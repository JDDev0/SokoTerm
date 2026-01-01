use std::collections::HashMap;
use std::error::Error;
use std::ffi::OsString;
use std::fmt::{Debug, Display, Formatter};
use std::mem;
use std::fs::File;
use std::io::Write;
use std::str::FromStr;
use crate::game::audio::{AudioHandler, BackgroundMusic, BackgroundMusicId};
use crate::game::help_page::HelpPage;
use crate::game::level::{LevelPack, SoundEffect};
use crate::game::screen::*;
use crate::game::screen::dialog::{DialogType, RenderedDialog, Dialog};
use crate::io::{Console, Key};

#[cfg(feature = "gui")]
use bevy::prelude::*;
#[cfg(feature = "steam")]
use bevy_steamworks::*;

pub mod level;
pub(crate) mod screen;
mod help_page;
pub mod audio;
pub mod console_extension;

#[cfg(feature = "steam")]
pub mod steam;

#[derive(Default, Debug, Clone, Copy, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub enum TileMode {
    Ascii,
    #[default]
    Graphical,
}

impl TileMode {
    pub fn display_name(self) -> &'static str {
        match self {
            TileMode::Ascii => "Ascii",
            TileMode::Graphical => "Graphical",
        }
    }

    #[must_use]
    pub fn toggle(self) -> Self {
        match self {
            TileMode::Ascii => TileMode::Graphical,
            TileMode::Graphical => TileMode::Ascii,
        }
    }
}

impl Display for TileMode {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.display_name())
    }
}

impl FromStr for TileMode {
    type Err = GameError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "Ascii" => Ok(TileMode::Ascii),
            "Graphical" => Ok(TileMode::Graphical),

            _ => Err(GameError::new("Invalid tile mode \"{s}\"")),
        }
    }
}

#[derive(Default, Debug, Clone, Copy, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub enum AnimationSpeed {
    Slow,
    #[default]
    Normal,
    Fast,
    VeryFast,
}

impl AnimationSpeed {
    pub fn display_name(self) -> &'static str {
        match self {
            AnimationSpeed::Slow => "Slow",
            AnimationSpeed::Normal => "Normal",
            AnimationSpeed::Fast => "Fast",
            AnimationSpeed::VeryFast => "Very fast",
        }
    }

    pub fn animation_count_per_update(self) -> f32 {
        match self {
            AnimationSpeed::Slow => 0.75,
            AnimationSpeed::Normal => 1.0,
            AnimationSpeed::Fast => 1.5,
            AnimationSpeed::VeryFast => 2.0,
        }
    }

    #[must_use]
    fn next_setting(self) -> Self {
        match self {
            AnimationSpeed::Slow => AnimationSpeed::Normal,
            AnimationSpeed::Normal => AnimationSpeed::Fast,
            AnimationSpeed::Fast => AnimationSpeed::VeryFast,
            AnimationSpeed::VeryFast => AnimationSpeed::Slow,
        }
    }
}

impl Display for AnimationSpeed {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.display_name())
    }
}

impl FromStr for AnimationSpeed {
    type Err = GameError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "Slow" => Ok(AnimationSpeed::Slow),
            "Normal" => Ok(AnimationSpeed::Normal),
            "Fast" => Ok(AnimationSpeed::Fast),
            "VeryFast" => Ok(AnimationSpeed::VeryFast),

            _ => Err(GameError::new("Invalid animation speed \"{s}\"")),
        }
    }
}

pub struct GameSettings {
    color_scheme_index: usize,
    tile_mode: TileMode,

    background_music: bool,

    animation_speed: AnimationSpeed,
}

impl GameSettings {
    pub fn new() -> GameSettings {
        Self {
            color_scheme_index: 0,
            tile_mode: TileMode::default(),

            background_music: true,

            animation_speed: AnimationSpeed::default(),
        }
    }

