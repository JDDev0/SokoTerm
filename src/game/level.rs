use crate::game::{audio, Game, GameError};
use std::error::Error;
use std::ffi::OsString;
use std::fmt::{Debug, Display, Formatter, Write as _};
use std::fs::File;
use std::io::Write;
use std::str::FromStr;
use crate::collections::UndoHistory;
use crate::game::audio::BackgroundMusicId;
use crate::game::console_extension::ConsoleExtension;
use crate::io::{Color, Console};

#[cfg(feature = "steam")]
use bevy_steamworks::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tile {
    Empty,
    FragileFloor,
    Ice,

    OneWayLeft,
    OneWayUp,
    OneWayRight,
    OneWayDown,

    Wall,

    Player,
    PlayerOnFragileFloor,
    PlayerOnIce,

    Key,
    KeyInGoal,
    KeyOnFragileFloor,
    KeyOnIce,
    LockedDoor,

    Box,
    BoxInGoal,
    BoxOnFragileFloor,
    BoxOnIce,
    Goal,

    Hole,
    BoxInHole,

    DecorationBlank,

    Secret,
}

impl Tile {
    pub fn floor_tile(self) -> Self {
        match self {
            Tile::Empty => Tile::Empty,
            Tile::FragileFloor => Tile::FragileFloor,
            Tile::Ice => Tile::Ice,

            Tile::OneWayLeft => Tile::OneWayLeft,
            Tile::OneWayUp => Tile::OneWayUp,
            Tile::OneWayRight => Tile::OneWayRight,
            Tile::OneWayDown => Tile::OneWayDown,

            Tile::Wall => Tile::Wall,

            Tile::Player => Tile::Player,
            Tile::PlayerOnFragileFloor => Tile::FragileFloor,
            Tile::PlayerOnIce => Tile::Ice,

            Tile::Key => Tile::Key,
            Tile::KeyInGoal => Tile::Goal,
            Tile::KeyOnFragileFloor => Tile::FragileFloor,
            Tile::KeyOnIce => Tile::Ice,
            Tile::LockedDoor => Tile::LockedDoor,

            Tile::Box => Tile::Box,
            Tile::BoxInGoal => Tile::Goal,
            Tile::BoxOnFragileFloor => Tile::FragileFloor,
            Tile::BoxOnIce => Tile::Ice,
            Tile::Goal => Tile::Goal,

            Tile::Hole => Tile::Hole,
            Tile::BoxInHole => Tile::BoxInHole,

            Tile::DecorationBlank => Tile::DecorationBlank,

            Tile::Secret => Tile::Secret,
        }
    }

    pub fn from_ascii(a: u8) -> Result<Self, LevelLoadingError> {
        match a {
            b'-' => Ok(Tile::Empty),
            //Different ASCII char than display for compatibility with old level packs
            b':' => Ok(Tile::FragileFloor),
            b'%' => Ok(Tile::Ice),

            b'<' => Ok(Tile::OneWayLeft),
            b'^' => Ok(Tile::OneWayUp),
            b'>' => Ok(Tile::OneWayRight),
            b'v' => Ok(Tile::OneWayDown),

            b'#' => Ok(Tile::Wall),

            b'p' | b'P' => Ok(Tile::Player),
            b',' => Ok(Tile::PlayerOnFragileFloor),
            b'&' => Ok(Tile::PlayerOnIce),

            b'*' => Ok(Tile::Key),
            b'~' => Ok(Tile::KeyInGoal),
            b';' => Ok(Tile::KeyOnFragileFloor),
            b'\\' => Ok(Tile::KeyOnIce),
            b'=' => Ok(Tile::LockedDoor),

            b'@' => Ok(Tile::Box),
            b'+' => Ok(Tile::BoxInGoal),
            b'!' => Ok(Tile::BoxOnFragileFloor),
            b'/' => Ok(Tile::BoxOnIce),
            b'x' | b'X' => Ok(Tile::Goal),

            b'o' | b'O' => Ok(Tile::Hole),
            b'.' => Ok(Tile::BoxInHole),

            b'b' | b'B' => Ok(Tile::DecorationBlank),

            b's' | b'S' => Ok(Tile::Secret),

            _ => Err(LevelLoadingError::new("Invalid tile")),
        }
    }

    pub fn to_ascii(self) -> u8 {
        match self {
            Tile::Empty => b'-',
            //Different ASCII char than display for compatibility with old level packs
            Tile::FragileFloor => b':',
            Tile::Ice => b'%',

            Tile::OneWayLeft => b'<',
            Tile::OneWayUp => b'^',
            Tile::OneWayRight => b'>',
            Tile::OneWayDown => b'v',

            Tile::Wall => b'#',

            Tile::Player => b'P',
            Tile::PlayerOnFragileFloor => b',',
            Tile::PlayerOnIce => b'&',

            Tile::Key => b'*',
            Tile::KeyInGoal => b'~',
            Tile::KeyOnFragileFloor => b';',
            Tile::KeyOnIce => b'\\',
            Tile::LockedDoor => b'=',

            Tile::Box => b'@',
            Tile::BoxInGoal => b'+',
            Tile::BoxOnFragileFloor => b'!',
            Tile::BoxOnIce => b'/',
            Tile::Goal => b'x',

            Tile::Hole => b'o',
            Tile::BoxInHole => b'.',

            Tile::DecorationBlank => b'b',

            Tile::Secret => b's',
        }
    }

    pub fn draw(self, console: &Console, is_player_background: bool, inverted: bool) {
        console.draw_tile(self, is_player_background, inverted);
    }

