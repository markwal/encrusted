#![allow(dead_code)]

use std::boxed::Box;
use std::io;
use std::io::{stdout, Write};

use crossterm::{execute, terminal, terminal::ClearType, tty::IsTty};
use crossterm::style::{style, Color, Attribute, ContentStyle};
use regex::Regex;
use termbuffer::{TermBuffer, WrapBuffer, Rect, count_graphemes};

use traits::UI;

lazy_static! {
    static ref ANSI_RE: Regex = Regex::new(
        r"[\x1b\x9b][\[()#;?]*(?:[0-9]{1,4}(?:;[0-9]{0,4})*)?[0-9A-PRZcf-nqry=><]"
    ).unwrap();
}

#[derive(Debug)]
pub struct TerminalUI {
    isatty: bool,
    buffer: WrapBuffer,
    window: Window,
}

#[derive(Debug)]
struct Point {
    x: u16,
    y: u16,
}

#[derive(Debug)]
struct Window {
    buffer: TermBuffer,
    cursor: Point,
    style: ContentStyle,
}

impl TerminalUI {
    pub fn new_with_width(width: u16) -> Box<TerminalUI> {
        let width = if width == 0 { u16::MAX } else { width };
        let mut isatty = false;

        let area = if let Ok((w, h)) = terminal::size() {
            isatty = stdout().is_tty();
            let margin = if w > width { (w - width) / 2 } else { 0 }; // round to equal margins
            Rect {
                x: margin,
                y: 1,
                width: w - margin * 2,
                height: h,
            }
        }
        else {
            Rect {
                x: 0,
                y: 0,
                width: 60,
                height: 25,
            }
        };

        Box::new(TerminalUI {
            isatty: isatty,
            buffer: WrapBuffer::new(area),
            window: Window {
                buffer: TermBuffer::new(Rect { x: area.x, y: 0, width: area.width, height: 1 }),
                cursor: Point { x: 0, y: 0 },
                style: ContentStyle::new(),
            },
        })
    }

    fn print_raw(&self, raw: &str) {
        print!("{}", raw);
        io::stdout().flush().unwrap();
    }

    fn is_term(&self) -> bool {
        self.isatty
    }
}

impl UI for TerminalUI {
    fn new() -> Box<TerminalUI> {
        Self::new_with_width(55)
    }

    fn clear(&self) {
        if self.is_term() {
            execute!(stdout(), terminal::Clear(ClearType::All)).unwrap();
        }
    }

    fn print(&mut self, text: &str) {
        if !self.is_term() {
            self.print_raw(text);
            return;
        }

        self.buffer.print(text);
    }

    fn debug(&mut self, text: &str) {
        self.print(text);
    }

    fn print_object(&mut self, object: &str) {
        if self.is_term() {
            self.buffer.print_styled(&style(object).with(Color::White).attribute(Attribute::Bold));
        }
        else {
            self.print(object);
        }
    }

    fn set_status_bar(&mut self, left: &str, right: &str) {
        if self.is_term() {
            let width = self.window.buffer.area.width;
            self.window.buffer.print_at(0, 0,
                &format!(" {:width$}", left, width = (width - 1) as usize),
                ContentStyle::new().attribute(Attribute::Reverse)
            );

            let right_width = count_graphemes(right) as u16 + 1;
            self.window.buffer.print_at(width - right_width, 0, right, ContentStyle::new().attribute(Attribute::Reverse));
        }
    }

    fn get_user_input(&mut self) -> String {
        let mut input = String::new();
        io::stdin()
            .read_line(&mut input)
            .expect("Error reading input");

        // trim, strip and control sequences that might have gotten in,
        // and then trim once more to get rid of any excess whitespace
        let input = ANSI_RE
            .replace_all(input.trim(), "")
            .to_string()
            .trim()
            .to_string();

        if self.is_term() {
            self.buffer.print(&format!("{}\n\n", &input));
        }

        input
    }

    fn reset(&self) {
        println!();
    }

    // unimplemented, only used in web ui
    fn erase_window(&mut self, _window: i16) {}
    fn flush(&mut self) {}
    fn message(&self, _mtype: &str, _msg: &str) {}
}
