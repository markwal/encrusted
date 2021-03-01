/*
The MIT License (MIT)

Copyright (c) 2021 Mark Walker

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
*/

//! Very minimal TUI (Terminal User Interface) abstraction
//!
//! Define rectangular regions of the terminal screen (TermBuffer) that contain the text, colors
//! and attributes to allow that portion of the screen to be redrawn, scrolled, partially
//! rewritten, etc.
#![allow(dead_code)]

use std::io::{Write, stdout};
use std::cmp;
use std::ops::Range;
use std::iter::{Enumerate, Peekable};
use crossterm::{QueueableCommand, cursor, execute, queue, terminal};
use crossterm::style::{style, Color, Attribute, ContentStyle, StyledContent, Print};
use unicode_segmentation::{UnicodeSegmentation, UWordBoundIndices};

#[derive(Debug, Copy, Clone)]
pub struct Rect {
    pub x: u16,
    pub y: u16,
    pub width: u16,
    pub height: u16,
}

#[derive(Debug)]
/// Terminal UI text and style buffer
pub struct TermBuffer {
    pub area: Rect,
    rows: Vec<Row>,
    first_row: u32,
}

impl TermBuffer {
    /// Create a new TermBuffer covering the indicated area of the terminal
    /// screen
    pub fn new(area: Rect) -> TermBuffer {
        TermBuffer {
            area: area,
            rows: Vec::new(),
            first_row: 0,
        }
    }

    /// Change the location and/or extent of this TermBuffer on the terminal
    /// screen.
    ///
    /// Doesn't draw the changes until refresh is called, leaving behind old
    /// contents, if any
    ///
    /// if keep_last, will try to keep the bottom row with the same content
    /// as the old bottom row.  Otherwise, it'll be the top row that is 
    /// maintained
    pub fn resize(&mut self, area: Rect, keep_last: bool) {
        if keep_last {
            self.first_row += area.height as u32;
            self.first_row -= cmp::min(self.first_row, self.area.height as u32);
        }
        self.area = area;
    }

    /// Print styled text at a particular place within the term buffer area
    pub fn print_at(&mut self, x: u16, y: u16, s: &str, style: ContentStyle) {
        let irow = self.first_row as usize + y as usize;
        if irow as usize >= self.rows.len() {
            for _i in self.rows.len()..(irow + 1) {
                self.rows.push(Row::new());
            }
        }
        self.rows[irow].overwrite_at(x, &s, &style);

        // ignore output errors
        queue!(stdout(),
            cursor::MoveTo(x + self.area.x, y + self.area.y),
            Print(style.apply(s))
        ).unwrap_or(());
        stdout().flush().unwrap_or(());
    }

    /// Redraw the entire area covered by this TermBuffer
    pub fn refresh(&self) {
        let mut stdout = stdout();
        let mut y = self.area.y;
        for row in &self.rows[self.first_row as usize..] {
            queue!(stdout, cursor::MoveTo(self.area.x, y)).unwrap_or(());
            for s in row.iter_width(self.area.width) {
                queue!(stdout, Print(s)).unwrap_or(());
            }
            let l = row.text.len();
            if l < self.area.width as usize {
                queue!(stdout, Print(style(" ".repeat(self.area.width as usize - l)))).unwrap_or(());
            }
            y += 1;
        }
        let empty_line = style(" ".repeat(self.area.width as usize));
        while y < self.area.y + self.area.height {
            queue!(stdout, cursor::MoveTo(self.area.x, y)).unwrap_or(());
            queue!(stdout, Print(&empty_line)).unwrap_or(());
            y += 1;
        }
        stdout.flush().unwrap_or(());
    }
}

#[derive(Debug)]
/// A Terminal UI text buffer that word wraps its contents
pub struct WrapBuffer {
    termbuf: TermBuffer, // wrapped, display ready rows
    lines: Vec<Row>,     // unwrapped, newline terminated lines
}

impl WrapBuffer {
    pub fn new(area: Rect) -> WrapBuffer {
        WrapBuffer {
            termbuf: TermBuffer::new(area),
            lines: Vec::new(),
        }
    }

    fn last_line_terminated(&self) -> bool {
        let l = self.lines.len();
        if l == 0 {
            return true;
        }
        return self.lines[l - 1].text.ends_with("\n");
    }

    pub fn print_styled(&mut self, s: &StyledContent<&str>) {
        self.print_style(&s.content(), &s.style());
    }

    pub fn print_style(&mut self, s: &str, style: &ContentStyle) {
        if self.last_line_terminated() {
            self.lines.push(Row::new());
        }
        self.lines.last_mut().unwrap().append(s, style);
        self.wrap_append(s, style);
    }

