#![allow(dead_code)]

use std::boxed::Box;
use std::io;
use std::io::{stdout, Write};
use std::process;

use crossterm::{execute, terminal, terminal::ClearType, tty::IsTty};
use crossterm::style::{style, Color, Attribute, ContentStyle};
use crossterm::event;
use crossterm::event::{Event, KeyEvent, KeyCode, KeyModifiers, MouseEvent};
use bitflags::bitflags;
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
    zwindow: u16,
    style: ContentStyle,
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
}

mod zscii {
    // from frotz
    pub const BAD: char = '\u{0000}';
    pub const NEW_STYLE: char = '\u{0001}';
    pub const NEW_FONT: char = '\u{0002}';
    pub const BACKSPACE: char = '\u{0008}';
    pub const INDENT: char = '\u{0009}';
    pub const GAP: char = '\u{000b}';
    pub const RETURN: char = '\u{000d}';
    pub const TIME_OUT: char = '\u{0018}';
    pub const ESCAPE: char = '\u{001b}';
    pub const DEL_WORD: char = '\u{001c}';
    pub const WORD_RIGHT: char = '\u{001d}';
    pub const WORD_LEFT: char = '\u{001e}';
    pub const DEL_TO_BOL: char = '\u{001f}';
    pub const ASCII_MIN: u8 = 0x20;
    pub const ASCII_MAX: u8 = 0x7e;
    pub const DEL: char = '\u{007f}';
    pub const ARROW_MIN: u8 = 0x81;
    pub const ARROW_UP: char = '\u{0081}';
    pub const ARROW_DOWN: char = '\u{0082}';
    pub const ARROW_LEFT: char = '\u{0083}';
    pub const ARROW_RIGHT: char = '\u{0084}';
    pub const ARROW_MAX: u8 = 0x84;
    pub const FKEY_MIN: u8 = 0x85;
    pub const FKEY_F1: char = '\u{0085}';
    pub const FKEY_F2: char = '\u{0086}';
    pub const FKEY_F3: char = '\u{0087}';
    pub const FKEY_F4: char = '\u{0088}';
    pub const FKEY_F5: char = '\u{0089}';
    pub const FKEY_F6: char = '\u{008a}';
    pub const FKEY_F7: char = '\u{008b}';
    pub const FKEY_F8: char = '\u{008c}';
    pub const FKEY_F9: char = '\u{008d}';
    pub const FKEY_F10: char = '\u{008e}';
    pub const FKEY_F11: char = '\u{008f}';
    pub const FKEY_F12: char = '\u{0090}';
    pub const FKEY_MAX: u8 = 0x90;
    pub const NUMPAD_MIN: u8 = 0x91;
    pub const NUMPAD_0: char = '\u{0091}';
    pub const NUMPAD_1: char = '\u{0092}';
    pub const NUMPAD_2: char = '\u{0093}';
    pub const NUMPAD_3: char = '\u{0094}';
    pub const NUMPAD_4: char = '\u{0095}';
    pub const NUMPAD_5: char = '\u{0096}';
    pub const NUMPAD_6: char = '\u{0097}';
    pub const NUMPAD_7: char = '\u{0098}';
    pub const NUMPAD_8: char = '\u{0099}';
    pub const NUMPAD_9: char = '\u{009a}';
    pub const NUMPAD_MAX: u8 = 0x9a;
    pub const SINGLE_CLICK: char = '\u{009b}';
    pub const DOUBLE_CLICK: char = '\u{009c}';
    pub const MENU_CLICK: char = '\u{009d}';
    pub const LATIN1_MIN: char = '\u{00a0}';
    pub const LATIN1_MAX: char = '\u{00ff}';
}

bitflags! {
    #[derive(Default)]
    struct Zstyle: u16 {
        const ROMAN = 0;
        const REVERSE = 1;
        const BOLDFACE = 2;
        const EMPHASIS = 4;
        const FIXED_WIDTH = 8;
    }
}

