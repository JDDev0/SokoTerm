use std::cmp;
use std::collections::VecDeque;
use std::marker::PhantomData;
use std::sync::{Arc, Mutex};
use smol_str::SmolStr;

pub struct ConsoleState {
    dirty: bool,

    curser_pos: (usize, usize),
    current_color_pair: (Color, Color),

    text_buffer: Box<[u8]>,
    text_color_buffer: Box<[(Color, Color)]>,
    //TODO: underline

    input_queue_keyboard: VecDeque<Key>,
    input_queue_mouse: VecDeque<(usize, usize)>,
}

impl ConsoleState {
    pub fn new<const A: usize, const B: usize>() -> Self {
        Self {
            dirty: true,

            curser_pos: (0, 0),
            current_color_pair: (Color::White, Color::Black),

            text_buffer: vec![b' '; A * B].into_boxed_slice(),
            text_color_buffer: vec![(Color::White, Color::Black); A * B].into_boxed_slice(),
            //TODO: underline

            input_queue_keyboard: VecDeque::default(),
            input_queue_mouse: VecDeque::default(),
        }
    }

    pub fn dirty(&self) -> bool {
        self.dirty
    }

    pub fn mark_not_dirty(&mut self) {
        self.dirty = false;
    }

    pub fn clear_screen(&mut self) {
        self.dirty = true;
        self.curser_pos = (0, 0);

        self.text_buffer.fill(b' ');
        self.text_color_buffer.fill((Color::White, Color::Black));
        //TODO: underline
    }

    pub fn text_buffer(&self) -> &[u8] {
        &self.text_buffer
    }

    pub fn text_color_buffer(&self) -> &[(Color, Color)] {
        &self.text_color_buffer
    }

    pub fn input_queue_keyboard_mut(&mut self) -> &mut VecDeque<Key> {
        &mut self.input_queue_keyboard
    }

    pub fn input_queue_mouse_mut(&mut self) -> &mut VecDeque<(usize, usize)> {
        &mut self.input_queue_mouse
    }
}

pub struct Console<'a> {
    state: Arc<Mutex<ConsoleState>>,

    phantom: PhantomData<&'a ()>,
}

impl <'a> Console<'a> {
    pub fn new(state: Arc<Mutex<ConsoleState>>) -> Self {
        Self {
            state,
            phantom: PhantomData,
        }
    }

    /// Repaints the screen
    pub fn repaint(&self) {
        self.state.lock().unwrap().clear_screen();
    }

    /// Always returns fixed size of 74 chars in width and 23 chars in height
    pub const fn get_console_size(&self) -> (usize, usize) {
        (74, 23)
    }

    /// Checks if key input is available
    pub fn has_input(&self) -> bool {
        !self.state.lock().unwrap().input_queue_keyboard.is_empty()
    }

    /// Returns the key which was pressed or None
    pub fn get_key(&self) -> Option<Key> {
        self.state.lock().unwrap().input_queue_keyboard.pop_front()
    }

    /// Returns the coordinates of the pos where a left click occurred as (x, y).
    ///
    /// x and y represent character positions.
    ///
    /// If None, no left click occurred.
    pub fn get_mouse_pos_clicked(&self) -> Option<(usize, usize)> {
        self.state.lock().unwrap().input_queue_mouse.pop_front()
    }

    /// Draws text at the current cursor position.
    ///
    /// Behavior for Non-ASCII strings is terminal dependent.
    ///
    /// Characters which are out of bounds will be ignored and not drawn.
    pub fn draw_text(&self, text: impl Into<String>) {
        let text = text.into();
        if text.is_empty() {
            //Ignore empty text

            return;
        }

        let (width, height) = self.get_console_size();

        let mut state = self.state.lock().unwrap();
        let mut cursor_pos_x_before_newline = state.curser_pos.0;
        for line in text.split("\n") {
            if state.curser_pos.0 > width || state.curser_pos.1 >= height {
                //Allow writing to x = width (for trailing "\n") [out of bounds write will be prevented by cmp::min check below]

                //Ignore out of bounds writes
                return;
            }

            let start_index = state.curser_pos.0 + width * state.curser_pos.1;

            let len = cmp::min(line.len(), width - state.curser_pos.0);
            if len > 0 {
                let line = &line.as_bytes()[..len];
                let color = state.current_color_pair;

                state.text_buffer[start_index..start_index + len].copy_from_slice(line);
                state.text_color_buffer[start_index..start_index + len].fill(color);
            }

            cursor_pos_x_before_newline = state.curser_pos.0 + len;
            state.curser_pos.0 = 0;
            state.curser_pos.1 += 1;
        }

        //After loop y pos will always be one to high
        state.curser_pos.1 -= 1;

        if !text.ends_with("\n") {
            state.curser_pos.0 = cursor_pos_x_before_newline;
        }
    }

