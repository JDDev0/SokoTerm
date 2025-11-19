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
#[cfg(feature = "steam")]
use crate::game::steam;

pub mod dialog;
pub mod utils;

#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub enum ScreenId {
    StartMenu,
    About,

    SelectLevelPack,
    SelectLevel,

    InGame,

    SelectLevelPackEditor,
    SelectLevelPackBackgroundMusic,
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
        console.set_cursor_pos(65, 20);
        console.draw_text("About: ");
        console.set_color(Color::LightRed, Color::Default);
        console.draw_text("a");

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

        if key == Key::A {
            game_state.set_screen(ScreenId::About);

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

        if row == 20 && column > 64 && column < 73 {
            game_state.set_screen(ScreenId::About);
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

        console.set_color(Color::Red, Color::Default);
        console.set_cursor_pos(Game::CONSOLE_MIN_WIDTH - 1, 2);
        console.draw_text("^");

        console.set_color(Color::Red, Color::Default);
        console.set_cursor_pos(Game::CONSOLE_MIN_WIDTH - 1, Game::CONSOLE_MIN_HEIGHT - 1);
        console.draw_text("v");

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
        console.draw_text("About Console Sokoban:");
        console.set_underline(false);

        self.draw_scrollbar(console);

        let mut current_row = 2;
        if self.set_cursor_pos_if_visible(console, 0, current_row) {
            console.set_color(Color::LightYellow, Color::Default);
            console.draw_text("Console Sokoban");

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
            console.draw_text("https://github.com/JDDev0/ConsoleSokoban");
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
            console.draw_text("Console Sokoban");

            console.reset_color();
            console.draw_text("!");
        }
    }

    fn on_key_pressed(&mut self, game_state: &mut GameState, key: Key) {
        if key == Key::UP && self.scroll_position_row > 0 {
            self.scroll_position_row -= 1;
        }else if key == Key::DOWN && self.scroll_position_row < self.scroll_position_row_max {
            self.scroll_position_row += 1;
        }

        if key == Key::ESC {
            game_state.set_screen(ScreenId::StartMenu);
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

        //Include Level Pack Editor entry (And Steam Workshop entry on steam build)
        let entry_count = game_state.get_level_pack_count() + if cfg!(feature = "steam") { 2 } else { 1 };

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
            if i == game_state.get_level_pack_count() + 1 {
                //And Steam Workshop entry on steam build
                console.set_color(Color::White, Color::LightBlue);
                console.draw_text("[]");
            }else if i == game_state.get_level_pack_count() {
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
                console.draw_text(utils::number_to_string_leading_ascii(2, i as u32 + 1, false));
            }

            console.reset_color();
            console.draw_text("|");

            console.set_cursor_pos(x, y + 1);
            console.draw_text("---");
        }

        //Mark selected level pack
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

        if game_state.get_level_pack_index() >= game_state.get_level_pack_count() {
            if game_state.get_level_pack_index() == game_state.get_level_pack_count() + 1 {
                #[cfg(feature = "steam")]
                {
                    //And Steam Workshop entry on steam build
                    console.set_cursor_pos(14, y + 1);
                    console.draw_text("Download level packs from the Steam Workshop");

                    console.set_cursor_pos(8, y + 3);
                    console.set_color(Color::LightBlack, Color::Default);
                    console.draw_text("You must relaunch the game after downloading level packs.");
                }

                #[cfg(not(feature = "steam"))]
                unreachable!();
            }else {
                //Level Pack Editor entry
                console.set_cursor_pos(23, y + 2);
                console.draw_text("Create or edit level packs");
            }
        }else {
            //Draw sum of best time and sum of best moves
            console.set_cursor_pos(1, y + 1);
            console.draw_text(format!("Selected level pack: {}", game_state.level_packs().get(game_state.get_level_pack_index()).unwrap().name()));

            #[cfg(feature = "steam")]
            if game_state.level_packs().get(game_state.get_level_pack_index()).unwrap().steam_workshop_id().is_some() {
                console.draw_text(" [");

                console.set_color(Color::Red, Color::Default);
                console.draw_text("o");

                console.reset_color();
                console.draw_text(": open Steam Workshop]");
            }

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

        #[cfg(feature = "steam")]
        if key == Key::O && game_state.get_level_pack_index() < game_state.get_level_pack_count() &&
                let Some(id) = game_state.level_packs().get(game_state.get_level_pack_index()).unwrap().steam_workshop_id() {
            game_state.steam_client.friends().activate_game_overlay_to_web_page(&format!("steam://url/CommunityFilePage/{}", id.0));
        }

        'outer: {
            //Include Level Pack Editor entry (And Steam Workshop entry on steam build)
            let entry_count = game_state.get_level_pack_count() + if cfg!(feature = "steam") { 2 } else { 1 };

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

                    if game_state.get_level_pack_index() >= game_state.get_level_pack_count() {
                        if game_state.get_level_pack_index() == game_state.get_level_pack_count() + 1 {
                            #[cfg(feature = "steam")]
                            {
                                //And Steam Workshop entry on steam build
                                game_state.steam_client.friends().activate_game_overlay_to_web_page(&format!("steam://url/SteamWorkshopPage/{}", steam::APP_ID.0));
                            }

                            #[cfg(not(feature = "steam"))]
                            unreachable!();
                        }else {
                            //Level Pack Editor entry
                            game_state.set_level_index(0);

                            game_state.set_screen(ScreenId::SelectLevelPackEditor);
                        }
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

        //Include Level Pack Editor entry (And Steam Workshop entry on steam build)
        let entry_count = game_state.get_level_pack_count() + if cfg!(feature = "steam") { 2 } else { 1 };

        let level_pack_index = column/3 + (row - 1)/2*24;
        if level_pack_index < entry_count {
            game_state.set_level_pack_index(level_pack_index);
            self.on_key_pressed(game_state, Key::ENTER);
        }
    }

    fn on_set_screen(&mut self, game_state: &mut GameState) {
        game_state.set_background_music_loop(&audio::BACKGROUND_MUSIC_FIELDS_OF_ICE);
    }
}

pub struct ScreenSelectLevel {
    selected_level: usize,
    level_preview: bool,
}

impl ScreenSelectLevel {
    pub fn new() -> Self {
        Self {
            selected_level: Default::default(),
            level_preview: false,
        }
    }

    fn draw_overview(&self, game_state: &GameState, console: &Console) {
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
        console.draw_text("Selected level:       ");
        let selected_level = self.selected_level;
        console.draw_text(format!("{:03}", selected_level as u32 + 1));

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

        console.reset_color();
        console.set_cursor_pos(29, y + 1);
        console.draw_text("Press ");

        console.set_color(Color::Red, Color::Default);
        console.draw_text("p");

        console.reset_color();
        console.draw_text(" for level preview");
    }

    fn draw_level_preview(&self, game_state: &GameState, console: &Console) {
        if self.selected_level > 0 {
            console.set_color(Color::Red, Color::Default);
            console.draw_text("<");

            console.reset_color();
            console.draw_text(format!(" Level {:03}", self.selected_level));
        }

        if self.selected_level < game_state.get_current_level_pack().unwrap().level_count() - 1 {
            console.reset_color();
            console.set_cursor_pos(Game::CONSOLE_MIN_WIDTH - 11, 0);
            console.draw_text(format!("Level {:03} ", self.selected_level + 2));

            console.set_color(Color::Red, Color::Default);
            console.draw_text(">");
        }

        console.reset_color();
        console.set_cursor_pos(((Game::CONSOLE_MIN_WIDTH - 23) as f64 * 0.5) as usize, 0);
        console.draw_text("Preview (");

        console.set_color(Color::Red, Color::Default);
        console.draw_text("p");

        console.reset_color();
        console.draw_text(format!(") [Level {:03}]", self.selected_level + 1));

        let min_level_not_completed = game_state.get_current_level_pack().as_ref().unwrap().min_level_not_completed();
        let level = game_state.get_current_level_pack().unwrap().levels()[self.selected_level].level();

        if self.selected_level > min_level_not_completed {
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
            console.draw_text(format!("Beat level {:03} to unlock this level.", self.selected_level));
        }else {
            let x_offset = ((Game::CONSOLE_MIN_WIDTH - level.width()) as f64 * 0.5) as usize;
            let y_offset = 1;

            level.draw(console, x_offset, y_offset, game_state.is_player_background(), None);
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
                game_state.set_screen(ScreenId::SelectLevelPack);
            }

            return;
        }

        if key == Key::F1 {
            game_state.open_help_page();

            return;
        }

        if key == Key::P {
            game_state.play_sound_effect(audio::UI_SELECT_EFFECT);

            self.level_preview = !self.level_preview;

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
                    self.level_preview = false;

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

        if row == 0 {
            return;
        }

        let level_count = game_state.get_current_level_pack().as_ref().unwrap().level_count();
        let y = 4 + ((level_count - 1)/24)*2;
        if row == y + 1 && (29..54).contains(&column) {
            self.level_preview = true;
        }

        let level_index = column/3 + (row - 1)/2*24;
        if level_index < level_count {
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
        if game_state.get_level_pack_index() == 0 { //Built-in Tutorial pack
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
        }else if game_state.get_level_pack_index() == 1 { //Built-in Main pack
            console.reset_color();
            if game_state.current_level_index < 3 {
                let start_y = if game_state.current_level_index < 2 { 8 } else { 11 };

                console.set_cursor_pos(29, start_y);
                console.set_color(Color::Red, Color::Default);
                console.draw_text("z");

                console.reset_color();
                console.draw_text(": Undo, ");

                console.set_color(Color::Red, Color::Default);
                console.draw_text("y");

                console.reset_color();
                console.draw_text(": Redo");

                console.set_cursor_pos(29, start_y + 1);
                console.set_color(Color::Red, Color::Default);
                console.draw_text("r");

                console.reset_color();
                console.draw_text(": Restart Level");
            }
        }else if game_state.get_level_pack_index() == 2 { //Built-in Special pack
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
            let should_play_sound_effect = self.level.as_ref().unwrap().current_index() > 0 &&
                    ((self.time_min * 60) + self.time_sec) * 1000 + self.time_millis > 50;

            self.start_level(level_pack.levels()[current_level_index].level());

            if should_play_sound_effect {
                game_state.play_sound_effect(audio::LEVEL_RESET);
            }

            return;
        }

        //Level end
        if self.continue_flag {
            if key == Key::ENTER {
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

                #[cfg(feature = "steam")]
                if level_pack.id() == "main" && current_level_index == 95 && moves < 160 {
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

                    if level_pack.steam_workshop_id().is_some() {
                        Achievement::STEAM_WORKSHOP_LEVEL_PACK_COMPLETED.unlock(steam_client.clone());
                    }
                }

                if let Err(err) = level_pack.save_save_game(false) {
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

        #[cfg(feature = "steam")]
        if game_state.get_current_level_pack().unwrap().steam_workshop_id().is_some() {
            Achievement::STEAM_WORKSHOP_LEVEL_PACK_PLAYED.unlock(game_state.steam_client.clone());
        }

        if let Some(background_music_id) = game_state.get_current_level_pack().as_ref().unwrap().background_music_id() {
            game_state.set_background_music_loop(audio::BACKGROUND_MUSIC_TRACKS.get_track_by_id(background_music_id));
        }else {
            game_state.stop_background_music();
        }
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
                console.set_color(Color::Black,  if game_state.editor_state.level_packs.get(i).
                        unwrap().level_pack_best_moves_sum().is_some() {
                    Color::Green
                }else {
                    Color::Yellow
                });
                console.draw_text(utils::number_to_string_leading_ascii(2, i as u32 + 1, false));
            }

            console.reset_color();
            console.draw_text("|");

            console.set_cursor_pos(x, y + 1);
            console.draw_text("---");
        }

        //Mark selected level pack
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
        let y = 4 + ((entry_count - 1)/24)*2;

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

            console.set_cursor_pos(1, y + 3);
            console.draw_text("Background music: ");

            match game_state.editor_state.get_current_level_pack().unwrap().
                    background_music_id().
                    map(|background_music_id| audio::BACKGROUND_MUSIC_TRACKS.get_track_by_id(background_music_id)) {
                Some(background_music) => {
                    console.set_color(Color::LightCyan, Color::Default);
                    console.draw_text(background_music.display_name());

                    console.reset_color();
                    console.draw_text(" [by ");

                    console.set_color(Color::LightPink, Color::Default);
                    console.draw_text(background_music.creator());

                    console.reset_color();
                    console.draw_text("]");
                },

                None => {
                    console.draw_text("None");
                },
            }

            console.set_color(Color::Red, Color::Default);
            console.set_cursor_pos(46, y + 1);
            console.draw_text("s");

            console.reset_color();
            console.draw_text(":  Select background music");

            #[cfg(feature = "steam")]
            {
                console.set_color(Color::Red, Color::Default);
                console.set_cursor_pos(46, y + 2);
                console.draw_text("u");

                console.reset_color();
                console.draw_text(": Upload to Steam Workshop");
            }
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

                    //self.is_creating_new_level_pack with be set to false in on_set_screen after background music selection
                    self.new_level_pack_id = String::new();

                    game_state.editor_state.set_level_pack_index(index);
                    game_state.editor_state.set_level_index(0);
                    game_state.set_screen(ScreenId::SelectLevelPackBackgroundMusic);
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

        if key == Key::S && game_state.editor_state.selected_level_pack_index != game_state.editor_state.get_level_pack_count() {
            match game_state.editor_state.get_current_level_pack().unwrap().
                    background_music_id().
                    map(|background_music_id| audio::BACKGROUND_MUSIC_TRACKS.get_track_by_id(background_music_id)) {
                Some(background_music) => game_state.set_background_music_loop(background_music),
                None => game_state.stop_background_music(),
            }

            game_state.set_screen(ScreenId::SelectLevelPackBackgroundMusic);
        }

        if key == Key::E && game_state.editor_state.selected_level_pack_index != game_state.editor_state.get_level_pack_count() {
            self.is_exporting_level_pack = true;

            game_state.open_dialog(Box::new(DialogYesNo::new("Do you want to export the level pack to the current directory?")));
        }

        #[cfg(feature = "steam")]
        if key == Key::U && game_state.editor_state.selected_level_pack_index != game_state.editor_state.get_level_pack_count() {
            let level_stats = &game_state.editor_state.get_current_level_pack().unwrap();
            if level_stats.level_pack_best_moves_sum().is_none() {
                game_state.open_dialog(Box::new(DialogOk::new_error(
                    "Level pack was not validated yet! All levels must be validated.",
                )));

                return;
            }

            let ret = steam::prepare_workshop_upload_temp_data(
                game_state.editor_state.get_current_level_pack().unwrap(),
            );
            if let Err(err) = ret {
                game_state.open_dialog(Box::new(DialogOk::new_error(
                    "Could not prepare files for upload to steam workshop!",
                )));
                println!("{err}"); //TODO remove and create proper popup with erro text

                return;
            }

            game_state.play_sound_effect_ui_dialog_open();
            game_state.show_workshop_upload_popup = true;
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
                    if game_state.editor_state.selected_level_pack_index < 24 {
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

        let y = 4 + ((entry_count - 1)/24)*2;
        if row == y + 1 && (46..Game::CONSOLE_MIN_WIDTH - 1).contains(&column) {
            self.on_key_pressed(game_state, Key::S);
        }

        #[cfg(feature = "steam")]
        {
            if row == y + 2 && (46..Game::CONSOLE_MIN_WIDTH - 1).contains(&column) {
                self.on_key_pressed(game_state, Key::U);
            }
        }

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

                if let Err(err) = level_pack.export_editor_level_pack_to_path(path) {
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

    fn on_set_screen(&mut self, game_state: &mut GameState) {
        if self.is_creating_new_level_pack {
            //Background music was selected for newly created level pack -> Do not change music and enter level pack editor

            self.is_creating_new_level_pack = false;
            game_state.set_screen(ScreenId::LevelPackEditor);
        }else {
            game_state.set_background_music_loop(&audio::BACKGROUND_MUSIC_FIELDS_OF_ICE);
        }
    }
}

pub struct ScreenSelectLevelPackBackgroundMusic {}

impl ScreenSelectLevelPackBackgroundMusic {
    pub fn new() -> Self {
        Self {}
    }
}

impl Screen for ScreenSelectLevelPackBackgroundMusic {
    fn draw(&self, game_state: &GameState, console: &Console) {
        console.reset_color();
        console.set_underline(true);
        console.draw_text("Select the background music for the level pack:");
        console.set_underline(false);

        console.set_color(Color::Red, Color::Default);
        console.set_cursor_pos(0, 1);
        console.draw_text("ENTER");

        console.reset_color();
        console.draw_text(": Save selection");

        console.set_color(Color::Red, Color::Default);
        console.set_cursor_pos(0, 2);
        console.draw_text("ESC");

        console.reset_color();
        console.draw_text(": Cancel");

        console.reset_color();
        console.set_cursor_pos(0, 4);
        console.draw_text("( ) None");

        let current_selected_music_index = game_state.current_background_music_id().
                map(|id| id.id()).
                unwrap_or(0);

        for track in audio::BACKGROUND_MUSIC_TRACKS.tracks() {
            console.reset_color();
            console.set_cursor_pos(0, track.id().id() + 4);
            console.reset_color();
            console.draw_text("( ) ");

            console.set_color(Color::LightCyan, Color::Default);
            console.draw_text(format!("{:35}", track.display_name()));

            console.reset_color();
            console.draw_text(" [by ");

            console.set_color(Color::LightPink, Color::Default);
            console.draw_text(track.creator());

            console.reset_color();
            console.draw_text("]");
        }

        console.set_color(Color::Yellow, Color::Default);
        console.set_cursor_pos(1, current_selected_music_index + 4);
        console.draw_text("X");
    }

    fn on_key_pressed(&mut self, game_state: &mut GameState, key: Key) {
        let current_background_music_id = game_state.current_background_music_id();
        let mut current_selected_music_index = current_background_music_id.
                map(|id| id.id()).
                unwrap_or(0);

        if key == Key::UP && current_selected_music_index > 0 {
            current_selected_music_index -= 1;
        }else if key == Key::DOWN && current_selected_music_index < audio::BACKGROUND_MUSIC_TRACKS.tracks().len() {
            current_selected_music_index += 1;
        }

        if current_selected_music_index == 0 {
            game_state.stop_background_music();
        }else {
            game_state.set_background_music_loop(audio::BACKGROUND_MUSIC_TRACKS.get_track_by_id(
                audio::BACKGROUND_MUSIC_TRACKS.check_id(current_selected_music_index).unwrap(),
            ));
        }

        if key == Key::ENTER {
            game_state.editor_state.get_current_level_pack_mut().unwrap().set_background_music_id(current_background_music_id);

            if let Err(err) = game_state.editor_state.get_current_level_pack().unwrap().save_editor_level_pack() {
                game_state.open_dialog(Box::new(DialogOk::new_error(format!("Cannot save: {}", err))));
            }
        }

        if key == Key::ENTER || key == Key::ESC {
            game_state.set_screen(ScreenId::SelectLevelPackEditor);
        }
    }

    fn on_mouse_pressed(&mut self, game_state: &mut GameState, column: usize, row: usize) {
        if row == 1 && column < 5 {
            self.on_key_pressed(game_state, Key::ENTER);
        }else if row == 2 && column < 3 {
            self.on_key_pressed(game_state, Key::ESC);
        }

        if row < 4 {
            return;
        }

        let background_music_selection_index = row - 4;
        if background_music_selection_index > audio::BACKGROUND_MUSIC_TRACKS.tracks().len() {
            return;
        }

        if background_music_selection_index == 0 {
            game_state.stop_background_music();
        }else {
            game_state.set_background_music_loop(audio::BACKGROUND_MUSIC_TRACKS.get_track_by_id(
                audio::BACKGROUND_MUSIC_TRACKS.check_id(background_music_selection_index).unwrap(),
            ));
        }
    }
}

pub struct ScreenLevelPackEditor {
    level_preview: bool,
    is_creating_new_level: bool,
    is_editing_height: bool,
    is_deleting_level: bool,
    new_level_width_str: String,
    new_level_height_str: String,
}

impl ScreenLevelPackEditor {
    pub fn new() -> Self {
        Self {
            level_preview: false,
            is_creating_new_level: Default::default(),
            is_editing_height: Default::default(),
            is_deleting_level: Default::default(),
            new_level_width_str: String::new(),
            new_level_height_str: String::new(),
        }
    }

    fn draw_overview(&self, game_state: &GameState, console: &Console) {
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
                console.set_color(Color::Black, if game_state.editor_state.get_current_level_pack().
                        unwrap().levels()[i].best_moves().is_some() {
                    Color::Green
                }else {
                    Color::Yellow
                });
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
        let y = 4 + ((entry_count - 1)/24)*2;

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
                console.set_cursor_pos(30, y + 2);
                console.draw_text("Create a level");
            }
        }else {
            //Draw best time and best moves
            console.set_cursor_pos(1, y + 1);
            console.draw_text("Selected level: ");
            let selected_level = game_state.editor_state.selected_level_index;
            console.draw_text(format!("{:03}", selected_level as u32 + 1));

            if game_state.editor_state.get_current_level_pack().unwrap().
                    thumbnail_level_index().is_some_and(|index| index == game_state.editor_state.selected_level_index) {
                console.draw_text(" [Thumbnail]");

                console.reset_color();
                console.set_cursor_pos(Game::CONSOLE_MIN_WIDTH - 38, y + 2);
                console.draw_text("Press ");

                console.set_color(Color::Red, Color::Default);
                console.draw_text("t");

                console.reset_color();
                console.draw_text(" to unset level pack thumbnail");
            }else {
                console.reset_color();
                console.set_cursor_pos(Game::CONSOLE_MIN_WIDTH - 36, y + 2);
                console.draw_text("Press ");

                console.set_color(Color::Red, Color::Default);
                console.draw_text("t");

                console.reset_color();
                console.draw_text(" to set level pack thumbnail");
            }

            console.set_cursor_pos(1, y + 2);
            console.draw_text(format!(
                "Size: {} x {}",
                game_state.editor_state.get_current_level().unwrap().width(),
                game_state.editor_state.get_current_level().unwrap().height(),
            ));

            console.set_cursor_pos(1, y + 3);
            console.draw_text("Validation: ");
            {
                let level_stats = &game_state.editor_state.get_current_level_pack().unwrap().
                        levels()[game_state.editor_state.selected_level_index];

                if let Some(best_moves) = level_stats.best_moves() {
                    console.set_color(Color::Green, Color::Default);
                    console.draw_text(format!("Best moves: {best_moves}"));
                }else {
                    console.set_color(Color::Red, Color::Default);
                    console.draw_text("You need to complete this level to validate it");
                }
            }

            console.reset_color();
            console.set_cursor_pos(Game::CONSOLE_MIN_WIDTH - 26, y + 1);
            console.draw_text("Press ");

            console.set_color(Color::Red, Color::Default);
            console.draw_text("p");

            console.reset_color();
            console.draw_text(" for level preview");
        }
    }

    fn draw_level_preview(&self, game_state: &GameState, console: &Console) {
        let selected_level = game_state.editor_state.get_level_index();

        if selected_level > 0 {
            console.set_color(Color::Red, Color::Default);
            console.draw_text("<");

            console.reset_color();
            console.draw_text(format!(" Level {:03}", selected_level));
        }

        if game_state.editor_state.get_current_level_pack().unwrap().level_count() > 0 &&
                selected_level < game_state.editor_state.get_current_level_pack().unwrap().level_count() - 1 {
            console.reset_color();
            console.set_cursor_pos(Game::CONSOLE_MIN_WIDTH - 11, 0);
            console.draw_text(format!("Level {:03} ", selected_level + 2));

            console.set_color(Color::Red, Color::Default);
            console.draw_text(">");
        }

        if game_state.editor_state.get_current_level_pack().unwrap().level_count() > 0 &&
                selected_level == game_state.editor_state.get_current_level_pack().unwrap().level_count() - 1 {
            console.reset_color();
            console.set_cursor_pos(Game::CONSOLE_MIN_WIDTH - 16, 0);
            console.draw_text("Create a level ");

            console.set_color(Color::Red, Color::Default);
            console.draw_text(">");
        }

        if selected_level == game_state.editor_state.get_current_level_pack().unwrap().level_count() {
            let has_max_level_count = game_state.editor_state.get_current_level_pack().unwrap().level_count() == LevelPack::MAX_LEVEL_COUNT_PER_PACK;

            console.reset_color();
            console.set_cursor_pos(((Game::CONSOLE_MIN_WIDTH - 11) as f64 * 0.5) as usize, 0);
            console.draw_text("Preview (");

            console.set_color(Color::Red, Color::Default);
            console.draw_text("p");

            console.reset_color();
            console.draw_text(")");

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
                console.set_cursor_pos(30, y + 2);
                console.reset_color();
                console.draw_text("Create a level");
            }
        }else {
            console.reset_color();
            console.set_cursor_pos(((Game::CONSOLE_MIN_WIDTH - 23) as f64 * 0.5) as usize, 0);
            console.draw_text("Preview (");

            console.set_color(Color::Red, Color::Default);
            console.draw_text("p");

            console.reset_color();
            console.draw_text(format!(") [Level {:03}]", selected_level + 1));

            let level = game_state.editor_state.get_current_level_pack().unwrap().levels()[selected_level].level();

            let x_offset = ((Game::CONSOLE_MIN_WIDTH - level.width()) as f64 * 0.5) as usize;
            let y_offset = 1;

            level.draw(console, x_offset, y_offset, game_state.is_player_background(), None);
        }
    }
}

impl Screen for ScreenLevelPackEditor {
    fn draw(&self, game_state: &GameState, console: &Console) {
        if self.level_preview {
            self.draw_level_preview(game_state, console);
        }else {
            self.draw_overview(game_state, console);
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

            if self.level_preview {
                self.level_preview = false;
            }else {
                game_state.set_screen(ScreenId::SelectLevelPackEditor);
            }

            return;
        }

        if key == Key::F1 {
            game_state.open_help_page();

            return;
        }

        if key == Key::P {
            game_state.play_sound_effect(audio::UI_SELECT_EFFECT);

            self.level_preview = !self.level_preview;

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
                    if game_state.editor_state.selected_level_index < 24 {
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
                    self.level_preview = false;

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

                Key::T => {
                    let selected_level_index = game_state.editor_state.selected_level_index;
                    if selected_level_index != game_state.editor_state.get_current_level_pack().unwrap().level_count() {
                        game_state.play_sound_effect(audio::UI_SELECT_EFFECT);

                        if game_state.editor_state.get_current_level_pack().unwrap().
                                thumbnail_level_index().is_some_and(|index| index == selected_level_index) {
                            game_state.editor_state.get_current_level_pack_mut().unwrap().set_thumbnail_level_index(None);
                        }else {
                            game_state.editor_state.get_current_level_pack_mut().unwrap().set_thumbnail_level_index(Some(selected_level_index));
                        }

                        if let Err(err) = game_state.editor_state.get_current_level_pack().unwrap().save_editor_level_pack() {
                            game_state.open_dialog(Box::new(DialogOk::new_error(format!("Cannot save: {}", err))));
                        }
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

                let selected_level = game_state.editor_state.get_level_index();
                if game_state.editor_state.get_current_level_pack().unwrap().level_count() > 0 &&
                        selected_level == game_state.editor_state.get_current_level_pack().unwrap().level_count() - 1 &&
                        column > Game::CONSOLE_MIN_WIDTH - 17 {
                    self.on_key_pressed(game_state, Key::RIGHT);
                }
            }

            return;
        }

        if row == 0 {
            return;
        }

        //Include create Level entry
        let entry_count = game_state.editor_state.get_current_level_pack().unwrap().level_count() + 1;

        let y = 4 + ((entry_count - 1)/24)*2;
        if row == y + 1 && (Game::CONSOLE_MIN_WIDTH - 26..Game::CONSOLE_MIN_WIDTH - 1).contains(&column) {
            self.level_preview = true;
        }

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
                let level_pack = game_state.editor_state.get_current_level_pack_mut().unwrap();
                level_pack.levels_mut().remove(index);
                level_pack.calculate_stats_sum();

                if let Err(err) = game_state.editor_state.get_current_level_pack().unwrap().save_editor_level_pack() {
                    game_state.open_dialog(Box::new(DialogOk::new_error(format!("Cannot save: {}", err))));
                }
            }
        }
    }

    fn on_set_screen(&mut self, game_state: &mut GameState) {
        if let Some(background_music_id) = game_state.editor_state.get_current_level_pack().as_ref().unwrap().background_music_id() {
            game_state.set_background_music_loop(audio::BACKGROUND_MUSIC_TRACKS.get_track_by_id(background_music_id));
        }else {
            game_state.stop_background_music();
        }
    }
}

pub struct ScreenLevelEditor {
    level: UndoHistory<Level>,
    is_vertical_input: bool,
    is_reverse_input: bool,
    continue_flag: bool,
    validation_result_history_index: usize,
    //TODO best time
    validation_best_moves: Option<u32>,
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
            continue_flag: false,
            validation_result_history_index: 0,
            //TODO best time
            validation_best_moves: None,
            playing_level: Default::default(),
            cursor_pos: Default::default(),
        }
    }

    fn on_key_pressed_playing(&mut self, game_state: &mut GameState, key: Key) {
        if self.continue_flag {
            return;
        }

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

                let has_player_moved = player_pos != (x_from, y_from);
                if has_player_moved {
                    level_history.commit_change((level, player_pos));
                }

                if has_won {
                    self.continue_flag = true;

                    //Update validation
                    self.validation_result_history_index = self.level.current_index(); //Use current index of editor level history

                    //TODO best time

                    //Use current index of playing level history
                    let moves = level_history.current_index() as u32;
                    if self.validation_best_moves.is_none_or(|best_moves| moves < best_moves) {
                        self.validation_best_moves = Some(moves);
                    }

                    game_state.play_sound_effect(audio::LEVEL_COMPLETE_EFFECT);
                }

                if has_player_moved {
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

            if self.continue_flag {
                console.set_cursor_pos(((Game::CONSOLE_MIN_WIDTH - 16) as f64 * 0.5) as usize, 0);
                console.draw_text("Level validated!");
            }

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

            console.set_cursor_pos(Game::CONSOLE_MIN_WIDTH - 14, 0);
            console.draw_text("Validated: ");
            {
                let validated = self.validation_result_history_index == self.level.current_index() &&
                        self.validation_best_moves.is_some();

                if validated {
                    console.set_color(Color::Green, Color::Default);
                    console.draw_text("Yes");
                }else {
                    console.set_color(Color::Red, Color::Default);
                    console.draw_text(" No");
                }
            }
        }

        console.reset_color();
        console.set_cursor_pos(((Game::CONSOLE_MIN_WIDTH - 9) as f64 * 0.25) as usize, 0);
        console.draw_text("Level: ");
        console.draw_text(utils::number_to_string_leading_ascii(2, game_state.editor_state.selected_level_index as u32 + 1, true));

        let x_offset = ((Game::CONSOLE_MIN_WIDTH - self.level.current().width()) as f64 * 0.5) as usize;
        let y_offset = 1;

        self.playing_level.as_ref().map_or(self.level.current(), |level| &level.current().0).
                draw(console, x_offset, y_offset, game_state.is_player_background(),
                     self.playing_level.as_ref().map_or(Some(self.cursor_pos), |_| None));
    }

    fn on_key_pressed(&mut self, game_state: &mut GameState, key: Key) {
        if key == Key::ESC {
            game_state.open_dialog(Box::new(DialogYesCancelNo::new("Exiting (Save changes and level validation state?)")));

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
                self.continue_flag = false;

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
            let index = game_state.editor_state.selected_level_index;
            let level_pack = game_state.editor_state.get_current_level_pack_mut().unwrap();
            let level = level_pack.levels_mut().get_mut(index).unwrap();

            *level.level_mut() = self.level.current().clone();

            if self.validation_result_history_index == self.level.current_index() {
                //TODO best time
                level.set_best_moves(self.validation_best_moves);
            }else {
                //Reset validation if editor level current history index does not match validation history index
                //TODO best time
                level.set_best_moves(None);
            }
            level_pack.calculate_stats_sum();

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

        let level = game_state.editor_state.get_current_level_pack().
                unwrap().levels().get(game_state.editor_state.selected_level_index).unwrap();
        self.level.clear_with_new_initial(level.level().clone());

        //Validation is valid for first history element
        self.validation_result_history_index = 0;
        //TODO best time
        self.validation_best_moves = level.best_moves();
    }
}
