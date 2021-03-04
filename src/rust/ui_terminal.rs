#![allow(dead_code)]

use std::boxed::Box;
use std::io;
use std::io::{stdout, Write};
use std::process;

use crossterm::{execute, terminal, terminal::ClearType, tty::IsTty};
use crossterm::style::{style, Color, Attribute, ContentStyle};
use crossterm::event;
use crossterm::event::{Event, KeyEvent, KeyCode, KeyModifiers, MouseEvent};
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
    pub height: u16,
    pub width: u16,
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

// from frotz
const ZC_BAD: char = '\u{0000}';
const ZC_NEW_STYLE: char = '\u{0001}';
const ZC_NEW_FONT: char = '\u{0002}';
const ZC_BACKSPACE: char = '\u{0008}';
const ZC_INDENT: char = '\u{0009}';
const ZC_GAP: char = '\u{000b}';
const ZC_RETURN: char = '\u{000d}';
const ZC_TIME_OUT: char = '\u{0018}';
const ZC_ESCAPE: char = '\u{001b}';
const ZC_DEL_WORD: char = '\u{001c}';
const ZC_WORD_RIGHT: char = '\u{001d}';
const ZC_WORD_LEFT: char = '\u{001e}';
const ZC_DEL_TO_BOL: char = '\u{001f}';
const ZC_ASCII_MIN: u8 = 0x20;
const ZC_ASCII_MAX: u8 = 0x7e;
const ZC_DEL: char = '\u{007f}';
const ZC_ARROW_MIN: u8 = 0x81;
const ZC_ARROW_UP: char = '\u{0081}';
const ZC_ARROW_DOWN: char = '\u{0082}';
const ZC_ARROW_LEFT: char = '\u{0083}';
const ZC_ARROW_RIGHT: char = '\u{0084}';
const ZC_ARROW_MAX: u8 = 0x84;
const ZC_FKEY_MIN: u8 = 0x85;
const ZC_FKEY_F1: char = '\u{0085}';
const ZC_FKEY_F2: char = '\u{0086}';
const ZC_FKEY_F3: char = '\u{0087}';
const ZC_FKEY_F4: char = '\u{0088}';
const ZC_FKEY_F5: char = '\u{0089}';
const ZC_FKEY_F6: char = '\u{008a}';
const ZC_FKEY_F7: char = '\u{008b}';
const ZC_FKEY_F8: char = '\u{008c}';
const ZC_FKEY_F9: char = '\u{008d}';
const ZC_FKEY_F10: char = '\u{008e}';
const ZC_FKEY_F11: char = '\u{008f}';
const ZC_FKEY_F12: char = '\u{0090}';
const ZC_FKEY_MAX: u8 = 0x90;
const ZC_NUMPAD_MIN: u8 = 0x91;
const ZC_NUMPAD_0: char = '\u{0091}';
const ZC_NUMPAD_1: char = '\u{0092}';
const ZC_NUMPAD_2: char = '\u{0093}';
const ZC_NUMPAD_3: char = '\u{0094}';
const ZC_NUMPAD_4: char = '\u{0095}';
const ZC_NUMPAD_5: char = '\u{0096}';
const ZC_NUMPAD_6: char = '\u{0097}';
const ZC_NUMPAD_7: char = '\u{0098}';
const ZC_NUMPAD_8: char = '\u{0099}';
const ZC_NUMPAD_9: char = '\u{009a}';
const ZC_NUMPAD_MAX: u8 = 0x9a;
const ZC_SINGLE_CLICK: char = '\u{009b}';
const ZC_DOUBLE_CLICK: char = '\u{009c}';
const ZC_MENU_CLICK: char = '\u{009d}';
const ZC_LATIN1_MIN: char = '\u{00a0}';
const ZC_LATIN1_MAX: char = '\u{00ff}';

impl TerminalUI {
    pub fn new_with_width(width: u16) -> Box<TerminalUI> {
        let mut width = if width == 0 { u16::MAX } else { width };
        let mut height = 25;
        let mut isatty = false;

        let area = if let Ok((w, h)) = terminal::size() {
            isatty = stdout().is_tty();
            let margin = if w > width { (w - width) / 2 } else { 0 }; // round to equal margins
            width = w - margin * 2;
            height = h;
            Self::print_raw(&format!("\x1B[{};{}r", 2, h));
            Rect {
                x: margin,
                y: 1,
                width: w - margin * 2,
                height: h - 1,
            }
        }
        else {
            width = 60;
            Rect {
                x: 0,
                y: 0,
                width: width,
                height: height,
            }
        };

        Box::new(TerminalUI {
            isatty: isatty,
            height: height,
            width: width,
            buffer: WrapBuffer::new(area),
            window: Window {
                buffer: TermBuffer::new(Rect { x: area.x, y: 0, width: area.width, height: 1 }),
                cursor: Point { x: 0, y: 0 },
                style: ContentStyle::new(),
            },
        })
    }

    fn print_raw(raw: &str) {
        print!("{}", raw);
        io::stdout().flush().unwrap();
    }

