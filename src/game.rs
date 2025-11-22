use std::collections::HashMap;
use std::error::Error;
use std::ffi::OsString;
use std::fmt::{Debug, Display, Formatter};
use std::mem;
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;
use std::str::FromStr;
use crate::game::audio::{AudioHandler, BackgroundMusic, BackgroundMusicId};
use crate::game::help_page::HelpPage;
use crate::game::level::{Level, LevelPack};
use crate::game::screen::*;
use crate::game::screen::dialog::{Dialog, DialogOk, DialogType};
use crate::io::{Console, Key};

#[cfg(feature = "gui")]
use bevy::prelude::*;
#[cfg(feature = "steam")]
use bevy_steamworks::*;

pub mod level;
mod screen;
mod help_page;
pub mod audio;
pub mod console_extension;

#[cfg(feature = "steam")]
pub mod steam;

pub struct EditorState {
    level_packs: Vec<LevelPack>,
    selected_level_pack_index: usize,
    selected_level_index: usize,
}

impl EditorState {
    pub fn new(level_packs: Vec<LevelPack>) -> Self {
        Self {
            level_packs,
            selected_level_pack_index: Default::default(),
            selected_level_index: Default::default(),
        }
    }

    pub fn get_level_pack_count(&self) -> usize {
        self.level_packs.len()
    }

    pub fn get_level_pack_index(&self) -> usize {
        self.selected_level_pack_index
    }

    pub fn get_current_level_pack(&self) -> Option<&LevelPack> {
        self.level_packs.get(self.selected_level_pack_index)
    }

    pub fn get_current_level_pack_mut(&mut self) -> Option<&mut LevelPack> {
        self.level_packs.get_mut(self.selected_level_pack_index)
    }

    pub fn set_level_pack_index(&mut self, level_pack_index: usize) {
        self.selected_level_pack_index = level_pack_index;
    }

    pub fn get_level_index(&self) -> usize {
        self.selected_level_index
    }

    pub fn set_level_index(&mut self, level_index: usize) {
        self.selected_level_index = level_index;
    }

    pub fn get_current_level(&self) -> Option<&Level> {
        self.level_packs.get(self.selected_level_pack_index).
                and_then(|level_pack| level_pack.levels().get(self.selected_level_index)).
                map(|level_with_stats| level_with_stats.level())
    }

    pub fn get_current_level_mut(&mut self) -> Option<&mut Level> {
        self.level_packs.get_mut(self.selected_level_pack_index).
                and_then(|level_pack| level_pack.levels_mut().get_mut(self.selected_level_index)).
                map(|level_with_stats| level_with_stats.level_mut())
    }
}

pub struct GameSettings {
    color_scheme_index: usize,

    background_music: bool,
}

impl GameSettings {
    pub fn new() -> GameSettings {
        Self {
            color_scheme_index: 0,

            background_music: true,
        }
    }

    pub fn read_from_file() -> Result<Self, Box<dyn Error>> {
        let mut settings_save_file = Game::get_or_create_save_game_folder()?;
        settings_save_file.push("settings.sav");

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
                                    warn!("\"settings.sav\" contains invalid value for option \"{key}\": \"{value}\": Using default");
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

                        "background_music" => {
                            let Ok(value) = bool::from_str(value) else {
                                #[cfg(feature = "gui")]
                                {
                                    warn!("\"settings.sav\" contains invalid value for option \"{key}\": \"{value}\": Using default");
                                }

                                //TODO warning in cli version

                                continue;
                            };

                            settings.background_music = value;
                        },

                        _ => {
                            #[cfg(feature = "gui")]
                            {
                                warn!("\"settings.sav\" contains invalid settings option: \"{key}\" with value \"{value}\": Ignoring");
                            }

                            //TODO warning in cli version
                        }
                    }
                }else {
                    #[cfg(feature = "gui")]
                    {
                        warn!("\"settings.sav\" contains invalid data: \"{line}\": Ignoring");
                    }

                    //TODO warning in cli version
                }
            }
        }

        Ok(settings)
    }

    pub fn save_to_file(&self) -> Result<(), Box<dyn Error>> {
        let mut settings_save_file = Game::get_or_create_save_game_folder()?;
        settings_save_file.push("settings.sav");
        let mut file = File::create(settings_save_file)?;

        writeln!(file, "color_scheme_index = {}", self.color_scheme_index)?;
        writeln!(file, "background_music = {}", self.background_music)?;

        Ok(())
    }

    pub fn color_scheme_index(&self) -> usize {
        self.color_scheme_index
    }

    pub fn background_music(&self) -> bool {
        self.background_music
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
    dialog: Option<Box<dyn Dialog>>,

    current_level_pack_index: usize,
    level_packs: Vec<LevelPack>,

    current_level_index: usize,

    is_player_background: bool,
    player_background_tmp: i32,

    found_secret_main_level_pack: bool,

    should_exit: bool,

    editor_state: EditorState,
    settings: GameSettings,

    audio_handler: Option<AudioHandler>,
    current_background_music_id: Option<BackgroundMusicId>,

    #[cfg(feature = "steam")]
    steam_client: Client,
    #[cfg(feature = "steam")]
    pub show_workshop_upload_popup: bool,
}

