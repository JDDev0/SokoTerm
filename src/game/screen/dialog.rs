use crate::io::{Color, Console, Key};

#[derive(Debug, Clone, Copy, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub enum DialogType {
    Information,
    Error,
    SecretFound,
}

#[derive(Debug, Clone, Copy, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub enum DialogSelection {
    No,
    Yes,
    Ok,
    Cancel,
}

pub trait Dialog {
    fn dialog_type(&self) -> DialogType;

    fn draw_border(&self, console: &Console, x: usize, y: usize, width: usize, height: usize) {
        console.set_cursor_pos(x, y);
        console.draw_text(" ".repeat(width));

        console.set_cursor_pos(x, y + height);
        console.draw_text(" ".repeat(width));
        for i in y + 1..y + height {
            console.set_cursor_pos(x, i);
            console.draw_text(" ");

            console.set_cursor_pos(x + width - 1, i);
            console.draw_text(" ");
        }
    }

    fn draw(&self, console: &Console, console_width: usize, console_height: usize);

    fn on_key_pressed(&self, console_width: usize, console_height: usize, key: Key) -> Option<DialogSelection>;
    fn on_mouse_pressed(&self, console_width: usize, console_height: usize, column: usize, row: usize) -> Option<DialogSelection>;
}

pub struct DialogYesNo {
    message: String,
    dialog_type: DialogType,
}

impl DialogYesNo {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            dialog_type: DialogType::Information,
        }
    }
}

impl Dialog for DialogYesNo {
    fn dialog_type(&self) -> DialogType {
        self.dialog_type
    }

    fn draw(&self, console: &Console, console_width: usize, console_height: usize) {
        let char_count = self.message.chars().count();

        let width = char_count.max(16);
        let width_with_border = width + 2;

        let x_start = ((console_width - width_with_border) as f64 * 0.5) as usize;
        let y_start = ((console_height - 6) as f64 * 0.5) as usize;

        let whitespace_count_half = ((width - char_count) as f64 * 0.5) as usize;

        console.set_color(Color::Black, Color::Yellow);
        console.set_cursor_pos(x_start + 1, y_start + 1);
        console.draw_text(format!(
            "{}{}{}",
            " ".repeat(whitespace_count_half),
            self.message,
            " ".repeat(width - char_count - whitespace_count_half),
        ));

        console.set_cursor_pos(x_start + 1, y_start + 2);
        console.draw_text(format!(
            "{}{}{}",
            " ".repeat(whitespace_count_half),
            "-".repeat(char_count),
            " ".repeat(width - char_count - whitespace_count_half),
        ));

        console.set_cursor_pos(x_start + 1, y_start + 3);
        console.draw_text(" ".repeat(width));

        console.set_cursor_pos(x_start + 1, y_start + 4);
        console.draw_text(format!(
            "[y]es{}[n]o",
            " ".repeat(width - 9),
        ));

        console.set_color(Color::LightBlack, Color::Red);
        self.draw_border(console, x_start, y_start, width_with_border, 5);
    }

    fn on_key_pressed(&self, _: usize, _: usize, key: Key) -> Option<DialogSelection> {
        if key == Key::Y {
            return Some(DialogSelection::Yes);
        }else if key == Key::N {
            return Some(DialogSelection::No);
        }

        None
    }

    fn on_mouse_pressed(&self, console_width: usize, console_height: usize, column: usize, row: usize) -> Option<DialogSelection> {
        let char_count = self.message.chars().count();

        let width = char_count.max(16);
        let width_with_border = width + 2;

        let x_start = ((console_width - width_with_border) as f64 * 0.5) as usize;
        let y_start = ((console_height - 6) as f64 * 0.5) as usize;

        if row == y_start + 4 {
            if (x_start + 1..x_start + 6).contains(&column) {
                return Some(DialogSelection::Yes);
            }else if (x_start + width - 3..x_start + width + 1).contains(&column) {
                return Some(DialogSelection::No);
            }
        }

        None
    }
}

pub struct DialogOk {
    message: String,
    fg_color: Color,
    dialog_type: DialogType,
}

impl DialogOk {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            fg_color: Color::Black,
            dialog_type: DialogType::Information,
        }
    }

    pub fn new_error(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            fg_color: Color::LightRed,
            dialog_type: DialogType::Error,
        }
    }

    pub fn new_secret_found(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            fg_color: Color::Black,
            dialog_type: DialogType::SecretFound,
        }
    }
}

impl Dialog for DialogOk {
    fn dialog_type(&self) -> DialogType {
        self.dialog_type
    }