    /// Sets the color for foreground and background
    pub fn set_color(&self, fg: Color, bg: Color) {
        self.state.lock().unwrap().current_color_pair = (fg, bg);
    }

    /// Sets the color for foreground and background
    ///
    /// Foreground and background colors are swapped if inverted is true
    pub fn set_color_invertible(&self, fg: Color, bg: Color, inverted: bool) {
        if inverted {
            self.set_color(bg, fg);
        }else {
            self.set_color(fg, bg);
        }
    }

    /// Resets the color for foreground to [Color::White] and background to [Color::Black]
    pub fn reset_color(&self) {
        self.set_color(Color::White, Color::Black)
    }

    pub fn set_underline(&self, _underline: bool) {
        //TODO underline
    }

    /// Sets the cursor pos to x and y
    pub fn set_cursor_pos(&self, x: usize, y: usize) {
        self.state.lock().unwrap().curser_pos = (x, y);
    }
}

/// A representation of a key code for the bevy console lib abstraction.
///
/// The key should be checked with the constants provided in the [Key] implementation (Like [Key::SPACE]).
///
/// Unknown keys map to undefined values.
#[derive(Debug, Clone, Copy, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct Key(u16);

impl Key {
    //Ascii
    pub const SPACE: Key = Key(b' ' as u16);
    pub const EXCLAMATION_MARK: Key = Key(b'!' as u16);
    pub const QUOTATION_MARK: Key = Key(b'"' as u16);
    pub const NUMBER_SIGN: Key = Key(b'#' as u16);
    pub const DOLLAR: Key = Key(b'$' as u16);
    pub const PERCENT_SIGN: Key = Key(b'%' as u16);
    pub const AMPERSAND: Key = Key(b'&' as u16);
    pub const APOSTROPHE: Key = Key(b'\'' as u16);
    pub const LEFT_PARENTHESIS: Key = Key(b'(' as u16);
    pub const RIGHT_PARENTHESIS: Key = Key(b')' as u16);
    pub const ASTERISK: Key = Key(b'*' as u16);
    pub const PLUS: Key = Key(b'+' as u16);
    pub const COMMA: Key = Key(b',' as u16);
    pub const MINUS: Key = Key(b'-' as u16);
    pub const DOT: Key = Key(b'.' as u16);
    pub const SLASH: Key = Key(b'/' as u16);

    pub const COLON: Key = Key(b':' as u16);
    pub const SEMICOLON: Key = Key(b';' as u16);
    pub const LESS_THAN_SIGN: Key = Key(b'<' as u16);
    pub const EQUALS_SIGN: Key = Key(b'=' as u16);
    pub const GREATER_THAN_SIGN: Key = Key(b'>' as u16);
    pub const QUESTION_MARK: Key = Key(b'?' as u16);
    pub const AT_SIGN: Key = Key(b'@' as u16);

    pub const LEFT_BRACKET: Key = Key(b'[' as u16);
    pub const BACKSLASH: Key = Key(b'\\' as u16);
    pub const RIGHT_BRACKET: Key = Key(b']' as u16);
    pub const CARET: Key = Key(b'^' as u16);
    pub const UNDERSCORE: Key = Key(b'_' as u16);
    pub const BACKTICK: Key = Key(b'`' as u16);

    pub const LEFT_CURLY_BRACKET: Key = Key(b'{' as u16);
    pub const VERTICAL_BAR: Key = Key(b'|' as u16);
    pub const RIGHT_CURLY_BRACKET: Key = Key(b'}' as u16);
    pub const TILDE: Key = Key(b'~' as u16);

    pub const DIGIT_0: Key = Key(b'0' as u16);
    pub const DIGIT_1: Key = Key(b'1' as u16);
    pub const DIGIT_2: Key = Key(b'2' as u16);
    pub const DIGIT_3: Key = Key(b'3' as u16);
    pub const DIGIT_4: Key = Key(b'4' as u16);
    pub const DIGIT_5: Key = Key(b'5' as u16);
    pub const DIGIT_6: Key = Key(b'6' as u16);
    pub const DIGIT_7: Key = Key(b'7' as u16);
    pub const DIGIT_8: Key = Key(b'8' as u16);
    pub const DIGIT_9: Key = Key(b'9' as u16);

