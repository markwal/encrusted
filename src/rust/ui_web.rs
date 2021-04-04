use std::boxed::Box;
use std::ffi::CString;
use std::fmt::Write;

use serde_json;

use js_message;
use traits::{UI, Zstyle};
use chgrid::{Rect, ChGrid};

#[allow(dead_code)]
#[derive(Debug)]
enum Token {
    Newline,
    Text(String),
    Object(String),
    Debug(String),
    TextProps(ZTextProps),
    Erase,
}

enum_from_primitive! {
    #[derive(Clone, Copy, Debug, PartialEq)]
    enum Zfont {
        NoChange = 0,
        Normal = 1,
        Picture = 2,
        CharGraphics = 3,
        Fixed = 4,
    }
}

impl Default for Zfont {
    fn default() -> Self { Zfont::Normal }
}

#[allow(dead_code)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ZTextProps {
    font: Zfont,
    style: Zstyle,
    color: u32,
}

#[derive(Debug)]
pub struct WebUI {
    buffer: Vec<Token>,
    windows: Vec<Window>,
    cur_window: usize,
    props: ZTextProps,
}

impl WebUI {
    pub fn new() -> Box<WebUI> {
        Box::new(WebUI { 
            buffer: Vec::new(),
            windows: Vec::new(),
            cur_window: 0,
            props: Default::default(),
        })
    }

    #[allow(dead_code)]
    fn css_from_zprops(zprops: &ZTextProps) -> String {
        let mut v = Vec::new();
        if zprops.style.contains(Zstyle::REVERSE) {
            v.push("reverse");
        }
        if zprops.style.contains(Zstyle::BOLDFACE) {
            v.push("bold");
        }
        if zprops.style.contains(Zstyle::EMPHASIS) {
            v.push("emphasis");
        }

        v.join(" ")
    }