    pub fn draw_raw(self, console: &Console, is_player_background: bool, inverted: bool) {
        match self {
            Tile::Empty => {
                console.set_color_invertible(Color::LightBlue, Color::Default, inverted);
                console.draw_text("-");
            },
            Tile::FragileFloor => {
                console.set_color_invertible(Color::LightBlue, Color::Default, inverted);
                console.draw_text("~");
            },
            Tile::Ice => {
                console.set_color_invertible(Color::LightBlue, Color::Default, inverted);
                console.draw_text("%");
            },
            Tile::OneWayLeft => {
                console.set_color_invertible(Color::LightBlue, Color::Default, inverted);
                console.draw_text("<");
            },
            Tile::OneWayUp => {
                console.set_color_invertible(Color::LightBlue, Color::Default, inverted);
                console.draw_text("^");
            },
            Tile::OneWayRight => {
                console.set_color_invertible(Color::LightBlue, Color::Default, inverted);
                console.draw_text(">");
            },
            Tile::OneWayDown => {
                console.set_color_invertible(Color::LightBlue, Color::Default, inverted);
                console.draw_text("v");
            },
            Tile::Wall => {
                console.set_color_invertible(Color::LightGreen, Color::Default, inverted);
                console.draw_text("#");
            },
            Tile::Player | Tile::PlayerOnFragileFloor | Tile::PlayerOnIce => {
                if is_player_background {
                    console.set_color_invertible(Color::Default, Color::Yellow, inverted);
                }else {
                    console.set_color_invertible(Color::Yellow, Color::Default, inverted);
                }
                console.draw_text("P");
            },
            Tile::Key | Tile::KeyOnFragileFloor | Tile::KeyOnIce => {
                console.set_color_invertible(Color::LightCyan, Color::Default, inverted);
                console.draw_text("*");
            },
            Tile::KeyInGoal => {
                console.set_color_invertible(Color::LightPink, Color::Default, inverted);
                console.draw_text("*");
            },
            Tile::LockedDoor => {
                console.set_color_invertible(Color::LightRed, Color::Default, inverted);
                console.draw_text("=");
            },
            Tile::Box | Tile::BoxOnFragileFloor | Tile::BoxOnIce => {
                console.set_color_invertible(Color::LightCyan, Color::Default, inverted);
                console.draw_text("@");
            },
            Tile::BoxInGoal => {
                console.set_color_invertible(Color::LightPink, Color::Default, inverted);
                console.draw_text("@");
            },
            Tile::Goal => {
                console.set_color_invertible(Color::LightRed, Color::Default, inverted);
                console.draw_text("x");
            },
            Tile::Hole => {
                console.set_color_invertible(Color::LightBlue, Color::Default, inverted);
                console.draw_text("O");
            },
            Tile::BoxInHole => {
                console.set_color_invertible(Color::Default, Color::LightBlue, inverted);
                console.draw_text("@");
            },
            Tile::DecorationBlank => {
                console.set_color_invertible(Color::LightBlue, Color::Default, inverted);
                console.draw_text(" ");
            },
            Tile::Secret => {
                console.set_color_invertible(Color::LightBlue, Color::Default, inverted);
                console.draw_text("+");
            },
        };
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Left,
    Up,
    Right,
    Down,
}

impl Direction {
    pub fn update_x(self, x: usize, width: usize) -> usize {
        match self {
            Direction::Left => if x == 0 {
                width - 1
            }else {
                x - 1
            },
            Direction::Right => if x == width - 1 {
                0
            }else {
                x + 1
            },
            _ => x,
        }
    }

    pub fn update_y(self, y: usize, height: usize) -> usize {
        match self {
            Direction::Up => if y == 0 {
                height - 1
            }else {
                y - 1
            },
            Direction::Down => if y == height - 1 {
                0
            }else {
                y + 1
            },
            _ => y,
        }
    }

