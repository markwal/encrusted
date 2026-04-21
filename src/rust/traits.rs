use bitflags::bitflags;

pub trait UI {
    fn clear(&self);
    fn print(&mut self, text: &str);
    fn debug(&mut self, text: &str);
    fn print_object(&mut self, object: &str);
    fn set_status_bar(&mut self, left: &str, right: &str);
    fn erase_window(&mut self, window: i16);

    // only used by terminal ui
    fn reset(&self);
    fn get_user_input(&mut self) -> String;
    fn split_window(&mut self, height: u16);
    fn read_char(&self) -> char;
    fn set_text_style(&mut self, zstyle: Zstyle);
    fn set_window(&mut self, zwindow: u16);
    fn get_window(&mut self) -> u16;
    fn set_cursor(&mut self, zwindow: i16, x: i16, y: i16);
    fn get_cursor(&mut self, zwindow: i16) -> (u16, u16);

    // only used by web ui
    #[allow(dead_code)]
    fn flush(&mut self);
    fn message(&self, mtype: &str, msg: &str);
}

bitflags! {
    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
    pub struct Zstyle: u16 {
        const ROMAN = 0;
        const REVERSE = 1;
        const BOLDFACE = 2;
        const EMPHASIS = 4;
        const FIXED_WIDTH = 8;
    }
}

impl Zstyle {
    pub fn new(bits: u16) -> Zstyle {
        Zstyle::from_bits_retain(bits)
    }
}