    pub fn read_from_file() -> Result<Self, Box<dyn Error>> {
        let mut settings_save_file = Game::get_or_create_save_game_folder()?;
        settings_save_file.push("settings.data");

        let mut settings = GameSettings::new();

        if std::fs::exists(&settings_save_file)? {
            let settings_data = std::fs::read_to_string(&settings_save_file)?;
            for line in settings_data.split("\n").
                    filter(|line| !line.trim().is_empty()) {
                let mut tokens = line.splitn(2, " = ");

                let key = tokens.next();
                let value = tokens.next();

                if let Some(key) = key && let Some(value) = value {
                    match key {
                        "color_scheme_index" => {
                            let Ok(value) = usize::from_str(value) else {
                                #[cfg(feature = "gui")]
                                {
                                    warn!("\"settings.data\" contains invalid value for option \"{key}\": \"{value}\": Using default");
                                }

                                //TODO warning in cli version

                                continue;
                            };

                            #[cfg(feature = "gui")]
                            {
                                settings.color_scheme_index = value % crate::io::bevy_abstraction::COLOR_SCHEMES.len();
                            }

                            #[cfg(feature = "cli")]
                            {
                                //Not used in CLI build, but keep value as is for saving (CLI and GUI builds might both be played)
                                settings.color_scheme_index = value;
                            }
                        },

                        "tile_mode" => {
                            let Ok(value) = TileMode::from_str(value) else {
                                #[cfg(feature = "gui")]
                                {
                                    warn!("\"settings.data\" contains invalid value for option \"{key}\": \"{value}\": Using default");
                                }

                                //TODO warning in cli version

                                continue;
                            };

                            settings.tile_mode = value;
                        },

                        "background_music" => {
                            let Ok(value) = bool::from_str(value) else {
                                #[cfg(feature = "gui")]
                                {
                                    warn!("\"settings.data\" contains invalid value for option \"{key}\": \"{value}\": Using default");
                                }

                                //TODO warning in cli version

                                continue;
                            };

                            settings.background_music = value;
                        },

                        "animation_speed" => {
                            let Ok(value) = AnimationSpeed::from_str(value) else {
                                #[cfg(feature = "gui")]
                                {
                                    warn!("\"settings.data\" contains invalid value for option \"{key}\": \"{value}\": Using default");
                                }

                                //TODO warning in cli version

                                continue;
                            };

                            settings.animation_speed = value;
                        },

                        _ => {
                            #[cfg(feature = "gui")]
                            {
                                warn!("\"settings.data\" contains invalid settings option: \"{key}\" with value \"{value}\": Ignoring");
                            }

                            //TODO warning in cli version
                        }
                    }
                }else {
                    #[cfg(feature = "gui")]
                    {
                        warn!("\"settings.data\" contains invalid data: \"{line}\": Ignoring");
                    }

                    //TODO warning in cli version
                }
            }
        }

        Ok(settings)
    }

    pub fn save_to_file(&self) -> Result<(), Box<dyn Error>> {
        let mut settings_save_file = Game::get_or_create_save_game_folder()?;
        settings_save_file.push("settings.data");
        let mut file = File::create(settings_save_file)?;

        writeln!(file, "color_scheme_index = {}", self.color_scheme_index)?;
        writeln!(file, "tile_mode = {}", self.tile_mode)?;
        writeln!(file, "background_music = {}", self.background_music)?;
        writeln!(file, "animation_speed = {:?}", self.animation_speed)?;

        Ok(())
    }

    pub fn color_scheme_index(&self) -> usize {
        self.color_scheme_index
    }

    pub fn tile_mode(&self) -> TileMode {
        self.tile_mode
    }

    pub fn background_music(&self) -> bool {
        self.background_music
    }

    pub fn animation_speed(&self) -> AnimationSpeed {
        self.animation_speed
    }
}

impl Default for GameSettings {
    fn default() -> Self {
        GameSettings::new()
    }
}

pub struct GameState {
    current_screen_id: ScreenId,
    should_call_on_set_screen: bool,

    is_help: bool,
    dialog: Option<RenderedDialog>,

    current_level_pack_index: usize,
    level_pack: Box<LevelPack>,

    current_level_index: usize,

