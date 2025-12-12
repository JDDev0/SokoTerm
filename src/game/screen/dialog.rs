use crate::io::{Color, Console, Key};

#[derive(Debug, Clone, Copy, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub enum DialogType {
    Information,
    Error,
}

impl DialogType {
    pub fn text_color(self) -> Color {
        match self {
            DialogType::Information => Color::Black,
            DialogType::Error => Color::LightRed,
        }
    }
}

#[derive(Debug, Clone, Copy, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub enum DialogSelection {
    No,
    Yes,
    Ok,
    Cancel,
}

#[derive(Debug, Clone)]
pub struct DialogOption {
    text: &'static str,
    action: DialogSelection,
    keys: &'static [Key],
}

impl DialogOption {
    pub const OK: &'static DialogOption = &DialogOption::new("[O]k", DialogSelection::Ok, &[Key::O, Key::ENTER, Key::SPACE, Key::ESC]);
    pub const YES: &'static DialogOption = &DialogOption::new("[Y]es", DialogSelection::Yes, &[Key::Y]);
    pub const NO: &'static DialogOption = &DialogOption::new("[N]o", DialogSelection::No, &[Key::N]);
    pub const CANCEL: &'static DialogOption = &DialogOption::new("[C]ancel", DialogSelection::Cancel, &[Key::C, Key::ESC]);

    pub const fn new(text: &'static str, action: DialogSelection, keys: &'static [Key]) -> Self {
        Self { text, action, keys }
    }

    pub fn text(&self) -> &str {
        self.text
    }

    pub fn action(&self) -> DialogSelection {
        self.action
    }

    pub fn keys(&self) -> &[Key] {
        self.keys
    }
}

#[derive(Debug, Clone)]
pub struct Dialog {
    dialog_type: DialogType,
    message: Box<str>,
    options: Box<[&'static DialogOption]>,
}

impl Dialog {
    pub fn new(dialog_type: DialogType, message: impl Into<Box<str>>, options: Box<[&'static DialogOption]>) -> Self {
        Self {
            dialog_type,
            message: message.into(),
            options,
        }
    }

    pub fn new_ok(message: impl Into<Box<str>>) -> Self {
        Self {
            dialog_type: DialogType::Information,
            message: message.into(),
            options: Box::from([DialogOption::OK]),
        }
    }

    pub fn new_ok_error(message: impl Into<Box<str>>) -> Self {
        Self {
            dialog_type: DialogType::Error,
            message: message.into(),
            options: Box::from([DialogOption::OK]),
        }
    }

    pub fn new_yes_no(message: impl Into<Box<str>>) -> Self {
        Self {
            dialog_type: DialogType::Information,
            message: message.into(),
            options: Box::from([DialogOption::YES, DialogOption::NO]),
        }
    }

    pub fn new_yes_cancel_no(message: impl Into<Box<str>>) -> Self {
        Self {
            dialog_type: DialogType::Information,
            message: message.into(),
            options: Box::from([DialogOption::YES, DialogOption::CANCEL, DialogOption::NO]),
        }
    }

    pub fn dialog_type(&self) -> DialogType {
        self.dialog_type
    }

    pub fn message(&self) -> &str {
        &self.message
    }

    pub fn options(&self) -> &[&DialogOption] {
        &self.options
    }

    pub fn render(self, width: usize, height: usize) -> RenderedDialog {
        RenderedDialog::new(self, width, height)
    }
}

#[derive(Debug, Clone)]
pub struct RenderedDialog {
    dialog: Dialog,
    lines: Box<[Box<str>]>,
    width: usize,
    height: usize,
}

impl RenderedDialog {
    fn split_message(message: &str, width: usize) -> Box<[Box<str>]> {
        if message.len() <= width && !message.contains("\n") {
            return vec![Box::from(message)].into_boxed_slice();
        }

        let mut lines = Vec::new();
        for line in message.split("\n") {
            let mut start_index = 0;

            'line_split:
            while line[start_index..].len() > width {
                let line = &line[start_index..];
                let orig_len = line.len();

                //Skip whitespace in front
                let line = line.trim_start();
                start_index += orig_len - line.len();

                for i in (0..width).rev() {
                    if matches!(line.as_bytes()[i], b' ' | b',' | b';' | b'.' | b'!' | b'?') {
                        start_index += i + 1;
                        lines.push(Box::from(line[..=i].trim()));

                        continue 'line_split;
                    }
                }

                for i in (0..width).rev() {
                    if matches!(line.as_bytes()[i], b'-' | b'=') {
                        start_index += i + 1;
                        lines.push(Box::from(line[..=i].trim()));

                        continue 'line_split;
                    }
                }

                for i in (0..width).rev() {
                    if matches!(line.as_bytes()[i], b'\\' | b'/' | b':') {
                        start_index += i + 1;
                        lines.push(Box::from(line[..=i].trim()));

                        continue 'line_split;
                    }
                }

                start_index += width;
                lines.push(Box::from(line[..width].trim()));
            }

            lines.push(Box::from(line[start_index..].trim()));
        }

        lines.into_boxed_slice()
    }