impl GameState {
    fn new(
        level_packs: Vec<LevelPack>, editor_level_packs: Vec<LevelPack>,

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
            level_packs,

            current_level_index: Default::default(),

            is_player_background: Default::default(),
            player_background_tmp: Default::default(),

            found_secret_main_level_pack: Default::default(),

            should_exit: Default::default(),

            settings,
            editor_state: EditorState::new(editor_level_packs),

            audio_handler: AudioHandler::new().ok(),
            current_background_music_id: None,

            #[cfg(feature = "steam")]
            steam_client,
            #[cfg(feature = "steam")]
            show_workshop_upload_popup: false,
        }
    }

    pub fn set_screen(&mut self, screen_id: ScreenId) {
        self.current_screen_id = screen_id;
        self.should_call_on_set_screen = true;
    }

    pub fn level_packs(&self) -> &[LevelPack] {
        &self.level_packs
    }

    pub fn get_level_pack_count(&self) -> usize {
        self.level_packs.len()
    }

    pub fn get_level_pack_index(&self) -> usize {
        self.current_level_pack_index
    }

    pub fn set_level_pack_index(&mut self, level_pack_index: usize) {
        self.current_level_pack_index = level_pack_index;
    }

    pub fn get_current_level_pack(&self) -> Option<&LevelPack> {
        self.level_packs.get(self.current_level_pack_index)
    }

    pub fn get_current_level_pack_mut(&mut self) -> Option<&mut LevelPack> {
        self.level_packs.get_mut(self.current_level_pack_index)
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

    pub fn open_dialog(&mut self, dialog: Box<dyn Dialog>) {
        let dialog_type = dialog.dialog_type();

        self.dialog = Some(dialog);

        match dialog_type {
            DialogType::Information => {
                self.play_sound_effect_ui_dialog_open();
            },
            DialogType::Error => {
                self.play_sound_effect_ui_error();
            },
            DialogType::SecretFound => {
                self.play_sound_effect(audio::SECRET_FOUND_EFFECT);
            },
        }
    }

    pub fn close_dialog(&mut self) {
        self.dialog = None;
    }

    pub fn exit(&mut self) {
        self.should_exit = true;
    }

    fn on_found_secret_for_level_pack(&mut self, level_pack_index: usize) -> Result<(), Box<dyn Error>> {
        if level_pack_index == 1 && !self.found_secret_main_level_pack {
            self.found_secret_main_level_pack = true;

            let secret_level_pack = LevelPack::read_from_save_game(
                "secret", "built-in:secret", Game::MAP_SECRET, false,

                #[cfg(feature = "steam")]
                None,
            )?;

            //Save immediately in order to keep secret level pack after game restart if not yet played
            secret_level_pack.save_save_game(false)?;

            self.level_packs.insert(4, secret_level_pack);
        }

        Ok(())
    }

    pub fn on_found_secret(&mut self) -> Result<(), Box<dyn Error>> {
        self.on_found_secret_for_level_pack(self.current_level_pack_index)
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

    pub fn editor_state(&self) -> &EditorState {
        &self.editor_state
    }

    pub fn set_and_save_color_scheme_index(&mut self, color_scheme_index: usize) -> Result<(), Box<dyn Error>> {
        self.settings.color_scheme_index = color_scheme_index;
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

    const MAP_TUTORIAL: &'static str = include_str!("../resources/tutorial.lvl");
    const MAP_MAIN: &'static str = include_str!("../resources/main.lvl");
    const MAP_SPECIAL: &'static str = include_str!("../resources/special.lvl");
    const MAP_DEMON: &'static str = include_str!("../resources/demon.lvl");

    const MAP_SECRET: &'static str = include_str!("../resources/secret.lvl");

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

            (ScreenId::SelectLevelPack, Box::new(ScreenSelectLevelPack::new()) as Box<dyn Screen>),
            (ScreenId::SelectLevel, Box::new(ScreenSelectLevel::new()) as Box<dyn Screen>),

            (ScreenId::InGame, Box::new(ScreenInGame::new()) as Box<dyn Screen>),

            (ScreenId::SelectLevelPackEditor, Box::new(ScreenSelectLevelPackEditor::new()) as Box<dyn Screen>),
            (ScreenId::SelectLevelPackBackgroundMusic, Box::new(ScreenSelectLevelPackBackgroundMusic::new()) as Box<dyn Screen>),
            (ScreenId::LevelPackEditor, Box::new(ScreenLevelPackEditor::new()) as Box<dyn Screen>),
            (ScreenId::LevelEditor, Box::new(ScreenLevelEditor::new()) as Box<dyn Screen>),
        ]);

        let mut level_packs = Vec::with_capacity(LevelPack::MAX_LEVEL_PACK_COUNT);
        level_packs.append(&mut vec![
            LevelPack::read_from_save_game(
                "tutorial", "built-in:tutorial", Self::MAP_TUTORIAL, false,

                #[cfg(feature = "steam")]
                None,
            )?,
            LevelPack::read_from_save_game(
                "main", "built-in:main", Self::MAP_MAIN, false,

                #[cfg(feature = "steam")]
                None,
            )?,
            LevelPack::read_from_save_game(
                "special", "built-in:special", Self::MAP_SPECIAL, false,

                #[cfg(feature = "steam")]
                None,
            )?,
            LevelPack::read_from_save_game(
                "demon", "built-in:demon", Self::MAP_DEMON, false,

                #[cfg(feature = "steam")]
                None,
            )?,
        ]);

        for arg in std::env::args().
                skip(1) {
            if !arg.ends_with(".lvl") {
                return Err(Box::new(GameError::new(format!(
                    "Invalid level pack \"{}\": The file extension of level pack must be \".lvl\"",
                    arg
                ))));
            }

            let level_pack_path = Path::new(&arg);

            let level_pack_file_name = if let Some(file_name) = level_pack_path.file_name() {
                if let Some(file_name) = file_name.to_str() {
                    file_name
                }else {
                    return Err(Box::new(GameError::new(format!(
                        "Error while loading level pack \"{}\": Invalid file name",
                        arg
                    ))));
                }
            }else {
                return Err(Box::new(GameError::new(format!(
                    "Error while loading level pack \"{}\": File name is missing",
                    arg
                ))));
            };

            let mut level_pack_file = match File::open(level_pack_path) {
                Ok(file) => file,
                Err(err) => return Err(Box::new(GameError::new(format!(
                    "Error while loading level pack \"{}\": {}",
                    arg, err
                )))),
            };

            let mut level_pack_data = String::new();
            if let Err(err) = level_pack_file.read_to_string(&mut level_pack_data) {
                return Err(Box::new(GameError::new(format!(
                    "Error while loading level pack \"{}\": {}",
                    arg, err
                ))));
            };

            let level_pack_id = &level_pack_file_name[..level_pack_file_name.len() - 4];
            if level_pack_id.len() > LevelPack::MAX_LEVEL_PACK_NAME_LEN {
                return Err(Box::new(GameError::new(format!(
                    "Error while loading level pack \"{}\": Level pack ID is too long (Max: {})",
                    arg, LevelPack::MAX_LEVEL_PACK_NAME_LEN
                ))));
            }

            if level_pack_id == "secret" {
                return Err(Box::new(GameError::new(format!("Level pack \"{}\" already exists!", level_pack_id))));
            }

            for id in level_packs.iter().
                    map(|level_pack| level_pack.id()) {
                if id == level_pack_id {
                    return Err(Box::new(GameError::new(format!("Level pack \"{}\" already exists!", level_pack_id))));
                }
            }

            level_packs.push(LevelPack::read_from_save_game(
                level_pack_id, &arg, level_pack_data, false,

                #[cfg(feature = "steam")]
                None,
            )?);
        }

        if level_packs.len() > LevelPack::MAX_LEVEL_PACK_COUNT {
            return Err(Box::new(GameError::new(format!(
                "Too many level packs ({}, max: {})",
                level_packs.len(),
                LevelPack::MAX_LEVEL_PACK_COUNT,
            ))));
        }

        for level_pack in level_packs.iter() {
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
        }

        let mut editor_level_packs = Vec::with_capacity(LevelPack::MAX_LEVEL_PACK_COUNT);

        let save_game_folder = Game::get_or_create_save_game_folder()?;
        for entry in std::fs::read_dir(save_game_folder)?.
                filter(|entry| entry.as_ref().
                        is_ok_and(|entry| entry.path().is_file())).
                map(|entry| entry.unwrap()) {
            if entry.file_name().to_str().is_some_and(|file_name| file_name.ends_with(".lvl.edit")) {
                let file_name = entry.file_name();
                let file_name = file_name.to_str().unwrap();
                let level_pack_id = &file_name[..file_name.len() - 9];

                let mut level_pack_file = match File::open(entry.path()) {
                    Ok(file) => file,
                    Err(err) => return Err(Box::new(GameError::new(format!(
                        "Error while loading editor level pack \"{}\": {}",
                        file_name, err
                    )))),
                };

                let mut level_pack_data = String::new();
                if let Err(err) = level_pack_file.read_to_string(&mut level_pack_data) {
                    return Err(Box::new(GameError::new(format!(
                        "Error while loading editor level pack \"{}\": {}",
                        file_name, err
                    ))));
                };

                editor_level_packs.push(LevelPack::read_from_save_game(
                    level_pack_id, entry.path().to_str().unwrap(), level_pack_data, true,

                    #[cfg(feature = "steam")]
                    None,
                )?);
            }
        }

        if editor_level_packs.len() > LevelPack::MAX_LEVEL_PACK_COUNT {
            return Err(Box::new(GameError::new(format!(
                "Too many level packs ({}, max: {})",
                editor_level_packs.len(),
                LevelPack::MAX_LEVEL_PACK_COUNT,
            ))));
        }

        for level_pack in editor_level_packs.iter() {
            //Level pack for editor might be empty and might contain no player tile

            if level_pack.level_count() > LevelPack::MAX_LEVEL_COUNT_PER_PACK {
                return Err(Box::new(GameError::new(format!(
                    "Error while loading editor level pack \"{}\": Level pack contains too many levels ({}, max: {})",
                    level_pack.id(),
                    level_pack.level_count(),
                    LevelPack::MAX_LEVEL_COUNT_PER_PACK,
                ))));
            }

            for (i, level) in level_pack.levels().iter().
                    map(|level| level.level()).
                    enumerate() {
                if level.width() > Self::LEVEL_MAX_WIDTH || level.height() > Self::LEVEL_MAX_HEIGHT {
                    return Err(Box::new(GameError::new(format!(
                        "Error while loading editor level pack \"{}\": Level {} is too large (Max: {}x{})",
                        level_pack.id(),
                        i + 1,
                        Self::LEVEL_MAX_WIDTH,
                        Self::LEVEL_MAX_HEIGHT,
                    ))));
                }
            }
        }
        
        editor_level_packs.sort_by_key(|level_pack| level_pack.id().to_string());

        let settings = GameSettings::read_from_file()?;

        let mut game_state = GameState::new(
            level_packs, editor_level_packs,

            settings,

            #[cfg(feature = "steam")]
            steam_client,
        );

        let mut save_game_file = Game::get_or_create_save_game_folder()?;
        save_game_file.push("secret.lvl.sav");
        if std::fs::exists(&save_game_file).is_ok_and(|exists| exists) {
            game_state.on_found_secret_for_level_pack(1)?;
        }

        game_state.set_background_music_loop(&audio::BACKGROUND_MUSIC_FIELDS_OF_ICE);

        Ok(Self {
            console,

            screens,
            help_page: HelpPage::new(Self::CONSOLE_MIN_WIDTH, Self::CONSOLE_MIN_HEIGHT),

            game_state,
        })
    }

    #[cfg(feature = "steam")]
    pub fn load_steam_workshop_level_pack(
        &mut self,

        item: QueryResult,
    ) -> Result<(), Box<dyn Error>> {
        info!("Loading steam workshop level pack (ID: {}, Name: \"{}\")", item.published_file_id.0, item.title);

        let install_info = self.game_state.steam_client.ugc().item_install_info(item.published_file_id);
        let Some(install_info) = install_info else {
            return Err(Box::new(GameError::new(format!(
                "Steam workshop level pack (ID: {}, Name: \"{}\") does not exist or is invalid!",
                item.published_file_id.0, item.title,
            ))));
        };

        let mut level_pack_path = OsString::from(install_info.folder);
        level_pack_path.push("/pack.lvl");

        let level_pack_path = Path::new(&level_pack_path);
        if !std::fs::exists(level_pack_path)? || !level_pack_path.is_file() {
            return Err(Box::new(GameError::new(format!(
                "Steam workshop level pack (ID: {}, Name: \"{}\") is invalid!",
                item.published_file_id.0, item.title,
            ))));
        }

        let mut level_pack_file = match File::open(level_pack_path) {
            Ok(file) => file,
            Err(err) => return Err(Box::new(GameError::new(format!(
                "Error while loading level pack from steam workshop (ID: {}, Name: \"{}\"): {}",
                item.published_file_id.0, item.title,
                err,
            )))),
        };

        let mut level_pack_data = String::new();
        if let Err(err) = level_pack_file.read_to_string(&mut level_pack_data) {
            return Err(Box::new(GameError::new(format!(
                "Error while loading level pack from steam workshop (ID: {}, Name: \"{}\"): {}",
                item.published_file_id.0, item.title,
                err,
            ))));
        };

        let level_pack_id = format!("workshop:{}", item.published_file_id.0);

        let ascii_level_title = item.title.replace(|c: char| !c.is_ascii(), "?");

        let mut truncated_workshop_item_name = ascii_level_title;
        if truncated_workshop_item_name.len() > LevelPack::MAX_LEVEL_PACK_NAME_LEN {
            truncated_workshop_item_name = truncated_workshop_item_name[..LevelPack::MAX_LEVEL_PACK_NAME_LEN - 3].to_string() + "...";
        }

        for id in self.game_state.level_packs.iter().
                map(|level_pack| level_pack.id()) {
            if id == level_pack_id {
                return Err(Box::new(GameError::new(format!("Level pack \"{}\" already exists!", level_pack_id))));
            }
        }

        let mut level_pack = LevelPack::read_from_save_game(
            level_pack_id, level_pack_path.to_str().unwrap(), level_pack_data, false,

            Some(item.published_file_id),
        )?;
        level_pack.set_name(truncated_workshop_item_name);

        for (i, level) in level_pack.levels().iter().
                map(|level| level.level()).
                enumerate() {
            if level.width() > Self::LEVEL_MAX_WIDTH || level.height() > Self::LEVEL_MAX_HEIGHT {
                return Err(Box::new(GameError::new(format!(
                    "Error while loading level pack from steam workshop (ID: {}, Name: \"{}\"): Level {} is too large (Max: {}x{})",
                    item.published_file_id.0, item.title,
                    i + 1,
                    Self::LEVEL_MAX_WIDTH,
                    Self::LEVEL_MAX_HEIGHT,
                ))));
            }
        }

        self.game_state.level_packs.push(level_pack);

        Ok(())
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
            if let Some(screen) = screen {
                if mem::replace(&mut self.game_state.should_call_on_set_screen, false) {
                    screen.on_set_screen(&mut self.game_state);
                }

                screen.update(&mut self.game_state);
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
        if key == Key::F9 {
            self.game_state.play_sound_effect_ui_select();

            if let Err(err) = self.game_state.set_and_save_background_music_enabled(!self.game_state.settings.background_music) {
                self.game_state.open_dialog(Box::new(DialogOk::new_error(format!("Cannot save settings: {}", err))));
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
            if let Some(dialog_selection) = dialog.on_key_pressed(Self::CONSOLE_MIN_WIDTH, Self::CONSOLE_MIN_HEIGHT, key) {
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
            if let Some(dialog_selection) = dialog.on_mouse_pressed(Self::CONSOLE_MIN_WIDTH, Self::CONSOLE_MIN_HEIGHT, column, row) {
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
            dialog.draw(self.console, Self::CONSOLE_MIN_WIDTH, Self::CONSOLE_MIN_HEIGHT);
        }
    }

    #[cfg(feature = "steam")]
    #[must_use]
    pub fn draw_level_pack_thumbnail_screenshot(&self) -> Option<(usize, usize)> {
        self.console.repaint();

        if let Some(level_pack) = self.game_state.editor_state.get_current_level_pack() {
            let level_index = level_pack.thumbnail_level_index().unwrap_or(0);
            if let Some(level) = level_pack.levels().get(level_index) {
                let level = level.level();

                //Always draw to top left: Screenshot will be trimmed to the level size
                level.draw(self.console, 0, 0, false, None);

                return Some((level.width(), level.height()))
            }
        }

        None
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