    is_player_background: bool,
    player_background_tmp: i32,

    pending_animation_play_count: f32,

    should_exit: bool,

    settings: GameSettings,

    audio_handler: Option<AudioHandler>,
    current_background_music_id: Option<BackgroundMusicId>,

    #[cfg(feature = "steam")]
    steam_client: Client,
}

impl GameState {
    fn new(
        level_pack: Box<LevelPack>,

        settings: GameSettings,

        #[cfg(feature = "steam")]
        steam_client: Client,
    ) -> Self {
        Self {
            current_screen_id: ScreenId::StartMenu,
            should_call_on_set_screen: Default::default(),

            is_help: Default::default(),
            dialog: Default::default(),

            current_level_pack_index: Default::default(),
            level_pack,

            current_level_index: Default::default(),

            is_player_background: Default::default(),
            player_background_tmp: Default::default(),

            pending_animation_play_count: 0.0,

            should_exit: Default::default(),

            settings,

            audio_handler: AudioHandler::new().ok(),
            current_background_music_id: None,

            #[cfg(feature = "steam")]
            steam_client,
        }
    }

    pub fn set_screen(&mut self, screen_id: ScreenId) {
        self.current_screen_id = screen_id;
        self.should_call_on_set_screen = true;
    }

    pub fn get_level_pack_index(&self) -> usize {
        self.current_level_pack_index
    }

    pub fn set_level_pack_index(&mut self, level_pack_index: usize) {
        self.current_level_pack_index = level_pack_index;
    }

    pub fn get_current_level_pack(&self) -> Option<&LevelPack> {
        Some(&self.level_pack)
    }

    pub fn get_current_level_pack_mut(&mut self) -> Option<&mut LevelPack> {
        Some(&mut self.level_pack)
    }

    pub fn get_level_index(&self) -> usize {
        self.current_level_index
    }

    pub fn set_level_index(&mut self, level_index: usize) {
        self.current_level_index = level_index;
    }

    pub fn is_player_background(&self) -> bool {
        self.is_player_background
    }

    pub fn open_help_page(&mut self) {
        self.play_sound_effect(audio::BOOK_OPEN_EFFECT);

        self.is_help = true;
    }

    pub fn close_help_page(&mut self) {
        self.play_sound_effect(audio::UI_SELECT_EFFECT);

        self.is_help = false;
    }

    pub fn is_dialog_opened(&self) -> bool {
        self.dialog.is_some()
    }

    pub fn open_dialog(&mut self, dialog: Dialog) {
        let dialog_type = dialog.dialog_type();

        self.dialog = Some(dialog.render(Game::CONSOLE_MIN_WIDTH, Game::CONSOLE_MIN_HEIGHT));

        match dialog_type {
            DialogType::Information => {
                self.play_sound_effect_ui_dialog_open();
            },
            DialogType::Error => {
                self.play_sound_effect_ui_error();
            },
        }
    }

    pub fn close_dialog(&mut self) {
        self.dialog = None;
    }

    pub fn exit(&mut self) {
        self.should_exit = true;
    }

    pub fn play_sound_effect_ui_dialog_open(&self) {
        self.play_sound_effect(audio::UI_DIALOG_OPEN_EFFECT);
    }

    pub fn play_sound_effect_ui_select(&self) {
        self.play_sound_effect(audio::UI_SELECT_EFFECT);
    }

    pub fn play_sound_effect_ui_error(&self) {
        self.play_sound_effect(audio::UI_ERROR_EFFECT);
    }

    pub fn play_sound_effect(&self, sound_effect: &'static [u8]) {
        if let Some(audio_handler) = &self.audio_handler {
            let _ = audio_handler.play_sound_effect(sound_effect);
        }
    }

    pub fn play_level_sound_effect(&self, sound_effect: SoundEffect) {
        if let Some(audio_handler) = &self.audio_handler {
            let _ = audio_handler.play_sound_effect(match sound_effect {
                SoundEffect::BoxFall => audio::BOX_FALL_EFFECT,
                SoundEffect::DoorUnlocked => audio::DOOR_OPEN_EFFECT,
                SoundEffect::FloorBroken => audio::FLOOR_BROKEN_EFFECT,
            });
        }
    }

