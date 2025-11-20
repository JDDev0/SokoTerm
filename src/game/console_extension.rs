use crate::io::{Color, Console};

pub trait ConsoleExtension {
    fn draw_key_input_text(&self, input_text: &str);
}

impl<'a> ConsoleExtension for Console<'a> {
    fn draw_key_input_text(&self, input_text: &str) {
        self.set_color(Color::LightRed, Color::Default);
        self.draw_text(input_text);
    }
}
