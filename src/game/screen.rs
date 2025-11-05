use std::cmp::Ordering;
use std::fmt::Write as _;
use std::str::FromStr;
use std::time::SystemTime;
use dialog::DialogYesNo;
use crate::game::{audio, Game, GameState};
use crate::game::level::{Level, LevelPack, Tile};
use crate::game::screen::dialog::{DialogOk, DialogSelection, DialogYesCancelNo};
use crate::collections::UndoHistory;
use crate::io::{Color, Console, Key};

#[cfg(feature = "steam")]
use crate::game::steam::achievement::Achievement;

pub mod dialog;
pub mod utils;

#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub enum ScreenId {
    StartMenu,

    SelectLevelPack,
    SelectLevel,

    InGame,

    SelectLevelPackEditor,
    LevelPackEditor,
    LevelEditor,
}

#[allow(unused_variables)]
pub trait Screen {
    fn draw(&self, game_state: &GameState, console: &Console);

    fn update(&mut self, game_state: &mut GameState) {}

    fn on_key_pressed(&mut self, game_state: &mut GameState, key: Key) {}
    fn on_mouse_pressed(&mut self, game_state: &mut GameState, column: usize, row: usize) {}

    fn on_dialog_selection(&mut self, game_state: &mut GameState, selection: DialogSelection) {}

    fn on_continue(&mut self, game_state: &mut GameState) {}
    fn on_set_screen(&mut self, game_state: &mut GameState) {}
}

pub struct ScreenStartMenu {}

impl ScreenStartMenu {
    pub fn new() -> Self {
        Self {}
    }
}

impl Screen for ScreenStartMenu {
    fn draw(&self, _: &GameState, console: &Console) {
        //Draw border (top)
        console.set_color(Color::White, Color::Blue);
        console.draw_text(
            "/------------------------------------------------------------------------\\\n"
        );

        //Draw text
        console.set_color(Color::LightYellow, Color::Default);
        console.draw_text(
            "                -----------------------------------------\n                \
            .---- .---. |  ./ .---. .--.  .---. .   .\n                |     \
            |   | | /'  |   | |   : |   | |\\  |\n                '---. |   | :\
            {    |   | +---+ +---+ | \\ |\n                    | |   | | \\.  |   \
            | |   : |   | |  \\|\n                ----' '---' |  '\\ '---' '--'  \
            |   | '   '\n                ---------------------------------------\
            --\n\n\n\n\n\n-------------------------------------------------------\
            ------------------"
        );

        //Draw infos
        console.reset_color();
        let version = "Version: ".to_string() + Game::VERSION;
        console.set_cursor_pos(
            Game::CONSOLE_MIN_WIDTH - version.chars().count() - 3,
            14
        );
        console.draw_text(&version);

        console.set_cursor_pos(21, 16);
        console.draw_text("Press ");
        console.set_color(Color::LightRed, Color::Default);
        console.draw_text("ENTER");
        console.reset_color();
        console.draw_text(" to start the game!");

        console.set_cursor_pos(1, 21);
        console.draw_text("By ");
        console.set_color(Color::Default, Color::Yellow);
        console.draw_text("JDDev0");

        console.reset_color();
        console.set_cursor_pos(65, 21);
        console.draw_text("Help: ");
        console.set_color(Color::LightRed, Color::Default);
        console.draw_text("F1");

        //Draw border
        console.set_color(Color::White, Color::Blue);
        for i in 1..Game::CONSOLE_MIN_HEIGHT - 1 {
            console.set_cursor_pos(0, i);
            console.draw_text("|");

            console.set_cursor_pos(Game::CONSOLE_MIN_WIDTH - 1, i);
            console.draw_text("|");
        }
        console.draw_text("\n\\------------------------------------------------------------------------/");
    }

    fn on_key_pressed(&mut self, game_state: &mut GameState, key: Key) {
        if key == Key::ESC {
            game_state.open_dialog(Box::new(DialogYesNo::new("Exit game?")));

            return;
        }

        if key == Key::F1 {
            game_state.open_help_page();

            return;
        }

        if key == Key::ENTER {
            game_state.play_sound_effect_ui_select();

            game_state.set_screen(ScreenId::SelectLevelPack);
        }
    }

    fn on_mouse_pressed(&mut self, game_state: &mut GameState, column: usize, row: usize) {
        if row == 16 && column > 26 && column < 32 {
            self.on_key_pressed(game_state, Key::ENTER);
        }

        if row == 21 && column > 64 && column < 73 {
            game_state.open_help_page();
        }
    }

    fn on_dialog_selection(&mut self, game_state: &mut GameState, selection: DialogSelection) {
        if selection == DialogSelection::Yes {
            game_state.exit();
        }
    }
}

pub struct ScreenSelectLevelPack {}

impl ScreenSelectLevelPack {
    pub fn new() -> Self {
        Self {}
    }
}

impl Screen for ScreenSelectLevelPack {
    fn draw(&self, game_state: &GameState, console: &Console) {
        console.reset_color();
        console.set_underline(true);
        console.draw_text("Select a level pack:");
        console.set_underline(false);

        //Include Level Pack Editor entry
        let entry_count = game_state.get_level_pack_count() + 1;

        //Draw first line
        console.set_cursor_pos(0, 1);
        console.draw_text("-");
        let mut max = entry_count%24;
        if entry_count/24 > 0 {
            max = 24;
        }

        for i in 0..max  {
            let x = 1 + (i%24)*3;

            console.set_cursor_pos(x, 1);
            console.draw_text("---");
        }

        for i in 0..entry_count {
            let x = 1 + (i%24)*3;
            let y = 2 + (i/24)*2;

            //First box
            if x == 1 {
                console.set_cursor_pos(x - 1, y);
                console.draw_text("|");

                console.set_cursor_pos(x - 1, y + 1);
                console.draw_text("-");
            }

            console.set_cursor_pos(x, y);
            if i == game_state.get_level_pack_count() {
                //Level Pack Editor entry
                console.set_color(Color::White, Color::LightBlue);
                console.draw_text(" +");
            }else {
                console.set_color(Color::Black, if game_state.level_packs().get(i).
                        unwrap().level_pack_best_moves_sum().is_some() {
                    Color::Green
                }else {
                    Color::Yellow
                });
                console.draw_text(format!("{:2}", i + 1));
            }

            console.reset_color();
            console.draw_text("|");

            console.set_cursor_pos(x, y + 1);
            console.draw_text("---");
        }

        //Mark selected level
        let x = (game_state.get_level_pack_index()%24)*3;
        let y = 1 + (game_state.get_level_pack_index()/24)*2;

        console.set_color(Color::Cyan, Color::Default);
        console.set_cursor_pos(x, y);
        console.draw_text("----");
        console.set_cursor_pos(x, y + 1);
        console.draw_text("|");
        console.set_cursor_pos(x + 3, y + 1);
        console.draw_text("|");
        console.set_cursor_pos(x, y + 2);
        console.draw_text("----");

        //Draw border for best time and best moves
        let y = 4 + (entry_count/24)*2;

        console.set_cursor_pos(0, y);
        console.set_color(Color::Cyan, Color::Default);
        console.draw_text(".------------------------------------------------------------------------.");
        for i in 1..4 {
            console.set_cursor_pos(0, y + i);
            console.draw_text("|                                                                        |");
        }
        console.set_cursor_pos(0, y + 4);
        console.draw_text("\'------------------------------------------------------------------------\'");
        console.reset_color();

        if game_state.get_level_pack_index() == game_state.get_level_pack_count() {
            //Level Pack Editor entry
            console.set_cursor_pos(23, y + 2);
            console.draw_text("Create or edit level packs");
        }else {
            //Draw sum of best time and sum of best moves
            console.set_cursor_pos(1, y + 1);
            console.draw_text(format!("Selected level pack: {}", game_state.level_packs().get(game_state.get_level_pack_index()).unwrap().name()));
            console.set_cursor_pos(1, y + 2);
            console.draw_text("Sum of best time   : ");
            match game_state.get_current_level_pack().as_ref().unwrap().level_pack_best_time_sum() {
                None => console.draw_text("X:XX:XX:XX.XXX"),
                Some(best_time_sum) => {
                    console.draw_text(format!(
                        "{:01}:{:02}:{:02}:{:02}.{:03}",
                        best_time_sum/86400000,
                        (best_time_sum/3600000)%24,
                        (best_time_sum/60000)%60,
                        (best_time_sum/1000)%60,
                        best_time_sum%1000
                    ));
                },
            }
            console.set_cursor_pos(1, y + 3);
            console.draw_text("Sum of best moves  : ");
            match game_state.get_current_level_pack().as_ref().unwrap().level_pack_best_moves_sum() {
                None => console.draw_text("XXXXXXX"),
                Some(best_moves_sum) => console.draw_text(format!("{:07}", best_moves_sum)),
            }
        }
    }