    pub fn print(&mut self, s: &str) {
        self.print_style(s, &ContentStyle::new());
    }

    fn wrap_append(&mut self, s: &str, style: &ContentStyle) {
        let mut width_row = 0;

        if let Some(row) = self.termbuf.rows.last() {
            width_row = count_graphemes(&row.text);
        }

        let mut scroll_up = false;
        for (_, row_text) in s.wrap_to_width_offset(self.termbuf.area.width as usize, width_row) {
            if scroll_up || row_text.len() == 0 {
                self.termbuf.first_row += 1;
                self.termbuf.refresh();
            }
            scroll_up = true;
            self.termbuf.print_at(width_row as u16, self.termbuf.area.height - 1, row_text, *style);
            width_row = 0;
        }
    }

    fn rewrap(&mut self) {
        let mut rows: Vec<Row> = Vec::new();
        let mut row = Row::new();
        let width = self.termbuf.area.width as usize;

        for line in &self.lines {
            let mut iter = line.iter_run_ranges().enumerate();
            let next_run = iter.next().unwrap(); // must be at least one run
            let mut irun = next_run.0;
            let mut run_range = next_run.1;

            for (row_start, row_text) in line.text.wrap_to_width(width) {
                row.text = row_text.to_string();
                let mut line_pos = row_start;
                let mut row_pos = 0;

                while line_pos < line.text.len() {
                    while line_pos >= run_range.end {
                        let next_run = iter.next().unwrap(); // must have runs to cover text
                        irun = next_run.0;
                        run_range = next_run.1;
                    }
                    let run_end = cmp::min(row.text.len(), run_range.end - row_start);
                    row.apply_style(row_pos, run_end, &line.runs[irun].style);
                    row_pos = run_end;
                    line_pos = run_range.end;
                }
                rows.push(row);
                row = Row::new();
            }
        }

        let height = self.termbuf.area.height as usize;
        if rows.len() < height {
            let mut empty_rows = (0..height - rows.len()).map(|_| { Row::new() }).collect::<Vec<Row>>();
            empty_rows.append(&mut rows);
            rows = empty_rows;
        }
        self.termbuf.first_row = (rows.len() - height) as u32;
        self.termbuf.rows = rows;
        self.termbuf.refresh();
    }
}

#[derive(Debug, Copy, Clone)]
struct Run {
    start: u16,
    style: ContentStyle,
}

#[derive(Debug)]
struct Row {
    runs: Vec<Run>,
    text: String,
}

struct RowIterRunRanges<'a> {
    row: &'a Row,
    cur: usize,
}

impl<'a> Iterator for RowIterRunRanges<'a> {
    type Item = Range<usize>;

    fn next(&mut self) -> Option<Self::Item> {
        let l = self.row.runs.len();
        let i = self.cur;
        if i >= l {
            return None
        }
        let end = if i + 1 >= l { self.row.text.len() } else { self.row.runs[i + 1].start as usize };
        self.cur += 1;
        Some(self.row.runs[i].start as usize..end)
    }
}

struct RowIter<'a> {
    row: &'a Row,
    width: u16,
    iter: Enumerate<RowIterRunRanges<'a>>,
}

impl<'a> Iterator for RowIter<'a> {
    type Item = StyledContent<&'a str>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some((i, mut r)) = self.iter.next() {
            if r.start > self.width as usize {
                return None;
            }
            if r.end > self.width as usize {
                r.end = self.width as usize;
            }
            Some(self.row.runs[i].style.apply(&self.row.text[r]))
        }
        else {
            None
        }
    }
}

pub fn count_graphemes(s: &str) -> usize {
    UnicodeSegmentation::grapheme_indices(s, true).count()
}

struct UnicodeWordIter<'a> {
    s: &'a str,
    iter: Peekable<UWordBoundIndices<'a>>,
}

impl<'a> Iterator for UnicodeWordIter<'a> {
    type Item = (usize, &'a str);

    fn next(&mut self) -> Option<Self::Item> {
        if let Some((i, w)) = self.iter.next() {
            let mut end = i + w.len();
            let mut eat_one = false;

            if let Some((_, wnext)) = self.iter.peek() {
                if wnext.chars().all(|c| c.is_ascii_punctuation()) {
                    end += wnext.len();
                    eat_one = true;
                }
            }
            if eat_one {
                self.iter.next();
            }
            Some((i, &self.s[i..end]))
        }
        else { None }
    }
}

struct WordWrapIter<'a> {
    s: &'a str,
    pos: usize,
    width: usize,
    cur_width: usize,
}

