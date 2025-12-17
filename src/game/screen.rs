use std::cmp::Ordering;
use std::time::SystemTime;
use crate::game::{audio, Game, GameState, TileMode};
use crate::game::level::{Direction, Level, MoveResult, PlayingLevel, Tile};
use crate::game::screen::dialog::{Dialog, DialogSelection};
use crate::game::console_extension::ConsoleExtension;
use crate::game::screen::components::{Rect, UIList, UIListElement};
use crate::io::{Color, Console, Key};

#[cfg(feature = "steam")]
use crate::game::steam::stats::*;

pub mod dialog;
pub mod utils;
pub mod components;

#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub enum ScreenId {
    StartMenu,
    About,
    Settings,

    SelectLevel,

    InGame,
}

#[allow(unused_variables)]
pub trait Screen {
    fn draw(&self, game_state: &GameState, console: &Console);

    fn update(&mut self, game_state: &mut GameState) {}
    fn animate(&mut self, game_state: &mut GameState) {}

    fn on_key_pressed(&mut self, game_state: &mut GameState, key: Key) {}
    fn on_mouse_pressed(&mut self, game_state: &mut GameState, column: usize, row: usize) {}

    fn on_dialog_selection(&mut self, game_state: &mut GameState, selection: DialogSelection) {}

    fn on_pause(&mut self, game_state: &mut GameState) {}
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
            r#"
              ----------------------------------------------
              .---- .---. .  . .---.  --+-- .---- .--. .   .
              |     |   | | /  |   |    |   |     |  | |\ /|
              '---. |   | :{   |   |    |   +---- +--' | v |
                  | |   | | \  |   |    |   |     | \  |   |
              ----' '---' '  ' '---'    '   '---- '  ' '   '
              ----------------------------------------------
            "#[1..].trim_end() /* Remove leading newline and trailing spaces */);
        console.set_color(Color::LightRed, Color::Default);
        console.draw_text(
            r#"
                          .--.  .---- .   . .---.
                          |   | |     |\ /| |   |
                          |   | +---- | v | |   |
                          |   | |     |   | |   |
                          '--'  '---- '   ' '---'
            "#.trim_end() /* Remove trailing spaces */);
        console.set_color(Color::LightYellow, Color::Default);
        console.set_cursor_pos(1, 13);
        console.draw_text("------------------------------------------------------------------------");

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
        console.draw_key_input_text("ENTER");
        console.reset_color();
        console.draw_text(" to start the game!");

        console.set_cursor_pos(1, 21);
        console.draw_text("By ");
        console.set_color(Color::Default, Color::Yellow);
        console.draw_text("JDDev0");

        console.reset_color();
        console.set_cursor_pos(62, 19);
        console.draw_text("Settings: ");
        console.draw_key_input_text("s");

        console.reset_color();
        console.set_cursor_pos(65, 20);
        console.draw_text("About: ");
        console.draw_key_input_text("a");

        console.reset_color();
        console.set_cursor_pos(65, 21);
        console.draw_text("Help: ");
        console.draw_key_input_text("F1");

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
            game_state.open_dialog(Dialog::new_yes_no("Exit game?"));

            return;
        }

        if key == Key::A {
            game_state.play_sound_effect_ui_select();

            game_state.set_screen(ScreenId::About);

            return;
        }

        if key == Key::S {
            game_state.play_sound_effect_ui_select();

            game_state.set_screen(ScreenId::Settings);

            return;
        }

        if key == Key::ENTER || key == Key::SPACE {
            game_state.play_sound_effect_ui_select();

            //Select first level which is not yet completed
            let level_pack = game_state.get_current_level_pack().unwrap();
            let min_level_not_completed = level_pack.min_level_not_completed();
            if min_level_not_completed >= level_pack.level_count() {
                game_state.set_level_index(0);
            }else {
                game_state.set_level_index(min_level_not_completed);
            }

            game_state.set_screen(ScreenId::SelectLevel);
        }
    }

    fn on_mouse_pressed(&mut self, game_state: &mut GameState, column: usize, row: usize) {
        if row == 16 && (21..51).contains(&column) {
            self.on_key_pressed(game_state, Key::ENTER);
        }

        if row == 21 && column > 64 && column < 73 {
            game_state.open_help_page();
        }

        if row == 20 && column > 64 && column < 73 {
            self.on_key_pressed(game_state, Key::A);
        }

        if row == 19 && column > 61 && column < 73 {
            self.on_key_pressed(game_state, Key::S);
        }
    }

    fn on_dialog_selection(&mut self, game_state: &mut GameState, selection: DialogSelection) {
        if selection == DialogSelection::Yes {
            game_state.exit();
        }
    }

    fn on_set_screen(&mut self, game_state: &mut GameState) {
        game_state.set_background_music_loop(&audio::BACKGROUND_MUSIC_FIELDS_OF_ICE);
    }
}

mod attribution {
    use std::sync::LazyLock;

    const ATTRIBUTION_AUDIO_SOUND_EFFECTS_TEXT: &str = include_str!("../../resources/attribution/audio_sound_effects.txt");
    const ATTRIBUTION_AUDIO_BACKGROUND_MUSIC_TEXT: &str = include_str!("../../resources/attribution/audio_background_music.txt");

    #[cfg(all(feature = "gui", not(feature = "steam")))]
    const ATTRIBUTION_FONTS_TEXT: &str = include_str!("../../resources/attribution/fonts_gui.txt");
    #[cfg(feature = "steam")]
    const ATTRIBUTION_FONTS_TEXT: &str = include_str!("../../resources/attribution/fonts_steam.txt");