    pub fn update_xy(self, x: usize, y: usize, width: usize, height: usize) -> (usize, usize) {
        (self.update_x(x, width), self.update_y(y, height))
    }
}

#[derive(Debug, Clone)]
enum AnimationState {
    Player {
        last_valid_move_result: MoveResult,
        direction: Direction,
    },
    BoxOrKey {
        last_valid_move_result: MoveResult,
        x_from: usize,
        y_from: usize,
        direction: Direction,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LevelSoundEffect {
    BoxFall,
    KeyFall,
    DoorUnlocked,
    FloorBroken,
}

#[derive(Debug, Clone)]
pub enum MoveResult {
    Valid {
        has_won: bool,
        secret_found: bool,
        sound_effect: Option<LevelSoundEffect>,
    },
    Invalid,
    Animation {
        player_animation: bool,
        sound_effect: Option<LevelSoundEffect>,
    },
}

impl MoveResult {
    pub fn is_valid(&self) -> bool {
        matches!(self, MoveResult::Valid { .. })
    }

    pub fn has_won(&self) -> bool {
        matches!(self, MoveResult::Valid {has_won: true, ..})
    }

    pub fn secret_found(&self) -> bool {
        matches!(self, MoveResult::Valid {secret_found: true, ..})
    }

    pub fn is_invalid(&self) -> bool {
        matches!(self, MoveResult::Invalid)
    }

    pub fn is_animation(&self) -> bool {
        matches!(self, MoveResult::Animation {..})
    }
}

#[derive(Debug, Clone)]
pub struct Level {
    width: usize,
    height: usize,
    tiles: Vec<Tile>,
}

impl Level {
    pub fn new(width: usize, height: usize) -> Self {
        if width == 0 {
            panic!("Width must be > 0!");
        }

        if height == 0 {
            panic!("Height must be > 0!");
        }

        let tiles = vec![Tile::Empty; width * height];

        Level { width, height, tiles }
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn height(&self) -> usize {
        self.height
    }

    pub fn tiles(&self) -> &[Tile] {
        &self.tiles
    }

    pub fn get_tile(&self, x: usize, y: usize) -> Option<Tile> {
        self.tiles.get(x + y * self.width).copied()
    }

    pub fn get_tile_mut(&mut self, x: usize, y: usize) -> Option<&mut Tile> {
        self.tiles.get_mut(x + y * self.width)
    }

    pub fn set_tile(&mut self, x: usize, y: usize, tile: Tile) {
        self.tiles[x + y * self.width] = tile;
    }

    pub fn draw(&self, console: &Console, x_offset: usize, y_offset: usize, is_player_background: bool, cursor_pos: Option<(usize, usize)>) {
        let mut tile_iter = self.tiles.iter();

        for i in 0..self.height {
            console.set_cursor_pos(x_offset, i + y_offset);

            for j in 0..self.width {
                if let Some(tile) = tile_iter.next() {
                    tile.draw(console, is_player_background, cursor_pos.is_some_and(|(x, y)| x == j && y == i));
                }
            }

            console.draw_text("\n");
        }
    }

    pub fn draw_floor(&self, console: &Console, x_offset: usize, y_offset: usize, is_player_background: bool, original_level: &Level, cursor_pos: Option<(usize, usize)>) {
        let mut tile_iter = self.tiles.iter().copied();

        for i in 0..self.height {
            console.set_cursor_pos(x_offset, i + y_offset);

            for j in 0..self.width {
                if let Some(tile) = tile_iter.next() {
                    let tile = match tile.floor_tile() {
                        Tile::Player => match original_level.get_tile(j, i) {
                            Some(Tile::KeyOnIce | Tile::BoxOnIce | Tile::Ice | Tile::PlayerOnIce) => Tile::Ice,

                            Some(Tile::OneWayLeft) => Tile::OneWayLeft,
                            Some(Tile::OneWayUp) => Tile::OneWayUp,
                            Some(Tile::OneWayRight) => Tile::OneWayRight,
                            Some(Tile::OneWayDown) => Tile::OneWayDown,

                            Some(Tile::KeyInGoal | Tile::BoxInGoal | Tile::Goal) => Tile::Goal,

                            Some(
                                Tile::Hole | Tile::BoxInHole |
                                Tile::KeyOnFragileFloor | Tile::BoxOnFragileFloor
                            ) => Tile::BoxInHole,

                            _ => Tile::Empty,
                        },

                        Tile::Box | Tile::Key => match original_level.get_tile(j, i) {
                            Some(Tile::Hole | Tile::BoxInHole) => Tile::BoxInHole,

                            _ => Tile::Empty,
                        },

                        tile => tile,
                    };

                    tile.draw(console, is_player_background, cursor_pos.is_some_and(|(x, y)| x == j && y == i));
                }
            }

            console.draw_text("\n");
        }
    }

    pub fn draw_ascii_art_background(
        console: &Console,

        (screen_x, screen_y, screen_width, screen_height): (usize, usize, usize, usize),

        background_music_id: BackgroundMusicId,
        is_player_background: bool,
    ) {
        if background_music_id == audio::BACKGROUND_MUSIC_LONELY_NIGHT.id() {
            //Moon
            let moon_str = r#"
'::.
 `::.
  :::
 .::'
.::'
            "#[1..].trim_end(); //Remove leading newline and trailing spaces

            let start_x = screen_x + screen_width - 5;
            let start_y = screen_y + 1;

            for (y, line) in moon_str.split("\n").enumerate() {
                for (x, c) in line.bytes().enumerate() {
                    if c == b' ' {
                        continue;
                    }

                    console.set_cursor_pos(start_x + x, start_y + y);
                    console.set_color(Color::LightYellow, Color::Default);

                    console.draw_text(c as char);
                }
            }

            //Small stars (yellow, dark)
            console.set_color(Color::Yellow, Color::Default);
            for (x, y) in [
                (0, 5),
                (5, 16),
                (15, 18),
                (25, 6),
                (32, 8),
                (34, 3),
                (36, 0),
                (37, 18),
                (45, 9),
                (49, 0),
                (50, 20),
                (54, 13),
                (56, 9),
                (60, 2),
                (62, 19),
                (63, 10),
                (64, 13),
                (65, 4),
                (72, 18),
            ] {
                console.set_cursor_pos(screen_x + x, screen_y + y);
                console.draw_text("*");
            }

            //Small stars (yellow, bright)
            console.set_color(Color::LightYellow, Color::Default);
            for (x, y) in [
                (0, 14),
                (1, 18),
                (7, 4),
                (10, 9),
                (12, 6),
                (16, 20),
                (19, 12),
                (22, 7),
                (24, 4),
                (29, 17),
                (30, 2),
                (35, 6),
                (50, 7),
                (51, 15),
                (53, 18),
                (55, 3),
                (59, 8),
                (64, 17),
                (67, 15),
                (70, 12),
                (73, 20),
            ] {
                console.set_cursor_pos(screen_x + x, screen_y + y);
                console.draw_text("*");
            }

            //Small stars (yellow, blinking)
            console.set_color(if is_player_background { Color::Yellow } else { Color::LightYellow }, Color::Default);
            for (x, y) in [
                (0, 21),
                (2, 2),
                (4, 7),
                (7, 18),
                (9, 14),
                (14, 10),
                (15, 3),
                (19, 9),
                (22, 16),
                (25, 0),
                (24, 10),
                (30, 21),
                (42, 5),
                (45, 3),
                (46, 16),
                (48, 13),
                (55, 5),
                (58, 11),
                (60, 21),
                (64, 0),
                (65, 9),
                (67, 3),
                (68, 19),
                (71, 14),
            ] {
                console.set_cursor_pos(screen_x + x, screen_y + y);
                console.draw_text("*");
            }

            //Small stars (blue, dark)
            console.set_color(Color::Blue, Color::Default);
            for (x, y) in [
                (4, 20),
                (20, 2),
                (57, 16),
            ] {
                console.set_cursor_pos(screen_x + x, screen_y + y);
                console.draw_text("*");
            }

            //Small stars (blue, bright)
            console.set_color(Color::LightBlue, Color::Default);
            for (x, y) in [
                (17, 15),
                (16, 5),
                (44, 19),
                (64, 6),
            ] {
                console.set_cursor_pos(screen_x + x, screen_y + y);
                console.draw_text("*");
            }

            //Small stars (blue, blinking)
            console.set_color(if is_player_background { Color::Blue } else { Color::LightBlue }, Color::Default);
            for (x, y) in [
                (0, 11),
                (40, 1),
                (52, 10),
            ] {
                console.set_cursor_pos(screen_x + x, screen_y + y);
                console.draw_text("*");
            }

            //Small stars (red, dark)
            console.set_color(Color::Red, Color::Default);
            for (x, y) in [
                (10, 1),
                (12, 21),
                (39, 13),
            ] {
                console.set_cursor_pos(screen_x + x, screen_y + y);
                console.draw_text("*");
            }

            //Small stars (red, bright)
            console.set_color(Color::LightRed, Color::Default);
            for (x, y) in [
                (24, 19),
                (61, 15),
            ] {
                console.set_cursor_pos(screen_x + x, screen_y + y);
                console.draw_text("*");
            }

            //Small stars (red, blinking)
            console.set_color(if is_player_background { Color::Red } else { Color::LightRed }, Color::Default);
            for (x, y) in [
                (8, 12),
                (33, 14),
                (70, 7),
            ] {
                console.set_cursor_pos(screen_x + x, screen_y + y);
                console.draw_text("*");
            }
        }else if background_music_id == audio::BACKGROUND_MUSIC_CATCHY.id() {
            //Grass
            console.set_cursor_pos(screen_x, screen_y + screen_height - 1);
            for x in screen_x..screen_x + screen_width {
                if (((x % 7) % 3) + x) % 5 < 3 {
                    console.set_color(Color::Green, Color::Default);
                }else {
                    console.set_color(Color::LightGreen, Color::Default);
                }

                console.draw_text("_");
            }

            //Trees
            let tree_1_str = r#"
     @%&%
   ###&%&&%`     #%@
 &%%.:.&%%&#&` &#&%##@
&&#'#@':.@&%%#@&%%##@&&#
%#:%#@&#:~~._##@.-~-@&&#
 @@#%%:'&#%'~\./*#&#&`
  ##@/%&#%&%#'\.#`@&&
   %%'@@@&&`#.||@&##&%%
    #%%`    `||'  ####&&
            .|| #@_=~~~_&
            ||/--'&&#@@``
           ~||' #&@``
           |||
           |||
        _./|||\._
            "#[1..].trim_end(); //Remove leading newline and trailing spaces

            let tree_2_str = r#"
             ##&&&`
          ##&&&&@#@&
     &@%&%##%.~##@#%%%`
   &%%@#%#@./%#@@#.:.@@#`
  %##&%%@@/&%@@#&.:'&%'%##
  %##&-~-:&%%_.~~:%#&%@:%@
    `#%#%*\./~'@%#':@@%&&
   %%##&`%./'%@#@%#@\&%%
 %%%&&%%#&||.%`##&&&'@@
&_=~-=_&%'||.%#`%`@@%
``&&%##'--/||###@%%
    ``&#% '||\=~-.%%#
           ||:/%##'.#%#
           |||%##@@@%#`
           |||     #%`
        _./|||\._
            "#[1..].trim_end(); //Remove leading newline and trailing spaces

            for (i, tree) in [tree_1_str, tree_2_str].into_iter().enumerate() {
                let start_x = if i == 0 { screen_x } else { screen_x + screen_width - 25 };
                let start_y = screen_y + screen_height - 15 - i;

                for (y, line) in tree.split("\n").enumerate() {
                    for (x, c) in line.bytes().enumerate() {
                        if c == b' ' {
                            continue;
                        }

                        console.set_cursor_pos(start_x + x, start_y + y);

                        let is_stem = matches!(c, b':' | b'.' | b'\'' | b'~' | b'-' | b'_' | b'=' | b'/' | b'|' | b'\\');

                        if (((y + i) % 3) + x + 2 * i) % 5 < 3 {
                            console.set_color(if is_stem { Color::Yellow } else { Color::Pink }, Color::Default);
                        }else {
                            console.set_color(if is_stem { Color::LightYellow } else { Color::LightPink }, Color::Default);
                        }

                        console.draw_text(c as char);
                    }
                }
            }
        }
    }

    pub fn draw_level_ascii_art_background(
        &self,

        console: &Console,

        (screen_x, screen_y, screen_width, screen_height): (usize, usize, usize, usize),
        (level_x_start, level_y_start): (usize, usize),

        background_music_id: BackgroundMusicId,
        is_player_background: bool,
    ) {
        Self::draw_ascii_art_background(
            console,

            (screen_x, screen_y, screen_width, screen_height),

            background_music_id,
            is_player_background,
        );

        console.set_color(Color::Default, Color::Default);
        for y in level_y_start.saturating_sub(1)..level_y_start + self.height + 1 {
            if y < screen_y {
                continue;
            }

            if y == level_y_start.saturating_sub(1) || y == level_y_start + self.height {
                for x in level_x_start.saturating_sub(1)..level_x_start + self.width + 1 {
                    if x < screen_x {
                        continue;
                    }

                    console.set_cursor_pos(x, y);
                    console.draw_text(" ");
                }
            }else {
                for x in [level_x_start.saturating_sub(1), level_x_start + self.width] {
                    if x < screen_x {
                        continue;
                    }

                    console.set_cursor_pos(x, y);
                    console.draw_text(" ");
                }
            }
        }
    }

    pub fn to_str(&self) -> String {
        let mut out = String::with_capacity(14 + self.width * self.height);

        let _ = writeln!(out, "w: {}, h: {}", self.width, self.height);
        for row in self.tiles.chunks(self.width) {
            row.iter().map(|tile| (tile.to_ascii() as char).to_string()).for_each(|tile| out += &tile);
            out += "\n";
        }

        out
    }
}

impl FromStr for Level {
    type Err = LevelLoadingError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let lines = s.lines().collect::<Vec<_>>();
        if lines.is_empty() {
            return Err(LevelLoadingError::new("Level is invalid!"));
        }

        let line = lines.first().unwrap().trim();
        if !line.starts_with("w: ") || !line.contains(", h: ") {
            return Err(LevelLoadingError::new("Level is invalid!"));
        }

        let index = line.to_string().find(", h: ").unwrap();

        let (width, height) = (&line[3..index], &line[index + 5..]);
        let height = if let Ok(height) = usize::from_str(height) {
            height
        }else {
            return Err(LevelLoadingError::new("Level is invalid!"));
        };
        let width = if let Ok(width) = usize::from_str(width) {
            width
        }else {
            return Err(LevelLoadingError::new("Level is invalid!"));
        };

        if width == 0 || height == 0 {
            return Err(LevelLoadingError::new("Level is invalid!"));
        }

        let mut tiles = Vec::with_capacity(width * height);

        for line in lines.into_iter().
                skip(1).
                map(|line| line.trim()) {
            if line.len() != width {
                return Err(LevelLoadingError::new("Level is invalid!"));
            }

            for tile in line.bytes() {
                tiles.push(Tile::from_ascii(tile)?);
            }
        }

        if tiles.len() != width * height {
            return Err(LevelLoadingError::new("Level is invalid!"));
        }

        Ok(Self { width, height, tiles })
    }
}

#[derive(Debug)]
pub struct PlayingLevel {
    original_level: Level,
    animation_state: Option<AnimationState>,
    playing_level: UndoHistory<(Level, (usize, usize))>,
}

impl PlayingLevel {
    pub fn new(level: &Level, history_size: usize) -> Result<Self, LevelLoadingError> {
        let player_tile_count = level.tiles().iter().filter(|tile| matches!(tile, Tile::Player | Tile::PlayerOnFragileFloor | Tile::PlayerOnIce)).count();
        if player_tile_count == 0 {
            return Err(LevelLoadingError::new("Level does not contain a player tile!"));
        }else if player_tile_count > 1 {
            return Err(LevelLoadingError::new("Level contains too many player tiles!"));
        }

        let mut player_pos = None;

        'outer:
        for i in 0..level.width() {
            for j in 0..level.height() {
                if let Some(tile) = level.get_tile(i, j) && matches!(tile, Tile::Player | Tile::PlayerOnFragileFloor | Tile::PlayerOnIce) {
                    player_pos = Some((i, j));

                    break 'outer;
                }
            }
        }

        Ok(PlayingLevel {
            original_level: level.clone(),
            animation_state: None,
            playing_level: UndoHistory::new(history_size, (level.clone(), player_pos.unwrap())),
        })
    }

    pub fn is_playing_animation(&self) -> bool {
        self.animation_state.is_some()
    }

    #[must_use]
    pub fn continue_animation(&mut self) -> MoveResult {
        let Some(animation_state) = self.animation_state.clone() else {
            return MoveResult::Invalid;
        };

        let move_result = match animation_state {
            AnimationState::Player {
                last_valid_move_result, direction,
            } => {
                let move_result = self.move_player_internal(direction);
                if move_result.is_invalid() {
                    //Animation finished
                    self.animation_state = None;

                    //No changes happened to level -> return result directly
                    return last_valid_move_result;
                }

                if move_result.is_valid() {
                    //Animation finished
                    self.animation_state = None;
                }

                move_result
            },

            AnimationState::BoxOrKey {
                last_valid_move_result,
                x_from, y_from,
                direction,
            } => {
                let (mut level, player_pos) = self.playing_level.current().clone();

                let move_result = self.move_box_or_key(&mut level, x_from, y_from, direction);
                if move_result.is_invalid() {
                    //Animation finished
                    self.animation_state = None;

                    //No changes happened to level -> return result directly
                    return last_valid_move_result;
                }

                if move_result.is_valid() {
                    //Animation finished
                    self.animation_state = None;
                }

                self.playing_level.commit_change((level, player_pos));

                move_result
            },
        };

        //Undo temporary change from last animation iteration
        let current_playing_level = self.playing_level.current().clone();
        self.playing_level.undo();
        self.playing_level.undo();
        self.playing_level.commit_change(current_playing_level);

        move_result
    }

    pub fn cancel_animation_and_undo_move(&mut self) -> Option<&(Level, (usize, usize))> {
        if !self.is_playing_animation() {
            return None;
        }

        self.animation_state = None;

        //Undo temporary change from last animation iteration
        self.playing_level.undo();

        //Prevent redo into animation frame by commiting change after undo
        let current_playing_level = self.playing_level.current().clone();
        self.playing_level.undo();
        self.playing_level.commit_change(current_playing_level);

        Some(self.playing_level.current())
    }

    #[must_use]
    pub fn move_player(&mut self, direction: Direction) -> MoveResult {
        if self.is_playing_animation() {
            return MoveResult::Invalid;
        }

        self.move_player_internal(direction)
    }

    #[must_use]
    fn move_player_internal(&mut self, direction: Direction) -> MoveResult {
        let (mut level, mut player_pos) = self.playing_level.current().clone();

        let (x_from, y_from) = player_pos;
        let (x_to, y_to) = direction.update_xy(x_from, y_from, level.width, level.height);

        let one_way_door_tile = match direction {
            Direction::Left => Tile::OneWayLeft,
            Direction::Up => Tile::OneWayUp,
            Direction::Right => Tile::OneWayRight,
            Direction::Down => Tile::OneWayDown,
        };

        //Set players old position to old level data
        let mut tile = self.original_level.get_tile(x_from, y_from).unwrap();
        let player_tile = level.get_tile(x_from, y_from).unwrap();
        if matches!(tile, Tile::Player | Tile::Box | Tile::Key | Tile::LockedDoor) {
            tile = Tile::Empty;
        }else if matches!(tile, Tile::BoxInGoal | Tile::KeyInGoal) {
            tile = Tile::Goal;
        }else if matches!(tile, Tile::Hole | Tile::BoxInHole) {
            tile = Tile::BoxInHole;
        }else if matches!(tile, Tile::FragileFloor | Tile::PlayerOnFragileFloor | Tile::BoxOnFragileFloor | Tile::KeyOnFragileFloor) {
            tile = if player_tile == Tile::PlayerOnFragileFloor {
                Tile::Hole //First time player is on tile -> Replace with Hole
            }else {
                Tile::BoxInHole //Hole from Fragile Floor usage must already have been filled with box
            };
        }else if matches!(tile, Tile::Ice | Tile::PlayerOnIce | Tile::BoxOnIce | Tile::KeyOnIce) {
            tile = Tile::Ice;
        }

        level.set_tile(x_from, y_from, tile);

        let was_floor_broken = tile == Tile::Hole;

        let tile = level.get_tile(x_to, y_to).unwrap();
        let move_result = if matches!(tile, Tile::Empty | Tile::FragileFloor | Tile::Ice | Tile::Goal | Tile::Secret | Tile::BoxInHole) || tile == one_way_door_tile {
            MoveResult::Valid { has_won: false, secret_found: tile == Tile::Secret, sound_effect: was_floor_broken.then_some(LevelSoundEffect::FloorBroken) }
        }else if matches!(tile, Tile::Box | Tile::BoxInGoal | Tile::BoxOnFragileFloor | Tile::BoxOnIce | Tile::Key | Tile::KeyInGoal | Tile::KeyOnFragileFloor | Tile::KeyOnIce) {
            let move_result = self.move_box_or_key(&mut level, x_to, y_to, direction);
            match move_result {
                MoveResult::Valid {
                    has_won, secret_found, sound_effect,
                } if was_floor_broken && sound_effect.is_none() => MoveResult::Valid {
                    has_won, secret_found, sound_effect: Some(LevelSoundEffect::FloorBroken),
                },

                _ => move_result,
            }
        }else {
            MoveResult::Invalid
        };

        if move_result.is_valid() || move_result.is_animation() {
            player_pos = (x_to, y_to);
        }

        //Set player to new position
        if matches!(level.get_tile(x_to, y_to).unwrap(), Tile::FragileFloor | Tile::PlayerOnFragileFloor | Tile::BoxOnFragileFloor | Tile::KeyOnFragileFloor) {
            level.set_tile(player_pos.0, player_pos.1, Tile::PlayerOnFragileFloor);
        }else {
            level.set_tile(player_pos.0, player_pos.1, Tile::Player);
        }

        if move_result.is_valid() || move_result.is_animation() {
            self.playing_level.commit_change((level, player_pos));

            //If ice tile: move forwards until no longer ice (Start animation)
            if tile == Tile::Ice {
                self.animation_state = Some(AnimationState::Player {
                    last_valid_move_result: move_result,
                    direction,
                });

                return MoveResult::Animation { player_animation: true, sound_effect: was_floor_broken.then_some(LevelSoundEffect::FloorBroken) };
            }
        }

        move_result
    }

    #[must_use]
    fn move_box_or_key(&mut self, level: &mut Level, x_from: usize, y_from: usize, direction: Direction) -> MoveResult {
        if level.width != self.original_level.width || level.height != self.original_level.height {
            panic!("Original level must have the same width and height as the modified level!");
        }

        let (x_to, y_to) = direction.update_xy(x_from, y_from, level.width, level.height);

        let index_from = x_from + y_from * level.width;
        let index_to = x_to + y_to * level.width;

        let Some(tile_from) = level.tiles.get(index_from) else {
            return MoveResult::Invalid;
        };
        let Some(tile_to) = level.tiles.get(index_to) else {
            return MoveResult::Invalid;
        };

        let is_box = matches!(*tile_from, Tile::Box | Tile::BoxInGoal | Tile::BoxOnFragileFloor | Tile::BoxOnIce);

        let tile_from_new_value;
        let tile_to_new_value;

        let mut has_won = false;

        if matches!(*tile_to, Tile::Empty | Tile::FragileFloor | Tile::Ice | Tile::Goal | Tile::BoxInHole | Tile::Hole) ||
                (!is_box && *tile_to == Tile::LockedDoor) {
            if is_box && *tile_to == Tile::Goal {
                tile_to_new_value = Tile::BoxInGoal;

                has_won = true;
                for (index, tile) in level.tiles.iter().
                        enumerate() {
                    if index == index_to {
                        continue;
                    }

                    if *tile == Tile::Goal || *tile == Tile::KeyInGoal {
                        has_won = false;

                        break;
                    }

                    let tile_original = &self.original_level.tiles[index];

                    //If player is on GOAL -> check level field
                    if (*tile == Tile::Player || index == index_from) &&
                            matches!(*tile_original, Tile::Goal | Tile::BoxInGoal | Tile::KeyInGoal) {
                        has_won = false;

                        break;
                    }
                }
            }else if !is_box && *tile_to == Tile::Goal {
                tile_to_new_value = Tile::KeyInGoal;
            }else if *tile_to == Tile::FragileFloor {
                if is_box {
                    tile_to_new_value = Tile::BoxOnFragileFloor;
                }else {
                    tile_to_new_value = Tile::KeyOnFragileFloor;
                }
            }else if *tile_to == Tile::Ice {
                if is_box {
                    tile_to_new_value = Tile::BoxOnIce;
                }else {
                    tile_to_new_value = Tile::KeyOnIce;
                }
            }else if *tile_to == Tile::Hole {
                if is_box {
                    tile_to_new_value = Tile::BoxInHole;
                }else {
                    //Key will be destroyed, only boxes can fill holes
                    tile_to_new_value = Tile::Hole;
                }
            }else if is_box {
                tile_to_new_value = Tile::Box;
            }else if *tile_to == Tile::LockedDoor {
                //Open door and destroy key
                tile_to_new_value = Tile::Empty;
            }else {
                tile_to_new_value = Tile::Key;
            }

            if *tile_from == Tile::Box || *tile_from == Tile::Key {
                tile_from_new_value = Tile::Empty;
            }else if *tile_from == Tile::BoxInHole {
                tile_from_new_value = Tile::BoxInHole;
            }else if *tile_from == Tile::BoxOnFragileFloor || *tile_from == Tile::KeyOnFragileFloor {
                tile_from_new_value = Tile::FragileFloor;
            }else if *tile_from == Tile::BoxOnIce || *tile_from == Tile::KeyOnIce {
                tile_from_new_value = Tile::Ice;
            }else {
                tile_from_new_value = Tile::Goal;
            }

            level.tiles[index_from] = tile_from_new_value;
            level.tiles[index_to] = tile_to_new_value;

            let move_result = MoveResult::Valid { has_won, secret_found: false, sound_effect: match tile_to_new_value {
                Tile::BoxInHole => Some(LevelSoundEffect::BoxFall),
                Tile::Hole => Some(LevelSoundEffect::KeyFall),
                Tile::Empty => Some(LevelSoundEffect::DoorUnlocked),

                _ => None,
            }};

            //If ice tile: move forwards until no longer ice
            if matches!(tile_to_new_value, Tile::BoxOnIce | Tile::KeyOnIce) {
                self.animation_state = Some(AnimationState::BoxOrKey {
                    last_valid_move_result: move_result,
                    x_from: x_to, y_from: y_to,
                    direction,
                });

                return MoveResult::Animation { player_animation: false, sound_effect: None };
            }

            return move_result;
        }

        MoveResult::Invalid
    }

    pub fn original_level(&self) -> &Level {
        &self.original_level
    }

    pub fn current_playing_level(&self) -> &(Level, (usize, usize)) {
        self.playing_level.current()
    }

    pub fn current_move_index(&self) -> usize {
        self.playing_level.current_index()
    }

    pub fn undo_move(&mut self) -> Option<&(Level, (usize, usize))> {
        if self.is_playing_animation() {
            return None;
        }

        self.playing_level.undo()
    }

    pub fn redo_move(&mut self) -> Option<&(Level, (usize, usize))> {
        if self.is_playing_animation() {
            return None;
        }

        self.playing_level.redo()
    }
}

#[derive(Debug, Clone)]
pub struct LevelWithStats {
    level: Level,
    best_time: Option<u64>,
    best_moves: Option<u32>
}

impl LevelWithStats {
    pub fn new(level: Level, best_time: Option<u64>, best_moves: Option<u32>) -> Self {
        Self { level, best_time, best_moves }
    }

    pub fn level(&self) -> &Level {
        &self.level
    }

    pub fn level_mut(&mut self) -> &mut Level {
        &mut self.level
    }

    pub fn best_time(&self) -> Option<u64> {
        self.best_time
    }

    pub fn best_moves(&self) -> Option<u32> {
        self.best_moves
    }

    pub fn set_best_time(&mut self, best_time: Option<u64>) {
        self.best_time = best_time;
    }

    pub fn set_best_moves(&mut self, best_moves: Option<u32>) {
        self.best_moves = best_moves;
    }
}

#[cfg(feature = "steam")]
#[derive(Debug)]
pub struct SteamLevelPackData {
    workshop_id: PublishedFileId,
}

#[cfg(feature = "steam")]
impl SteamLevelPackData {
    pub fn workshop_id(&self) -> PublishedFileId {
        self.workshop_id
    }
}

#[cfg(feature = "steam")]
impl From<QueryResult> for SteamLevelPackData {
    fn from(value: QueryResult) -> Self {
        SteamLevelPackData {
            workshop_id: value.published_file_id,
        }
    }
}

#[derive(Debug)]
pub struct LevelPack {
    name: String,
    id: String,
    path: String,

    thumbnail_level_index: Option<usize>,
    background_music_id: Option<BackgroundMusicId>,

    levels: Vec<LevelWithStats>,

    min_level_not_completed: usize,

    level_pack_best_time_sum: Option<u64>,
    level_pack_best_moves_sum: Option<u32>,

    #[cfg(feature = "steam")]
    steam_level_pack_data: Option<SteamLevelPackData>,
}

impl LevelPack {
    pub const MAX_LEVEL_PACK_NAME_LEN: usize = 25;

    pub const MAX_LEVEL_PACK_COUNT: usize = 190;
    pub const MAX_LEVEL_COUNT_PER_PACK: usize = 190;

    pub fn new(name: impl Into<String>, id: impl Into<String>, path: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            id: id.into(),
            path: path.into(),
            levels: vec![],

            thumbnail_level_index: None,
            background_music_id: None,

            min_level_not_completed: Default::default(),
            level_pack_best_time_sum: Default::default(),
            level_pack_best_moves_sum: Default::default(),

            #[cfg(feature = "steam")]
            steam_level_pack_data: None,
        }
    }

    pub fn read_from_save_game(
        id: impl Into<String>, path: impl Into<String>, lvl_data: impl Into<String>, editor_level_pack: bool,

        #[cfg(feature = "steam")]
        steam_level_pack_data: Option<SteamLevelPackData>,
    ) -> Result<Self, Box<dyn Error>> {
        let mut lvl_name = None;
        let id = id.into();
        let path = path.into();

        let mut pack_thumbnail_level_index = None;
        let mut pack_background_music_id = None;

        let lvl_data = lvl_data.into();

        let mut levels = Vec::with_capacity(Self::MAX_LEVEL_COUNT_PER_PACK);
        {
            let lines = lvl_data.lines().collect::<Vec<_>>();
            if lines.is_empty() {
                return Err(Box::new(LevelLoadingError::new(format!(
                    "The level pack file \"{path}\" is empty!"
                ))));
            }

            let mut lines = lines.into_iter();

            let mut line = lines.next().unwrap().trim();
            if let Some(name) = line.strip_prefix("Name: ") {
                let name = name.trim();
                if name.len() > Self::MAX_LEVEL_PACK_NAME_LEN {
                    return Err(Box::new(LevelLoadingError::new(format!(
                        "The level pack name \"{name}\" is too long!"
                    ))));
                }

                lvl_name = Some(name);

                let next_line = lines.next();
                let Some(next_line) = next_line else {
                    return Err(Box::new(LevelLoadingError::new(format!(
                        "The level pack file \"{path}\" does not contain level count!"
                    ))));
                };
                line = next_line.trim();
            }

            if let Some(thumbnail_level) = line.strip_prefix("Thumbnail Level: ") {
                let Ok(thumbnail_level_index) = usize::from_str(thumbnail_level.trim()) else {
                    return Err(Box::new(LevelLoadingError::new(format!(
                        "The thumbnail level index \"{line}\" is invalid in the level pack file \"{path}\"!"
                    ))));
                };

                pack_thumbnail_level_index = Some(thumbnail_level_index);

                let next_line = lines.next();
                let Some(next_line) = next_line else {
                    return Err(Box::new(LevelLoadingError::new(format!(
                        "The level pack file \"{path}\" does not contain level count!"
                    ))));
                };
                line = next_line.trim();
            }

            if let Some(background_music) = line.strip_prefix("Background Music: ") {
                let Ok(background_music_id) = usize::from_str(background_music.trim()) else {
                    return Err(Box::new(LevelLoadingError::new(format!(
                        "The background music id \"{line}\" is invalid in the level pack file \"{path}\"!"
                    ))));
                };

                pack_background_music_id = audio::BACKGROUND_MUSIC_TRACKS.check_id(background_music_id);
                if pack_background_music_id.is_none() {
                    return Err(Box::new(LevelLoadingError::new(format!(
                        "The background music \"{background_music_id}\" from level pack file \"{path}\" does not exist \
                        (Make sure that you are playing the latest version of SokoTerm)!"
                    ))));
                }

                let next_line = lines.next();
                let Some(next_line) = next_line else {
                    return Err(Box::new(LevelLoadingError::new(format!(
                        "The level pack file \"{path}\" does not contain level count!"
                    ))));
                };
                line = next_line.trim();
            }

            if !line.starts_with("Levels: ") {
                return Err(Box::new(LevelLoadingError::new(format!(
                    "The level count is missing in the level pack file \"{path}\"!"
                ))));
            }

            let line = &line[8..];

            let level_count = if let Ok(level_count) = usize::from_str(line) {
                if level_count > Self::MAX_LEVEL_COUNT_PER_PACK {
                    return Err(Box::new(LevelLoadingError::new(format!(
                        "There are too many levels in the level pack file \"{path}\" (Count: {line}, Max: {})!",
                        Self::MAX_LEVEL_COUNT_PER_PACK
                    ))));
                }else {
                    level_count
                }
            }else {
                return Err(Box::new(LevelLoadingError::new(format!(
                    "The level count \"{line}\" is invalid in the level pack file \"{path}\"!"
                ))));
            };

            if let Some(index) = pack_thumbnail_level_index && level_count <= index {
                return Err(Box::new(LevelLoadingError::new(format!(
                    "The thumbnail level index {index} is out of bounds (Should be less then {level_count}) in the level pack file \"{path}\"!"
                ))));
            }

            let mut line_iter = lines.
                    filter(|line| !line.trim().is_empty());
            for i in 0..level_count {
                let line = line_iter.next();
                let Some(line) = line else {
                    return Err(Box::new(LevelLoadingError::new(format!(
                        "EOF was reached early in the level pack file \"{path}\" (Read: {} levels, Expected: {level_count} levels)!",
                        i + 1
                    ))));
                };

                if !line.starts_with("w: ") || !line.contains(", h: ") {
                    return Err(Box::new(LevelLoadingError::new(format!(
                        "Level {} is invalid in the level pack file \"{path}\"!",
                        i + 1
                    ))));
                }

                let index = line.to_string().find(", h: ").unwrap() + 5;
                let height = if let Ok(height) = usize::from_str(&line[index..]) {
                    height
                }else {
                    return Err(Box::new(LevelLoadingError::new(format!(
                        "Level {} is invalid in the level pack file \"{path}\"!",
                        i + 1
                    ))));
                };

                let mut level_str = Vec::with_capacity(1 + height);
                level_str.push(line);
                for _ in 0..height {
                    if let Some(line) = line_iter.next() {
                        level_str.push(line);
                    }else {
                        return Err(Box::new(LevelLoadingError::new(format!(
                            "EOF was reached early during parsing of level {} is invalid in the level pack file \"{path}\"!",
                            i + 1
                        ))));
                    }
                }

                let level = Level::from_str(&level_str.join("\n"));
                let level = match level {
                    Ok(level) => level,
                    Err(err) => {
                        return Err(Box::new(LevelLoadingError::new(format!(
                            "\"{}\" occurred during parsing of level {} is invalid in the level pack file \"{path}\"!",
                            err, i + 1
                        ))));
                    },
                };

                if !editor_level_pack {
                    let player_tile_count = level.tiles().iter().filter(|tile| matches!(tile, Tile::Player | Tile::PlayerOnFragileFloor | Tile::PlayerOnIce)).count();
                    if player_tile_count == 0 {
                        return Err(Box::new(GameError::new(format!(
                            "Error while loading level pack \"{}\": Level {} does not contain a player tile",
                            id,
                            i + 1,
                        ))));
                    }else if player_tile_count > 1 {
                        return Err(Box::new(GameError::new(format!(
                            "Error while loading level pack \"{}\": Level {} contains too many player tiles",
                            id,
                            i + 1,
                        ))));
                    }
                }

                levels.push(level);
            }

            if line_iter.next().is_some() {
                return Err(Box::new(LevelLoadingError::new(format!(
                    "Additional data was found after last level was parsed in the level pack file \"{path}\"!"
                ))));
            }
        }

        if !editor_level_pack && levels.is_empty() {
            return Err(Box::new(GameError::new(format!(
                "Error while loading level pack \"{}\": Level pack contains no levels",
                id,
            ))));
        }

        let level_save_file_postfix = if editor_level_pack {
            ".lvl.edit.sav"
        }else {
            ".lvl.sav"
        };

        let mut save_game_file = Game::get_or_create_save_game_folder()?;
        {
            #[cfg(not(feature = "steam"))]
            {
                save_game_file.push(&id);
                save_game_file.push(level_save_file_postfix);
            }

            #[cfg(feature = "steam")]
            if let Some(steam_level_pack_data) = &steam_level_pack_data {
                save_game_file.push("SteamWorkshop/");
                save_game_file.push(steam_level_pack_data.workshop_id.0.to_string());
                save_game_file.push(level_save_file_postfix);
            }else {
                save_game_file.push(&id);
                save_game_file.push(level_save_file_postfix);
            }
        }

        let mut min_level_not_completed= Default::default();
        let mut level_stats: Vec<(Option<u64>, Option<u32>)> = vec![Default::default(); Self::MAX_LEVEL_COUNT_PER_PACK];
        'read_save_game: {
            if std::fs::exists(&save_game_file)? {
                let save_game_data = std::fs::read_to_string(&save_game_file)?;

                let lines = save_game_data.lines().collect::<Vec<_>>();
                if lines.is_empty() {
                    //TODO add warning message (could not load save file '&id + level_save_file_postfix')

                    break 'read_save_game;
                }

                let line = lines.first().unwrap().trim();

                if !editor_level_pack {
                    min_level_not_completed = if let Ok(min_level_not_completed) = usize::from_str(line) {
                        min_level_not_completed
                    }else {
                        //TODO add warning message (could not load save file '&id + level_save_file_postfix')

                        break 'read_save_game;
                    };
                }

                for (i, mut line) in lines.iter().
                        skip(if editor_level_pack { 0 } else { 1 }).
                        take(Self::MAX_LEVEL_COUNT_PER_PACK).
                        map(|line| line.trim()).
                        enumerate() {
                    let is_new_format = line.starts_with("ms");
                    if is_new_format {
                        line = &line[2..];
                    }

                    let tokens = line.split(",").collect::<Vec<_>>();
                    if tokens.len() != 2 {
                        continue;
                    }

                    let best_time = u64::from_str(tokens[0]).ok().map(|best_time| {
                        if is_new_format {
                            best_time
                        }else {
                            best_time * 1000 + 999
                        }
                    });
                    let best_moves = u32::from_str(tokens[1]).ok();

                    level_stats[i] = (best_time, best_moves);
                }
            }
        }

        let levels = levels.into_iter().
                zip(level_stats).
                map(|(level, (best_time, best_moves))| {
                    LevelWithStats::new(level, best_time, best_moves)
                }).collect::<Vec<_>>();

        let mut level_pack = Self {
            name: lvl_name.map(ToString::to_string).unwrap_or_else(|| id.clone()),
            id,
            path,

            thumbnail_level_index: pack_thumbnail_level_index,
            background_music_id: pack_background_music_id,

            levels,

            min_level_not_completed,
            level_pack_best_time_sum: Default::default(),
            level_pack_best_moves_sum: Default::default(),

            #[cfg(feature = "steam")]
            steam_level_pack_data,
        };
        level_pack.calculate_stats_sum();

        Ok(level_pack)
    }

