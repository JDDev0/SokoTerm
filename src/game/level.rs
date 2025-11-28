use crate::game::{audio, Game, GameError};
use std::error::Error;
use std::ffi::OsString;
use std::fmt::{Debug, Display, Formatter, Write as _};
use std::fs::File;
use std::io::Write;
use std::str::FromStr;
use crate::collections::UndoHistory;
use crate::game::audio::BackgroundMusicId;
use crate::io::{Color, Console};

#[cfg(feature = "steam")]
use bevy_steamworks::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tile {
    Empty,
    FragileFloor,

    OneWayLeft,
    OneWayUp,
    OneWayRight,
    OneWayDown,

    Wall,

    Player,
    PlayerOnFragileFloor,

    Key,
    KeyInGoal,
    KeyOnFragileFloor,
    LockedDoor,

    Box,
    BoxInGoal,
    BoxOnFragileFloor,
    Goal,

    Hole,
    BoxInHole,

    DecorationBlank,

    Secret,
}

impl Tile {
    pub fn from_ascii(a: u8) -> Result<Self, LevelLoadingError> {
        match a {
            b'-' => Ok(Tile::Empty),
            //Different ASCII char than display for compatibility with old level packs
            b':' => Ok(Tile::FragileFloor),

            b'<' => Ok(Tile::OneWayLeft),
            b'^' => Ok(Tile::OneWayUp),
            b'>' => Ok(Tile::OneWayRight),
            b'v' => Ok(Tile::OneWayDown),

            b'#' => Ok(Tile::Wall),

            b'p' | b'P' => Ok(Tile::Player),
            b',' => Ok(Tile::PlayerOnFragileFloor),

            b'*' => Ok(Tile::Key),
            b'~' => Ok(Tile::KeyInGoal),
            b';' => Ok(Tile::KeyOnFragileFloor),
            b'=' => Ok(Tile::LockedDoor),

            b'@' => Ok(Tile::Box),
            b'+' => Ok(Tile::BoxInGoal),
            b'!' => Ok(Tile::BoxOnFragileFloor),
            b'x' | b'X' => Ok(Tile::Goal),

            b'o' | b'O' => Ok(Tile::Hole),
            b'.' => Ok(Tile::BoxInHole),

            b'b' | b'B' => Ok(Tile::DecorationBlank),

            b's' | b'S' => Ok(Tile::Secret),

            _ => Err(LevelLoadingError::new("Invalid tile")),
        }
    }

    pub fn to_ascii(&self) -> u8 {
        match self {
            Tile::Empty => b'-',
            //Different ASCII char than display for compatibility with old level packs
            Tile::FragileFloor => b':',

            Tile::OneWayLeft => b'<',
            Tile::OneWayUp => b'^',
            Tile::OneWayRight => b'>',
            Tile::OneWayDown => b'v',

            Tile::Wall => b'#',

            Tile::Player => b'P',
            Tile::PlayerOnFragileFloor => b',',

            Tile::Key => b'*',
            Tile::KeyInGoal => b'~',
            Tile::KeyOnFragileFloor => b';',
            Tile::LockedDoor => b'=',

            Tile::Box => b'@',
            Tile::BoxInGoal => b'+',
            Tile::BoxOnFragileFloor => b'!',
            Tile::Goal => b'x',

            Tile::Hole => b'o',
            Tile::BoxInHole => b'.',

            Tile::DecorationBlank => b'b',

            Tile::Secret => b's',
        }
    }