    #[cfg(feature = "cli")]
    const ATTRIBUTION_LIBRARIES_TEXT: &str = include_str!("../../resources/attribution/libraries_cli.txt");
    #[cfg(all(feature = "gui", not(feature = "steam")))]
    const ATTRIBUTION_LIBRARIES_TEXT: &str = include_str!("../../resources/attribution/libraries_gui.txt");
    #[cfg(feature = "steam")]
    const ATTRIBUTION_LIBRARIES_TEXT: &str = include_str!("../../resources/attribution/libraries_steam.txt");

    pub static ATTRIBUTION_AUDIO_SOUND_EFFECTS_TOKENS: LazyLock<Box<[[&str; 4]]>> = LazyLock::new(|| {
        ATTRIBUTION_AUDIO_SOUND_EFFECTS_TEXT.split("\n").
                filter(|line| !line.trim().is_empty()).
                map(|line| {
                    line.split(";").
                            map(|token| token.trim()).
                            collect::<Box<[&str]>>().
                            as_ref().try_into().unwrap()
                }).collect()
    });

    pub static ATTRIBUTION_AUDIO_BACKGROUND_MUSIC_TOKENS: LazyLock<Box<[[&str; 4]]>> = LazyLock::new(|| {
        ATTRIBUTION_AUDIO_BACKGROUND_MUSIC_TEXT.split("\n").
                filter(|line| !line.trim().is_empty()).
                map(|line| {
                    line.split(";").
                            map(|token| token.trim()).
                            collect::<Box<[&str]>>().
                            as_ref().try_into().unwrap()
                }).collect()
    });

    #[cfg(feature = "gui")]
    pub static ATTRIBUTION_FONTS_TOKENS: LazyLock<Box<[[&str; 4]]>> = LazyLock::new(|| {
        ATTRIBUTION_FONTS_TEXT.split("\n").
                filter(|line| !line.trim().is_empty()).
                map(|line| {
                    line.split(";").
                            map(|token| token.trim()).
                            collect::<Box<[&str]>>().
                            as_ref().try_into().unwrap()
                }).collect()
    });

    pub static ATTRIBUTION_LIBRARIES_TOKENS: LazyLock<Box<[[&str; 3]]>> = LazyLock::new(|| {
        ATTRIBUTION_LIBRARIES_TEXT.split("\n").
                filter(|line| !line.trim().is_empty()).
                map(|line| {
                    line.split(";").
                            map(|token| token.trim()).
                            collect::<Box<[&str]>>().
                            as_ref().try_into().unwrap()
                }).collect()
    });
}

pub struct ScreenAbout {
    scroll_position_row: usize,
    scroll_position_row_max: usize,
}

impl ScreenAbout {
    pub fn new() -> Self {
        let mut scroll_position_row_max = 11; //Open source game, version, link, build info, thank you text

        scroll_position_row_max += 3 + 5 * attribution::ATTRIBUTION_AUDIO_SOUND_EFFECTS_TOKENS.len();
        scroll_position_row_max += 3 + 5 * attribution::ATTRIBUTION_AUDIO_BACKGROUND_MUSIC_TOKENS.len();

        #[cfg(feature = "gui")]
        {
            scroll_position_row_max += 3 + 5 * attribution::ATTRIBUTION_FONTS_TOKENS.len();
        }

        scroll_position_row_max += 3 + 4 * attribution::ATTRIBUTION_LIBRARIES_TOKENS.len();

        //Do not allow scrolling past the last line of text
        scroll_position_row_max -= Game::CONSOLE_MIN_HEIGHT - 1;

        Self {
            scroll_position_row: 0,
            scroll_position_row_max,
        }
    }

    fn draw_scrollbar(&self, console: &Console) {
        console.reset_color();
        for y in 2..Game::CONSOLE_MIN_HEIGHT - 1 {
            console.set_cursor_pos(Game::CONSOLE_MIN_WIDTH - 1, y);
            console.draw_text("|");
        }

        console.set_cursor_pos(Game::CONSOLE_MIN_WIDTH - 1, 2);
        console.draw_key_input_text("^");

        console.set_cursor_pos(Game::CONSOLE_MIN_WIDTH - 1, Game::CONSOLE_MIN_HEIGHT - 1);
        console.draw_key_input_text("v");

        let scrollbar_indicator_y_pos = (self.scroll_position_row as f64
                / self.scroll_position_row_max as f64
                //"-1": One less than count
                //"-2": Ignore two top rows
                //"-1": One less than sum, because 1 is added at bottom if not at very top
                * (Game::CONSOLE_MIN_HEIGHT - 1 - 2 - 1) as f64
        ).floor() as usize
                + 2
                + if self.scroll_position_row == 0 { 0 } else { 1 };

        console.set_color(Color::LightCyan, Color::Default);
        console.set_cursor_pos(Game::CONSOLE_MIN_WIDTH - 1, scrollbar_indicator_y_pos);
        console.draw_text("*");
    }

    fn set_cursor_pos_if_visible(&self, console: &Console, column: usize, row: usize) -> bool {
        let min_visible_y = self.scroll_position_row;
        let max_visible_y = self.scroll_position_row + Game::CONSOLE_MIN_HEIGHT;

        if row > min_visible_y + 1 && row < max_visible_y {
            console.set_cursor_pos(column, row - min_visible_y);

            true
        }else {
            false
        }
    }
}