    fn on_key_pressed(&mut self, game_state: &mut GameState, key: Key) {
        if key == Key::ESC {
            game_state.play_sound_effect(audio::UI_SELECT_EFFECT);

            game_state.set_screen(ScreenId::StartMenu);

            return;
        }

        if key == Key::F1 {
            game_state.open_help_page();

            return;
        }

        'outer: {
            //Include Level Pack Editor entry
            let entry_count = game_state.get_level_pack_count() + 1;

            match key {
                Key::LEFT => {
                    if game_state.current_level_pack_index == 0 {
                        break 'outer;
                    }

                    game_state.current_level_pack_index -= 1;
                },
                Key::UP => {
                    if game_state.current_level_pack_index <= 24 {
                        break 'outer;
                    }

                    game_state.current_level_pack_index -= 24;
                },
                Key::RIGHT => {
                    if game_state.current_level_pack_index + 1 >= entry_count {
                        break 'outer;
                    }

                    game_state.current_level_pack_index += 1;
                },
                Key::DOWN => {
                    if game_state.current_level_pack_index + 24 >= entry_count {
                        break 'outer;
                    }

                    game_state.current_level_pack_index += 24;
                },

                Key::ENTER => {
                    game_state.play_sound_effect_ui_select();

                    if game_state.get_level_pack_index() == game_state.get_level_pack_count() {
                        //Level Pack Editor entry
                        game_state.set_level_index(0);

                        game_state.set_screen(ScreenId::SelectLevelPackEditor);
                    }else {
                        //Set selected level
                        let min_level_not_completed = game_state.get_current_level_pack().as_ref().unwrap().min_level_not_completed();
                        if min_level_not_completed >= game_state.get_current_level_pack().as_ref().unwrap().level_count() {
                            game_state.set_level_index(0);
                        }else {
                            game_state.set_level_index(min_level_not_completed);
                        }

                        game_state.set_screen(ScreenId::SelectLevel);
                    }
                },

                _ => {},
            }
        }
    }

    fn on_mouse_pressed(&mut self, game_state: &mut GameState, column: usize, row: usize) {
        if row == 0 {
            return;
        }
        //Include Level Pack Editor entry
        let entry_count = game_state.get_level_pack_count() + 1;

        let level_pack_index = column/3 + (row - 1)/2*24;
        if level_pack_index < entry_count {
            game_state.set_level_pack_index(level_pack_index);
            self.on_key_pressed(game_state, Key::ENTER);
        }
    }
}

pub struct ScreenSelectLevel {
    selected_level: usize,
}

impl ScreenSelectLevel {
    pub fn new() -> Self {
        Self {
            selected_level: Default::default(),
        }
    }
}

impl Screen for ScreenSelectLevel {
    fn draw(&self, game_state: &GameState, console: &Console) {
        console.reset_color();
        console.set_underline(true);
        console.draw_text(format!("Select a level (Level pack \"{}\"):", game_state.get_current_level_pack().unwrap().name()));
        console.set_underline(false);

        let level_count = game_state.get_current_level_pack().as_ref().unwrap().level_count();

        //Draw first line
        console.set_cursor_pos(0, 1);
        console.draw_text("-");
        let mut max = level_count%24;
        if level_count/24 > 0 {
            max = 24;
        }
        for i in 0..max {
            let x = 1 + (i%24)*3;

            console.set_cursor_pos(x, 1);
            console.draw_text("---");
        }

        for i in 0..level_count {
            let x = 1 + (i%24)*3;
            let y = 2 + (i/24)*2;

            //First box
            if x == 1 {
                console.set_cursor_pos(x - 1, y);
                console.draw_text("|");

                console.set_cursor_pos(x - 1, y + 1);
                console.draw_text("-");
            }

            let min_level_not_completed = game_state.get_current_level_pack().as_ref().unwrap().min_level_not_completed();
            console.set_color(
                Color::Black,
                match i.cmp(&min_level_not_completed) {
                    Ordering::Less => Color::Green,
                    Ordering::Equal => Color::Yellow,
                    Ordering::Greater => Color::Red,
                }
            );
            console.set_cursor_pos(x, y);
            console.draw_text(utils::number_to_string_leading_ascii(2, i as u32 + 1, false));

            console.reset_color();
            console.draw_text("|");

            console.set_cursor_pos(x, y + 1);
            console.draw_text("---");
        }

        //Mark selected level
        let x = (self.selected_level%24)*3;
        let y = 1 + (self.selected_level/24)*2;

        console.set_color(Color::Cyan, Color::Default);
        console.set_cursor_pos(x, y);
        console.draw_text("----");
        console.set_cursor_pos(x, y + 1);
        console.draw_text("|");
        console.set_cursor_pos(x + 3, y + 1);
        console.draw_text("|");
        console.set_cursor_pos(x, y + 2);
        console.draw_text("----");

        //Draw border for best time and best moves
        let y = 4 + ((level_count - 1)/24)*2;

        console.set_cursor_pos(0, y);
        console.set_color(Color::Cyan, Color::Default);
        console.draw_text(".-------------------------.");
        for i in 1..4 {
            console.set_cursor_pos(0, y + i);
            console.draw_text("|                         |");
        }
        console.set_cursor_pos(0, y + 4);
        console.draw_text("\'-------------------------\'");

        //Draw best time and best moves
        console.reset_color();
        console.set_cursor_pos(1, y + 1);
        console.draw_text("Selected level:        ");
        let selected_level = self.selected_level;
        console.draw_text(utils::number_to_string_leading_ascii(2, selected_level as u32 + 1, true));

        console.set_cursor_pos(1, y + 2);
        console.draw_text("Best time     : ");
        match game_state.get_current_level_pack().as_ref().unwrap().levels().get(selected_level).unwrap().best_time() {
            None => console.draw_text("XX:XX.XXX"),
            Some(best_time) => {
                console.draw_text(format!(
                    "{:02}:{:02}.{:03}",
                    best_time/60000,
                    (best_time%60000)/1000,
                    best_time%1000
                ));
            },
        }
        console.set_cursor_pos(1, y + 3);
        console.draw_text("Best moves    :      ");
        match game_state.get_current_level_pack().as_ref().unwrap().levels().get(selected_level).unwrap().best_moves() {
            None => console.draw_text("XXXX"),
            Some(best_moves) => {
                console.draw_text(format!("{:04}", best_moves));
            },
        }
    }

    fn on_key_pressed(&mut self, game_state: &mut GameState, key: Key) {
        if key == Key::ESC {
            game_state.play_sound_effect(audio::UI_SELECT_EFFECT);

            game_state.set_screen(ScreenId::SelectLevelPack);

            return;
        }

        if key == Key::F1 {
            game_state.open_help_page();

            return;
        }

        'outer: {
            match key {
                Key::LEFT => {
                    if self.selected_level == 0 {
                        break 'outer;
                    }

                    self.selected_level -= 1;
                },
                Key::UP => {
                    if self.selected_level < 24 {
                        break 'outer;
                    }

                    self.selected_level -= 24;
                },
                Key::RIGHT => {
                    if self.selected_level + 1 >= game_state.get_current_level_pack().
                            as_ref().unwrap().level_count() {
                        break 'outer;
                    }

                    self.selected_level += 1;
                },
                Key::DOWN => {
                    if self.selected_level + 24 >= game_state.get_current_level_pack().
                            as_ref().unwrap().level_count() {
                        break 'outer;
                    }

                    self.selected_level += 24;
                },

                Key::ENTER => {
                    if self.selected_level <= game_state.get_current_level_pack().
                            as_ref().unwrap().min_level_not_completed() {
                        game_state.play_sound_effect_ui_select();

                        game_state.set_level_index(self.selected_level);
                        game_state.set_screen(ScreenId::InGame);
                    }else {
                        game_state.play_sound_effect_ui_error();
                    }
                },

                _ => {},
            }
        }
    }

    fn on_mouse_pressed(&mut self, game_state: &mut GameState, column: usize, row: usize) {
        if row == 0 {
            return;
        }

        let level_index = column/3 + (row - 1)/2*24;
        if level_index < game_state.get_current_level_pack().as_ref().unwrap().level_count() {
            self.selected_level = level_index;
            self.on_key_pressed(game_state, Key::ENTER);
        }
    }

    fn on_set_screen(&mut self, game_state: &mut GameState) {
        self.selected_level = game_state.get_level_index();
    }
}

pub struct ScreenInGame {
    time_start_in_menu: Option<SystemTime>,
    time_start: Option<SystemTime>,
    time_millis: u32,
    time_sec: u32,
    time_min: u32,

    level: Option<UndoHistory<(Level, (usize, usize))>>,

    continue_flag: bool,
    secret_found_flag: bool,
    game_over_flag: bool,
}

impl ScreenInGame {
    pub const UNDO_HISTORY_SIZE_PLAYING: usize = 10000;

    pub fn new() -> Self {
        Self {
            time_start_in_menu: Default::default(),
            time_start: Default::default(),
            time_millis: Default::default(),
            time_sec: Default::default(),
            time_min: Default::default(),

            level: Default::default(),

            continue_flag: Default::default(),
            secret_found_flag: Default::default(),
            game_over_flag: Default::default(),
        }
    }

    pub fn start_level(&mut self, level: &Level) {
        //Reset stats
        self.time_start = None;
        self.time_millis = 0;
        self.time_sec = 0;
        self.time_min = 0;

        let level = level.clone();

        self.continue_flag = false;
        self.game_over_flag = false;

        let mut player_pos = None;

        'outer:
        for i in 0..level.width() {
            for j in 0..level.height() {
                if let Some(tile) = level.get_tile(i, j) && *tile == Tile::Player {
                    player_pos = Some((i, j));

                    break 'outer;
                }
            }
        }

        self.level = Some(UndoHistory::new(Self::UNDO_HISTORY_SIZE_PLAYING, (level, player_pos.unwrap())));
    }

