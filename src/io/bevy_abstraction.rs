use std::{cmp, mem};
use std::collections::VecDeque;
use std::marker::PhantomData;
use std::sync::{Arc, Mutex};
use bevy::asset::{AssetServer, Handle};
use bevy::image::Image;
use smol_str::SmolStr;
use crate::game::level::Tile;
use crate::game::TileMode;

#[derive(Debug, Clone)]
pub struct ColorScheme {
    color_mapping: [bevy::color::Color; 17],
}

impl ColorScheme {
    const fn new(color_mapping: [bevy::color::Color; 17]) -> Self {
        Self { color_mapping }
    }

     fn convert_console_color_to_bevy_color(&self, color: Color) -> bevy::color::Color {
        let index = color as i8;

        self.color_mapping[(index + 1) as usize]
    }
}

macro_rules! color_scheme {
    {
        Default: ($cdr:literal, $cdg:literal, $cdb:literal),
        Black: ($c0r:literal, $c0g:literal, $c0b:literal),
        Blue: ($c1r:literal, $c1g:literal, $c1b:literal),
        Green: ($c2r:literal, $c2g:literal, $c2b:literal),
        Cyan: ($c3r:literal, $c3g:literal, $c3b:literal),
        Red: ($c4r:literal, $c4g:literal, $c4b:literal),
        Pink: ($c5r:literal, $c5g:literal, $c5b:literal),
        Yellow: ($c6r:literal, $c6g:literal, $c6b:literal),
        White: ($c7r:literal, $c7g:literal, $c7b:literal),
        LightBlack: ($c8r:literal, $c8g:literal, $c8b:literal),
        LightBlue: ($c9r:literal, $c9g:literal, $c9b:literal),
        LightGreen: ($c10r:literal, $c10g:literal, $c10b:literal),
        LightCyan: ($c11r:literal, $c11g:literal, $c11b:literal),
        LightRed: ($c12r:literal, $c12g:literal, $c12b:literal),
        LightPink: ($c13r:literal, $c13g:literal, $c13b:literal),
        LightYellow: ($c14r:literal, $c14g:literal, $c14b:literal),
        LightWhite: ($c15r:literal, $c15g:literal, $c15b:literal),
    } => {
        ColorScheme::new([
            bevy::color::Color::srgb_u8($cdr, $cdg, $cdb),

            bevy::color::Color::srgb_u8($c0r, $c0g, $c0b),
            bevy::color::Color::srgb_u8($c1r, $c1g, $c1b),
            bevy::color::Color::srgb_u8($c2r, $c2g, $c2b),
            bevy::color::Color::srgb_u8($c3r, $c3g, $c3b),
            bevy::color::Color::srgb_u8($c4r, $c4g, $c4b),
            bevy::color::Color::srgb_u8($c5r, $c5g, $c5b),
            bevy::color::Color::srgb_u8($c6r, $c6g, $c6b),
            bevy::color::Color::srgb_u8($c7r, $c7g, $c7b),
            bevy::color::Color::srgb_u8($c8r, $c8g, $c8b),
            bevy::color::Color::srgb_u8($c9r, $c9g, $c9b),
            bevy::color::Color::srgb_u8($c10r, $c10g, $c10b),
            bevy::color::Color::srgb_u8($c11r, $c11g, $c11b),
            bevy::color::Color::srgb_u8($c12r, $c12g, $c12b),
            bevy::color::Color::srgb_u8($c13r, $c13g, $c13b),
            bevy::color::Color::srgb_u8($c14r, $c14g, $c14b),
            bevy::color::Color::srgb_u8($c15r, $c15g, $c15b),
        ])
    };
}