impl Screen for ScreenAbout {
    fn draw(&self, _game_state: &GameState, console: &Console) {
        console.reset_color();
        console.set_underline(true);
        console.draw_text("About SokoTerm:");
        console.set_underline(false);

        self.draw_scrollbar(console);

        let mut current_row = 2;
        if self.set_cursor_pos_if_visible(console, 0, current_row) {
            console.set_color(Color::LightYellow, Color::Default);
            console.draw_text("SokoTerm");

            console.reset_color();
            console.draw_text(" is an open-source game (licensed under the GPLv3)!");
        }

        current_row += 1;
        if self.set_cursor_pos_if_visible(console, 0, current_row) {
            console.reset_color();
            console.draw_text(format!("Version: {}", Game::VERSION));
        }

        current_row += 2;
        if self.set_cursor_pos_if_visible(console, 0, current_row) {
            console.reset_color();
            console.draw_text("You can view the source code here: ");
        }

        current_row += 1;
        if self.set_cursor_pos_if_visible(console, 0, current_row) {
            console.set_color(Color::LightBlue, Color::Default);
            console.set_underline(true);
            console.draw_text("https://github.com/JDDev0/SokoTerm");
            console.set_underline(false);
        }

        current_row += 2;
        if self.set_cursor_pos_if_visible(console, 0, current_row) {
            console.reset_color();
            console.draw_text("This is the ");

            if cfg!(feature = "steam") {
                console.set_color(Color::LightBlue, Color::Default);
                console.draw_text("Steam");
            }else if cfg!(feature = "gui") {
                console.set_color(Color::Red, Color::Default);
                console.draw_text("GUI");
            }else if cfg!(feature = "cli") {
                console.set_color(Color::Yellow, Color::Default);
                console.draw_text("CLI");
            }

            console.set_color(Color::LightRed, Color::Default);
            console.draw_text(" [Demo]");

            console.reset_color();
            console.draw_text(" build of this game.");
        }

        current_row += 3;
        if self.set_cursor_pos_if_visible(console, 0, current_row) {
            console.reset_color();
            console.draw_text("This build of the game uses the following sound effects:");
        }

        current_row += 1;
        if self.set_cursor_pos_if_visible(console, 0, current_row) {
            console.reset_color();
            console.draw_text("========================================================");
        }

        for [name, creator, license, project_link] in attribution::ATTRIBUTION_AUDIO_SOUND_EFFECTS_TOKENS.iter() {
            current_row += 1;
            if self.set_cursor_pos_if_visible(console, 0, current_row) {
                console.set_color(Color::LightCyan, Color::Default);
                console.draw_text(*name);
            }

            current_row += 1;
            if self.set_cursor_pos_if_visible(console, 0, current_row) {
                console.reset_color();
                console.draw_text("[by ");

                console.set_color(Color::LightPink, Color::Default);
                console.draw_text(*creator);

                console.reset_color();
                console.draw_text("]");
            }

            current_row += 1;
            if self.set_cursor_pos_if_visible(console, 0, current_row) {
                console.set_color(Color::LightYellow, Color::Default);
                console.draw_text(*license);
            }

            current_row += 1;
            if self.set_cursor_pos_if_visible(console, 0, current_row) {
                console.set_color(Color::LightBlue, Color::Default);
                console.set_underline(true);
                console.draw_text(*project_link);
                console.set_underline(false);
            }

            current_row += 1;
        }

        current_row += 2;
        if self.set_cursor_pos_if_visible(console, 0, current_row) {
            console.reset_color();
            console.draw_text("This build of the game uses the following background music tracks:");
        }

        current_row += 1;
        if self.set_cursor_pos_if_visible(console, 0, current_row) {
            console.reset_color();
            console.draw_text("==================================================================");
        }

        for [name, creator, license, project_link] in attribution::ATTRIBUTION_AUDIO_BACKGROUND_MUSIC_TOKENS.iter() {
            current_row += 1;
            if self.set_cursor_pos_if_visible(console, 0, current_row) {
                console.set_color(Color::LightCyan, Color::Default);
                console.draw_text(*name);
            }

            current_row += 1;
            if self.set_cursor_pos_if_visible(console, 0, current_row) {
                console.reset_color();
                console.draw_text("[by ");

                console.set_color(Color::LightPink, Color::Default);
                console.draw_text(*creator);

                console.reset_color();
                console.draw_text("]");
            }

            current_row += 1;
            if self.set_cursor_pos_if_visible(console, 0, current_row) {
                console.set_color(Color::LightYellow, Color::Default);
                console.draw_text(*license);
            }

            current_row += 1;
            if self.set_cursor_pos_if_visible(console, 0, current_row) {
                console.set_color(Color::LightBlue, Color::Default);
                console.set_underline(true);
                console.draw_text(*project_link);
                console.set_underline(false);
            }

            current_row += 1;
        }

        #[cfg(feature = "gui")]
        {
            current_row += 2;
            if self.set_cursor_pos_if_visible(console, 0, current_row) {
                console.reset_color();
                console.draw_text("This build of the game uses the following text fonts:");
            }

            current_row += 1;
            if self.set_cursor_pos_if_visible(console, 0, current_row) {
                console.reset_color();
                console.draw_text("=====================================================");
            }

            for [name, creator, license, project_link] in attribution::ATTRIBUTION_FONTS_TOKENS.iter() {
                current_row += 1;
                if self.set_cursor_pos_if_visible(console, 0, current_row) {
                    console.set_color(Color::LightCyan, Color::Default);
                    console.draw_text(*name);
                }

                current_row += 1;
                if self.set_cursor_pos_if_visible(console, 0, current_row) {
                    console.reset_color();
                    console.draw_text("[by ");

                    console.set_color(Color::LightPink, Color::Default);
                    console.draw_text(*creator);

                    console.reset_color();
                    console.draw_text("]");
                }

                current_row += 1;
                if self.set_cursor_pos_if_visible(console, 0, current_row) {
                    console.set_color(Color::LightYellow, Color::Default);
                    console.draw_text(*license);
                }

                current_row += 1;
                if self.set_cursor_pos_if_visible(console, 0, current_row) {
                    console.set_color(Color::LightBlue, Color::Default);
                    console.set_underline(true);
                    console.draw_text(*project_link);
                    console.set_underline(false);
                }

                current_row += 1;
            }
        }

        current_row += 2;
        if self.set_cursor_pos_if_visible(console, 0, current_row) {
            console.reset_color();
            console.draw_text("This build of the game uses the following open-source libraries:");
        }

        current_row += 1;
        if self.set_cursor_pos_if_visible(console, 0, current_row) {
            console.reset_color();
            console.draw_text("================================================================");
        }

        for [name, license, project_link] in attribution::ATTRIBUTION_LIBRARIES_TOKENS.iter() {
            current_row += 1;
            if self.set_cursor_pos_if_visible(console, 0, current_row) {
                console.set_color(Color::LightCyan, Color::Default);
                console.draw_text(*name);
            }

            current_row += 1;
            if self.set_cursor_pos_if_visible(console, 0, current_row) {
                console.set_color(Color::LightYellow, Color::Default);
                console.draw_text(*license);
            }

            current_row += 1;
            if self.set_cursor_pos_if_visible(console, 0, current_row) {
                console.set_color(Color::LightBlue, Color::Default);
                console.set_underline(true);
                console.draw_text(*project_link);
                console.set_underline(false);
            }

            current_row += 1;
        }

        current_row += 2;
        if self.set_cursor_pos_if_visible(console, 0, current_row) {
            console.reset_color();
            console.draw_text("Thank you for playing ");

            console.set_color(Color::LightYellow, Color::Default);
            console.draw_text("SokoTerm");

            console.reset_color();
            console.draw_text("!");
        }
    }