    fn draw_tutorial_level_text(&self, game_state: &GameState, console: &Console) {
        //Draw special help text for tutorial levels (tutorial pack and tutorial levels in special pack)
        if game_state.get_level_pack_index() == 0 { //Tutorial pack
            console.reset_color();
            match game_state.current_level_index {
                0 => {
                    if self.continue_flag {
                        console.set_cursor_pos(18, 8);
                        console.draw_text("Press ");

                        console.set_color(Color::Red, Color::Default);
                        console.draw_text("ENTER");

                        console.reset_color();
                        console.draw_text(" to go to the next level...");
                    }else {
                        console.set_cursor_pos(17, 8);
                        console.draw_text("Use the arrow keys (< ^ > v) to move...");
                    }
                },
                1 => {
                    console.set_cursor_pos(16, 8);
                    console.draw_text("Boxes (");

                    console.set_color(Color::LightCyan, Color::Default);
                    console.draw_text("@");

                    console.reset_color();
                    console.draw_text(") must be placed on ");

                    console.set_color(Color::Red, Color::Default);
                    console.draw_text("all");

                    console.reset_color();
                    console.draw_text(" goals (");

                    console.set_color(Color::Red, Color::Default);
                    console.draw_text("x");

                    console.reset_color();
                    console.draw_text(")");
                },
                2 => {
                    console.set_cursor_pos(14, 8);
                    console.draw_text("Some boxes (");

                    console.set_color(Color::LightPink, Color::Default);
                    console.draw_text("@");

                    console.reset_color();
                    console.draw_text(") might already be in a goal (");

                    console.set_color(Color::Red, Color::Default);
                    console.draw_text("x");

                    console.reset_color();
                    console.draw_text(")");
                },
                3 => {
                    console.set_cursor_pos(14, 8);
                    console.draw_text("Not all boxes (");

                    console.set_color(Color::LightCyan, Color::Default);
                    console.draw_text("@");

                    console.reset_color();
                    console.draw_text(") must be in a goal (");

                    console.set_color(Color::Red, Color::Default);
                    console.draw_text("x");

                    console.reset_color();
                    console.draw_text(") to win");
                },
                4 => {
                    console.set_cursor_pos(5, 8);
                    console.draw_text("One-way doors (");

                    console.set_color(Color::Blue, Color::Default);
                    console.draw_text("< ^ > v");

                    console.reset_color();
                    console.draw_text(") can only be entered from the opened side");
                },
                5 => {
                    if self.game_over_flag {
                        console.set_cursor_pos(12, 8);
                        console.draw_text("Press ");

                        console.set_color(Color::Red, Color::Default);
                        console.draw_text("ESC");

                        console.reset_color();
                        console.draw_text(" to go back to the level selection screen");
                    }else {
                        console.set_cursor_pos(8, 8);
                        console.draw_text("Boxes (");

                        console.set_color(Color::LightCyan, Color::Default);
                        console.draw_text("@");

                        console.reset_color();
                        console.draw_text(") cannot be moved through one-way doors (");

                        console.set_color(Color::Blue, Color::Default);
                        console.draw_text("< ^ > v");

                        console.reset_color();
                        console.draw_text(")");
                    }
                },
                _ => {},
            }
        }else if game_state.get_level_pack_index() == 2 { //Built-in special pack
            console.reset_color();
            match game_state.current_level_index {
                0 => {
                    console.set_cursor_pos(18, 8);
                    console.draw_text("Keys (");

                    console.set_color(Color::LightCyan, Color::Default);
                    console.draw_text("*");

                    console.reset_color();
                    console.draw_text(") can be used to open doors (");

                    console.set_color(Color::Red, Color::Default);
                    console.draw_text("=");

                    console.reset_color();
                    console.draw_text(")");
                },
                1 => {
                    console.set_cursor_pos(19, 8);
                    console.draw_text("Every key (");

                    console.set_color(Color::LightCyan, Color::Default);
                    console.draw_text("*");

                    console.reset_color();
                    console.draw_text(") can open any door (");

                    console.set_color(Color::Red, Color::Default);
                    console.draw_text("=");

                    console.reset_color();
                    console.draw_text(")");
                },
                2 => {
                    console.set_cursor_pos(21, 8);
                    console.draw_text("Keys (");

                    console.set_color(Color::LightPink, Color::Default);
                    console.draw_text("*");

                    console.reset_color();
                    console.draw_text(") might be in a goal (");

                    console.set_color(Color::Red, Color::Default);
                    console.draw_text("x");

                    console.reset_color();
                    console.draw_text(")");
                },
                8 => {
                    console.set_cursor_pos(23, 8);
                    console.draw_text("Holes (");

                    console.set_color(Color::LightBlue, Color::Default);
                    console.draw_text("O");

                    console.reset_color();
                    console.draw_text(") cannot be crossed");
                },
                9 => {
                    console.set_cursor_pos(21, 8);
                    console.draw_text("Filled holes (");

                    console.set_color(Color::Default, Color::LightBlue);
                    console.draw_text("@");

                    console.reset_color();
                    console.draw_text(") can be crossed");
                },
                10 => {
                    console.set_cursor_pos(23, 8);
                    console.draw_text("Boxes (");

                    console.set_color(Color::LightCyan, Color::Default);
                    console.draw_text("@");

                    console.reset_color();
                    console.draw_text(") can fill holes (");

                    console.set_color(Color::LightBlue, Color::Default);
                    console.draw_text("O");

                    console.reset_color();
                    console.draw_text(")");
                },
                11 => {
                    console.set_cursor_pos(13, 8);
                    console.draw_text("Keys (");

                    console.set_color(Color::LightCyan, Color::Default);
                    console.draw_text("*");

                    console.reset_color();
                    console.draw_text(") cannot fill holes (");

                    console.set_color(Color::LightBlue, Color::Default);
                    console.draw_text("O");

                    console.reset_color();
                    console.draw_text(") and will be lost");
                },
                _ => {},
            }
        }
    }
}

impl Screen for ScreenInGame {
    fn draw(&self, game_state: &GameState, console: &Console) {
        console.reset_color();
        console.draw_text(format!("Pack: {:02}", game_state.get_level_pack_index() + 1));

        console.set_cursor_pos(((Game::CONSOLE_MIN_WIDTH - 9) as f64 * 0.25) as usize, 0);
        console.draw_text("Level: ");
        console.draw_text(utils::number_to_string_leading_ascii(2, game_state.current_level_index as u32 + 1, true));

        console.set_cursor_pos(((Game::CONSOLE_MIN_WIDTH - 11) as f64 * 0.75) as usize, 0);
        console.draw_text(format!("Moves: {:04}", self.level.as_ref().unwrap().current_index()));

        console.set_cursor_pos(Game::CONSOLE_MIN_WIDTH - 15, 0);
        console.draw_text(format!(
            "Time: {:02}:{:02}.{:03}",
            self.time_min,
            self.time_sec,
            self.time_millis,
        ));

        if self.continue_flag {
            console.set_cursor_pos(((Game::CONSOLE_MIN_WIDTH - 16) as f64 * 0.5) as usize, 0);
            console.draw_text("Level completed!");
        }

        if self.game_over_flag {
            if self.secret_found_flag {
                console.set_cursor_pos(((Game::CONSOLE_MIN_WIDTH - 13) as f64 * 0.5) as usize, 0);
                console.draw_text("Secret found!");
            }else {
                console.set_cursor_pos(((Game::CONSOLE_MIN_WIDTH - 13) as f64 * 0.5) as usize, 0);
                console.draw_text("You have won!");
            }
        }

        if let Some(level) = self.level.as_ref().map(|level| &level.current().0) {
            let x_offset = ((Game::CONSOLE_MIN_WIDTH - level.width()) as f64 * 0.5) as usize;
            let y_offset = 1;

            level.draw(console, x_offset, y_offset, game_state.is_player_background(), None);

            self.draw_tutorial_level_text(game_state, console);
        }
    }

    fn update(&mut self, game_state: &mut GameState) {
        if game_state.is_dialog_opened() || self.game_over_flag || self.continue_flag {
            return;
        }

        if let Some(ref time_start) = self.time_start {
            let time_current = SystemTime::now();

            let diff = time_current.duration_since(*time_start).
                    expect("Time manipulation detected (Start time is in the future)!").
                    as_millis();

            self.time_millis = (diff % 1000) as u32;
            self.time_sec = (diff / 1000 % 60) as u32;
            self.time_min = (diff / 1000 / 60 % 60) as u32;

            if self.time_min >= 60 {
                self.time_millis = 999;
                self.time_sec = 59;
                self.time_min = 59;
            }
        }
    }

