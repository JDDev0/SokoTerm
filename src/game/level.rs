use crate::game::Game;
use std::error::Error;
use std::fmt::{Debug, Display, Formatter, Write as _};
use std::fs::File;
use std::io::Write;
use std::str::FromStr;
use crate::io::{Color, Console};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Tile {
    Empty,

    OneWayLeft,
    OneWayUp,
    OneWayRight,
    OneWayDown,

    Wall,

    Player,

    Key,
    KeyInGoal,
    LockedDoor,

    Box,
    BoxInGoal,
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

            b'<' => Ok(Tile::OneWayLeft),
            b'^' => Ok(Tile::OneWayUp),
            b'>' => Ok(Tile::OneWayRight),
            b'v' => Ok(Tile::OneWayDown),

            b'#' => Ok(Tile::Wall),

            b'p' | b'P' => Ok(Tile::Player),

            b'*' => Ok(Tile::Key),
            b'~' => Ok(Tile::KeyInGoal),
            b'=' => Ok(Tile::LockedDoor),

            b'@' => Ok(Tile::Box),
            b'+' => Ok(Tile::BoxInGoal),
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

            Tile::OneWayLeft => b'<',
            Tile::OneWayUp => b'^',
            Tile::OneWayRight => b'>',
            Tile::OneWayDown => b'v',

            Tile::Wall => b'#',

            Tile::Player => b'P',

            Tile::Key => b'*',
            Tile::KeyInGoal => b'~',
            Tile::LockedDoor => b'=',

            Tile::Box => b'@',
            Tile::BoxInGoal => b'+',
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
            Tile::Player => {
                if is_player_background {
                    console.set_color_invertible(Color::Default, Color::Yellow, inverted);
                }else {
                    console.set_color_invertible(Color::Yellow, Color::Default, inverted);
                }
                console.draw_text("P");
            },
            Tile::Key => {
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
            Tile::Box => {
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

    pub fn get_tile(&self, x: usize, y: usize) -> Option<&Tile> {
        self.tiles.get(x + y * self.width)
    }

    pub fn get_tile_mut(&mut self, x: usize, y: usize) -> Option<&mut Tile> {
        self.tiles.get_mut(x + y * self.width)
    }

    pub fn set_tile(&mut self, x: usize, y: usize, tile: Tile) {
        self.tiles[x + y * self.width] = tile;
    }

    pub fn move_box_or_key(&mut self, level_original: &Level, has_won: &mut bool, from_pos_x: usize, from_pos_y: usize, to_pos_x: usize, to_pos_y: usize) -> bool {
        if self.width != level_original.width || self.height != level_original.height {
            panic!("Original level must have the same width and height as the modified level!");
        }

        let move_x = to_pos_x as isize - from_pos_x as isize;
        let move_y = to_pos_y as isize - from_pos_y as isize;
        let index_from = to_pos_x + to_pos_y * self.width;
        let index_to = ((to_pos_x as isize + move_x + self.width as isize) % self.width as isize) as usize +
                ((to_pos_y as isize + move_y + self.height as isize) % self.height as isize) as usize * self.width;

        let Some(tile_from) = self.tiles.get(index_from) else {
            return false;
        };
        let Some(tile_to) = self.tiles.get(index_to) else {
            return false;
        };

        let is_box = *tile_from == Tile::Box || *tile_from == Tile::BoxInGoal;

        let tile_from_new_value;
        let tile_to_new_value;

        if *tile_to == Tile::Empty ||*tile_to == Tile::Goal ||  *tile_to == Tile::BoxInHole ||
                *tile_to == Tile::Hole || (!is_box && *tile_to == Tile::LockedDoor) {
            if is_box && *tile_to == Tile::Goal {
                tile_to_new_value = Tile::BoxInGoal;

                *has_won = true;
                for (index, tile) in self.tiles.iter().
                        enumerate() {
                    if index == index_to {
                        continue;
                    }

                    if *tile == Tile::Goal || *tile == Tile::KeyInGoal {
                        *has_won = false;

                        break;
                    }

                    let tile_original = &level_original.tiles[index];

                    //If player is on GOAL -> check level field
                    if index == index_from && (*tile_original == Tile::Goal ||
                            *tile_original == Tile::BoxInGoal || *tile_original == Tile::KeyInGoal) {
                        *has_won = false;

                        break;
                    }
                }
            }else if !is_box && *tile_to == Tile::Goal {
                tile_to_new_value = Tile::KeyInGoal;
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
            }else {
                tile_from_new_value = Tile::Goal;
            }

            self.tiles[index_from] = tile_from_new_value;
            self.tiles[index_to] = tile_to_new_value;

            return true;
        }

        false
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
}

#[derive(Debug)]
pub struct LevelPack {
    name: String,
    id: String,
    path: String,
    levels: Vec<LevelWithStats>,

    min_level_not_completed: usize,

    level_pack_best_time_sum: Option<u64>,
    level_pack_best_moves_sum: Option<u32>,
}

impl LevelPack {
    pub const MAX_LEVEL_PACK_NAME_LEN: usize = 25;

    pub const MAX_LEVEL_PACK_COUNT: usize = 64;
    pub const MAX_LEVEL_COUNT_PER_PACK: usize = 191;

    pub fn new(name: impl Into<String>, id: impl Into<String>, path: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            id: id.into(),
            path: path.into(),
            levels: vec![],

            min_level_not_completed: Default::default(),
            level_pack_best_time_sum: Default::default(),
            level_pack_best_moves_sum: Default::default(),
        }
    }

    pub fn read_from_save_game(id: impl Into<String>, path: impl Into<String>, lvl_data: impl Into<String>) -> Result<Self, Box<dyn Error>> {
        let mut lvl_name = None;
        let id = id.into();
        let path = path.into();
        let lvl_data = lvl_data.into();

        let mut levels = Vec::with_capacity(Self::MAX_LEVEL_COUNT_PER_PACK);
        {
            let lines = lvl_data.lines().collect::<Vec<_>>();
            if lines.is_empty() {
                return Err(Box::new(LevelLoadingError::new(format!(
                    "The level pack file \"{path}\" is empty!"
                ))));
            }

            let mut line = lines.first().unwrap().trim();
            if let Some(name) = line.strip_prefix("Name: ") {
                let name = name.trim();
                if name.len() > Self::MAX_LEVEL_PACK_NAME_LEN {
                    return Err(Box::new(LevelLoadingError::new(format!(
                        "The level pack name \"{name}\" is too long!"
                    ))));
                }

                lvl_name = Some(name);

                if lines.len() == 1 {
                    return Err(Box::new(LevelLoadingError::new(format!(
                        "The level pack file \"{path}\" does not contain level count!"
                    ))));
                }
                line = lines.get(1).unwrap().trim();
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

            let mut line_iter = lines.into_iter().
                    skip(if lvl_name.is_none() { 1 } else { 2 }).
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
                match level {
                    Ok(level) => levels.push(level),
                    Err(err) => {
                        return Err(Box::new(LevelLoadingError::new(format!(
                            "\"{}\" occurred during parsing of level {} is invalid in the level pack file \"{path}\"!",
                            err, i + 1
                        ))));
                    },
                }
            }

            if line_iter.next().is_some() {
                return Err(Box::new(LevelLoadingError::new(format!(
                    "Additional data was found after last level was parsed in the level pack file \"{path}\"!"
                ))));
            }
        }

        let mut save_game_file = Game::get_or_create_save_game_folder()?;
        save_game_file.push(&id);
        save_game_file.push(".lvl.sav");

        let mut min_level_not_completed= Default::default();
        let mut level_stats: Vec<(Option<u64>, Option<u32>)> = vec![Default::default(); Self::MAX_LEVEL_COUNT_PER_PACK];
        'read_save_game: {
            if std::fs::exists(&save_game_file)? {
                let save_game_data = std::fs::read_to_string(&save_game_file)?;

                let lines = save_game_data.lines().collect::<Vec<_>>();
                if lines.is_empty() {
                    //TODO add warning message (could not load save file '&id + ".lvl.sav"')

                    break 'read_save_game;
                }

                let line = lines.first().unwrap().trim();

                min_level_not_completed = if let Ok(min_level_not_completed) = usize::from_str(line) {
                    min_level_not_completed
                }else {
                    //TODO add warning message (could not load save file '&id + ".lvl.sav"')

                    break 'read_save_game;
                };

                for (i, mut line) in lines.iter().
                        skip(1).
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
            levels,

            min_level_not_completed,
            level_pack_best_time_sum: Default::default(),
            level_pack_best_moves_sum: Default::default(),
        };
        level_pack.calculate_stats_sum();

        Ok(level_pack)
    }

    pub fn save_editor_level_pack(&self) -> Result<(), Box<dyn Error>> {
        self.save_editor_level_pack_to_path(&self.path)
    }

    pub fn save_editor_level_pack_to_path(&self, path: impl Into<String>) -> Result<(), Box<dyn Error>> {
        let mut file = File::create(path.into())?;

        writeln!(file, "Name: {}", self.name)?;
        writeln!(file, "Levels: {}", self.levels.len())?;

        for level in self.levels.iter().
                map(|level| level.level()) {
            write!(file, "\n{}", level.to_str())?;
        }
        file.flush()?;

        Ok(())
    }

    pub fn save_save_game(&self) -> Result<(), Box<dyn Error>> {
        let mut save_game_file = Game::get_or_create_save_game_folder()?;
        save_game_file.push(&self.id);
        save_game_file.push(".lvl.sav");

        let mut file = File::create(save_game_file)?;

        writeln!(file, "{}", self.min_level_not_completed)?;

        for level in self.levels.iter().
                take(self.min_level_not_completed) {
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

    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn path(&self) -> &str {
        &self.path
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

    fn calculate_stats_sum(&mut self) {
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