trait WordWrapper {
    fn word_bounds_for_wrapping<'a>(&'a self) -> UnicodeWordIter<'a>;
    fn wrap_to_width<'a>(&'a self, width: usize) -> WordWrapIter<'a>;
    fn wrap_to_width_offset<'a>(&'a self, width: usize, offset: usize) -> WordWrapIter<'a>;
}

impl WordWrapper for str {
    fn word_bounds_for_wrapping<'a>(&'a self) -> UnicodeWordIter<'a> {
        UnicodeWordIter {
            s: self,
            iter: self.split_word_bound_indices().peekable(),
        }
    }

    fn wrap_to_width_offset<'a>(&'a self, width: usize, offset: usize) -> WordWrapIter<'a> {
        WordWrapIter {
            s: self,
            pos: 0,
            width: width,
            cur_width: offset,
        }
    }

    fn wrap_to_width<'a>(&'a self, width: usize) -> WordWrapIter<'a> {
        self.wrap_to_width_offset(width, 0)
    }
}

impl<'a> Iterator for WordWrapIter<'a> {
    type Item = (usize, &'a str);

    fn next(&mut self) -> Option<Self::Item> {
        if self.pos >= self.s.len() {
            return None; // done
        }

        let start = self.pos;

        // special case of a \n at the end. An extra empty string to signal that next
        // run of text shouldn't be appended to the last run from this one
        if &self.s[self.pos..] == "\n" {
            self.pos += 1;
            return Some((start, &self.s[start..start]))
        }

        // for each word in the remaining input
        for (i, word) in self.s[start..].word_bounds_for_wrapping() {
            let next_pos = start + i;
            let word_graphemes = count_graphemes(word);

            // new line
            if word == "\n" {
                // min allows a final new line to come through twice (for that empty string above)
                self.pos = cmp::min(next_pos + 1, self.s.len() - 1);
                self.cur_width = 0;
                return Some((start, &self.s[start..next_pos]));
            }

            // if the next word doesn't fit, return what we have so far
            if self.cur_width + word_graphemes > self.width {

                // do we have a wider than the screen word?
                if self.cur_width == 0 {
                    // nth(width) grapheme
                    let mut grapheme_indices = word.grapheme_indices(true);
                    if let Some((next, _)) = &mut grapheme_indices.nth(self.width) {
                        self.pos = *next;
                    }
                    else {
                        self.pos = next_pos + word.len();
                    }
                    return Some((next_pos, &self.s[next_pos..self.pos]));
                }

                // first eat all of the trailing whitepace
                let end = next_pos;
                let next_pos = if let Some((i, _)) = self.s[end..].split_word_bound_indices()
                        .find(|(_, w)| !w.chars().all(char::is_whitespace) || w == &"\n")
                {
                    end + i
                }
                else {
                    self.s.len()
                };

                // return the accumulated line
                self.pos = next_pos;
                self.cur_width = 0;
                return Some((start, &self.s[start..end]));
            }

            // keep this word
            self.cur_width += word_graphemes;
        }

        self.pos = self.s.len();
        return if self.pos > start { Some((start, &self.s[start..self.pos])) } else { None };
    }
}

impl Row {
    fn new() -> Row {
        let run = Run {
            start: 0,
            style: ContentStyle::new(),
        };
        Row {
            runs: vec![run],
            text: "".to_string(),
        }
    }

    fn iter_run_ranges(&self) -> RowIterRunRanges {
        RowIterRunRanges {
            row: self,
            cur: 0,
        }
    }

    fn iter_width(&self, width: u16) -> RowIter {
        RowIter {
            row: self,
            iter: self.iter_run_ranges().enumerate(),
            width: width,
        }
    }

    fn iter(&self) -> RowIter {
        self.iter_width(u16::MAX)
    }

    /// return the run indices that span the indicated start and end 
    fn run_range(&self, start: usize, end: usize) -> Range<usize> {
        let mut start_run = 0;
        for (i, r) in self.iter_run_ranges().enumerate() {
            if r.contains(&start) {
                start_run = i;
            }
            if r.contains(&end) {
                return start_run..i;
            }
        }
        start_run..self.runs.len()
    }