    pub const A: Key = Key(b'a' as u16);
    pub const B: Key = Key(b'b' as u16);
    pub const C: Key = Key(b'c' as u16);
    pub const D: Key = Key(b'd' as u16);
    pub const E: Key = Key(b'e' as u16);
    pub const F: Key = Key(b'f' as u16);
    pub const G: Key = Key(b'g' as u16);
    pub const H: Key = Key(b'h' as u16);
    pub const I: Key = Key(b'i' as u16);
    pub const J: Key = Key(b'j' as u16);
    pub const K: Key = Key(b'k' as u16);
    pub const L: Key = Key(b'l' as u16);
    pub const M: Key = Key(b'm' as u16);
    pub const N: Key = Key(b'n' as u16);
    pub const O: Key = Key(b'o' as u16);
    pub const P: Key = Key(b'p' as u16);
    pub const Q: Key = Key(b'q' as u16);
    pub const R: Key = Key(b'r' as u16);
    pub const S: Key = Key(b's' as u16);
    pub const T: Key = Key(b't' as u16);
    pub const U: Key = Key(b'u' as u16);
    pub const V: Key = Key(b'v' as u16);
    pub const W: Key = Key(b'w' as u16);
    pub const X: Key = Key(b'x' as u16);
    pub const Y: Key = Key(b'y' as u16);
    pub const Z: Key = Key(b'z' as u16);

    //Arrow keys
    pub const LEFT: Key = Key(5000);
    pub const UP: Key = Key(5001);
    pub const RIGHT: Key = Key(5002);
    pub const DOWN: Key = Key(5003);

    //F keys
    pub const F1: Key = Key(5004);
    pub const F2: Key = Key(5005);
    pub const F3: Key = Key(5006);
    pub const F4: Key = Key(5007);
    pub const F5: Key = Key(5008);
    pub const F6: Key = Key(5009);
    pub const F7: Key = Key(5010);
    pub const F8: Key = Key(5011);
    pub const F9: Key = Key(5012);
    pub const F10: Key = Key(5013);
    pub const F11: Key = Key(5014);
    pub const F12: Key = Key(5015);

    //Other keys
    pub const ESC: Key = Key(5016);
    pub const DELETE: Key = Key(5017);
    pub const ENTER: Key = Key(5018);
    pub const TAB: Key = Key(5019);
}

impl Key {
    pub fn from_bevy_key(key: &bevy::input::keyboard::Key, text: Option<&SmolStr>) -> Option<Self> {
        'ascii_chars: {
            if let Some(text) = text {
                let key = match &*text.to_ascii_lowercase() {
                    " " => Key::SPACE,
                    "!" => Key::EXCLAMATION_MARK,
                    "\"" => Key::QUOTATION_MARK,
                    "#" => Key::NUMBER_SIGN,
                    "$" => Key::DOLLAR,
                    "%" => Key::PERCENT_SIGN,
                    "&" => Key::AMPERSAND,
                    "'" => Key::APOSTROPHE,
                    "(" => Key::LEFT_PARENTHESIS,
                    ")" => Key::RIGHT_PARENTHESIS,
                    "*" => Key::ASTERISK,
                    "+" => Key::PLUS,
                    "," => Key::COMMA,
                    "-" => Key::MINUS,
                    "." => Key::DOT,
                    "/" => Key::SLASH,

                    ":" => Key::COLON,
                    ";" => Key::SEMICOLON,
                    "<" => Key::LESS_THAN_SIGN,
                    "=" => Key::EQUALS_SIGN,
                    ">" => Key::GREATER_THAN_SIGN,
                    "?" => Key::QUESTION_MARK,
                    "@" => Key::AT_SIGN,

                    "[" => Key::LEFT_BRACKET,
                    "\\" => Key::BACKSLASH,
                    "]" => Key::RIGHT_BRACKET,
                    "^" => Key::CARET,
                    "_" => Key::UNDERSCORE,
                    "`" => Key::BACKTICK,

                    "{" => Key::LEFT_CURLY_BRACKET,
                    "|" => Key::VERTICAL_BAR,
                    "}" => Key::RIGHT_CURLY_BRACKET,
                    "~" => Key::TILDE,

                    "0" => Key::DIGIT_0,
                    "1" => Key::DIGIT_1,
                    "2" => Key::DIGIT_2,
                    "3" => Key::DIGIT_3,
                    "4" => Key::DIGIT_4,
                    "5" => Key::DIGIT_5,
                    "6" => Key::DIGIT_6,
                    "7" => Key::DIGIT_7,
                    "8" => Key::DIGIT_8,
                    "9" => Key::DIGIT_9,

                    "a" => Key::A,
                    "b" => Key::B,
                    "c" => Key::C,
                    "d" => Key::D,
                    "e" => Key::E,
                    "f" => Key::F,
                    "g" => Key::G,
                    "h" => Key::H,
                    "i" => Key::I,
                    "j" => Key::J,
                    "k" => Key::K,
                    "l" => Key::L,
                    "m" => Key::M,
                    "n" => Key::N,
                    "o" => Key::O,
                    "p" => Key::P,
                    "q" => Key::Q,
                    "r" => Key::R,
                    "s" => Key::S,
                    "t" => Key::T,
                    "u" => Key::U,
                    "v" => Key::V,
                    "w" => Key::W,
                    "x" => Key::X,
                    "y" => Key::Y,
                    "z" => Key::Z,

                    _ => break 'ascii_chars,
                };
                return Some(key);
            }
        }

