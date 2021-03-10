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

//! A text buffer grid
//!
//! Stores characters and associated style specifiers for a rectangular
//! grid of text (for display with a fixed width font, for example).

use std::ops::Range;
use std::iter::Enumerate;
use unicode_segmentation::UnicodeSegmentation;

#[derive(Debug, Copy, Clone)]
pub struct Rect {
    pub x: u16,
    pub y: u16,
    pub width: u16,
    pub height: u16,
}

#[derive(Debug, Copy, Clone)]
pub struct Run<T> where T: PartialEq {
    pub start: u16,
    pub style: T,
}

#[derive(Debug)]
pub struct Row<T> where T: PartialEq {
    pub runs: Vec<Run<T>>,
    pub text: String,
}

pub struct RowIterRunRanges<'a, T> where T: PartialEq {
    row: &'a Row<T>,
    cur: usize,
}

impl<'a, T> Iterator for RowIterRunRanges<'a, T> where T: PartialEq {
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

pub struct RowIter<'a, T> where T: PartialEq {
    row: &'a Row<T>,
    width: usize,
    graphemes: usize,
    iter: Enumerate<RowIterRunRanges<'a, T>>,
}

impl<'a, T> Iterator for RowIter<'a, T> where T: PartialEq {
    type Item = (&'a str, &'a T);

    fn next(&mut self) -> Option<Self::Item> {
        if self.graphemes >= self.width {
            return None;
        }

        if let Some((i, mut r)) = self.iter.next() {
            let run_text = &self.row.text[r.start..r.end];
            let mut grapheme_indices = run_text.grapheme_indices(true);
            if let Some((next, _)) = &mut grapheme_indices.nth((self.width - self.graphemes).into()) {
                r.end = r.start + *next;
            }

            let run_text = &self.row.text[r];
            self.graphemes += count_graphemes(&run_text);
            Some((&run_text, &self.row.runs[i].style))
        }
        else {
            None
        }
    }
}

pub fn count_graphemes(s: &str) -> usize {
    UnicodeSegmentation::grapheme_indices(s, true).count()
}

impl<T> Row<T> where T: PartialEq + Default {
    pub fn new() -> Row<T> {
        let run = Run::<T> {
            start: 0,
            style: Default::default(),
        };
        Row {
            runs: vec![run],
            text: "".to_string(),
        }
    }

    pub fn iter_run_ranges(&self) -> RowIterRunRanges<T> {
        RowIterRunRanges {
            row: self,
            cur: 0,
        }
    }

    pub fn iter_width(&self, width: u16) -> RowIter<T> {
        RowIter {
            row: self,
            iter: self.iter_run_ranges().enumerate(),
            width: width as usize,
            graphemes: 0,
        }
    }

    #[allow(dead_code)]
    pub fn iter(&self) -> RowIter<T> {
        self.iter_width(u16::MAX)
    }

    pub fn apply_style(&mut self, start: usize, end: usize, style: &T) where T: Copy {
        let mut new_runs: Vec<Run<T>> = Vec::new();
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

    fn find_grapheme_index(&self, grapheme_index: u16) -> (usize, usize) {
        let mut count = 0;
        let mut iter = self.text.grapheme_indices(true);

        // count up until we find grapheme_index
        loop {
            if let Some((pos, _grapheme)) = iter.next() {
                if count >= grapheme_index {
                    break (pos, 0);
                }
                count += 1;
            }
            else {
                let pad = grapheme_index - count;
                break (self.text.len() + pad as usize, pad.into());
            }
        }
    }

    pub fn overwrite_at(&mut self, grapheme_index: u16, s: &str, style: &T) -> u16 where T: Copy {
        let s_len = count_graphemes(s);
        if s_len == 0 {
            return grapheme_index;
        }

        // count up until we find grapheme_index
        let (start, pad) = self.find_grapheme_index(grapheme_index);

        // count down until we've depleted s_len
        let end = if start >= self.text.len() {
            start
        }
        else {
            let mut iter = self.text[start..].grapheme_indices(true);
            let mut count = s_len as u16;
            loop {
                if let Some((pos, _)) = iter.next() {
                    if count <= 0 {
                        break start + pos;
                    }
                    count -= 1;
                }
                else {
                    break self.text.len() + pad as usize;
                }
            }
        };

        // pad out with spaces
        self.text.push_str(&" ".repeat(pad.into()));
        self.text.replace_range(start..end, &s);
        self.apply_style(start, start + s.len(), style);
        return grapheme_index + s_len as u16;
    }

    pub fn truncate_at(&mut self, grapheme_index: u16) {
        let (start, _) = self.find_grapheme_index(grapheme_index);
        self.text = self.text[0..start].to_string();
    }

    pub fn append(&mut self, s: &str, style: &T) where T: Copy {
        let l = self.text.len();
        self.text.push_str(s);
        self.apply_style(l, l + s.len(), style);
    }
}

