pub trait UI {
    fn new() -> Box<Self>
    where
        Self: Sized;
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

    // only used by web ui
    fn flush(&mut self);
    fn message(&self, mtype: &str, msg: &str);
}