    fn is_term(&self) -> bool {
        self.isatty
    }

    fn char_from_ucs2(ucs2: u16) -> char {
        String::from_utf16_lossy(&[ucs2]).chars().next().unwrap_or('?')
    }

    fn char_from_key_event(key: KeyEvent) -> char {
        match key {
            KeyEvent { code: KeyCode::Char(ch @ 'a'..='z'), modifiers } =>
                if !(modifiers & KeyModifiers::ALT).is_empty() as bool { ZC_BAD }
                else if !(modifiers & KeyModifiers::CONTROL).is_empty() { Self::char_from_ucs2(ch as u16 - 'a' as u16 + 1) }
                else if !(modifiers & KeyModifiers::SHIFT).is_empty() { ch.to_uppercase().next().unwrap_or('?') }
                else { ch },
            KeyEvent { code: KeyCode::Char(ch @ 'A'..='Z'), modifiers } =>
                if !(modifiers & KeyModifiers::ALT).is_empty() as bool { ZC_BAD }
                else if !(modifiers & KeyModifiers::CONTROL).is_empty() { Self::char_from_ucs2(ch as u16 - 'A' as u16 + 1) }
                else { ch },
            KeyEvent { code: KeyCode::Char(ch), modifiers: KeyModifiers::NONE } => { ch },
            KeyEvent { code: KeyCode::Esc, .. } => { ZC_ESCAPE },
            KeyEvent { code: KeyCode::Up, .. } => { ZC_ARROW_UP },
            KeyEvent { code: KeyCode::Down, .. } => { ZC_ARROW_DOWN },
            KeyEvent { code: KeyCode::Left, .. } => { ZC_ARROW_LEFT },
            KeyEvent { code: KeyCode::Right, .. } => { ZC_ARROW_RIGHT },
            KeyEvent { code: KeyCode::Backspace, .. } => { ZC_BACKSPACE },
            KeyEvent { code: KeyCode::Enter, .. } => { ZC_RETURN },
            KeyEvent { code: KeyCode::Tab, .. } => { ZC_INDENT },
            KeyEvent { code: KeyCode::Delete, .. } => { ZC_DEL },
            KeyEvent { code: KeyCode::F(n), .. } => { Self::char_from_ucs2((ZC_FKEY_MIN + n - 1).into()) },
            _ => ZC_BAD,
        }
    }

    fn char_from_mouse_event(_mouse: MouseEvent) -> char {
        return ZC_BAD;
    }
}

impl Drop for TerminalUI {
    fn drop(&mut self) {
        if self.is_term() {
            println!("[Hit any key to exit.]");
            terminal::enable_raw_mode().unwrap_or(());
            loop {
                match event::read().unwrap() {
                    Event::Key(_) => break,
                    Event::Mouse(_) => break,
                    _ => continue,
                }
            }
            terminal::disable_raw_mode().unwrap_or(());
            Self::print_raw(&format!("\x1B[r"));
        }
    }
}

impl UI for TerminalUI {
    fn new() -> Box<TerminalUI> {
        Self::new_with_width(55)
    }

    fn clear(&self) {
        if self.is_term() {
            execute!(stdout(), terminal::Clear(ClearType::All)).unwrap();
            Self::print_raw(&format!("\x1B[{};{}r", self.window.buffer.area.height + 1, self.height));
        }
    }

    fn print(&mut self, text: &str) {
        if !self.is_term() {
            Self::print_raw(text);
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

    fn split_window(&mut self, height: u16) {
        if self.is_term() {
            self.window.buffer.resize(Rect {
                x: 0, y:0,
                width: self.width,
                height: height,
            }, false);
            self.buffer.resize(Rect {
                x: 0, y: height,
                width: self.width,
                height: self.height - height,
            }, true);
            Self::print_raw(&format!("\x1B[{};{}r", height + 1, self.height));
            self.window.buffer.refresh();
            self.buffer.refresh();
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
            // reverse what the enter did at the end of input
            if let Err(_) = execute!(stdout(), terminal::ScrollDown(1)) {
                self.buffer.refresh();
            }

            // write over what the user did again so that our buffers match
            // for reflow (rewrap)
            self.buffer.print(&format!("{}\n\n", &input));
        }

        input
    }

    fn read_char(&self) -> char {
        terminal::enable_raw_mode().unwrap_or(());
        let c = loop {
            let e = event::read();
            match e {
                Ok(Event::Key(key)) => break Self::char_from_key_event(key),
                Ok(Event::Mouse(mouse)) => break Self::char_from_mouse_event(mouse),
                _ => continue,
            }
        };
        terminal::disable_raw_mode().unwrap_or(());

        if c == '\u{3}' {
            process::exit(1);
        }

        c
    }

    fn reset(&self) {
        println!();
    }

    // unimplemented, only used in web ui
    fn erase_window(&mut self, _window: i16) {}
    fn flush(&mut self) {}
    fn message(&self, _mtype: &str, _msg: &str) {}
}