    #[allow(dead_code)]
    fn render_window(&self, zwindow: usize) -> String {
        // zwindow 0 is the scrolling transcript which is handled by the token
        // stream in self.buffer
        if zwindow == 0 || zwindow - 1 >= self.windows.len() {
            return String::new();
        }

        let window = &self.windows[zwindow - 1];
        let mut s = String::with_capacity(window.grid.rows.len() * 80);
        for row in &window.grid.rows {
            s.push_str(r#"<div className="window">"#);
            for (text, zprops) in row.iter() {
                write!(s, r#"<span className="{}">{}</span>"#, Self::css_from_zprops(&zprops), text);
            }
            s.push_str("</div>\n");
        }
        s.shrink_to_fit();
        return s;
    }
}

impl UI for WebUI {
    fn print(&mut self, text: &str) {
        if self.cur_window > 0 {
            if self.cur_window < self.windows.len() {
                let window = &mut self.windows[self.cur_window - 1];
                let cursor = &mut window.cursor;
                let x = window.grid.print_at(cursor.x - 1, cursor.y - 1, text, &self.props);
                window.cursor.x = std::cmp::min(x, window.get_width() - 1);
                window.dirty = true;
            }
            return;
        }

        if text.is_empty() {
            return;
        }

        if text == "\n" {
            self.buffer.push(Token::Newline);
            return;
        }

        if !text.contains('\n') {
            self.buffer.push(Token::Text(String::from(text)));
            return;
        }

        let lines = text.lines().collect::<Vec<_>>();

        for (index, line) in lines.iter().enumerate() {
            if !line.is_empty() {
                self.buffer.push(Token::Text(String::from(*line)));
            }

            if let Some(_) = lines.get(index + 1) {
                self.buffer.push(Token::Newline);
            }
        }

        if text.ends_with('\n') {
            self.buffer.push(Token::Newline);
        }
    }

    fn debug(&mut self, text: &str) {
        self.buffer.push(Token::Debug(String::from(text)));
    }

    fn print_object(&mut self, obj: &str) {
        if self.cur_window > 0 {
            self.print(obj);
            return;
        }
        self.buffer.push(Token::Object(String::from(obj)));
    }

    fn flush(&mut self) {
        if self.windows.len() > 0 && self.windows[0].dirty {
            self.message("window_update", &self.render_window(1));
            self.windows[0].dirty = false;
        }

        if self.buffer.is_empty() {
            return;
        }

        let mut html = String::new();

        for (index, item) in self.buffer.iter().enumerate() {
            let prev = if index == 0 {
                None
            } else {
                self.buffer.get(index - 1)
            };

            let next = self.buffer.get(index + 1);

            match *item {
                Token::TextProps(_) => {
                }
                Token::Newline => {
                    html.push_str("<br>");
                }
                Token::Text(ref text) => {
                    match prev {
                        Some(&Token::Text(_)) => (),
                        _ => html.push_str("<span>"),
                    }

                    html.push_str(&text);

                    match next {
                        Some(&Token::Text(_)) => (),
                        _ => html.push_str("</span>"),
                    }
                }
                Token::Object(ref obj) => {
                    let class = match (prev, next) {
                        (None, Some(&Token::Newline)) => "room",
                        (Some(&Token::Newline), Some(&Token::Newline)) => "room",
                        _ => "object",
                    };

                    write!(html, r#"<span class="{}">{}</span>"#, class, obj).unwrap();
                }
                Token::Debug(ref text) => {
                    write!(html, r#"<span class="debug">{}</span>"#, text).unwrap();
                }
                Token::Erase => {
                    html.push_str("<div height=\"100%\"></div>");
                }
            }
        }

        self.message("print", &html);
        self.buffer.clear();
    }

    fn set_status_bar(&mut self, left: &str, right: &str) {
        let msg = serde_json::to_string(&(left, right)).unwrap();
        self.message("header", &msg);
    }

    fn message(&self, mtype: &str, msg: &str) {
        let type_ptr = CString::new(mtype).unwrap().into_raw();
        let msg_ptr = CString::new(msg).unwrap().into_raw();

        unsafe {
            js_message(type_ptr, msg_ptr);
            CString::from_raw(type_ptr); // free memory
            CString::from_raw(msg_ptr);
        }
    }

    fn erase_window(&mut self, window: i16) {
        if window == 0 || window == -1 {
            self.buffer.push(Token::Erase);
            self.flush();
        }
    }

    fn split_window(&mut self, height: u16) {
        let rect = Rect {
            x: 0,
            y: 0,
            width: 60, // TODO use current screen width?
            height: height,
        };
        let mut window = Window::new(&rect);
        window.dirty = true;
        if self.windows.len() > 0 {
            self.windows[0] = window;
        }
        else {
            self.windows.push(window);
        }
    }

    fn set_text_style(&mut self, zstyle: Zstyle) {
        self.props.style = zstyle;
    }

    fn set_window(&mut self, zwindow: u16) {
        self.cur_window = zwindow.into();
    }

    fn get_window(&mut self) -> u16 {
        return (self.cur_window) as u16;
    }

    fn set_cursor(&mut self, zwindow: i16, x_in: i16, y_in: i16) {
        if zwindow == 0 || zwindow as usize > self.windows.len() {
            return;
        }
        let zwindow = zwindow as usize - 1;

        if y_in < 0 {
            // TODO v6 this turns on and off the cursor
            return;
        }

        let cursor = &self.windows[zwindow].cursor;

        let     y = if y_in == 0 { cursor.y } else { y_in as u16 };
        let mut x = if x_in == 0 { cursor.x } else { x_in as u16 };

        if x >= self.windows[zwindow].get_width() {
            x = 1;
        }

        self.windows[zwindow].cursor = Point { x, y };
    }

    fn get_cursor(&mut self, zwindow: i16) -> (u16, u16) {
        if zwindow > 0 && zwindow as usize - 1 < self.windows.len() {
            let cursor = &self.windows[zwindow as usize - 1].cursor;
            (cursor.x, cursor.y)
        }
        else {
            (1, 1)
        }
    }

    // Terminal UI only
    fn get_user_input(&mut self) -> String {
        unimplemented!();
    }
    fn read_char(&self) -> char {
        unimplemented!();
    }
    fn clear(&self) {}
    fn reset(&self) {}
}

#[derive(Debug, Clone)]
struct Point {
    x: u16,
    y: u16,
}

#[derive(Debug)]
struct Window {
    grid: ChGrid<ZTextProps>,
    cursor: Point,
    dirty: bool,
}

impl Window {
    fn new(rect: &Rect) -> Window {
        Window {
            grid: ChGrid::<ZTextProps>::new(*rect),
            cursor: Point { x: 1, y: 1, },
            dirty: false,
        }
    }

    fn get_width(&self) -> u16 {
        self.grid.area.width
    }
}