    pub fn new(dialog: Dialog, width: usize, height: usize) -> Self {
        let lines = Self::split_message(&dialog.message, width - 2);

        RenderedDialog {
            dialog,
            lines,
            width,
            height,
        }
    }

    pub fn draw(&self, console: &Console) {
        let option_len = self.dialog.options.iter().
                map(|option| option.text.len() + 3).
                sum::<usize>();

        let max_char_count = self.lines.iter().map(|line| line.len()).max().unwrap_or(0);
        let width = max_char_count.max(option_len + 2);
        let width_with_border = width + 2;

        let line_count = self.lines.len();

        let x_start = ((self.width - width - 2) as f64 * 0.5) as usize;
        let y_start = ((self.height - line_count - 5) as f64 * 0.5) as usize;

        for (i, line) in self.lines.iter().enumerate() {
            let char_count = line.len();

            let whitespace_count_half = ((width - char_count) as f64 * 0.5) as usize;

            console.set_color(self.dialog.dialog_type.text_color(), Color::Yellow);
            console.set_cursor_pos(x_start + 1, y_start + i + 1);
            console.draw_text(format!(
                "{}{}{}",
                " ".repeat(whitespace_count_half),
                line,
                " ".repeat(width - char_count - whitespace_count_half),
            ));
        }

        let whitespace_count_half = ((width - max_char_count) as f64 * 0.5) as usize;

        console.set_cursor_pos(x_start + 1, y_start + line_count + 1);
        console.draw_text(format!(
            "{}{}{}",
            " ".repeat(whitespace_count_half),
            "-".repeat(max_char_count),
            " ".repeat(width - max_char_count - whitespace_count_half),
        ));

        console.set_cursor_pos(x_start + 1, y_start + line_count + 2);
        console.draw_text(" ".repeat(width));

        let whitespace_count_half = ((width - option_len + 3) as f64 * 0.5) as usize;

        console.set_cursor_pos(x_start + 1, y_start + line_count + 3);
        console.draw_text(" ".repeat(whitespace_count_half));

        for option in self.dialog.options.iter() {
            console.draw_text(format!("{}   ", option.text));
        }

        console.draw_text(" ".repeat(width - option_len - whitespace_count_half));

        console.set_color(Color::LightBlack, Color::Red);
        self.draw_border(console, x_start, y_start, width_with_border, line_count + 4);
    }

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

    pub fn on_key_pressed(&self, key: Key) -> Option<DialogSelection> {
        for option in self.dialog.options.iter() {
            if option.keys.contains(&key) {
                return Some(option.action);
            }
        }

        None
    }

    pub fn on_mouse_pressed(&self, column: usize, row: usize) -> Option<DialogSelection> {
        let line_count = self.lines.len();

        let y_start = ((self.height - line_count - 5) as f64 * 0.5) as usize;

        let y_pos_options = y_start + line_count + 3;
        if row != y_pos_options {
            return None;
        }

        let option_len = self.dialog.options.iter().
                map(|option| option.text.len() + 3).
                sum::<usize>();

        let max_char_count = self.lines.iter().map(|line| line.len()).max().unwrap_or(0);
        let width = max_char_count.max(option_len + 2);

        let x_start = ((self.width - width - 2) as f64 * 0.5) as usize;

        let whitespace_count_half = ((width - option_len + 3) as f64 * 0.5) as usize;

        let x_start_options = x_start + whitespace_count_half + 1;
        if column < x_start_options {
            return None;
        }

        let mut x_pos_relative = column - x_start_options;
        for option in self.dialog.options.iter() {
            if x_pos_relative < option.text.len() {
                return Some(option.action);
            }

            x_pos_relative -= option.text.len();

            if x_pos_relative < 3 {
                return None;
            }

            x_pos_relative -= 3;
        }

        None
    }
}