pub const MUTED_COLOR_SCHEME: ColorScheme = color_scheme! {
    Default: (40, 42, 46),

    Black: (40, 42, 46),
    Blue: (95, 129, 157),
    Green: (140, 148, 64),
    Cyan: (94, 141, 135),
    Red: (165, 66, 66),
    Pink: (133, 103, 143),
    Yellow: (222, 147, 95),
    White: (208, 207, 204),
    LightBlack: (120, 120, 120),
    LightBlue: (129, 162, 190),
    LightGreen: (181, 189, 104),
    LightCyan: (138, 190, 183),
    LightRed: (204, 102, 102),
    LightPink: (178, 148, 187),
    LightYellow: (240, 198, 116),
    LightWhite: (255, 255, 255),
};

pub const BLUE_TINTED_COLOR_SCHEME: ColorScheme = color_scheme! {
    Default: (23, 20, 33),

    Black: (23, 20, 33),
    Blue: (18, 72, 139),
    Green: (38, 162, 105),
    Cyan: (42, 161, 179),
    Red: (192, 28, 40),
    Pink: (163, 71, 186),
    Yellow: (162, 115, 76),
    White: (208, 207, 204),
    LightBlack: (94, 92, 100),
    LightBlue: (42, 123, 222),
    LightGreen: (51, 218, 122),
    LightCyan: (51, 199, 222),
    LightRed: (246, 97, 81),
    LightPink: (192, 97, 203),
    LightYellow: (233, 173, 12),
    LightWhite: (255, 255, 255),
};

pub const DARK_COLOR_SCHEME: ColorScheme = color_scheme! {
    Default: (16, 16, 16),

    Black: (16, 16, 16),
    Blue: (16, 16, 150),
    Green: (16, 150, 16),
    Cyan: (16, 150, 150),
    Red: (150, 16, 16),
    Pink: (150, 16, 150),
    Yellow: (150, 120, 16),
    White: (192, 192, 192),
    LightBlack: (128, 128, 128),
    LightBlue: (32, 32, 200),
    LightGreen: (32, 200, 32),
    LightCyan: (32, 200, 200),
    LightRed: (200, 32, 32),
    LightPink: (200, 32, 200),
    LightYellow: (200, 200, 32),
    LightWhite: (240, 240, 240),
};

pub const HIGH_CONTRAST_COLOR_SCHEME: ColorScheme = color_scheme! {
    Default: (0, 0, 0),

    Black: (0, 0, 0),
    Blue: (128, 128, 128),
    Green: (60, 60, 60),
    Cyan: (80, 80, 80),
    Red: (160, 160, 160),
    Pink: (192, 192, 192),
    Yellow: (240, 240, 240),
    White: (240, 240, 240),
    LightBlack: (0, 0, 0),
    LightBlue: (128, 128, 128),
    LightGreen: (60, 60, 60),
    LightCyan: (80, 80, 80),
    LightRed: (160, 160, 160),
    LightPink: (192, 192, 192),
    LightYellow: (240, 240, 240),
    LightWhite: (240, 240, 240),
};

pub const COLOR_SCHEMES: [ColorScheme; 4] = [
    MUTED_COLOR_SCHEME,
    BLUE_TINTED_COLOR_SCHEME,
    DARK_COLOR_SCHEME,
    HIGH_CONTRAST_COLOR_SCHEME,
];

#[derive(Debug, Clone, Copy, Ord, PartialOrd, Eq, PartialEq, Hash)]
#[repr(u8)]
pub enum GraphicalCharacter {
    Empty = 0,
    FragileFloor = 1,
    Ice = 2,

    OneWayLeft = 3,
    OneWayUp = 4,
    OneWayRight = 5,
    OneWayDown = 6,

    Wall = 7,

    Key = 8,
    KeyInGoal = 9,
    KeyOnFragileFloor = 10,
    KeyOnIce = 11,
    LockedDoor = 12,

    Box = 13,
    BoxInGoal = 14,
    Goal = 15,

    Hole = 16,
    BoxInHole = 17,
}

impl GraphicalCharacter {
    pub fn id(self) -> u8 {
        self as u8
    }