    fn on_key_pressed(&mut self, game_state: &mut GameState, key: Key) {
        if key == Key::ESC {
            if self.game_over_flag {
                self.continue_flag = false;
                self.game_over_flag = false;

                game_state.play_sound_effect(audio::UI_SELECT_EFFECT);

                game_state.set_screen(ScreenId::SelectLevel);

                return;
            }

            self.time_start_in_menu = Some(SystemTime::now());

            game_state.open_dialog(Box::new(DialogYesNo::new("Back to level selection?")));

            return;
        }

        if key == Key::F1 {
            self.time_start_in_menu = Some(SystemTime::now());

            game_state.open_help_page();

            return;
        }

        #[cfg(feature = "steam")]
        let steam_client = game_state.steam_client.clone();

        let current_level_index = game_state.current_level_index;
        let Some(level_pack) = game_state.get_current_level_pack_mut() else {
            return;
        };

        //Reset
        if key == Key::R {
            self.start_level(level_pack.levels()[current_level_index].level());

            game_state.play_sound_effect(audio::LEVEL_RESET);

            return;
        }

        //Level end
        if self.continue_flag {
            if key == Key::ENTER {
                self.continue_flag = false;

                #[cfg(feature = "steam")]
                if level_pack.id() == "main" && current_level_index == 95 {
                    Achievement::LEVEL_PACK_MAIN_LEVEL_96_COMPLETED.unlock(steam_client.clone());
                }

                #[cfg(feature = "steam")]
                if level_pack.level_pack_best_moves_sum().is_some() && level_pack.level_pack_best_time_sum().is_some() {
                    match level_pack.id() {
                        "tutorial" => {
                            Achievement::LEVEL_PACK_TUTORIAL_COMPLETED.unlock(steam_client.clone());

                            if level_pack.level_pack_best_time_sum().unwrap() < 4000 {
                                Achievement::LEVEL_PACK_TUTORIAL_FAST.unlock(steam_client.clone());
                            }
                        },

                        "main" => {
                            Achievement::LEVEL_PACK_MAIN_COMPLETED.unlock(steam_client.clone());
                        },

                        "special" => {
                            Achievement::LEVEL_PACK_SPECIAL_COMPLETED.unlock(steam_client.clone());
                        },

                        "demon" => {
                            Achievement::LEVEL_PACK_DEMON_COMPLETED.unlock(steam_client.clone());
                        },

                        "secret" => {
                            Achievement::LEVEL_PACK_SECRET_COMPLETED.unlock(steam_client.clone());
                        },

                        _ => {},
                    }
                }

                //All levels completed
                if current_level_index + 1 == level_pack.level_count() {
                    self.game_over_flag = true;

                    game_state.play_sound_effect(audio::LEVEL_PACK_COMPLETE_EFFECT);

                    return;
                }else {
                    game_state.current_level_index += 1;
                }

                self.start_level(game_state.get_current_level_pack().unwrap().levels()[game_state.current_level_index].level());
            }

            return;
        }

        //Prevent movement after level complete
        if self.game_over_flag {
            return;
        }

        if key == Key::Z {
            let level = self.level.as_mut().unwrap().undo();
            if level.is_some() {
                game_state.play_sound_effect(audio::UNDO_REDO_EFFECT);
            }

            return;
        }else if key == Key::Y {
            let level = self.level.as_mut().unwrap().redo();
            if level.is_some() {
                game_state.play_sound_effect(audio::UNDO_REDO_EFFECT);
            }

            return;
        }

        if key.is_arrow_key() {
            let (mut level, mut player_pos) = self.level.as_ref().unwrap().current().clone();

            let width = level.width();
            let height = level.height();

            let (x_from, y_from) = player_pos;

            let x_to = match key {
                Key::LEFT => if x_from == 0 {
                    width - 1
                }else {
                    x_from - 1
                },
                Key::RIGHT => if x_from == width - 1 {
                    0
                }else {
                    x_from + 1
                },
                _ => x_from,
            };
            let y_to = match key {
                Key::UP => if y_from == 0 {
                    height - 1
                }else {
                    y_from - 1
                },
                Key::DOWN => if y_from == height - 1 {
                    0
                }else {
                    y_from + 1
                },
                _ => y_from,
            };

            let one_way_door_tile = match key {
                Key::LEFT => Tile::OneWayLeft,
                Key::UP => Tile::OneWayUp,
                Key::RIGHT => Tile::OneWayRight,
                Key::DOWN => Tile::OneWayDown,
                _ => return, //Should never happen
            };

            //Set players old position to old level data
            let mut tile = level_pack.levels()[current_level_index].level().get_tile(x_from, y_from).unwrap().clone();
            if tile == Tile::Player || tile == Tile::Box || tile == Tile::Key || tile == Tile::LockedDoor {
                tile = Tile::Empty;
            }else if tile == Tile::BoxInGoal || tile == Tile::KeyInGoal {
                tile = Tile::Goal;
            }else if tile == Tile::Hole || tile == Tile::BoxInHole {
                tile = Tile::BoxInHole;
            }

            level.set_tile(x_from, y_from, tile);

            self.time_start.get_or_insert_with(SystemTime::now);

            let mut has_won = false;
            let tile = level.get_tile(x_to, y_to).unwrap().clone();
            if matches!(tile, Tile::Empty | Tile::Goal | Tile::Secret | Tile::BoxInHole) || tile == one_way_door_tile ||
                    matches!(tile, Tile::Box | Tile::BoxInGoal | Tile::Key | Tile::KeyInGoal if level.move_box_or_key(
                        level_pack.levels().get(current_level_index).unwrap().level(), &mut has_won, x_from, y_from, x_to, y_to)) {
                if tile == Tile::Secret {
                    self.game_over_flag = true;
                    self.secret_found_flag = true;
                }

                player_pos = (x_to, y_to);
            }

            //Set player to new position
            level.set_tile(player_pos.0, player_pos.1, Tile::Player);

            let has_player_moved = player_pos != (x_from, y_from);
            if has_player_moved {
                self.level.as_mut().unwrap().commit_change((level, player_pos));
            }

            if has_won {
                self.continue_flag = true;

                //Update best scores
                let time = self.time_millis as u64 + 1000 * self.time_sec as u64 + 60000 * self.time_min as u64;
                let moves = self.level.as_ref().unwrap().current_index() as u32;

                level_pack.update_stats(current_level_index, time, moves);

                if current_level_index >= level_pack.min_level_not_completed() {
                    level_pack.set_min_level_not_completed(current_level_index + 1);
                }

                if let Err(err) = level_pack.save_save_game() {
                    game_state.open_dialog(Box::new(DialogOk::new_error(format!("Cannot save: {}", err))));
                }

                game_state.play_sound_effect(audio::LEVEL_COMPLETE_EFFECT);
            }

            if has_player_moved {
                game_state.play_sound_effect(audio::STEP_EFFECT);
            }else {
                game_state.play_sound_effect(audio::NO_PATH_EFFECT);
            }

            if self.secret_found_flag {
                #[cfg(feature = "steam")]
                Achievement::LEVEL_PACK_SECRET_DISCOVERED.unlock(steam_client.clone());

                game_state.open_dialog(Box::new(DialogOk::new_secret_found("You have found a secret!")));

                if let Err(err) = game_state.on_found_secret() {
                    game_state.open_dialog(Box::new(DialogOk::new_error(format!("Error: {}", err))));
                }
            }
        }
    }

    fn on_dialog_selection(&mut self, game_state: &mut GameState, selection: DialogSelection) {
        if self.secret_found_flag {
            self.continue_flag = false;
            self.game_over_flag = false;
            self.secret_found_flag = false;

            game_state.set_screen(ScreenId::SelectLevelPack);

            return;
        }

        if selection == DialogSelection::Yes {
            self.continue_flag = false;
            self.game_over_flag = false;

            game_state.set_screen(ScreenId::SelectLevel);
        }else if selection == DialogSelection::No {
            self.on_continue(game_state);
        }
    }

    fn on_continue(&mut self, _: &mut GameState) {
        if self.game_over_flag || self.continue_flag || self.time_start.is_none() || self.time_start_in_menu.is_none() {
            return;
        }

        let diff = SystemTime::now().duration_since(self.time_start_in_menu.take().unwrap()).
                expect("Time manipulation detected (Start time is in the future)!");

        self.time_start = self.time_start.map(|time_start| time_start + diff);
    }

    fn on_set_screen(&mut self, game_state: &mut GameState) {
        self.start_level(game_state.get_current_level_pack().as_ref().unwrap().levels().get(
            game_state.get_level_index()).unwrap().level());
    }
}

pub struct ScreenSelectLevelPackEditor {
    is_exporting_level_pack: bool,
    is_deleting_level_pack: bool,

    is_creating_new_level_pack: bool,
    new_level_pack_id: String,
}

impl ScreenSelectLevelPackEditor {
    pub fn new() -> Self {
        Self {
            is_exporting_level_pack: Default::default(),
            is_deleting_level_pack: Default::default(),

            is_creating_new_level_pack: Default::default(),
            new_level_pack_id: String::new(),
        }
    }
}