    fn on_key_pressed(&mut self, game_state: &mut GameState, key: Key) {
        if key == Key::ESC {
            game_state.play_sound_effect_ui_select();

            game_state.set_screen(ScreenId::StartMenu);

            return;
        }

        if key == Key::UP && self.scroll_position_row > 0 {
            self.scroll_position_row -= 1;
        }else if key == Key::DOWN && self.scroll_position_row < self.scroll_position_row_max {
            self.scroll_position_row += 1;
        }
    }

    fn on_mouse_pressed(&mut self, _game_state: &mut GameState, column: usize, row: usize) {
        if column == Game::CONSOLE_MIN_WIDTH - 1 && (2..Game::CONSOLE_MIN_HEIGHT).contains(&row) {
            let scrollbar_y_coord = row - 2;

            self.scroll_position_row = (scrollbar_y_coord as f64
                    //"-1": One less than count
                    //"-2": Ignore two top rows
                    / (Game::CONSOLE_MIN_HEIGHT - 1 - 2) as f64
                    //"-1": One less than sum, because 1 is added at bottom if not at very top
                    * (self.scroll_position_row_max - 1) as f64
            ).floor() as usize
                    + if scrollbar_y_coord == 0 { 0 } else { 1 };
        }
    }
}

pub struct ScreenSettings {}

impl ScreenSettings {
    pub fn new() -> Self {
        Self {}
    }
}

impl Screen for ScreenSettings {
    fn draw(&self, game_state: &GameState, console: &Console) {
        console.set_color(Color::Yellow, Color::Default);
        console.set_underline(true);
        console.draw_text("Settings menu");
        console.set_underline(false);

        console.reset_color();
        console.set_cursor_pos(0, 2);
        if cfg!(feature = "gui") {
            console.draw_text("Color scheme (Toggle with ");

            console.draw_key_input_text("F10");

            console.reset_color();
            console.draw_text("):");
        }else {
            console.draw_text("Color scheme:");
        }

        //Draw color scheme
        console.set_cursor_pos(0, 3);
        console.set_color(Color::Default, Color::Black);
        console.draw_text("   ");
        console.set_color(Color::Default, Color::Red);
        console.draw_text("   ");
        console.set_color(Color::Default, Color::Green);
        console.draw_text("   ");
        console.set_color(Color::Default, Color::Yellow);
        console.draw_text("   ");
        console.set_color(Color::Default, Color::Blue);
        console.draw_text("   ");
        console.set_color(Color::Default, Color::Pink);
        console.draw_text("   ");
        console.set_color(Color::Default, Color::Cyan);
        console.draw_text("   ");
        console.set_color(Color::Default, Color::White);
        console.draw_text("   ");

        console.set_cursor_pos(0, 4);
        console.set_color(Color::Default, Color::LightBlack);
        console.draw_text("   ");
        console.set_color(Color::Default, Color::LightRed);
        console.draw_text("   ");
        console.set_color(Color::Default, Color::LightGreen);
        console.draw_text("   ");
        console.set_color(Color::Default, Color::LightYellow);
        console.draw_text("   ");
        console.set_color(Color::Default, Color::LightBlue);
        console.draw_text("   ");
        console.set_color(Color::Default, Color::LightPink);
        console.draw_text("   ");
        console.set_color(Color::Default, Color::LightCyan);
        console.draw_text("   ");
        console.set_color(Color::Default, Color::LightWhite);
        console.draw_text("   ");

        console.reset_color();
        console.set_cursor_pos(0, 6);
        if cfg!(feature = "gui") {
            console.draw_text("Tile mode (Toggle with ");

            console.draw_key_input_text("F9");

            console.reset_color();
            console.draw_text("): ");
        }else {
            console.draw_text("Tile mode: ");
        }

        if game_state.settings.tile_mode == TileMode::Graphical && !cfg!(feature = "cli") {
            console.set_color(Color::Blue, Color::Default);
            console.draw_text("Graphical");
        }else {
            console.set_color(Color::Red, Color::Default);
            console.draw_text("ASCII");
        }

        console.reset_color();
        console.set_cursor_pos(0, 8);
        console.draw_text("Background Music: ");

        if game_state.settings.background_music {
            console.set_color(Color::Green, Color::Default);
            console.draw_text("Enabled");
        }else {
            console.set_color(Color::Red, Color::Default);
            console.draw_text("Disabled");
        }

        console.reset_color();
        console.draw_text(" (Toggle with ");

        console.draw_key_input_text("F8");

        console.reset_color();
        console.draw_text(")");

        console.reset_color();
        console.set_cursor_pos(0, 10);
        console.draw_text("Animation Speed: ");

        console.set_color(Color::Blue, Color::Default);
        console.draw_text(game_state.settings.animation_speed.display_name());

        console.reset_color();
        console.draw_text(" (Toggle with ");

        console.draw_key_input_text("F7");

        console.reset_color();
        console.draw_text(")");
    }