    pub fn from_id(val: u8) -> Option<Self> {
        match val {
            0 => Some(GraphicalCharacter::Empty),
            1 => Some(GraphicalCharacter::FragileFloor),
            2 => Some(GraphicalCharacter::Ice),

            3 => Some(GraphicalCharacter::OneWayLeft),
            4 => Some(GraphicalCharacter::OneWayUp),
            5 => Some(GraphicalCharacter::OneWayRight),
            6 => Some(GraphicalCharacter::OneWayDown),

            7 => Some(GraphicalCharacter::Wall),

            8 => Some(GraphicalCharacter::Key),
            9 => Some(GraphicalCharacter::KeyInGoal),
            10 => Some(GraphicalCharacter::KeyOnFragileFloor),
            11 => Some(GraphicalCharacter::KeyOnIce),
            12 => Some(GraphicalCharacter::LockedDoor),

            13 => Some(GraphicalCharacter::Box),
            14 => Some(GraphicalCharacter::BoxInGoal),
            15 => Some(GraphicalCharacter::Goal),

            16 => Some(GraphicalCharacter::Hole),
            17 => Some(GraphicalCharacter::BoxInHole),

            _ => None,
        }
    }

    pub fn from_tile(val: Tile) -> Option<Self> {
        match val {
            Tile::Empty => Some(GraphicalCharacter::Empty),
            Tile::FragileFloor => Some(GraphicalCharacter::FragileFloor),
            Tile::Ice => Some(GraphicalCharacter::Ice),

            Tile::OneWayLeft => Some(GraphicalCharacter::OneWayLeft),
            Tile::OneWayUp => Some(GraphicalCharacter::OneWayUp),
            Tile::OneWayRight => Some(GraphicalCharacter::OneWayRight),
            Tile::OneWayDown => Some(GraphicalCharacter::OneWayDown),

            Tile::Wall => Some(GraphicalCharacter::Wall),

            Tile::Key => Some(GraphicalCharacter::Key),
            Tile::KeyInGoal => Some(GraphicalCharacter::KeyInGoal),
            Tile::KeyOnFragileFloor => Some(GraphicalCharacter::KeyOnFragileFloor),
            Tile::KeyOnIce => Some(GraphicalCharacter::KeyOnIce),
            Tile::LockedDoor => Some(GraphicalCharacter::LockedDoor),

            Tile::Box | Tile::BoxOnFragileFloor | Tile::BoxOnIce  => Some(GraphicalCharacter::Box),
            Tile::BoxInGoal  => Some(GraphicalCharacter::BoxInGoal),
            Tile::Goal => Some(GraphicalCharacter::Goal),

            Tile::Hole => Some(GraphicalCharacter::Hole),
            Tile::BoxInHole => Some(GraphicalCharacter::BoxInHole),

            //TODO player tiles
            Tile::Player | Tile::PlayerOnFragileFloor | Tile::PlayerOnIce => None,

            Tile::DecorationBlank => None,
        }
    }

