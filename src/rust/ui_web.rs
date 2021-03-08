use std::boxed::Box;
use std::ffi::CString;
use std::fmt::Write;

use serde_json;

use js_message;
use traits::UI;

#[derive(Debug)]
enum Token {
    Newline,
    Text(String),
    Object(String),
    Debug(String),
    Erase,
}

#[derive(Debug)]
pub struct WebUI {
    buffer: Vec<Token>,
}

impl UI for WebUI {
    fn new() -> Box<WebUI> {
        Box::new(WebUI { buffer: Vec::new() })
    }

    fn print(&mut self, text: &str) {
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
        self.buffer.push(Token::Object(String::from(obj)));
    }

    fn flush(&mut self) {
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
        self.message("header", &msg)
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

    fn clear(&self) {}
    fn reset(&self) {}
    fn split_window(&mut self, _: u16) {}
    fn set_text_style(&mut self, _zstyle: u16) {}
    fn set_window(&mut self, _zwindow: u16) {}
    fn set_cursor(&mut self, _zwindow: i16, _x: i16, _y: i16) {}
    fn get_cursor(&mut self, _zwindow: i16) -> (u16, u16) {
        todo!();
    }
    fn get_user_input(&mut self) -> String {
        unimplemented!();
    }
    fn read_char(&self) -> char {
        unimplemented!();
    }
}