    fn draw(&self, console: &Console, console_width: usize, console_height: usize) {
        let char_count = self.message.chars().count();

        let width = char_count.max(16);
        let width_with_border = width + 2;

        let x_start = ((console_width - width_with_border) as f64 * 0.5) as usize;
        let y_start = ((console_height - 6) as f64 * 0.5) as usize;

        let whitespace_count_half = ((width - char_count) as f64 * 0.5) as usize;

        console.set_color(self.fg_color, Color::Yellow);
        console.set_cursor_pos(x_start + 1, y_start + 1);
        console.draw_text(format!(
            "{}{}{}",
            " ".repeat(whitespace_count_half),
            self.message,
            " ".repeat(width - char_count - whitespace_count_half),
        ));

        console.set_color(Color::Black, Color::Yellow);
        console.set_cursor_pos(x_start + 1, y_start + 2);
        console.draw_text(format!(
            "{}{}{}",
            " ".repeat(whitespace_count_half),
            "-".repeat(char_count),
            " ".repeat(width - char_count - whitespace_count_half),
        ));

        console.set_cursor_pos(x_start + 1, y_start + 3);
        console.draw_text(" ".repeat(width));

        let whitespace_count_half = ((width - 4) as f64 * 0.5) as usize;

        console.set_cursor_pos(x_start + 1, y_start + 4);
        console.draw_text(format!(
            "{}[o]k{}",
            " ".repeat(whitespace_count_half),
            " ".repeat(width - 4 - whitespace_count_half),
        ));

        console.set_color(Color::LightBlack, Color::Red);
        self.draw_border(console, x_start, y_start, width_with_border, 5);
    }

    fn on_key_pressed(&self, _: usize, _: usize, key: Key) -> Option<DialogSelection> {
        if key == Key::O || key == Key::ENTER || key == Key::ESC {
            return Some(DialogSelection::Ok);
        }

        None
    }

    fn on_mouse_pressed(&self, console_width: usize, console_height: usize, column: usize, row: usize) -> Option<DialogSelection> {
        let char_count = self.message.chars().count();

        let width = char_count.max(16);
        let width_with_border = width + 2;

        let x_start = ((console_width - width_with_border) as f64 * 0.5) as usize;
        let y_start = ((console_height - 6) as f64 * 0.5) as usize;

        let whitespace_count_half = ((width - 4) as f64 * 0.5) as usize;

        if row == y_start + 4 && (x_start + whitespace_count_half + 1..x_start + whitespace_count_half + 5).contains(&column) {
            return Some(DialogSelection::Ok);
        }

        None
    }
}

pub struct DialogYesCancelNo {
    message: String,
    dialog_type: DialogType,
}

impl DialogYesCancelNo {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            dialog_type: DialogType::Information,
        }
    }
}

impl Dialog for DialogYesCancelNo {
    fn dialog_type(&self) -> DialogType {
        self.dialog_type
    }

    fn draw(&self, console: &Console, console_width: usize, console_height: usize) {
        let char_count = self.message.chars().count();

        let width = char_count.max(31);
        let width_with_border = width + 2;

        let x_start = ((console_width - width_with_border) as f64 * 0.5) as usize;
        let y_start = ((console_height - 6) as f64 * 0.5) as usize;

        let whitespace_count_half = ((width - char_count) as f64 * 0.5) as usize;

        console.set_color(Color::Black, Color::Yellow);
        console.set_cursor_pos(x_start + 1, y_start + 1);
        console.draw_text(format!(
            "{}{}{}",
            " ".repeat(whitespace_count_half),
            self.message,
            " ".repeat(width - char_count - whitespace_count_half),
        ));

        console.set_cursor_pos(x_start + 1, y_start + 2);
        console.draw_text(format!(
            "{}{}{}",
            " ".repeat(whitespace_count_half),
            "-".repeat(char_count),
            " ".repeat(width - char_count - whitespace_count_half),
        ));

        console.set_cursor_pos(x_start + 1, y_start + 3);
        console.draw_text(" ".repeat(width));

        let first_half = ((width - 17) as f64 * 0.5) as usize;
        let second_half = width - 17 - first_half;

        console.set_cursor_pos(x_start + 1, y_start + 4);
        console.draw_text(format!(
            "[y]es{}[c]ancel{}[n]o",
            " ".repeat(first_half),
            " ".repeat(second_half),
        ));

        console.set_color(Color::LightBlack, Color::Red);
        self.draw_border(console, x_start, y_start, width_with_border, 5);
    }

    fn on_key_pressed(&self, _: usize, _: usize, key: Key) -> Option<DialogSelection> {
        if key == Key::Y {
            return Some(DialogSelection::Yes);
        }else if key == Key::C || key == Key::ESC  {
            return Some(DialogSelection::Cancel);
        }else if key == Key::N {
            return Some(DialogSelection::No);
        }

        None
    }

    fn on_mouse_pressed(&self, console_width: usize, console_height: usize, column: usize, row: usize) -> Option<DialogSelection> {
        let char_count = self.message.chars().count();

        let width = char_count.max(31);
        let width_with_border = width + 2;

        let x_start = ((console_width - width_with_border) as f64 * 0.5) as usize;
        let y_start = ((console_height - 6) as f64 * 0.5) as usize;

        let first_half = ((width - 17) as f64 * 0.5) as usize;

        if row == y_start + 4 {
            if(x_start + 1..x_start + 6).contains(&column) {
                return Some(DialogSelection::Yes);
            }else if(x_start + 6 + first_half..x_start + 6 + first_half + 8).contains(&column) {
                return Some(DialogSelection::Cancel);
            }else if(x_start + width - 3..x_start + width + 1).contains(&column) {
                return Some(DialogSelection::No);
            }
        }

        None
    }
}