    pub fn into_image(self, asset_server: &AssetServer) -> Handle<Image> {
        match self {
            GraphicalCharacter::Empty => asset_server.load("embedded://textures/tiles/empty.png"),
            GraphicalCharacter::FragileFloor => asset_server.load("embedded://textures/tiles/fragile_floor.png"),
            GraphicalCharacter::Ice => asset_server.load("embedded://textures/tiles/ice.png"),

            GraphicalCharacter::OneWayLeft => asset_server.load("embedded://textures/tiles/one_way_left.png"),
            GraphicalCharacter::OneWayUp => asset_server.load("embedded://textures/tiles/one_way_up.png"),
            GraphicalCharacter::OneWayRight => asset_server.load("embedded://textures/tiles/one_way_right.png"),
            GraphicalCharacter::OneWayDown => asset_server.load("embedded://textures/tiles/one_way_down.png"),

            GraphicalCharacter::Wall => asset_server.load("embedded://textures/tiles/wall.png"),

            GraphicalCharacter::Key => asset_server.load("embedded://textures/tiles/key.png"),
            GraphicalCharacter::KeyInGoal => asset_server.load("embedded://textures/tiles/key_in_goal.png"),
            GraphicalCharacter::KeyOnFragileFloor => asset_server.load("embedded://textures/tiles/key_on_fragile_floor.png"),
            GraphicalCharacter::KeyOnIce => asset_server.load("embedded://textures/tiles/key_on_ice.png"),

            GraphicalCharacter::LockedDoor => asset_server.load("embedded://textures/tiles/locked_door.png"),

            GraphicalCharacter::Box => asset_server.load("embedded://textures/tiles/box.png"),
            GraphicalCharacter::BoxInGoal => asset_server.load("embedded://textures/tiles/box_in_goal.png"),
            GraphicalCharacter::Goal => asset_server.load("embedded://textures/tiles/goal.png"),

            GraphicalCharacter::Hole => asset_server.load("embedded://textures/tiles/hole.png"),
            GraphicalCharacter::BoxInHole => asset_server.load("embedded://textures/tiles/box_in_hole.png"),
        }
    }
}

#[derive(Debug, Clone, Copy, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct ConsoleCharacter {
    data: u8,
}

impl ConsoleCharacter {
    ///Returns Ok(u8) for simple ASCII char and Err(GraphicalCharacter) for graphical char
    pub fn get(self) -> Result<u8, GraphicalCharacter> {
        if self.data & 128 == 0 {
            Ok(self.data)
        }else {
            GraphicalCharacter::from_id(self.data - 128).map(Err).unwrap_or_else(|| Ok(b'?'))
        }
    }
}

impl From<u8> for ConsoleCharacter {
    fn from(value: u8) -> Self {
        ConsoleCharacter { data: value }
    }
}

impl From<GraphicalCharacter> for ConsoleCharacter {
    fn from(value: GraphicalCharacter) -> Self {
        ConsoleCharacter { data: 128 + value.id() }
    }
}

#[derive(Clone)]
pub struct ConsoleDrawBuffer {
    text_buffer: Box<[ConsoleCharacter]>,
    text_color_buffer: Box<[(Color, Color)]>,
    //TODO: underline
}

impl ConsoleDrawBuffer {
    pub fn new<const W: usize, const H: usize>() -> Self {
        Self {
            text_buffer: vec![ConsoleCharacter { data: b' ' }; W * H].into_boxed_slice(),
            text_color_buffer: vec![(Color::White, Color::Black); W * H].into_boxed_slice(),
            //TODO: underline
        }
    }

    pub fn text_buffer(&self) -> &[ConsoleCharacter] {
        &self.text_buffer
    }

    pub fn text_color_buffer(&self) -> &[(Color, Color)] {
        &self.text_color_buffer
    }
}

#[derive(Debug, Clone, Copy, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub enum ConsoleBufferSelection {
    Primary,
    Secondary,
}

impl ConsoleBufferSelection {
    pub fn swap(self) -> Self {
        match self {
            ConsoleBufferSelection::Primary => ConsoleBufferSelection::Secondary,
            ConsoleBufferSelection::Secondary => ConsoleBufferSelection::Primary,
        }
    }
}

pub struct ConsoleState {
    tile_mode: TileMode,

    curser_pos: (usize, usize),
    current_color_pair: (Color, Color),

    buffer_selection: ConsoleBufferSelection,
    primary_buffer: ConsoleDrawBuffer,
    secondary_buffer: ConsoleDrawBuffer,

    input_queue_keyboard: VecDeque<Key>,
    input_queue_mouse: VecDeque<(usize, usize)>,
}