impl Screen for ScreenSelectLevelPackEditor {
    fn draw(&self, game_state: &GameState, console: &Console) {
        console.reset_color();
        console.set_underline(true);
        console.draw_text("Edit a level pack:");
        console.set_underline(false);

        let has_max_level_pack_count = game_state.editor_state.get_level_pack_count() == LevelPack::MAX_LEVEL_PACK_COUNT;

        //Include Create Level Pack entry
        let entry_count = game_state.editor_state.get_level_pack_count() + 1;

        //Draw first line
        console.set_cursor_pos(0, 1);
        console.draw_text("-");
        let mut max = entry_count%24;
        if entry_count/24 > 0 {
            max = 24;
        }

        for i in 0..max  {
            let x = 1 + (i%24)*3;

            console.set_cursor_pos(x, 1);
            console.draw_text("---");
        }

        for i in 0..entry_count {
            let x = 1 + (i%24)*3;
            let y = 2 + (i/24)*2;

            //First box
            if x == 1 {
                console.set_cursor_pos(x - 1, y);
                console.draw_text("|");

                console.set_cursor_pos(x - 1, y + 1);
                console.draw_text("-");
            }

            console.set_cursor_pos(x, y);
            if i == game_state.editor_state.get_level_pack_count() {
                //Level Pack Editor entry
                if has_max_level_pack_count {
                    console.set_color(Color::White, Color::LightRed);
                }else {
                    console.set_color(Color::White, Color::LightBlue);
                }
                console.draw_text(" +");
            }else {
                console.set_color(Color::Black, Color::Green);
                console.draw_text(format!("{:2}", i + 1));
            }

            console.reset_color();
            console.draw_text("|");

            console.set_cursor_pos(x, y + 1);
            console.draw_text("---");
        }

        //Mark selected level
        let x = (game_state.editor_state.get_level_pack_index()%24)*3;
        let y = 1 + (game_state.editor_state.get_level_pack_index()/24)*2;

        console.set_color(Color::Cyan, Color::Default);
        console.set_cursor_pos(x, y);
        console.draw_text("----");
        console.set_cursor_pos(x, y + 1);
        console.draw_text("|");
        console.set_cursor_pos(x + 3, y + 1);
        console.draw_text("|");
        console.set_cursor_pos(x, y + 2);
        console.draw_text("----");

        //Draw border for best time and best moves
        let y = 4 + (entry_count/24)*2;

        console.set_cursor_pos(0, y);
        console.set_color(Color::Cyan, Color::Default);
        console.draw_text(".------------------------------------------------------------------------.");
        for i in 1..4 {
            console.set_cursor_pos(0, y + i);
            console.draw_text("|                                                                        |");
        }
        console.set_cursor_pos(0, y + 4);
        console.draw_text("\'------------------------------------------------------------------------\'");
        console.reset_color();

        if self.is_creating_new_level_pack {
            console.set_cursor_pos(1, y + 1);
            console.draw_text("Enter a new level pack ID:");

            console.set_cursor_pos(1, y + 2);
            console.set_color(Color::Cyan, Color::Default);
            console.draw_text(format!("> {}", &self.new_level_pack_id));
        }else if game_state.editor_state.get_level_pack_index() == game_state.editor_state.get_level_pack_count() {
            //Level Pack Editor entry
            if has_max_level_pack_count {
                let error_msg = format!(
                    "Max level pack count ({}) reached",
                    LevelPack::MAX_LEVEL_PACK_COUNT,
                );

                let x_offset = ((Game::CONSOLE_MIN_WIDTH - error_msg.len()) as f64 * 0.5) as usize;
                console.set_cursor_pos(x_offset, y + 2);
                console.set_color(Color::LightRed, Color::Default);
                console.draw_text(error_msg);
            }else {
                console.set_cursor_pos(28, y + 2);
                console.draw_text("Create a level pack");
            }
        }else {
            console.set_cursor_pos(1, y + 1);
            console.draw_text(format!("Level Pack ID: {}", game_state.editor_state.get_current_level_pack().unwrap().id()));

            console.set_cursor_pos(1, y + 2);
            console.draw_text(format!("Levels: {}", game_state.editor_state.get_current_level_pack().unwrap().level_count()));
        }
    }

    fn on_key_pressed(&mut self, game_state: &mut GameState, key: Key) {
        if self.is_creating_new_level_pack {
            match key {
                key if key.is_ascii() && (key.is_alphanumeric() || key == Key::UNDERSCORE || key == Key::MINUS) => {
                    if self.new_level_pack_id.len() >= LevelPack::MAX_LEVEL_PACK_NAME_LEN {
                        return;
                    }
                    
                    let _ = write!(self.new_level_pack_id, "{}", key.to_ascii().unwrap() as char);
                },
                Key::DELETE => {
                    self.new_level_pack_id.pop();
                },

                Key::ENTER => {
                    if self.new_level_pack_id.len() < 3 {
                        game_state.open_dialog(Box::new(DialogOk::new_error("Level pack ID must have at least 3 characters!")));

                        return;
                    }

                    for id in game_state.editor_state.level_packs.iter().
                            map(|level_pack| level_pack.id()) {
                        if id == self.new_level_pack_id {
                            game_state.open_dialog(Box::new(DialogOk::new_error(format!("The level pack with the ID \"{}\" already exists!", id))));

                            return;
                        }
                    }

                    let Ok(mut save_game_file) = Game::get_or_create_save_game_folder() else {
                        game_state.open_dialog(Box::new(DialogOk::new_error("Cannot save!")));

                        return;
                    };
                    save_game_file.push(&self.new_level_pack_id);
                    save_game_file.push(".lvl.edit");

                    let Some(save_game_file) = save_game_file.to_str() else {
                        game_state.open_dialog(Box::new(DialogOk::new_error("Cannot save!")));

                        return;
                    };

                    let level_pack = LevelPack::new(&self.new_level_pack_id, &self.new_level_pack_id, save_game_file);
                    if let Err(err) = level_pack.save_editor_level_pack() {
                        game_state.open_dialog(Box::new(DialogOk::new_error(format!("Cannot save: {}", err))));
                    }

                    game_state.play_sound_effect_ui_select();

                    let index = game_state.editor_state.level_packs.binary_search_by_key(
                        &level_pack.id().to_string(),
                        |level_pack| level_pack.id().to_string(),
                    ).err().unwrap();

                    game_state.editor_state.level_packs.insert(index, level_pack);

                    self.is_creating_new_level_pack = false;
                    self.new_level_pack_id = String::new();

                    game_state.editor_state.set_level_pack_index(index);
                    game_state.editor_state.set_level_index(0);
                    game_state.set_screen(ScreenId::LevelPackEditor);
                },

                Key::ESC => {
                    game_state.play_sound_effect(audio::UI_SELECT_EFFECT);

                    self.is_creating_new_level_pack = false;
                    self.new_level_pack_id = String::new();
                },

                _ => {},
            }

            return;
        }

        if key == Key::ESC {
            game_state.play_sound_effect(audio::UI_SELECT_EFFECT);

            game_state.set_screen(ScreenId::SelectLevelPack);

            return;
        }

        if key == Key::F1 {
            game_state.open_help_page();

            return;
        }

        if key == Key::E && game_state.editor_state.selected_level_pack_index != game_state.editor_state.get_level_pack_count() {
            self.is_exporting_level_pack = true;

            game_state.open_dialog(Box::new(DialogYesNo::new("Do you want to export the level pack to the current directory?")));
        }

        if key == Key::DELETE && game_state.editor_state.selected_level_pack_index != game_state.editor_state.get_level_pack_count() {
            self.is_deleting_level_pack = true;

            game_state.open_dialog(Box::new(DialogYesNo::new(format!(
                "Do you really want to delete level pack \"{}\"?",
                game_state.editor_state.get_current_level_pack().unwrap().id(),
            ))));
        }

        'outer: {
            //Include Level Pack Editor entry
            let entry_count = game_state.editor_state.get_level_pack_count() + 1;

            match key {
                Key::LEFT => {
                    if game_state.editor_state.selected_level_pack_index == 0 {
                        break 'outer;
                    }

                    game_state.editor_state.selected_level_pack_index -= 1;
                },
                Key::UP => {
                    if game_state.editor_state.selected_level_pack_index <= 24 {
                        break 'outer;
                    }

                    game_state.editor_state.selected_level_pack_index -= 24;
                },
                Key::RIGHT => {
                    if game_state.editor_state.selected_level_pack_index + 1 >= entry_count {
                        break 'outer;
                    }

                    game_state.editor_state.selected_level_pack_index += 1;
                },
                Key::DOWN => {
                    if game_state.editor_state.selected_level_pack_index + 24 >= entry_count {
                        break 'outer;
                    }

                    game_state.editor_state.selected_level_pack_index += 24;
                },

                Key::ENTER => {
                    if game_state.editor_state.selected_level_pack_index == game_state.editor_state.get_level_pack_count() {
                        //Level Pack Editor entry
                        if game_state.editor_state.get_level_pack_count() == LevelPack::MAX_LEVEL_PACK_COUNT {
                            game_state.open_dialog(Box::new(DialogOk::new_error(format!(
                                "Cannot create new level packs (Max level pack count ({}) reached)",
                                LevelPack::MAX_LEVEL_PACK_COUNT,
                            ))));
                        }else {
                            game_state.play_sound_effect_ui_select();

                            self.is_creating_new_level_pack = true;
                        }
                    }else {
                        game_state.play_sound_effect_ui_select();

                        //Set selected level pack
                        game_state.editor_state.set_level_index(0);
                        game_state.set_screen(ScreenId::LevelPackEditor);
                    }
                },

                _ => {},
            }
        }
    }

    fn on_mouse_pressed(&mut self, game_state: &mut GameState, column: usize, row: usize) {
        if row == 0 {
            return;
        }
        //Include Level Pack Editor entry
        let entry_count = game_state.editor_state.get_level_pack_count() + 1;

        let level_pack_index = column/3 + (row - 1)/2*24;
        if level_pack_index < entry_count {
            game_state.editor_state.selected_level_pack_index = level_pack_index;
            self.on_key_pressed(game_state, Key::ENTER);
        }
    }

    fn on_dialog_selection(&mut self, game_state: &mut GameState, selection: DialogSelection) {
        if self.is_exporting_level_pack {
            self.is_exporting_level_pack = false;

            if selection == DialogSelection::Yes {
                let level_pack = game_state.editor_state.get_current_level_pack().unwrap();
                let path = level_pack.id().to_string() + ".lvl";

                if std::fs::exists(&path).ok().is_none_or(|exists| exists) {
                    game_state.open_dialog(Box::new(DialogOk::new_error(format!(
                        "File \"{}\" already exists!",
                        path,
                    ))));

                    return;
                }

                if let Err(err) = level_pack.save_editor_level_pack_to_path(path) {
                    game_state.open_dialog(Box::new(DialogOk::new_error(format!("Cannot export: {}", err))));
                }else {
                    game_state.open_dialog(Box::new(DialogOk::new("The level pack was exported successfully")));
                }
            }
        }else if self.is_deleting_level_pack {
            self.is_deleting_level_pack = false;

            if selection == DialogSelection::Yes {
                let path = game_state.editor_state.get_current_level_pack().unwrap().path();

                if let Err(err) = std::fs::remove_file(path) {
                    game_state.open_dialog(Box::new(DialogOk::new_error(format!("Cannot delete: {}", err))));
                }else {
                    let index = game_state.editor_state.selected_level_pack_index;
                    game_state.editor_state.level_packs.remove(index);
                }
            }
        }
    }
}