        let key = match key {
            //Arrow keys
            bevy::input::keyboard::Key::ArrowLeft => Key::LEFT,
            bevy::input::keyboard::Key::ArrowUp => Key::UP,
            bevy::input::keyboard::Key::ArrowRight => Key::RIGHT,
            bevy::input::keyboard::Key::ArrowDown => Key::DOWN,

            //F keys
            bevy::input::keyboard::Key::F1 => Key::F1,
            bevy::input::keyboard::Key::F2 => Key::F2,
            bevy::input::keyboard::Key::F3 => Key::F3,
            bevy::input::keyboard::Key::F4 => Key::F4,
            bevy::input::keyboard::Key::F5 => Key::F5,
            bevy::input::keyboard::Key::F6 => Key::F6,
            bevy::input::keyboard::Key::F7 => Key::F7,
            bevy::input::keyboard::Key::F8 => Key::F8,
            bevy::input::keyboard::Key::F9 => Key::F9,
            bevy::input::keyboard::Key::F10 => Key::F10,
            bevy::input::keyboard::Key::F11 => Key::F11,
            bevy::input::keyboard::Key::F12 => Key::F12,

            //Other keys
            bevy::input::keyboard::Key::Escape => Key::ESC,
            bevy::input::keyboard::Key::Delete => Key::DELETE,
            bevy::input::keyboard::Key::Backspace => Key::DELETE,
            bevy::input::keyboard::Key::Enter => Key::ENTER,
            bevy::input::keyboard::Key::Tab => Key::TAB,

            _ => return None,
        };
        Some(key)
    }

    pub fn is_arrow_key(&self) -> bool {
        (Key::LEFT..=Key::DOWN).contains(self)
    }

    /// Converts the keycode to an ASCII character if the key represents an ASCII character.
    pub fn to_ascii(&self) -> Option<u8> {
        self.is_ascii().then_some(self.0 as u8)
    }

    pub fn is_ascii(&self) -> bool {
        (0..=127).contains(&self.0)
    }

    /// Checks if a keycode is ASCII and numeric.
    pub fn is_numeric(&self) -> bool {
        self.is_ascii() && (self.0 as u8 as char).is_numeric()
    }

    /// Checks if a keycode is ASCII and alphanumeric.
    pub fn is_alphanumeric(&self) -> bool {
        self.is_ascii() && (self.0 as u8 as char).is_alphanumeric()
    }
}

/// 4-bit ANSI Color definitions for the bevy console lib abstraction.
#[repr(i8)]
#[derive(Debug, Clone, Copy, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub enum Color {
    Black,
    Blue,
    Green,
    Cyan,
    Red,
    Pink,
    Yellow,
    White,
    LightBlack,
    LightBlue,
    LightGreen,
    LightCyan,
    LightRed,
    LightPink,
    LightYellow,
    LightWhite,

    /// Default color is [Color::Black]
    Default = -1
}

impl From<Color> for bevy::color::Color {
    fn from(value: Color) -> Self {
        match value {
            Color::Black => bevy::color::Color::srgb_u8(23, 20, 33),
            Color::Red => bevy::color::Color::srgb_u8(192, 28, 40),
            Color::Green => bevy::color::Color::srgb_u8(38, 162, 105),
            Color::Yellow => bevy::color::Color::srgb_u8(162, 115, 76),
            Color::Blue => bevy::color::Color::srgb_u8(18, 72, 139),
            Color::Pink => bevy::color::Color::srgb_u8(163, 71, 186),
            Color::Cyan => bevy::color::Color::srgb_u8(42, 161, 179),
            Color::White => bevy::color::Color::srgb_u8(208, 207, 204),
            Color::LightBlack => bevy::color::Color::srgb_u8(94, 92, 100),
            Color::LightRed => bevy::color::Color::srgb_u8(246, 97, 81),
            Color::LightGreen => bevy::color::Color::srgb_u8(51, 218, 122),
            Color::LightYellow => bevy::color::Color::srgb_u8(233, 173, 12),
            Color::LightBlue => bevy::color::Color::srgb_u8(42, 123, 222),
            Color::LightPink => bevy::color::Color::srgb_u8(192, 97, 203),
            Color::LightCyan => bevy::color::Color::srgb_u8(51, 199, 222),
            Color::LightWhite => bevy::color::Color::srgb_u8(255, 255, 255),

            Color::Default => bevy::color::Color::srgb_u8(23, 20, 33),
        }
    }
}