    pub fn draw(&self, console: &Console, is_player_background: bool, inverted: bool) {
        match self {
            Tile::Empty => {
                console.set_color_invertible(Color::LightBlue, Color::Default, inverted);
                console.draw_text("-");
            },
            Tile::FragileFloor => {
                console.set_color_invertible(Color::LightBlue, Color::Default, inverted);
                console.draw_text("~");
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
            Tile::Player | Tile::PlayerOnFragileFloor => {
                if is_player_background {
                    console.set_color_invertible(Color::Default, Color::Yellow, inverted);
                }else {
                    console.set_color_invertible(Color::Yellow, Color::Default, inverted);
                }
                console.draw_text("P");
            },
            Tile::Key | Tile::KeyOnFragileFloor => {
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
            Tile::Box | Tile::BoxOnFragileFloor => {
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
    pub fn update_x(&self, x: usize, width: usize) -> usize {
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

    pub fn update_y(&self, y: usize, height: usize) -> usize {
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

    pub fn update_xy(&self, x: usize, y: usize, width: usize, height: usize) -> (usize, usize) {
        (self.update_x(x, width), self.update_y(y, height))
    }
}

#[derive(Debug, Clone, Copy)]
pub enum MoveResult {
    Valid {
        has_won: bool,
        secret_found: bool,
    },
    Invalid,
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

    fn move_box_or_key(&mut self, level_original: &Level, x_from: usize, y_from: usize, direction: Direction) -> MoveResult {
        if self.width != level_original.width || self.height != level_original.height {
            panic!("Original level must have the same width and height as the modified level!");
        }

        let (x_to, y_to) = direction.update_xy(x_from, y_from, self.width, self.height);

        let index_from = x_from + y_from * self.width;
        let index_to = x_to + y_to * self.width;

        let Some(tile_from) = self.tiles.get(index_from) else {
            return MoveResult::Invalid;
        };
        let Some(tile_to) = self.tiles.get(index_to) else {
            return MoveResult::Invalid;
        };

        let is_box = matches!(*tile_from, Tile::Box | Tile::BoxInGoal | Tile::BoxOnFragileFloor);

        let tile_from_new_value;
        let tile_to_new_value;

        let mut has_won = false;

        if matches!(*tile_to, Tile::Empty | Tile::FragileFloor | Tile::Goal | Tile::BoxInHole | Tile::Hole) ||
                (!is_box && *tile_to == Tile::LockedDoor) {
            if is_box && *tile_to == Tile::Goal {
                tile_to_new_value = Tile::BoxInGoal;

                has_won = true;
                for (index, tile) in self.tiles.iter().
                        enumerate() {
                    if index == index_to {
                        continue;
                    }

                    if *tile == Tile::Goal || *tile == Tile::KeyInGoal {
                        has_won = false;

                        break;
                    }

                    let tile_original = &level_original.tiles[index];

                    //If player is on GOAL -> check level field
                    if index == index_from && (*tile_original == Tile::Goal ||
                            *tile_original == Tile::BoxInGoal || *tile_original == Tile::KeyInGoal) {
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
            }else {
                tile_from_new_value = Tile::Goal;
            }

            self.tiles[index_from] = tile_from_new_value;
            self.tiles[index_to] = tile_to_new_value;

            return MoveResult::Valid { has_won, secret_found: false };
        }

        MoveResult::Invalid
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
    playing_level: UndoHistory<(Level, (usize, usize))>,
}

impl PlayingLevel {
    pub fn new(level: &Level, history_size: usize) -> Result<Self, LevelLoadingError> {
        let player_tile_count = level.tiles().iter().filter(|tile| **tile == Tile::Player || **tile == Tile::PlayerOnFragileFloor).count();
        if player_tile_count == 0 {
            return Err(LevelLoadingError::new("Level does not contain a player tile!"));
        }else if player_tile_count > 1 {
            return Err(LevelLoadingError::new("Level contains too many player tiles!"));
        }

        let mut player_pos = None;

        'outer:
        for i in 0..level.width() {
            for j in 0..level.height() {
                if let Some(tile) = level.get_tile(i, j) && matches!(tile, Tile::Player | Tile::PlayerOnFragileFloor) {
                    player_pos = Some((i, j));

                    break 'outer;
                }
            }
        }

        Ok(PlayingLevel {
            original_level: level.clone(),
            playing_level: UndoHistory::new(history_size, (level.clone(), player_pos.unwrap())),
        })
    }

    pub fn move_player(&mut self, direction: Direction) -> MoveResult {
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
        }

        level.set_tile(x_from, y_from, tile);

        let tile = level.get_tile(x_to, y_to).unwrap();
        let move_result = if matches!(tile, Tile::Empty | Tile::FragileFloor | Tile::Goal | Tile::Secret | Tile::BoxInHole) || tile == one_way_door_tile {
            MoveResult::Valid { has_won: false, secret_found: tile == Tile::Secret }
        }else if matches!(tile, Tile::Box | Tile::BoxInGoal | Tile::BoxOnFragileFloor | Tile::Key | Tile::KeyInGoal | Tile::KeyOnFragileFloor) {
            level.move_box_or_key(&self.original_level, x_to, y_to, direction)
        }else {
            MoveResult::Invalid
        };

        if move_result.is_valid() {
            player_pos = (x_to, y_to);
        }

        //Set player to new position
        if matches!(level.get_tile(x_to, y_to).unwrap(), Tile::FragileFloor | Tile::PlayerOnFragileFloor | Tile::BoxOnFragileFloor | Tile::KeyOnFragileFloor) {
            level.set_tile(player_pos.0, player_pos.1, Tile::PlayerOnFragileFloor);
        }else {
            level.set_tile(player_pos.0, player_pos.1, Tile::Player);
        }

        if move_result.is_valid() {
            self.playing_level.commit_change((level, player_pos));
        }

        move_result
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
        self.playing_level.undo()
    }

    pub fn redo_move(&mut self) -> Option<&(Level, (usize, usize))> {
        self.playing_level.redo()
    }
}

#[derive(Debug)]
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
    steam_workshop_id: Option<PublishedFileId>,
}

impl LevelPack {
    pub const MAX_LEVEL_PACK_NAME_LEN: usize = 25;

    pub const MAX_LEVEL_PACK_COUNT: usize = 191;
    pub const MAX_LEVEL_COUNT_PER_PACK: usize = 191;

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
            steam_workshop_id: None,
        }
    }

    pub fn read_from_save_game(
        id: impl Into<String>, path: impl Into<String>, lvl_data: impl Into<String>, editor_level_pack: bool,

        #[cfg(feature = "steam")]
        steam_workshop_id: Option<PublishedFileId>,
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
                    let player_tile_count = level.tiles().iter().filter(|tile| **tile == Tile::Player || **tile == Tile::PlayerOnFragileFloor).count();
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
            if let Some(steam_workshop_id) = steam_workshop_id {
                save_game_file.push("SteamWorkshop/");
                save_game_file.push(steam_workshop_id.0.to_string());
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
            steam_workshop_id,
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

        if let Some(thumbnail_level_index) = self.thumbnail_level_index {
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
            if let Some(steam_workshop_id) = self.steam_workshop_id {
                save_game_file.push("SteamWorkshop/");
                save_game_file.push(steam_workshop_id.0.to_string());
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
    pub fn steam_workshop_id(&self) -> Option<PublishedFileId> {
        self.steam_workshop_id
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