pub struct ScreenLevelPackEditor {
    is_creating_new_level: bool,
    is_editing_height: bool,
    is_deleting_level: bool,
    new_level_width_str: String,
    new_level_height_str: String,
}

impl ScreenLevelPackEditor {
    pub fn new() -> Self {
        Self {
            is_creating_new_level: Default::default(),
            is_editing_height: Default::default(),
            is_deleting_level: Default::default(),
            new_level_width_str: String::new(),
            new_level_height_str: String::new(),
        }
    }
}

impl Screen for ScreenLevelPackEditor {
    fn draw(&self, game_state: &GameState, console: &Console) {
        console.reset_color();
        console.set_underline(true);
        console.draw_text(format!("Edit a level (Level pack \"{}\"):", game_state.editor_state.get_current_level_pack().unwrap().id()));
        console.set_underline(false);

        let has_max_level_count = game_state.editor_state.get_current_level_pack().unwrap().level_count() == LevelPack::MAX_LEVEL_COUNT_PER_PACK;

        //Include Create Level entry
        let entry_count = game_state.editor_state.get_current_level_pack().unwrap().level_count() + 1;

        //Draw first line
        console.set_cursor_pos(0, 1);
        console.draw_text("-");
        let mut max = entry_count%24;
        if entry_count/24 > 0 {
            max = 24;
        }

        for i in 0..max  {
            let x = 1 + (i%24)*3;

            console.set_cursor_pos(x, 1);
            console.draw_text("---");
        }

        for i in 0..entry_count {
            let x = 1 + (i%24)*3;
            let y = 2 + (i/24)*2;

            //First box
            if x == 1 {
                console.set_cursor_pos(x - 1, y);
                console.draw_text("|");

                console.set_cursor_pos(x - 1, y + 1);
                console.draw_text("-");
            }

            console.set_cursor_pos(x, y);
            if i == game_state.editor_state.get_current_level_pack().unwrap().level_count() {
                //Level Editor entry
                if has_max_level_count {
                    console.set_color(Color::White, Color::LightRed);
                }else {
                    console.set_color(Color::White, Color::LightBlue);
                }
                console.draw_text(" +");
            }else {
                console.set_color(Color::Black, Color::Green);
                console.draw_text(utils::number_to_string_leading_ascii(2, i as u32 + 1, false));
            }

            console.reset_color();
            console.draw_text("|");

            console.set_cursor_pos(x, y + 1);
            console.draw_text("---");
        }

        //Mark selected level
        let x = (game_state.editor_state.get_level_index()%24)*3;
        let y = 1 + (game_state.editor_state.get_level_index()/24)*2;

        console.set_color(Color::Cyan, Color::Default);
        console.set_cursor_pos(x, y);
        console.draw_text("----");
        console.set_cursor_pos(x, y + 1);
        console.draw_text("|");
        console.set_cursor_pos(x + 3, y + 1);
        console.draw_text("|");
        console.set_cursor_pos(x, y + 2);
        console.draw_text("----");

        //Draw border for best time and best moves
        let y = 4 + (entry_count/24)*2;

        console.set_cursor_pos(0, y);
        console.set_color(Color::Cyan, Color::Default);
        console.draw_text(".------------------------------------------------------------------------.");
        for i in 1..4 {
            console.set_cursor_pos(0, y + i);
            console.draw_text("|                                                                        |");
        }
        console.set_cursor_pos(0, y + 4);
        console.draw_text("\'------------------------------------------------------------------------\'");
        console.reset_color();

        if self.is_creating_new_level {
            console.set_cursor_pos(1, y + 1);
            console.draw_text("Enter width and height for new level:");

            console.set_color(if self.is_editing_height {
                Color::LightBlue
            }else {
                Color::Cyan
            }, Color::Default);
            console.set_cursor_pos(1, y + 2);
            console.draw_text(format!("Width: {}", &self.new_level_width_str));

            console.set_color(if self.is_editing_height {
                Color::Cyan
            }else {
                Color::LightBlue
            }, Color::Default);
            console.set_cursor_pos(14, y + 2);
            console.draw_text(format!("Height: {}", &self.new_level_height_str));
        }else if game_state.editor_state.get_level_index() == game_state.editor_state.get_current_level_pack().unwrap().level_count() {
            //Level Editor entry
            if has_max_level_count {
                let error_msg = format!(
                    "Max level count ({}) reached",
                    LevelPack::MAX_LEVEL_COUNT_PER_PACK,
                );

                let x_offset = ((Game::CONSOLE_MIN_WIDTH - error_msg.len()) as f64 * 0.5) as usize;
                console.set_cursor_pos(x_offset, y + 2);
                console.set_color(Color::LightRed, Color::Default);
                console.draw_text(error_msg);
            }else {
                console.set_cursor_pos(31, y + 2);
                console.draw_text("Create level");
            }
        }else {
            //Draw best time and best moves
            console.set_cursor_pos(1, y + 1);
            console.draw_text("Selected level: ");
            let selected_level = game_state.editor_state.selected_level_index;
            console.draw_text(utils::number_to_string_leading_ascii(2, selected_level as u32 + 1, true));

            console.set_cursor_pos(1, y + 2);
            console.draw_text(format!(
                "Size: {} x {}",
                game_state.editor_state.get_current_level().unwrap().width(),
                game_state.editor_state.get_current_level().unwrap().height(),
            ));
        }
    }