    fn on_key_pressed(&mut self, game_state: &mut GameState, key: Key) {
        if key == Key::ESC {
            game_state.play_sound_effect_ui_select();

            game_state.set_screen(ScreenId::StartMenu);
        }
    }

    fn on_mouse_pressed(&mut self, _game_state: &mut GameState, _column: usize, _row: usize) {
        //TODO
    }
}

pub struct ScreenSelectLevel {
    level_list: UIList,
    level_preview: bool,
}

impl ScreenSelectLevel {
    pub fn new() -> Self {
        Self {
            level_list: UIList::new(
                Rect::new(0, 1, Game::CONSOLE_MIN_WIDTH, Game::CONSOLE_MIN_HEIGHT - 1),
                vec![
                    UIListElement::new("<<", Color::White, Color::LightBlue),
                    //[Level Entries]
                ],
                Box::new(|_, game_state: &mut GameState, cursor_index: usize| {
                    if cursor_index == 0 {
                        game_state.play_sound_effect_ui_select();
                        game_state.set_screen(ScreenId::StartMenu);

                        return;
                    }

                    let level_index = cursor_index - 1;

                    let level_pack = game_state.get_current_level_pack().unwrap();
                    let min_level_not_completed = level_pack.min_level_not_completed();

                    if level_index <= min_level_not_completed {
                        game_state.play_sound_effect_ui_select();

                        game_state.set_level_index(level_index);
                        game_state.set_screen(ScreenId::InGame);
                    }else {
                        game_state.play_sound_effect_ui_error();
                    }
                }),
            ),
            level_preview: false,
        }
    }

    fn update_list_elements(&mut self, game_state: &GameState) {
        let elements = self.level_list.elements_mut();

        //Remove all level entries
        elements.drain(1..);

        let level_pack = game_state.get_current_level_pack().unwrap();
        let min_level_not_completed = level_pack.min_level_not_completed();
        for i in 0..level_pack.level_count() {
            elements.push(UIListElement::new(
                utils::number_to_string_leading_ascii(2, i as u32 + 1, false),
                Color::Black,
                match i.cmp(&min_level_not_completed) {
                    Ordering::Less => Color::Green,
                    Ordering::Equal => Color::Yellow,
                    Ordering::Greater => Color::Red,
                },
            ));
        }
    }

    fn draw_overview(&self, game_state: &GameState, console: &Console) {
        console.reset_color();
        console.set_underline(true);
        console.draw_text(format!("Select a level (Level pack \"{}\"):", game_state.get_current_level_pack().unwrap().name()));
        console.set_underline(false);

        self.level_list.draw(console);

        let entry_count = self.level_list.elements().len();

        //Draw border for best time and best moves
        let y = 4 + ((entry_count - 1)/24)*2;

        console.set_cursor_pos(0, y);
        console.set_color(Color::Cyan, Color::Default);
        console.draw_text(".-------------------------.");
        for i in 1..4 {
            console.set_cursor_pos(0, y + i);
            console.draw_text("|                         |");
        }
        console.set_cursor_pos(0, y + 4);
        console.draw_text("\'-------------------------\'");

        let cursor_index = self.level_list.cursor_index();
        if cursor_index == 0 {
            console.reset_color();
            console.set_cursor_pos(11, y + 2);
            console.draw_text("Back");
        }else {
            //Draw best time and best moves
            console.reset_color();
            console.set_cursor_pos(1, y + 1);
            console.draw_text("Selected level:       ");
            console.draw_text(format!("{:03}", cursor_index));

            let level_pack = game_state.get_current_level_pack().unwrap();
            let level = level_pack.levels().get(cursor_index - 1).unwrap();

            console.set_cursor_pos(1, y + 2);
            console.draw_text("Best time     : ");
            match level.best_time() {
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
            match level.best_moves() {
                None => console.draw_text("XXXX"),
                Some(best_moves) => {
                    console.draw_text(format!("{:04}", best_moves));
                },
            }

            console.reset_color();
            console.set_cursor_pos(29, y + 1);
            console.draw_text("Press ");

            console.draw_key_input_text("p");

            console.reset_color();
            console.draw_text(" for level preview");
        }
    }

    fn draw_level_preview(&self, game_state: &GameState, console: &Console) {
        let cursor_index = self.level_list.cursor_index();

        if cursor_index == 1 {
            console.draw_key_input_text("<");

            console.reset_color();
            console.draw_text(" Back");
        }else if cursor_index > 1 {
            console.draw_key_input_text("<");

            console.reset_color();
            console.draw_text(format!(" Level {:03}", cursor_index - 1));
        }

        if cursor_index < game_state.get_current_level_pack().unwrap().level_count() {
            console.reset_color();
            console.set_cursor_pos(Game::CONSOLE_MIN_WIDTH - 11, 0);
            console.draw_text(format!("Level {:03} ", cursor_index + 1));

            console.draw_key_input_text(">");
        }

        console.reset_color();
        console.set_cursor_pos(((Game::CONSOLE_MIN_WIDTH - 23) as f64 * 0.5) as usize, 0);
        console.draw_text("Preview (");

        console.draw_key_input_text("p");

        console.reset_color();
        console.draw_text(format!(") [Level {:03}]", cursor_index));

        if cursor_index == 0 {
            let x = ((Game::CONSOLE_MIN_WIDTH - 40) as f64 * 0.5) as usize;
            let y = ((Game::CONSOLE_MIN_HEIGHT - 5) as f64 * 0.5) as usize;

            console.set_cursor_pos(x, y);
            console.set_color(Color::Cyan, Color::Default);
            console.draw_text(".--------------------------------------.");
            for i in 1..4 {
                console.set_cursor_pos(x, y + i);
                console.draw_text("|                                      |");
            }
            console.set_cursor_pos(x, y + 4);
            console.draw_text("\'--------------------------------------\'");

            console.reset_color();
            console.set_cursor_pos(35, y + 2);
            console.draw_text("Back");
        }else {
            let min_level_not_completed = game_state.get_current_level_pack().as_ref().unwrap().min_level_not_completed();
            let level = game_state.get_current_level_pack().unwrap().levels()[cursor_index - 1].level();

            if cursor_index - 1 > min_level_not_completed {
                let x = ((Game::CONSOLE_MIN_WIDTH - 40) as f64 * 0.5) as usize;
                let y = ((Game::CONSOLE_MIN_HEIGHT - 5) as f64 * 0.5) as usize;

                console.set_cursor_pos(x, y);
                console.set_color(Color::Cyan, Color::Default);
                console.draw_text(".--------------------------------------.");
                for i in 1..4 {
                    console.set_cursor_pos(x, y + i);
                    console.draw_text("|                                      |");
                }
                console.set_cursor_pos(x, y + 4);
                console.draw_text("\'--------------------------------------\'");

                console.reset_color();
                console.set_cursor_pos(x + 2, y + 2);
                console.draw_text(format!("Beat level {:03} to unlock this level.", cursor_index - 1));
            }else {
                let x_offset = ((Game::CONSOLE_MIN_WIDTH - level.width()) as f64 * 0.5) as usize;
                let y_offset = 1;

                level.draw(console, x_offset, y_offset, game_state.is_player_background(), None);
            }
        }
    }
}

impl Screen for ScreenSelectLevel {
    fn draw(&self, game_state: &GameState, console: &Console) {
        if self.level_preview {
            self.draw_level_preview(game_state, console);
        }else {
            self.draw_overview(game_state, console);
        }
    }