impl ConsoleState {
    pub fn new<const W: usize, const H: usize>() -> Self {
        Self {
            tile_mode: TileMode::default(),

            curser_pos: (0, 0),
            current_color_pair: (Color::White, Color::Black),

            buffer_selection: ConsoleBufferSelection::Primary,
            primary_buffer: ConsoleDrawBuffer::new::<W, H>(),
            secondary_buffer: ConsoleDrawBuffer::new::<W, H>(),

            input_queue_keyboard: VecDeque::default(),
            input_queue_mouse: VecDeque::default(),
        }
    }

    pub fn tile_mode(&self) -> TileMode {
        self.tile_mode
    }

    pub fn set_tile_mode(&mut self, tile_mode: TileMode) {
        self.tile_mode = tile_mode;
    }

    pub fn clear_screen(&mut self) {
        self.curser_pos = (0, 0);

        self.current_buffer_mut().text_buffer.fill(ConsoleCharacter { data: b' ' });
        self.current_buffer_mut().text_color_buffer.fill((Color::White, Color::Black));
        //TODO: underline
    }

    pub fn primary_buffer(&self) -> &ConsoleDrawBuffer {
        &self.primary_buffer
    }

    pub fn secondary_buffer(&self) -> &ConsoleDrawBuffer {
        &self.secondary_buffer
    }

    pub fn current_buffer(&self) -> &ConsoleDrawBuffer {
        match self.buffer_selection {
            ConsoleBufferSelection::Primary => &self.primary_buffer,
            ConsoleBufferSelection::Secondary => &self.secondary_buffer,
        }
    }

    fn current_buffer_mut(&mut self) -> &mut ConsoleDrawBuffer {
        match self.buffer_selection {
            ConsoleBufferSelection::Primary => &mut self.primary_buffer,
            ConsoleBufferSelection::Secondary => &mut self.secondary_buffer,
        }
    }

    pub fn swap_buffer_selection(&mut self) {
        self.buffer_selection = self.buffer_selection.swap();
    }

    pub fn swap_buffer(&mut self) {
        mem::swap(&mut self.secondary_buffer, &mut self.primary_buffer);
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

                state.current_buffer_mut().text_buffer[start_index..start_index + len].
                        //SAFETY: &[u8] and &[ConsoleCharacter] have the same byte representation and all u8s are valid ConsoleCharacters
                        copy_from_slice(unsafe { mem::transmute::<&[u8], &[ConsoleCharacter]>(line) } );
                state.current_buffer_mut().text_color_buffer[start_index..start_index + len].fill(color);
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

    pub fn draw_tile_internal(&self, tile: Tile, is_player_background: bool, inverted: bool) {
        let tile_mode = self.state.lock().unwrap().tile_mode;
        if tile_mode == TileMode::Graphical &&
                let Some(graphical_character) = GraphicalCharacter::from_tile(tile) {
            self.draw_graphical_character(
                graphical_character,
                if is_player_background { Color::Yellow } else { Color::Default },
                if inverted { Color::Black } else { Color::Default },
            );
        }else {
            tile.draw_raw(self, is_player_background, inverted);
        }
    }

    pub fn draw_graphical_character(&self, graphical_tile: GraphicalCharacter, fg: Color, bg: Color) {
        let mut state = self.state.lock().unwrap();

        let (width, _) = self.get_console_size();

        let index = state.curser_pos.0 + width * state.curser_pos.1;

        state.current_buffer_mut().text_buffer[index] = ConsoleCharacter::from(graphical_tile);
        state.current_buffer_mut().text_color_buffer[index] = (fg, bg);

        state.curser_pos.0 += 1;
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

impl Color {
    pub fn into_bevy_color(self, color_scheme: &ColorScheme) -> bevy::color::Color {
        color_scheme.convert_console_color_to_bevy_color(self)
    }
}

impl From<Color> for bevy::color::Color {
    fn from(value: Color) -> Self {
        MUTED_COLOR_SCHEME.convert_console_color_to_bevy_color(value)
    }
}