    fn apply_style(&mut self, start: usize, end: usize, style: &ContentStyle) {
        let mut new_runs: Vec<Run> = Vec::new();
        let mut pushed_start = false;

        for (i, r) in self.iter_run_ranges().enumerate() {
            if r.start < start {
                new_runs.push(self.runs[i]);
            }
            else {
                if !pushed_start {
                    pushed_start = true;
                    if i == 0 || self.runs[i - 1].style != *style {
                        new_runs.push(Run {
                            start: start as u16,
                            style: *style,
                        });
                    }
                }
                if r.contains(&end) && self.runs[i].style != *style {
                    new_runs.push(Run {
                        start: end as u16,
                        style: self.runs[i].style,
                    });
                }
                if r.start > end {
                    new_runs.push(self.runs[i]);
                }
            }
        }
        let last_style = &self.runs.last().unwrap().style;
        if !pushed_start && *last_style != *style {
            new_runs.push(Run {
                start: start as u16,
                style: *style,
            });
            if end < self.text.len() {
                new_runs.push(Run {
                    start: end as u16,
                    style: *last_style,
                });
            }
        }
        self.runs = new_runs;
    }

    fn overwrite_at(&mut self, grapheme_index: u16, s: &str, style: &ContentStyle) {
        let s_len = count_graphemes(s);
        if s_len == 0 {
            return;
        }

        let mut iter = self.text.grapheme_indices(true);
        let mut count = 0;
        let mut pad = 0;

        // count up until we find grapheme_index
        let start = loop {
            if let Some((pos, _grapheme)) = iter.next() {
                if count >= grapheme_index {
                    break pos;
                }
                count += 1;
            }
            else {
                pad = grapheme_index - count;
                break self.text.len() + pad as usize;
            }
        };

        count = s_len as u16;
        // count down until we've depleted s_len
        let end = loop {
            if let Some((pos, _grapheme)) = iter.next() {
                count -= 1;
                if count <= 0 {
                    break pos;
                }
            }
            else {
                break self.text.len() + pad as usize;
            }
        };

        // pad out with spaces
        self.text.push_str(&" ".repeat(pad.into()));
        self.text.replace_range(start..end, &s);
        self.apply_style(start, start + s.len(), style);
    }

    fn append(&mut self, s: &str, style: &ContentStyle) {
        let l = self.text.len();
        self.text.push_str(s);
        self.apply_style(l, l + s.len(), style);
    }
}

pub fn test_termbuffer() {
    let mut row = Row::new();
    row.overwrite_at(3, "hello", &ContentStyle::new().attribute(Attribute::Bold));
    row.overwrite_at(10, "there", &ContentStyle::new().background(Color::Red).attribute(Attribute::SlowBlink));
    row.overwrite_at(7, "wow", &ContentStyle::new().attribute(Attribute::Underlined));
    row.overwrite_at(12, "holymoly", &ContentStyle::new().background(Color::Blue));
    println!("row: {:?}", row);

    for r in row.iter_run_ranges() {
        println!("range: {:?}", r);
    }

    for (i, r) in row.iter_run_ranges().enumerate() {
        execute!(
            stdout(),
            Print(row.runs[i].style.apply(&row.text[r]))
        ).expect("failed to print");
    }
    println!("");

    let mut out = stdout();
    for s in row.iter() {
        out.queue(Print(s)).expect("failed to queue terminal command");
    }
    out.flush().expect("failed to flush terminal commands");
    println!("");

    let (cols, rows) = terminal::size().expect("failed to retrieve terminal size");

    let mut wrap = WrapBuffer::new(Rect {
        x: 0,
        y: 0,
        width: cols,
        height: rows,
    });
    for _ in 1..5 {
        for _ in 1..10 {
            wrap.print("The quick brown fox jumps over the lazy dog. ");
        }
        wrap.print("\n");
    }
    wrap.print("Jim quickly realized that the beautiful gowns are expensive. ");
    wrap.print("The quick brown fox jumps over the lazy dog. ");
    wrap.print("The quick brown fox jumps over the lazy dog.\n");
    wrap.print("The quick brown fox jumps over the lazy dog. ");
    println!(":");
    wrap.rewrap();

    let mut buf = TermBuffer::new(Rect {
        x: 0,
        y: 0,
        width: cols,
        height: rows,
    });
    buf.print_at(10, 5, "This is a test!", ContentStyle::new().background(Color::Blue));
    execute!(stdout(), cursor::MoveTo(0, rows - 3)).unwrap();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_overwrite() {
        let mut row = Row::new();
        row.overwrite_at(10, &"a".repeat(20), &ContentStyle::new());
        let mut iter = row.iter_run_ranges();
        assert_eq!(Some(0..30), iter.next());

        row.overwrite_at(15, &"b".repeat(5), &ContentStyle::new().background(Color::Red));
        println!("{:?}", row);
        let mut iter = row.iter_run_ranges();
        assert_eq!(Some(0..15), iter.next());
        assert_eq!(Some(15..20), iter.next());
        assert_eq!(Some(20..30), iter.next());
    }
}