    fn on_key_pressed(&mut self, game_state: &mut GameState, key: Key) {
        if key == Key::ESC {
            game_state.play_sound_effect(audio::UI_SELECT_EFFECT);

            if self.level_preview {
                self.level_preview = false;
            }else {
                game_state.set_screen(ScreenId::StartMenu);
            }

            return;
        }

        if key == Key::P {
            game_state.play_sound_effect_ui_select();

            self.level_preview = !self.level_preview;

            return;
        }

        self.level_list.on_key_press(&mut (), game_state, key);
    }

    fn on_mouse_pressed(&mut self, game_state: &mut GameState, column: usize, row: usize) {
        if self.level_preview {
            if row == 0 {
                let center_text_start = ((Game::CONSOLE_MIN_WIDTH - 23) as f64 * 0.5) as usize;

                if column < 11 {
                    self.on_key_pressed(game_state, Key::LEFT);
                }else if column >= center_text_start && column < center_text_start + 23 {
                    self.on_key_pressed(game_state, Key::ENTER);
                }else if column > Game::CONSOLE_MIN_WIDTH - 12 {
                    self.on_key_pressed(game_state, Key::RIGHT);
                }
            }

            return;
        }

        let element_count = self.level_list.elements().len();
        let y = 4 + ((element_count - 1)/24)*2;
        if row == y + 1 && (29..54).contains(&column) {
            self.on_key_pressed(game_state, Key::P);
        }

        self.level_list.on_mouse_pressed(&mut (), game_state, column, row);
    }

    fn on_set_screen(&mut self, game_state: &mut GameState) {
        self.update_list_elements(game_state);

        self.level_list.set_cursor_index(game_state.get_level_index() + 1);

        self.level_preview = false;
    }
}

pub struct ScreenInGame {
    time_start_in_menu: Option<SystemTime>,
    time_start: Option<SystemTime>,
    time_millis: u32,
    time_sec: u32,
    time_min: u32,

    animation_first_frame: bool,
    level: Option<PlayingLevel>,

    show_floor: bool,

    continue_flag: bool,
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

            animation_first_frame: false,
            level: Default::default(),

            show_floor: false,