    pub fn current_background_music_id(&self) -> Option<BackgroundMusicId> {
        self.current_background_music_id
    }

    pub fn stop_background_music(&mut self) {
        self.current_background_music_id = None;

        self.stop_background_music_internal();
    }

    fn stop_background_music_internal(&mut self) {
        if let Some(audio_handler) = &self.audio_handler {
            audio_handler.stop_background_music();
        }
    }

    pub fn set_background_music_loop(&mut self, background_music: &BackgroundMusic) {
        if self.current_background_music_id.is_some_and(|id| background_music.id() == id) {
            return;
        }

        self.current_background_music_id = Some(background_music.id());

        if !self.settings.background_music {
            return;
        }

        if let Some(audio_handler) = &self.audio_handler {
            let _ = audio_handler.set_background_music_loop(
                background_music.intro_audio_data(),
                background_music.main_loop_audio_data(),
            );
        }
    }

    pub fn settings(&self) -> &GameSettings {
        &self.settings
    }

    pub fn set_and_save_color_scheme_index(&mut self, color_scheme_index: usize) -> Result<(), Box<dyn Error>> {
        self.settings.color_scheme_index = color_scheme_index;
        self.settings.save_to_file()?;

        Ok(())
    }

    pub fn set_and_save_tile_mode(&mut self, tile_mode: TileMode) -> Result<(), Box<dyn Error>> {
        self.settings.tile_mode = tile_mode;
        self.settings.save_to_file()?;

        Ok(())
    }

    pub fn set_and_save_background_music_enabled(&mut self, background_music: bool) -> Result<(), Box<dyn Error>> {
        self.settings.background_music = background_music;

        if background_music {
            if let Some(current_background_music_id) = self.current_background_music_id {
                //Force restart current background music
                self.stop_background_music();
                self.set_background_music_loop(audio::BACKGROUND_MUSIC_TRACKS.get_track_by_id(current_background_music_id));
            }
        }else {
            self.stop_background_music_internal();
        }

        self.settings.save_to_file()?;

        Ok(())
    }

    pub fn set_and_save_animation_speed(&mut self, animation_speed: AnimationSpeed) -> Result<(), Box<dyn Error>> {
        self.settings.animation_speed = animation_speed;

        self.settings.save_to_file()?;

        Ok(())
    }
}

pub struct Game<'a> {
    console: &'a Console<'a>,

    screens: HashMap<ScreenId, Box<dyn Screen>>,
    help_page: HelpPage,

    game_state: GameState,
}

impl <'a> Game<'a> {
    pub const VERSION: &'static str = env!("CARGO_PKG_VERSION");

    pub const CONSOLE_MIN_WIDTH: usize = 74;
    pub const CONSOLE_MIN_HEIGHT: usize = 23;

    pub const LEVEL_MAX_WIDTH: usize = Self::CONSOLE_MIN_WIDTH;
    pub const LEVEL_MAX_HEIGHT: usize = Self::CONSOLE_MIN_HEIGHT - 1;

    const PLAYER_BACKGROUND_DELAY: i32 = 12;

    const SAVE_GAME_FOLDER: &'static str = "SokoTerm";

    const MAP_DEMO: &'static str = include_str!("../resources/demo.lvl");

    pub fn get_or_create_save_game_folder() -> Result<OsString, Box<dyn Error>> {
        let mut directory = if cfg!(windows) {
            std::env::var_os("USERPROFILE").
                    ok_or(GameError::new("%USERPROFILE% is not set!"))?
        }else {
            std::env::var_os("HOME").
                    ok_or(GameError::new("$HOME not set!"))?
        };

        directory.push("/.jddev0/");
        directory.push(Self::SAVE_GAME_FOLDER);
        std::fs::create_dir_all(&directory)?;

        #[cfg(feature = "steam")]
        {
            let mut directory = directory.clone();
            directory.push("/SteamWorkshop");
            std::fs::create_dir_all(&directory)?;
        }

        directory.push("/");
        Ok(directory)
    }