    fn on_key_pressed(&mut self, game_state: &mut GameState, key: Key) {
        if self.is_creating_new_level {
            match key {
                key if key.is_ascii() && key.is_numeric() => {
                    if self.is_editing_height {
                        if self.new_level_height_str.len() >= 2 {
                            return;
                        }

                        let _ = write!(self.new_level_height_str, "{}", key.to_ascii().unwrap() as char);
                    }else {
                        if self.new_level_width_str.len() >= 2 {
                            return;
                        }

                        let _ = write!(self.new_level_width_str, "{}", key.to_ascii().unwrap() as char);
                    }
                },
                Key::DELETE => {
                    if self.is_editing_height {
                        self.new_level_height_str.pop();
                    }else {
                        self.new_level_width_str.pop();
                    }
                },

                Key::TAB => {
                    self.is_editing_height = !self.is_editing_height;
                },

                Key::ENTER => {
                    if !(1..=2).contains(&self.new_level_width_str.len()) {
                        game_state.open_dialog(Box::new(DialogOk::new_error(format!("Width must be >= 3 and <= {}!", Game::LEVEL_MAX_WIDTH))));

                        return;
                    }

                    let Ok(width) = usize::from_str(&self.new_level_width_str) else {
                        game_state.open_dialog(Box::new(DialogOk::new_error("Width must be a number")));

                        return;
                    };

                    if !(3..=Game::LEVEL_MAX_WIDTH).contains(&width) {
                        game_state.open_dialog(Box::new(DialogOk::new_error(format!("Width must be >= 3 and <= {}!", Game::LEVEL_MAX_WIDTH))));

                        return;
                    }

                    if self.new_level_height_str.is_empty() && !self.is_editing_height {
                        self.is_editing_height = true;

                        return;
                    }

                    if !(1..=2).contains(&self.new_level_height_str.len()) {
                        game_state.open_dialog(Box::new(DialogOk::new_error(format!("Height must be >= 3 and <= {}!", Game::LEVEL_MAX_HEIGHT))));

                        return;
                    }

                    let Ok(height) = usize::from_str(&self.new_level_height_str) else {
                        game_state.open_dialog(Box::new(DialogOk::new_error("Height must be a number")));

                        return;
                    };

                    if !(3..=Game::LEVEL_MAX_HEIGHT).contains(&height) {
                        game_state.open_dialog(Box::new(DialogOk::new_error(format!("Height must be >= 3 and <= {}!", Game::LEVEL_MAX_HEIGHT))));

                        return;
                    }

                    game_state.play_sound_effect_ui_select();

                    game_state.editor_state.get_current_level_pack_mut().unwrap().add_level(Level::new(width, height));

                    self.is_creating_new_level = false;
                    self.is_editing_height = false;
                    self.new_level_width_str = String::new();
                    self.new_level_height_str = String::new();

                    game_state.set_screen(ScreenId::LevelEditor);
                },

                Key::ESC => {
                    game_state.play_sound_effect(audio::UI_SELECT_EFFECT);

                    self.is_creating_new_level = false;
                    self.is_editing_height = false;
                    self.new_level_width_str = String::new();
                    self.new_level_height_str = String::new();
                },

                _ => {},
            }

            return;
        }

        if key == Key::ESC {
            game_state.play_sound_effect(audio::UI_SELECT_EFFECT);

            game_state.set_screen(ScreenId::SelectLevelPackEditor);

            return;
        }

        if key == Key::F1 {
            game_state.open_help_page();

            return;
        }

        'outer: {
            //Include Level Pack Editor entry
            let entry_count = game_state.editor_state.get_current_level_pack().unwrap().level_count() + 1;

            match key {
                Key::LEFT => {
                    if game_state.editor_state.selected_level_index == 0 {
                        break 'outer;
                    }

                    game_state.editor_state.selected_level_index -= 1;
                },
                Key::UP => {
                    if game_state.editor_state.selected_level_index <= 24 {
                        break 'outer;
                    }

                    game_state.editor_state.selected_level_index -= 24;
                },
                Key::RIGHT => {
                    if game_state.editor_state.selected_level_index + 1 >= entry_count {
                        break 'outer;
                    }

                    game_state.editor_state.selected_level_index += 1;
                },
                Key::DOWN => {
                    if game_state.editor_state.selected_level_index + 24 >= entry_count {
                        break 'outer;
                    }

                    game_state.editor_state.selected_level_index += 24;
                },

                Key::ENTER => {
                    if game_state.editor_state.selected_level_index == game_state.editor_state.get_current_level_pack().unwrap().level_count() {
                        //Level Editor entry
                        if game_state.editor_state.get_current_level_pack().unwrap().level_count() == LevelPack::MAX_LEVEL_COUNT_PER_PACK {
                            game_state.open_dialog(Box::new(DialogOk::new_error(format!(
                                "Cannot create level packs (Max level count ({}) reached)",
                                LevelPack::MAX_LEVEL_COUNT_PER_PACK,
                            ))));
                        }else {
                            game_state.play_sound_effect_ui_select();

                            self.is_creating_new_level = true;
                        }
                    }else {
                        game_state.play_sound_effect_ui_select();

                        //Set selected level
                        game_state.set_screen(ScreenId::LevelEditor);
                    }
                },

                Key::DELETE => {
                    if game_state.editor_state.selected_level_index != game_state.editor_state.get_current_level_pack().unwrap().level_count() {
                        self.is_deleting_level = true;

                        game_state.open_dialog(Box::new(DialogYesNo::new(format!("Do you really want to delete level {}?", game_state.editor_state.selected_level_index + 1))));
                    }
                },

                _ => {},
            }
        }
    }

    fn on_mouse_pressed(&mut self, game_state: &mut GameState, column: usize, row: usize) {
        if row == 0 {
            return;
        }

        //Include create Level entry
        let entry_count = game_state.editor_state.get_current_level_pack().unwrap().level_count() + 1;

        let level_pack_index = column/3 + (row - 1)/2*24;
        if level_pack_index < entry_count {
            game_state.editor_state.selected_level_index = level_pack_index;
            self.on_key_pressed(game_state, Key::ENTER);
        }
    }

    fn on_dialog_selection(&mut self, game_state: &mut GameState, selection: DialogSelection) {
        if self.is_deleting_level {
            self.is_deleting_level = false;

            if selection == DialogSelection::Yes {
                let index = game_state.editor_state.selected_level_index;
                game_state.editor_state.get_current_level_pack_mut().unwrap().levels_mut().remove(index);
                if let Err(err) = game_state.editor_state.get_current_level_pack().unwrap().save_editor_level_pack() {
                    game_state.open_dialog(Box::new(DialogOk::new_error(format!("Cannot save: {}", err))));
                }
            }
        }
    }
}

pub struct ScreenLevelEditor {
    level: UndoHistory<Level>,
    is_vertical_input: bool,
    is_reverse_input: bool,
    playing_level: Option<UndoHistory<(Level, (usize, usize))>>,
    cursor_pos: (usize, usize),
}

impl ScreenLevelEditor {
    pub const UNDO_HISTORY_SIZE: usize = 256;
    pub const UNDO_HISTORY_SIZE_PLAYING: usize = 10000;

    pub fn new() -> Self {
        Self {
            level: UndoHistory::new(Self::UNDO_HISTORY_SIZE, Level::new(1, 1)),
            is_vertical_input: Default::default(),
            is_reverse_input: Default::default(),
            playing_level: Default::default(),
            cursor_pos: Default::default(),
        }
    }

    fn on_key_pressed_playing(&mut self, game_state: &mut GameState, key: Key) {
        if let Some(level_history) = self.playing_level.as_mut() {
            if matches!(key, Key::Z | Key::Y) {
                let is_undo = key == Key::Z;

                let level = if is_undo {
                    level_history.undo()
                }else {
                    level_history.redo()
                };

                if level.is_some() {
                    game_state.play_sound_effect(audio::UNDO_REDO_EFFECT);
                }
            }

            if key.is_arrow_key() {
                let (mut level, mut player_pos) = level_history.current().clone();

                let width = level.width();
                let height = level.height();

                let (x_from, y_from) = player_pos;

                let x_to = match key {
                    Key::LEFT => if x_from == 0 {
                        width - 1
                    }else {
                        x_from - 1
                    },
                    Key::RIGHT => if x_from == width - 1 {
                        0
                    }else {
                        x_from + 1
                    },
                    _ => x_from,
                };
                let y_to = match key {
                    Key::UP => if y_from == 0 {
                        height - 1
                    }else {
                        y_from - 1
                    },
                    Key::DOWN => if y_from == height - 1 {
                        0
                    }else {
                        y_from + 1
                    },
                    _ => y_from,
                };

                let one_way_door_tile = match key {
                    Key::LEFT => Tile::OneWayLeft,
                    Key::UP => Tile::OneWayUp,
                    Key::RIGHT => Tile::OneWayRight,
                    Key::DOWN => Tile::OneWayDown,
                    _ => return, //Should never happen
                };

                //Set players old position to old level data
                let mut tile = self.level.current().get_tile(x_from, y_from).unwrap().clone();
                if tile == Tile::Player || tile == Tile::Box || tile == Tile::Key || tile == Tile::LockedDoor {
                    tile = Tile::Empty;
                }else if tile == Tile::BoxInGoal || tile == Tile::KeyInGoal {
                    tile = Tile::Goal;
                }else if tile == Tile::Hole || tile == Tile::BoxInHole {
                    tile = Tile::BoxInHole;
                }

                level.set_tile(x_from, y_from, tile);

                let mut has_won = false;
                let tile = level.get_tile(x_to, y_to).unwrap().clone();
                if matches!(tile, Tile::Empty | Tile::Goal | Tile::Secret | Tile::BoxInHole) || tile == one_way_door_tile ||
                        matches!(tile, Tile::Box | Tile::BoxInGoal | Tile::Key | Tile::KeyInGoal if level.move_box_or_key(
                            self.level.current(), &mut has_won, x_from, y_from, x_to, y_to)) {
                    player_pos = (x_to, y_to);
                }

                //Set player to new position
                level.set_tile(player_pos.0, player_pos.1, Tile::Player);

                if player_pos != (x_from, y_from) {
                    level_history.commit_change((level, player_pos));

                    game_state.play_sound_effect(audio::STEP_EFFECT);
                }else {
                    game_state.play_sound_effect(audio::NO_PATH_EFFECT);
                }
            }
        }
    }