            continue_flag: Default::default(),
            game_over_flag: Default::default(),
        }
    }

    pub fn start_level(&mut self, level: &Level) {
        //Reset stats
        self.time_start = None;
        self.time_millis = 0;
        self.time_sec = 0;
        self.time_min = 0;

        self.continue_flag = false;
        self.game_over_flag = false;

        self.animation_first_frame = false;
        self.level = Some(PlayingLevel::new(level, Self::UNDO_HISTORY_SIZE_PLAYING).unwrap());

        self.show_floor = false;
    }

    fn draw_tutorial_level_text(&self, game_state: &GameState, console: &Console) {
        //Draw special help text for tutorial levels
        if game_state.get_level_pack_index() == 0 { //Built-in Demo pack
            console.reset_color();
            match game_state.current_level_index {
                0 => {
                    if self.continue_flag {
                        console.set_cursor_pos(13, 8);
                        console.draw_text("Press ");

                        console.draw_key_input_text("ENTER");
                        console.reset_color();
                        console.draw_text("/");
                        console.draw_key_input_text("SPACEBAR");

                        console.reset_color();
                        console.draw_text(" to go to the next level...");
                    }else {
                        console.set_cursor_pos(13, 8);
                        console.draw_text("Use ");

                        console.draw_key_input_text("Arrow Keys");

                        console.reset_color();
                        console.draw_text(" (< ^ > v) or ");

                        console.draw_key_input_text("WASD");

                        console.reset_color();
                        console.draw_text(" keys to move...");
                    }
                },
                1 => {
                    console.set_cursor_pos(16, 8);
                    console.draw_text("Boxes (");

                    Tile::Box.draw(console, false, false);

                    console.reset_color();
                    console.draw_text(") must be placed on ");

                    console.set_color(Color::LightRed, Color::Default);
                    console.draw_text("all");

                    console.reset_color();
                    console.draw_text(" goals (");

                    Tile::Goal.draw(console, false, false);

                    console.reset_color();
                    console.draw_text(")");
                },
                2 => {
                    console.set_cursor_pos(5, 8);
                    console.draw_text("One-way doors (");

                    Tile::OneWayLeft.draw(console, false, false);

                    console.reset_color();
                    console.draw_text(" ");

                    Tile::OneWayUp.draw(console, false, false);

                    console.reset_color();
                    console.draw_text(" ");

                    Tile::OneWayRight.draw(console, false, false);

                    console.reset_color();
                    console.draw_text(" ");

                    Tile::OneWayDown.draw(console, false, false);

                    console.reset_color();
                    console.draw_text(") can only be entered from the opened side");
                },
                3 => {
                    console.set_cursor_pos(8, 8);
                    console.draw_text("Boxes (");

                    Tile::Box.draw(console, false, false);

                    console.reset_color();
                    console.draw_text(") cannot be moved through one-way doors (");

                    Tile::OneWayLeft.draw(console, false, false);

                    console.reset_color();
                    console.draw_text(" ");

                    Tile::OneWayUp.draw(console, false, false);

                    console.reset_color();
                    console.draw_text(" ");

                    Tile::OneWayRight.draw(console, false, false);

                    console.reset_color();
                    console.draw_text(" ");

                    Tile::OneWayDown.draw(console, false, false);

                    console.reset_color();
                    console.draw_text(")");
                },
                4..=6 => {
                    console.set_cursor_pos(28, 11);
                    console.draw_key_input_text("z");
                    console.reset_color();
                    console.draw_text("/");
                    console.draw_key_input_text("u");

                    console.reset_color();
                    console.draw_text(": Undo, ");

                    console.draw_key_input_text("y");

                    console.reset_color();
                    console.draw_text(": Redo");

                    console.set_cursor_pos(29, 12);
                    console.draw_key_input_text("r");

                    console.reset_color();
                    console.draw_text(": Restart Level");
                },
                9 => {
                    console.set_cursor_pos(8, 13);
                    console.draw_text("Press ");

                    console.draw_key_input_text("F9");

                    console.reset_color();
                    console.draw_text(" to toggle between Graphical and ASCII tile modes");
                },
                10 => {
                    console.set_cursor_pos(18, 11);
                    console.draw_text("Keys (");

                    Tile::Key.draw(console, false, false);

                    console.reset_color();
                    console.draw_text(") can be used to open doors (");

                    Tile::LockedDoor.draw(console, false, false);

                    console.reset_color();
                    console.draw_text(")");
                },
                34 => {
                    if !self.game_over_flag {
                        console.set_cursor_pos(11, 9);
                        console.draw_text("Ice (");

                        Tile::Ice.draw(console, false, false);

                        console.reset_color();
                        console.draw_text(") causes the player (");

                        Tile::Player.draw(console, false, false);

                        console.reset_color();
                        console.draw_text(") and boxes (");

                        Tile::BoxOnIce.draw(console, false, false);

                        console.reset_color();
                        console.draw_text(") to slide");
                    }
                },
                _ => {},
            }
        }
    }

    fn handle_move_result(&mut self, game_state: &mut GameState, move_result: MoveResult) {
        let current_level_index = game_state.current_level_index;
        let Some(level_pack) = game_state.get_current_level_pack_mut() else {
            return;
        };

        match move_result {
            MoveResult::Valid { has_won, sound_effect } => {
                self.time_start.get_or_insert_with(SystemTime::now);

                if has_won {
                    self.continue_flag = true;

                    //Update best scores
                    let time = self.time_millis as u64 + 1000 * self.time_sec as u64 + 60000 * self.time_min as u64;
                    let moves = self.level.as_ref().unwrap().current_move_index() as u32;

                    level_pack.update_stats(current_level_index, time, moves);

                    if current_level_index >= level_pack.min_level_not_completed() {
                        level_pack.set_min_level_not_completed(current_level_index + 1);
                    }

                    if let Err(err) = level_pack.save_save_game(false) {
                        game_state.open_dialog(Dialog::new_ok_error(format!("Cannot save: {}", err)));
                    }

                    #[cfg(feature = "steam")]
                    {
                        let steam_client = game_state.steam_client.clone();

                        let val = Stat::MAX_COMPLETED_LEVEL.get(steam_client.clone());
                        if current_level_index as i32 + 1 > val {
                            Stat::MAX_COMPLETED_LEVEL.set(steam_client.clone(), current_level_index as i32 + 1)
                        }
                    }

                    game_state.play_sound_effect(audio::LEVEL_COMPLETE_EFFECT);
                }

                game_state.play_sound_effect(audio::STEP_EFFECT);

                if let Some(sound_effect) = sound_effect {
                    game_state.play_level_sound_effect(sound_effect);
                }
            },

            MoveResult::Invalid => {
                game_state.play_sound_effect(audio::NO_PATH_EFFECT);
            },

            MoveResult::Animation { sound_effect, .. } => {
                if self.animation_first_frame {
                    game_state.play_sound_effect(audio::STEP_EFFECT);
                }

                if let Some(sound_effect) = sound_effect {
                    game_state.play_level_sound_effect(sound_effect);
                }
            },
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
        console.draw_text(format!("Moves: {:04}", self.level.as_ref().unwrap().current_move_index()));

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
        }else if self.game_over_flag {
            console.set_cursor_pos(((Game::CONSOLE_MIN_WIDTH - 13) as f64 * 0.5) as usize, 0);
            console.draw_text("You have won!");
        }else if self.show_floor {
            console.set_cursor_pos(((Game::CONSOLE_MIN_WIDTH - 14) as f64 * 0.5) as usize, 0);
            console.draw_text("Show tiles (");
            console.draw_key_input_text("q");
            console.reset_color();
            console.draw_text(")");
        }

        if self.game_over_flag {
            console.set_cursor_pos(17, 6);
            console.reset_color();
            console.draw_text("Thank you for playing the ");

            console.set_color(Color::LightYellow, Color::Default);
            console.draw_text("SokoTerm");

            console.set_color(Color::LightRed, Color::Default);
            console.draw_text(" Demo");

            console.reset_color();
            console.draw_text("!");

            console.set_cursor_pos(15, 10);
            console.reset_color();
            console.draw_text("You have completed the demo in ");

            console.set_color(Color::LightCyan, Color::Default);
            match game_state.level_pack.level_pack_best_time_sum() {
                None => console.draw_text("XX:XX:XX.XXX"),
                Some(best_time_sum) => {
                    console.draw_text(format!(
                        "{:02}:{:02}:{:02}.{:03}",
                        (best_time_sum/3600000)%24,
                        (best_time_sum/60000)%60,
                        (best_time_sum/1000)%60,
                        best_time_sum%1000
                    ));
                },
            }

            console.set_cursor_pos(28, 11);
            console.reset_color();
            console.draw_text("with ");

            console.set_color(Color::LightCyan, Color::Default);
            match game_state.level_pack.level_pack_best_moves_sum() {
                None => console.draw_text("XXXXXX"),
                Some(best_moves_sum) => console.draw_text(format!("{:06}", best_moves_sum)),
            }

            console.reset_color();
            console.draw_text(" moves!");

            console.set_cursor_pos(6, 15);
            console.reset_color();
            console.draw_text("Press ");

            console.draw_key_input_text("ENTER");
            console.reset_color();
            console.draw_text("/");
            console.draw_key_input_text("SPACEBAR");

            console.reset_color();
            console.draw_text(" to go back to the level selection screen");
        }else if let Some(playing_level) = self.level.as_ref() {
            let level = &playing_level.current_playing_level().0;

            let x_offset = ((Game::CONSOLE_MIN_WIDTH - level.width()) as f64 * 0.5) as usize;
            let y_offset = 1;

            if self.show_floor {
                level.draw_floor(console, x_offset, y_offset, game_state.is_player_background(), playing_level.original_level(), None);
            }else {
                level.draw(console, x_offset, y_offset, game_state.is_player_background(), None);
            }

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

    fn animate(&mut self, game_state: &mut GameState) {
        if game_state.is_dialog_opened() || self.game_over_flag || self.continue_flag {
            return;
        }

        if let Some(playing_level) = &mut self.level &&
                playing_level.is_playing_animation() && !self.animation_first_frame {
            let move_result = playing_level.continue_animation();
            self.handle_move_result(game_state, move_result);
        }
        self.animation_first_frame = false;
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

            game_state.open_dialog(Dialog::new_yes_no("Back to level selection?"));

            return;
        }

        if self.game_over_flag {
            if key == Key::ENTER || key == Key::SPACE {
                self.continue_flag = false;
                self.game_over_flag = false;

                game_state.play_sound_effect(audio::UI_SELECT_EFFECT);

                game_state.set_screen(ScreenId::SelectLevel);
            }

            return;
        }

        let current_level_index = game_state.current_level_index;
        let Some(level_pack) = game_state.get_current_level_pack_mut() else {
            return;
        };

        //Reset
        if key == Key::R {
            let should_play_sound_effect = self.level.as_ref().unwrap().current_move_index() > 0 &&
                    ((self.time_min * 60) + self.time_sec) * 1000 + self.time_millis > 50;

            self.start_level(level_pack.levels()[current_level_index].level());

            if should_play_sound_effect {
                game_state.play_sound_effect(audio::LEVEL_RESET);
            }

            return;
        }

        if key == Key::Q {
            game_state.play_sound_effect_ui_select();
            self.show_floor = !self.show_floor;

            return;
        }

        //Level end (Prevent movement)
        if self.continue_flag {
            if key == Key::ENTER || key == Key::SPACE {
                self.continue_flag = false;

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

        //Prevent movement during animation
        if self.level.as_mut().unwrap().is_playing_animation() {
            return;
        }

        if key == Key::U || key == Key::Z {
            let level = self.level.as_mut().unwrap().undo_move();
            if level.is_some() {
                game_state.play_sound_effect(audio::UNDO_REDO_EFFECT);
            }

            return;
        }else if key == Key::Y {
            let level = self.level.as_mut().unwrap().redo_move();
            if level.is_some() {
                game_state.play_sound_effect(audio::UNDO_REDO_EFFECT);
            }

            return;
        }

        let direction = match key {
            Key::W | Key::UP => Some(Direction::Up),
            Key::A | Key::LEFT => Some(Direction::Left),
            Key::S | Key::DOWN => Some(Direction::Down),
            Key::D | Key::RIGHT => Some(Direction::Right),

            _ => None,
        };

        if let Some(direction) = direction {
            let move_result = self.level.as_mut().unwrap().move_player(direction);
            if move_result.is_animation() {
                self.animation_first_frame = true;
            }
            self.handle_move_result(game_state, move_result);
        }
    }

    fn on_dialog_selection(&mut self, game_state: &mut GameState, selection: DialogSelection) {
        if selection == DialogSelection::Yes {
            self.continue_flag = false;
            self.game_over_flag = false;

            game_state.set_screen(ScreenId::SelectLevel);
        }else if selection == DialogSelection::No {
            self.on_continue(game_state);
        }
    }

    fn on_pause(&mut self, _: &mut GameState) {
        self.time_start_in_menu = Some(SystemTime::now());
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

        if let Some(background_music_id) = game_state.get_current_level_pack().as_ref().unwrap().background_music_id() {
            game_state.set_background_music_loop(audio::BACKGROUND_MUSIC_TRACKS.get_track_by_id(background_music_id));
        }else {
            game_state.stop_background_music();
        }
    }
}