    pub fn new(
        console: &'a Console,

        #[cfg(feature = "steam")]
        steam_client: Client,
    ) -> Result<Self, Box<dyn Error>> {
        let (width, height) = console.get_console_size();
        if width < Self::CONSOLE_MIN_WIDTH || height < Self::CONSOLE_MIN_HEIGHT {
            return Err(Box::new(GameError::new(format!(
                "Console is to small (Min: {} x {})!",
                Self::CONSOLE_MIN_WIDTH,
                Self::CONSOLE_MIN_HEIGHT
            ))));
        }

        let screens = HashMap::from_iter([
            (ScreenId::StartMenu, Box::new(ScreenStartMenu::new()) as Box<dyn Screen>),
            (ScreenId::About, Box::new(ScreenAbout::new()) as Box<dyn Screen>),
            (ScreenId::Settings, Box::new(ScreenSettings::new()) as Box<dyn Screen>),

            (ScreenId::SelectLevel, Box::new(ScreenSelectLevel::new()) as Box<dyn Screen>),

            (ScreenId::InGame, Box::new(ScreenInGame::new()) as Box<dyn Screen>),
        ]);

        let level_pack = Box::new(LevelPack::read_from_save_game(
            "demo", "built-in:demo", Self::MAP_DEMO, false,
        )?);

        for (i, level) in level_pack.levels().iter().
                map(|level| level.level()).
                enumerate() {
            if level.width() > Self::LEVEL_MAX_WIDTH || level.height() > Self::LEVEL_MAX_HEIGHT {
                return Err(Box::new(GameError::new(format!(
                    "Error while loading level pack \"{}\": Level {} is too large (Max: {}x{})",
                    level_pack.id(),
                    i + 1,
                    Self::LEVEL_MAX_WIDTH,
                    Self::LEVEL_MAX_HEIGHT,
                ))));
            }
        }

        let settings = GameSettings::read_from_file()?;

        let mut game_state = GameState::new(
            level_pack,

            settings,

            #[cfg(feature = "steam")]
            steam_client,
        );

        game_state.set_background_music_loop(&audio::BACKGROUND_MUSIC_FIELDS_OF_ICE);

        Ok(Self {
            console,

            screens,
            help_page: HelpPage::new(Self::CONSOLE_MIN_WIDTH, Self::CONSOLE_MIN_HEIGHT),

            game_state,
        })
    }

    #[must_use]
    pub fn update(&mut self) -> bool {
        if self.game_state.should_exit {
            return true;
        }

        if self.console.has_input() && let Some(key) = self.console.get_key() {
            self.update_key(key);
        }

        self.update_mouse();

        if !self.game_state.is_help {
            let screen = self.screens.get_mut(&self.game_state.current_screen_id);
            if let Some(mut screen) = screen {
                //"while" instead of "if": This supports setting the screen in "on_set_screen"
                //Otherwise "on_set_screen" would not be called for the new screen
                while mem::replace(&mut self.game_state.should_call_on_set_screen, false) {
                    screen.on_set_screen(&mut self.game_state);

                    if self.game_state.should_call_on_set_screen {
                        //Change local current screen if screen was set in "on_set_screen"
                        screen = self.screens.get_mut(&self.game_state.current_screen_id).unwrap();
                    }
                }

                screen.update(&mut self.game_state);

                //Animations
                self.game_state.pending_animation_play_count += self.game_state.settings.animation_speed.animation_count_per_update();
                while self.game_state.pending_animation_play_count > 0.0 {
                    self.game_state.pending_animation_play_count -= 1.0;

                    screen.animate(&mut self.game_state);
                }
            }else {
                self.game_state.pending_animation_play_count = 0.0;
            }
        }

        //Player background
        self.game_state.player_background_tmp += 1;
        if self.game_state.player_background_tmp >= Self::PLAYER_BACKGROUND_DELAY + self.game_state.is_player_background as i32 {
            //If isPlayerBackground: wait an additional update (25 updates per second, every half
            //second: switch background/foreground colors [12 updates, 13 updates])
            self.game_state.player_background_tmp = 0;
            self.game_state.is_player_background = !self.game_state.is_player_background;
        }

        false
    }