    fn on_key_pressed_editing(&mut self, game_state: &mut GameState, key: Key) {
        match key {
            Key::LEFT => {
                if self.cursor_pos.0 > 0 {
                    self.cursor_pos.0 -= 1;
                }else {
                    self.cursor_pos.0 = self.level.current().width() - 1;
                }
            },
            Key::UP => {
                if self.cursor_pos.1 > 0 {
                    self.cursor_pos.1 -= 1;
                }else {
                    self.cursor_pos.1 = self.level.current().height() - 1;
                }
            },
            Key::RIGHT => {
                if self.cursor_pos.0 < self.level.current().width() - 1 {
                    self.cursor_pos.0 += 1;
                }else {
                    self.cursor_pos.0 = 0;
                }
            },
            Key::DOWN => {
                if self.cursor_pos.1 < self.level.current().height() - 1 {
                    self.cursor_pos.1 += 1;
                }else {
                    self.cursor_pos.1 = 0;
                }
            },

            Key::DELETE => {
                if self.is_vertical_input {
                    if self.level.current().width() == 3 {
                        game_state.open_dialog(Box::new(DialogOk::new_error(format!(
                            "Level width limit reached (min: {})",
                            3,
                        ))));

                        return;
                    }

                    let index = self.cursor_pos.0;

                    let level_orig = self.level.current().clone();
                    let mut new_level = Level::new(level_orig.width() - 1, level_orig.height());

                    if index == new_level.width() {
                        self.cursor_pos.0 -= 1;
                    }

                    for i in 0..level_orig.height() {
                        for mut j in 0..level_orig.width() {
                            let tile = level_orig.get_tile(j, i).unwrap().clone();

                            if j == index {
                                continue;
                            }

                            if j >= index {
                                j -= 1;
                            }

                            new_level.set_tile(j, i, tile);
                        }
                    }

                    self.level.commit_change(new_level);
                }else {
                    if self.level.current().height() == 3 {
                        game_state.open_dialog(Box::new(DialogOk::new_error(format!(
                            "Level height limit reached (min: {})",
                            3,
                        ))));

                        return;
                    }

                    let index = self.cursor_pos.1;

                    let level_orig = self.level.current().clone();
                    let mut new_level = Level::new(level_orig.width(), level_orig.height() - 1);

                    if index == new_level.height() {
                        self.cursor_pos.1 -= 1;
                    }

                    for i in 0..level_orig.width() {
                        for mut j in 0..level_orig.height() {
                            let tile = level_orig.get_tile(i, j).unwrap().clone();

                            if j == index {
                                continue;
                            }

                            if j >= index {
                                j -= 1;
                            }

                            new_level.set_tile(i, j, tile);
                        }
                    }

                    self.level.commit_change(new_level);
                }
            },

            Key::W | Key::A | Key::S | Key::D => {
                self.is_vertical_input = key == Key::W || key == Key::S;
                self.is_reverse_input = key == Key::W || key == Key::A;
            },

            Key::C | Key::I => {
                let is_copy = key == Key::C;

                if self.is_vertical_input {
                    if self.level.current().height() == Game::LEVEL_MAX_HEIGHT {
                        game_state.open_dialog(Box::new(DialogOk::new_error(format!(
                            "Level height limit reached (max: {})",
                            Game::LEVEL_MAX_HEIGHT,
                        ))));

                        return;
                    }

                    let index_orig = self.cursor_pos.1;
                    if !self.is_reverse_input {
                        self.cursor_pos.1 += 1;
                    }
                    let index = self.cursor_pos.1;

                    let level_orig = self.level.current().clone();
                    let mut new_level = Level::new(level_orig.width(), level_orig.height() + 1);

                    for i in 0..level_orig.width() {
                        for mut j in 0..level_orig.height() {
                            let tile = level_orig.get_tile(i, j).unwrap().clone();

                            if j >= index {
                                j += 1;
                            }

                            new_level.set_tile(i, j, tile);
                        }

                        let tile = if is_copy {
                            level_orig.get_tile(i, index_orig).unwrap().clone()
                        }else {
                            Tile::Empty
                        };
                        new_level.set_tile(i, index, tile);
                    }

                    self.level.commit_change(new_level);
                }else {
                    if self.level.current().width() == Game::LEVEL_MAX_WIDTH {
                        game_state.open_dialog(Box::new(DialogOk::new_error(format!(
                            "Level width limit reached (max: {})",
                            Game::LEVEL_MAX_WIDTH,
                        ))));

                        return;
                    }

                    let index_orig = self.cursor_pos.0;
                    if !self.is_reverse_input {
                        self.cursor_pos.0 += 1;
                    }
                    let index = self.cursor_pos.0;

                    let level_orig = self.level.current().clone();
                    let mut new_level = Level::new(level_orig.width() + 1, level_orig.height());

                    for i in 0..level_orig.height() {
                        for mut j in 0..level_orig.width() {
                            let tile = level_orig.get_tile(j, i).unwrap().clone();

                            if j >= index {
                                j += 1;
                            }

                            new_level.set_tile(j, i, tile);
                        }

                        let tile = if is_copy {
                            level_orig.get_tile(index_orig, i).unwrap().clone()
                        }else {
                            Tile::Empty
                        };
                        new_level.set_tile(index, i, tile);
                    }

                    self.level.commit_change(new_level);
                }
            },

            Key::Z | Key::Y => {
                let is_undo = key == Key::Z;

                let level = if is_undo {
                    self.level.undo()
                }else {
                    self.level.redo()
                };

                if let Some(level) = level {
                    if self.cursor_pos.0 >= level.width() {
                        self.cursor_pos.0 = level.width() - 1;
                    }

                    if self.cursor_pos.1 >= level.height() {
                        self.cursor_pos.1 = level.height() - 1;
                    }
                }
            },

            key if key.is_ascii() => {
                if let Ok(tile_input) = Tile::from_ascii(key.to_ascii().unwrap()) && tile_input != Tile::Secret {
                    let mut level = self.level.current().clone();
                    let tile = level.get_tile_mut(self.cursor_pos.0, self.cursor_pos.1).unwrap();

                    if *tile != tile_input {
                        *tile = tile_input;

                        self.level.commit_change(level);
                    }
                }

                if self.is_vertical_input {
                    self.on_key_pressed_editing(game_state, if self.is_reverse_input {
                        Key::UP
                    }else {
                        Key::DOWN
                    });
                }else {
                    self.on_key_pressed_editing(game_state, if self.is_reverse_input {
                        Key::LEFT
                    }else {
                        Key::RIGHT
                    });
                }
            },

            _ => {},
        }
    }
}

impl Screen for ScreenLevelEditor {
    fn draw(&self, game_state: &GameState, console: &Console) {
        console.reset_color();
        if let Some(level_history) = &self.playing_level {
            console.draw_text("Playing");

            console.set_cursor_pos(((Game::CONSOLE_MIN_WIDTH - 11) as f64 * 0.75) as usize, 0);
            console.draw_text(format!("Moves: {:04}", level_history.current_index()));
        }else {
            console.draw_text(format!(
                "Editing ({})",
                match self.is_vertical_input {
                    true if self.is_reverse_input => "^",
                    true => "v",
                    false if self.is_reverse_input => "<",
                    false => ">",
                }
            ));

            console.set_cursor_pos(((Game::CONSOLE_MIN_WIDTH - 14) as f64 * 0.5) as usize, 0);
            console.draw_text(format!("Cursor ({:02}:{:02})", self.cursor_pos.0 + 1, self.cursor_pos.1 + 1));
        }

        let x_offset = ((Game::CONSOLE_MIN_WIDTH - self.level.current().width()) as f64 * 0.5) as usize;
        let y_offset = 1;

        self.playing_level.as_ref().map_or(self.level.current(), |level| &level.current().0).
                draw(console, x_offset, y_offset, game_state.is_player_background(),
                     self.playing_level.as_ref().map_or(Some(self.cursor_pos), |_| None));
    }

    fn on_key_pressed(&mut self, game_state: &mut GameState, key: Key) {
        if key == Key::ESC {
            game_state.open_dialog(Box::new(DialogYesCancelNo::new("Exiting (Save changes?)")));

            return;
        }

        if key == Key::F1 {
            game_state.open_help_page();

            return;
        }

        if key == Key::R {
            self.playing_level = if self.playing_level.is_some() {
                game_state.play_sound_effect(audio::LEVEL_RESET);

                None
            }else {
                let player_tile_count = self.level.current().tiles().iter().filter(|tile| **tile == Tile::Player).count();
                if player_tile_count == 0 {
                    game_state.open_dialog(Box::new(DialogOk::new_error("Level does not contain a player tile!")));

                    return;
                }else if player_tile_count > 1 {
                    game_state.open_dialog(Box::new(DialogOk::new_error("Level contains too many player tiles!")));

                    return;
                }

                let mut player_pos = None;

                'outer:
                for i in 0..self.level.current().width() {
                    for j in 0..self.level.current().height() {
                        if let Some(tile) = self.level.current().get_tile(i, j) && *tile == Tile::Player {
                            player_pos = Some((i, j));

                            break 'outer;
                        }
                    }
                }

                Some(UndoHistory::new(Self::UNDO_HISTORY_SIZE_PLAYING, (self.level.current().clone(), player_pos.unwrap())))
            };

            return;
        }

        if self.playing_level.is_none() {
            self.on_key_pressed_editing(game_state, key);
        }else {
            self.on_key_pressed_playing(game_state, key);
        }
    }

    fn on_mouse_pressed(&mut self, _: &mut GameState, column: usize, row: usize) {
        if row == 0 || self.playing_level.is_some() {
            return;
        }

        let x_offset = ((Game::CONSOLE_MIN_WIDTH - self.level.current().width()) as f64 * 0.5) as usize;
        let y_offset = 1;

        if column < x_offset {
            return;
        }

        let x = column - x_offset;
        if x >= self.level.current().width() {
            return;
        }

        let y = row - y_offset;
        if y >= self.level.current().height() {
            return;
        }

        self.cursor_pos = (x, y);
    }

    fn on_dialog_selection(&mut self, game_state: &mut GameState, selection: DialogSelection) {
        if selection == DialogSelection::Yes {
            *game_state.editor_state.get_current_level_mut().unwrap() = self.level.current().clone();
            if let Err(err) = game_state.editor_state.get_current_level_pack().unwrap().save_editor_level_pack() {
                game_state.open_dialog(Box::new(DialogOk::new_error(format!("Cannot save: {}", err))));
            }

            self.level.clear();
            game_state.set_screen(ScreenId::LevelPackEditor);
        }else if selection == DialogSelection::No {
            self.level.clear();
            game_state.set_screen(ScreenId::LevelPackEditor);
        }

        //Cancel: Close dialog without doing anything
    }

    fn on_set_screen(&mut self, game_state: &mut GameState) {
        self.is_vertical_input = false;
        self.is_reverse_input = false;
        self.playing_level = None;
        self.cursor_pos = (0, 0);

        self.level.clear_with_new_initial(game_state.editor_state.get_current_level().unwrap().clone());
    }
}