impl Zstyle {
    fn new(bits: u16) -> Zstyle {
        let mut zstyle = Zstyle::ROMAN;
        zstyle.bits = bits;
        zstyle
    }
}

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
            },
            zwindow: 0,
            style: ContentStyle::new(),
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
                if !(modifiers & KeyModifiers::ALT).is_empty() as bool { zscii::BAD }
                else if !(modifiers & KeyModifiers::CONTROL).is_empty() { Self::char_from_ucs2(ch as u16 - 'a' as u16 + 1) }
                else if !(modifiers & KeyModifiers::SHIFT).is_empty() { ch.to_uppercase().next().unwrap_or('?') }
                else { ch },
            KeyEvent { code: KeyCode::Char(ch @ 'A'..='Z'), modifiers } =>
                if !(modifiers & KeyModifiers::ALT).is_empty() as bool { zscii::BAD }
                else if !(modifiers & KeyModifiers::CONTROL).is_empty() { Self::char_from_ucs2(ch as u16 - 'A' as u16 + 1) }
                else { ch },
            KeyEvent { code: KeyCode::Char(ch), modifiers: KeyModifiers::NONE } => { ch },
            KeyEvent { code: KeyCode::Esc, .. } => { zscii::ESCAPE },
            KeyEvent { code: KeyCode::Up, .. } => { zscii::ARROW_UP },
            KeyEvent { code: KeyCode::Down, .. } => { zscii::ARROW_DOWN },
            KeyEvent { code: KeyCode::Left, .. } => { zscii::ARROW_LEFT },
            KeyEvent { code: KeyCode::Right, .. } => { zscii::ARROW_RIGHT },
            KeyEvent { code: KeyCode::Backspace, .. } => { zscii::BACKSPACE },
            KeyEvent { code: KeyCode::Enter, .. } => { zscii::RETURN },
            KeyEvent { code: KeyCode::Tab, .. } => { zscii::INDENT },
            KeyEvent { code: KeyCode::Delete, .. } => { zscii::DEL },
            KeyEvent { code: KeyCode::F(n), .. } => { Self::char_from_ucs2((zscii::FKEY_MIN + n - 1).into()) },
            _ => zscii::BAD,
        }
    }

    fn char_from_mouse_event(_mouse: MouseEvent) -> char {
        return zscii::BAD;
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

        if self.zwindow == 0 {
            self.buffer.print_styled(&self.style.apply(text));
        }
        else {
            self.window.cursor.x = self.window.buffer.print_at(self.window.cursor.x, self.window.cursor.y,
                text, self.style);
            if self.window.cursor.x > self.window.buffer.area.width {
                self.window.cursor.x = self.window.buffer.area.width - 1;
            }
        }
    }

    fn debug(&mut self, text: &str) {
        let zwindow = self.zwindow;
        self.zwindow = 0;
        self.print(text);
        self.zwindow = zwindow;
    }

    fn print_object(&mut self, object: &str) {
        if self.zwindow == 0 && self.is_term() {
            self.buffer.print_styled(&style(object).with(Color::White).attribute(Attribute::Italic));
        }
        else {
            self.print(object);
        }
    }

    fn set_text_style(&mut self, zstyle: u16) {
        let zstyle = Zstyle::new(zstyle);
        let mut style = ContentStyle::new();
        if !(zstyle & Zstyle::REVERSE).is_empty() {
            style = style.attribute(Attribute::Reverse);
        }
        if !(zstyle & Zstyle::BOLDFACE).is_empty() {
            style = style.foreground(Color::Red).attribute(Attribute::Bold);
        }
        if !(zstyle & Zstyle::EMPHASIS).is_empty() {
            style = style.attribute(Attribute::Italic);
        }
        // ignore FIXED_WIDTH because terminal
        self.style = style;
    }

    fn set_cursor(&mut self, _zwindow: i16, x_in: i16, y_in: i16) {
        if y_in < 0 {
            // v6 this turns on and off the cursor.  v6 isn't supported
            // in terminal mode
            return;
        }

        let     y = if y_in == 0 { self.window.cursor.y } else { y_in as u16 - 1 };
        let mut x = if x_in == 0 { self.window.cursor.x } else { x_in as u16 - 1 };

        if x >= self.window.buffer.area.width {
            x = 1;
        }

        self.window.cursor = Point {
            x: x,
            y: y,
        };
    }

    fn get_cursor(&mut self, _zwindow: i16) -> (u16, u16) {
        // only v6 supports a window param
        return (self.window.cursor.x + 1, self.window.cursor.y + 1);
    }

    fn erase_window(&mut self, zwindow: i16) {
        match zwindow {
            -2 => {
                self.buffer.clear();
                self.window.buffer.clear();
            },
            -1 => {
                self.split_window(0);
                self.clear();
            },
            0 => {
                self.buffer.clear();
            },
            1 => {
                self.window.buffer.clear();
            },
            _ => {
                self.debug(&format!("erase_window unknown window: {}\n", zwindow));
            },
        }
    }

    fn split_window(&mut self, height: u16) {
        if self.is_term() {
            let area = self.window.buffer.area;
            self.window.buffer.resize(Rect {
                x: area.x, y:0,
                width: area.width,
                height: height,
            }, false);
            self.buffer.resize(Rect {
                x: area.x, y: height,
                width: area.width,
                height: self.height - height,
            }, true);
            Self::print_raw(&format!("\x1B[{};{}r", height + 1, self.height));
            self.window.buffer.refresh();
            self.buffer.refresh();
        }
    }

    fn set_window(&mut self, zwindow: u16) {
        self.zwindow = zwindow;
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
        self.buffer.reset_more_counter();
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
    fn flush(&mut self) {}
    fn message(&self, _mtype: &str, _msg: &str) {}
}