    fn update_key(&mut self, key: Key) {
        if key == Key::F7 {
            self.game_state.play_sound_effect_ui_select();

            if let Err(err) = self.game_state.set_and_save_animation_speed(self.game_state.settings.animation_speed.next_setting()) {
                self.game_state.open_dialog(Dialog::new_ok_error(format!("Cannot save settings: {}", err)));
            }

            return;
        }else if key == Key::F8 {
            self.game_state.play_sound_effect_ui_select();

            if let Err(err) = self.game_state.set_and_save_background_music_enabled(!self.game_state.settings.background_music) {
                self.game_state.open_dialog(Dialog::new_ok_error(format!("Cannot save settings: {}", err)));
            }

            return;
        }

        let screen = self.screens.get_mut(&self.game_state.current_screen_id);
        if self.game_state.is_help {
            if key == Key::F1 || key == Key::ESC {
                self.game_state.close_help_page();

                if let Some(screen) = screen {
                    screen.on_continue(&mut self.game_state);
                }
            }else {
                self.help_page.on_key_pressed(&mut self.game_state, key);
            }

            return;
        }

        if let Some(dialog) = self.game_state.dialog.as_ref() {
            if let Some(dialog_selection) = dialog.on_key_pressed(key) {
                self.game_state.close_dialog();

                let screen = self.screens.get_mut(&self.game_state.current_screen_id);
                if let Some(screen) = screen {
                    screen.on_dialog_selection(&mut self.game_state, dialog_selection);

                    self.game_state.play_sound_effect_ui_select();
                }
            }

            return;
        }

        if key == Key::F1 {
            self.game_state.open_help_page();

            if let Some(screen) = screen {
                screen.on_pause(&mut self.game_state);
            }
        }else if let Some(screen) = screen {
            screen.on_key_pressed(&mut self.game_state, key);
        }
    }

    fn update_mouse(&mut self) {
        let Some((column, row)) = self.console.get_mouse_pos_clicked() else {
            return;
        };

        if self.game_state.is_help {
            self.help_page.on_mouse_pressed(Self::CONSOLE_MIN_WIDTH, Self::CONSOLE_MIN_HEIGHT, &mut self.game_state, column, row);

            return;
        }

        if let Some(dialog) = self.game_state.dialog.as_ref() {
            if let Some(dialog_selection) = dialog.on_mouse_pressed(column, row) {
                self.game_state.close_dialog();

                let screen = self.screens.get_mut(&self.game_state.current_screen_id);
                if let Some(screen) = screen {
                    screen.on_dialog_selection(&mut self.game_state, dialog_selection);

                    self.game_state.play_sound_effect_ui_select();
                }
            }

            return;
        }

        let screen = self.screens.get_mut(&self.game_state.current_screen_id);
        if let Some(screen) = screen {
            screen.on_mouse_pressed(&mut self.game_state, column, row);
        }
    }

    pub fn draw(&self) {
        self.console.repaint();

        if self.game_state.is_help {
            self.help_page.draw(self.console);

            return;
        }

        let screen = self.screens.get(&self.game_state.current_screen_id);
        if let Some(screen) = screen {
            screen.draw(&self.game_state, self.console);
        }

        if let Some(dialog) = self.game_state.dialog.as_ref() {
            dialog.draw(self.console);
        }
    }

    pub fn game_state(&self) -> &GameState {
        &self.game_state
    }

    pub fn game_state_mut(&mut self) -> &mut GameState {
        &mut self.game_state
    }
}

#[derive(Debug)]
pub struct GameError {
    message: String
}

impl GameError {
    pub fn new(message: impl Into<String>) -> Self {
        Self { message: message.into() }
    }
}

impl Display for GameError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message)
    }
}

impl Error for GameError {}