    /// This function is used for saving level pack editor state to the default save path, validation results are included
    pub fn save_editor_level_pack(&self) -> Result<(), Box<dyn Error>> {
        self.export_editor_level_pack_to_path(&self.path)?;

        self.save_save_game(true)
    }

    /// This function is used for saving level pack editor state and exporting, validation results are not included
    pub fn export_editor_level_pack_to_path(&self, path: impl Into<OsString>) -> Result<(), Box<dyn Error>> {
        let mut file = File::create(path.into())?;

        writeln!(file, "Name: {}", self.name)?;

        if let Some(thumbnail_level_index) = self.thumbnail_level_index && thumbnail_level_index < self.levels.len() {
            writeln!(file, "Thumbnail Level: {}", thumbnail_level_index)?;
        }

        if let Some(background_music_id) = self.background_music_id {
            writeln!(file, "Background Music: {}", background_music_id.id())?;
        }

        writeln!(file, "Levels: {}", self.levels.len())?;

        for level in self.levels.iter().
                map(|level| level.level()) {
            write!(file, "\n{}", level.to_str())?;
        }
        file.flush()?;

        Ok(())
    }

    pub fn save_save_game(&self, editor_validation: bool) -> Result<(), Box<dyn Error>> {
        let level_save_file_postfix = if editor_validation {
            ".lvl.edit.sav"
        }else {
            ".lvl.sav"
        };

        let mut save_game_file = Game::get_or_create_save_game_folder()?;
        {
            #[cfg(not(feature = "steam"))]
            {
                save_game_file.push(&self.id);
                save_game_file.push(level_save_file_postfix);
            }

            #[cfg(feature = "steam")]
            if let Some(steam_level_pack_data) = &self.steam_level_pack_data {
                save_game_file.push("SteamWorkshop/");
                save_game_file.push(steam_level_pack_data.workshop_id.0.to_string());
                save_game_file.push(level_save_file_postfix);
            }else {
                save_game_file.push(&self.id);
                save_game_file.push(level_save_file_postfix);
            }
        }

        let mut file = File::create(save_game_file)?;

        let level_score_count = if editor_validation {
            self.levels.len()
        }else {
            writeln!(file, "{}", self.min_level_not_completed)?;

            self.min_level_not_completed
        };

        for level in self.levels.iter().
                take(level_score_count) {
            writeln!(
                file, "ms{},{}",
                level.best_time.map_or(-1, |best_time| best_time as i64),
                level.best_moves.map_or(-1, |best_moves| best_moves as i32)
            )?;
        }
        file.flush()?;

        Ok(())
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn set_name(&mut self, name: impl Into<String>) {
        self.name = name.into();
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn path(&self) -> &str {
        &self.path
    }

    pub fn thumbnail_level_index(&self) -> Option<usize> {
        self.thumbnail_level_index
    }

    pub fn set_thumbnail_level_index(&mut self, thumbnail_level_index: Option<usize>) {
        self.thumbnail_level_index = thumbnail_level_index;
    }

    pub fn background_music_id(&self) -> Option<BackgroundMusicId> {
        self.background_music_id
    }

    pub fn set_background_music_id(&mut self, background_music_id: Option<BackgroundMusicId>) {
        self.background_music_id = background_music_id;
    }

    pub fn levels(&self) -> &[LevelWithStats] {
        &self.levels
    }

    pub fn levels_mut(&mut self) -> &mut Vec<LevelWithStats> {
        &mut self.levels
    }

    pub fn min_level_not_completed(&self) -> usize {
        self.min_level_not_completed
    }

    pub fn level_pack_best_time_sum(&self) -> Option<u64> {
        self.level_pack_best_time_sum
    }

    pub fn level_pack_best_moves_sum(&self) -> Option<u32> {
        self.level_pack_best_moves_sum
    }

    pub fn set_min_level_not_completed(&mut self, min_level_not_completed: usize) {
        self.min_level_not_completed = min_level_not_completed;
    }

    pub fn level_count(&self) -> usize {
        self.levels.len()
    }

    pub fn update_stats(&mut self, index: usize, best_time: u64, best_moves: u32) -> Option<()> {
        let level = self.levels.get_mut(index)?;

        level.best_time = if level.best_time.is_none_or(|level_best_time| best_time < level_best_time) {
            Some(best_time)
        }else {
            level.best_time
        };

        level.best_moves = if level.best_moves.is_none_or(|level_best_moves| best_moves < level_best_moves) {
            Some(best_moves)
        }else {
            level.best_moves
        };

        self.calculate_stats_sum();

        Some(())
    }

    pub fn add_level(&mut self, level: Level) {
        self.levels.push(LevelWithStats::new(level, None, None));

        self.calculate_stats_sum();
    }

    pub(super) fn calculate_stats_sum(&mut self) {
        if self.levels.is_empty() {
            self.level_pack_best_time_sum = None;
            self.level_pack_best_moves_sum = None;

            return;
        }

        let stats_sum = self.levels.iter().
                fold((Some(0), Some(0)), |mut sum, current| {
                    sum.0 = if let Some(best_time) = current.best_time {
                        sum.0.map(|sum| sum + best_time)
                    }else {
                        None
                    };

                    sum.1 = if let Some(best_moves) = current.best_moves {
                        sum.1.map(|sum| sum + best_moves)
                    }else {
                        None
                    };

                    sum
                });

        self.level_pack_best_time_sum = stats_sum.0;
        self.level_pack_best_moves_sum = stats_sum.1;
    }

    #[cfg(feature = "steam")]
    pub fn steam_level_pack_data(&self) -> Option<&SteamLevelPackData> {
        self.steam_level_pack_data.as_ref()
    }
}

#[derive(Debug)]
pub struct LevelLoadingError {
    message: String
}

impl LevelLoadingError {
    fn new(message: impl Into<String>) -> Self {
        Self { message: message.into() }
    }
}

impl Display for LevelLoadingError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message)
    }
}

impl Error for LevelLoadingError {}
