use std::cmp;
use std::error::Error;
use std::mem::ManuallyDrop;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::sync::{Arc, LazyLock, Mutex};
use bevy::prelude::*;
use bevy::input::ButtonState;
use bevy::input::keyboard::KeyboardInput;
use bevy::input::mouse::MouseButtonInput;
use bevy::window::{PrimaryWindow, WindowMode, WindowResized};
use bevy::asset::io::embedded::EmbeddedAssetRegistry;
use bevy::log::LogPlugin;
use crate::game::Game;
use crate::game::screen::dialog::Dialog;
use crate::io::bevy_abstraction::{ConsoleState, GraphicalCharacter, Key, COLOR_SCHEMES};
use crate::io::Console;

#[cfg(feature = "steam")]
use bevy_steamworks::*;
#[cfg(feature = "steam")]
use crate::ui::gui::steam_plugin::SteamPlugin;

mod assets;
mod startup_error;

#[cfg(feature = "steam")]
mod steam_plugin;

#[macro_export]
macro_rules! insert_embedded_asset {
    ( $embedded:ident, $asset:path$(,)? ) => {
        $embedded.insert_asset(
            PathBuf::from("../../assets/".to_string() + $asset.path()),
            Path::new($asset.path()),
            $asset.data(),
        );
    };
}

#[expect(clippy::type_complexity)]
static CONSOLE_STATE: LazyLock<Arc<Mutex<ConsoleState>>, fn() -> Arc<Mutex<ConsoleState>>> =
    LazyLock::new(|| Arc::new(Mutex::new(ConsoleState::new::<74, 23>())));

const BORDER_WIDTH: u32 = 5;

#[derive(Debug, Component)]
struct ConsoleTextCharacter {
    x: usize,
    y: usize,
}

#[derive(Debug, Component)]
struct ConsoleTileCharacter {
    x: usize,
    y: usize,
}

#[derive(Debug, Default, Resource)]
struct CharacterScaling {
    font_size: f32,

    char_width: f32,
    char_height: f32,

    x_offset: f32,
    y_offset: f32,
}

#[derive(Debug, Default, Clone, Copy, Resource)]
struct CurrentColorSchemeIndex(usize);

pub fn run_game() -> ExitCode {
    let mut app = App::new();

    app.add_plugins(LogPlugin::default());

    #[cfg(feature = "steam")]
    let steam_client = {
        if let Err(err) = steam_plugin::init(&mut app) {
            startup_error::show_startup_error_dialog(&mut app, &err.to_string());

            return ExitCode::FAILURE;
        }

        app.world().get_resource::<Client>().unwrap().clone()
    };

    let console = Box::leak(Box::new(Console::new(CONSOLE_STATE.clone())));
    let game = Game::new(
        console,

        #[cfg(feature = "steam")]
        steam_client.clone(),
    );
    let game = match game {
        Ok(game) => game,
        Err(err) => {
            startup_error::show_startup_error_dialog(&mut app, &format!("Could not initialize game: {err}"));

            return ExitCode::FAILURE;
        },
    };

    let settings = game.game_state().settings();
    CONSOLE_STATE.lock().unwrap().set_tile_mode(settings.tile_mode());

    #[cfg(feature = "steam")]
    {
        app.add_plugins(SteamPlugin);
    }

    app.
            add_plugins(DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    title: format!("SokoTerm (v{})", Game::VERSION),
                    resize_constraints: WindowResizeConstraints {
                        min_width: 480.0,
                        min_height: 360.0,
                        max_width: f32::INFINITY,
                        max_height: f32::INFINITY,
                    },
                    mode: WindowMode::BorderlessFullscreen(MonitorSelection::Primary),
                    ..default()
                }),
                ..default()
            }).disable::<LogPlugin>()).

            insert_resource(Time::<Fixed>::from_seconds(0.040)). //Run FixedUpdate every 40ms
            insert_resource(ClearColor(crate::io::bevy_abstraction::Color::Default.into_bevy_color(&COLOR_SCHEMES[settings.color_scheme_index()]))).
            insert_resource(CharacterScaling::default()).
            insert_resource(CurrentColorSchemeIndex(settings.color_scheme_index())).

            add_systems(Startup, spawn_camera).
            add_systems(Startup, preload_tiles).
            add_systems(Startup, update_text_entities).
            insert_non_send_resource(game).

            add_systems(FixedUpdate, update_game).

            add_systems(Update, draw_console_text).
            add_systems(Update, cycle_through_color_schemes.
                    pipe(handle_recoverable_error).
                    before(draw_console_text)).
            add_systems(Update, toggle_tile_mode.
                    pipe(handle_recoverable_error).
                    before(draw_console_text)).
            add_systems(Update, (on_resize, toggle_fullscreen));

    let embedded = app.world_mut().resource_mut::<EmbeddedAssetRegistry>();

    //Textures
    insert_embedded_asset!(embedded, assets::textures::tiles::EMPTY);
    insert_embedded_asset!(embedded, assets::textures::tiles::FRAGILE_FLOOR);
    insert_embedded_asset!(embedded, assets::textures::tiles::ICE);

    insert_embedded_asset!(embedded, assets::textures::tiles::ONE_WAY_LEFT);
    insert_embedded_asset!(embedded, assets::textures::tiles::ONE_WAY_UP);
    insert_embedded_asset!(embedded, assets::textures::tiles::ONE_WAY_RIGHT);
    insert_embedded_asset!(embedded, assets::textures::tiles::ONE_WAY_DOWN);

    insert_embedded_asset!(embedded, assets::textures::tiles::WALL);

    insert_embedded_asset!(embedded, assets::textures::tiles::KEY);
    insert_embedded_asset!(embedded, assets::textures::tiles::KEY_IN_GOAL);
    insert_embedded_asset!(embedded, assets::textures::tiles::KEY_ON_FRAGILE_FLOOR);
    insert_embedded_asset!(embedded, assets::textures::tiles::KEY_ON_ICE);
    insert_embedded_asset!(embedded, assets::textures::tiles::LOCKED_DOOR);

    insert_embedded_asset!(embedded, assets::textures::tiles::BOX);
    insert_embedded_asset!(embedded, assets::textures::tiles::BOX_IN_GOAL);
    insert_embedded_asset!(embedded, assets::textures::tiles::GOAL);

    insert_embedded_asset!(embedded, assets::textures::tiles::HOLE);
    insert_embedded_asset!(embedded, assets::textures::tiles::BOX_IN_HOLE);

    //Fonts
    insert_embedded_asset!(embedded, assets::font::JETBRAINS_MONO_BOLD_BYTES);

    #[cfg(feature = "steam")]
    {
        insert_embedded_asset!(embedded, assets::font::JETBRAINS_MONO_NL_EXTRA_LIGHT_BYTES);
    }

    let exit_code = app.run();
    match exit_code {
        AppExit::Success => ExitCode::SUCCESS,
        AppExit::Error(code) => ExitCode::from(code.get()),
    }
}

fn handle_recoverable_error(
    In(result): In<Result<(), Box<dyn Error>>>,

    mut game: NonSendMut<Game>,
) {
    let Err(err) = result else {
        return;
    };

    error!("An error occurred: {err}");
    game.game_state_mut().open_dialog(Dialog::new_ok_error(format!("An error occurred:\n{err}")));
}

fn spawn_camera(
    mut commands: Commands,
) {
    commands.spawn(Camera2d);
}

fn preload_tiles(
    asset_server: Res<AssetServer>,
) {
    for value in GraphicalCharacter::VALUES {
        //Preload tile images and prevent unloading
        let image = value.into_image(&asset_server);
        let _ = ManuallyDrop::new(image);
    }
}

#[expect(clippy::type_complexity)]
fn update_text_entities(
    mut commands: Commands,

    console_characters: Query<Entity, Or<(With<ConsoleTextCharacter>, With<ConsoleTileCharacter>)>>,
    window_query: Query<&Window, With<PrimaryWindow>>,

    asset_server: Res<AssetServer>,
    mut character_scaling: ResMut<CharacterScaling>,
    current_color_scheme_index: Res<CurrentColorSchemeIndex>,
) {
    for entity in console_characters.iter() {
        commands.entity(entity).despawn();
    }

    let window = window_query.single().unwrap();
    let window_width = window.width();
    let window_height = window.height();

    *character_scaling = calculate_character_scaling(window_width, window_height, 74, 23);

    let font = asset_server.load("embedded://font/JetBrainsMono-Bold.ttf");
    let text_font = TextFont {
        font: font.clone(),
        font_size: character_scaling.font_size,
        ..default()
    };

    let state = CONSOLE_STATE.lock().unwrap();
    let buffer = state.primary_buffer();
    let text_buffer = buffer.text_buffer();
    let text_color_buffer = buffer.text_color_buffer();

    let color_scheme = &COLOR_SCHEMES[current_color_scheme_index.0];

    let mut iter = text_buffer.iter().copied().zip(text_color_buffer.iter().copied());
    for y in 0..23 {
        for x in 0..74 {
            let (character, (fg, bg)) = iter.next().unwrap();

            let screen_x = character_scaling.x_offset + x as f32 * character_scaling.char_width - window_width * 0.5;
            let screen_y = window_height * 0.5 - (character_scaling.y_offset + y as f32 * character_scaling.char_height);

            let char = character.get();

            let inverted = bg == crate::io::bevy_abstraction::Color::Black;

            commands.spawn((
                Text2d::new(String::from_utf8_lossy(&[char.unwrap_or(b' ')])),
                text_font.clone(),
                Transform::from_translation(Vec3::new(screen_x, screen_y, 1.0)),
                TextColor(fg.into_bevy_color(color_scheme)),
                TextBackgroundColor(bg.into_bevy_color(color_scheme).with_alpha(if char.is_ok() || !inverted { 1.0 } else { 0.9 })),
                ConsoleTextCharacter { x, y },
                if char.is_ok() || inverted { Visibility::Visible } else { Visibility::Hidden },
            ));

            let mut sprite = Sprite {
                custom_size: Some(Vec2::new(character_scaling.char_width, character_scaling.char_height)),
                ..default()
            };

            if let Err(tile) = char {
                sprite.image = tile.into_image(&asset_server);
            }

            commands.spawn((
                sprite,
                Transform::from_translation(Vec3::new(screen_x, screen_y, 0.0)),
                ConsoleTileCharacter { x, y },
                if char.is_err() { Visibility::Visible } else { Visibility::Hidden },
            ));
        }
    }
}

fn update_game(
    window_query: Query<&Window, With<PrimaryWindow>>,

    mut game: NonSendMut<Game>,

    character_scaling: Res<CharacterScaling>,

    mut keyboard_event: MessageReader<KeyboardInput>,
    mut mouse_event: MessageReader<MouseButtonInput>,

    mut app_exit_event_writer: MessageWriter<AppExit>,
) {
    {
        let window = window_query.single().unwrap();

        let mut state = CONSOLE_STATE.lock().unwrap();
        let mut last_key_code = None;
        for event in keyboard_event.read() {
            if event.state == ButtonState::Released {
                continue;
            }

            //Limit repeated key to once per update
            if last_key_code == Some(event.key_code) && event.repeat {
                continue;
            }

            last_key_code = Some(event.key_code);

            if event.logical_key == bevy::input::keyboard::Key::F9 ||
                    event.logical_key == bevy::input::keyboard::Key::F10 ||
                    event.logical_key == bevy::input::keyboard::Key::F11 {
                continue;
            }

            let key = Key::from_bevy_key(&event.logical_key, event.text.as_ref());
            if let Some(key) = key {
                state.input_queue_keyboard_mut().push_back(key);
            }
        }

        for event in mouse_event.read() {
            if event.state == ButtonState::Released {
                continue;
            }

            if event.button == MouseButton::Left && let Some(pos) = window.cursor_position() {
                let x = pos.x - character_scaling.x_offset + character_scaling.char_width * 0.5;
                let y = pos.y - character_scaling.y_offset + character_scaling.char_height * 0.5;

                let column = x / character_scaling.char_width;
                let row = y / character_scaling.char_height;

                let column = column as i32;
                let row = row as i32;
                if column < 0 || row < 0 || column >= 74 || row >= 23 {
                    continue;
                }

                state.input_queue_mouse_mut().push_back((column as usize, row as usize));
            }
        }
    }

    let should_stop = game.update();
    game.draw();

    if should_stop {
        app_exit_event_writer.write(AppExit::Success);
    }
}

#[expect(clippy::type_complexity)]
fn on_resize(
    commands: Commands,

    console_characters: Query<Entity, Or<(With<ConsoleTextCharacter>, With<ConsoleTileCharacter>)>>,
    window_query: Query<&Window, With<PrimaryWindow>>,

    asset_server: Res<AssetServer>,
    character_scaling: ResMut<CharacterScaling>,
    current_color_scheme_index: Res<CurrentColorSchemeIndex>,

    mut resize_reader: MessageReader<WindowResized>,
) {
    let event = resize_reader.read().last();
    if event.is_some() {
        update_text_entities(
            commands,

            console_characters,
            window_query,

            asset_server,
            character_scaling,
            current_color_scheme_index,
        );
    }
}

fn toggle_fullscreen(
    mut window_query: Query<&mut Window, With<PrimaryWindow>>,

    keyboard_input: Res<ButtonInput<KeyCode>>,
) {
    let mut window = window_query.single_mut().unwrap();

    if keyboard_input.just_pressed(KeyCode::F11) {
        if window.mode == WindowMode::Windowed {
            window.mode = WindowMode::BorderlessFullscreen(MonitorSelection::Current);
        }else {
            window.mode = WindowMode::Windowed;
            window.position = WindowPosition::Centered(MonitorSelection::Current);
        }
    }
}

fn cycle_through_color_schemes(
    mut commands: Commands,

    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut current_color_scheme_index: ResMut<CurrentColorSchemeIndex>,

    mut game: NonSendMut<Game>,
) -> Result<(), Box<dyn Error>> {
    if keyboard_input.just_pressed(KeyCode::F10) {
        current_color_scheme_index.0 = (current_color_scheme_index.0 + 1) % COLOR_SCHEMES.len();
        game.game_state_mut().set_and_save_color_scheme_index(current_color_scheme_index.0)?;
        game.game_state().play_sound_effect_ui_select();

        commands.insert_resource(ClearColor(crate::io::bevy_abstraction::Color::Default.into_bevy_color(&COLOR_SCHEMES[current_color_scheme_index.0])));
    }

    Ok(())
}

fn toggle_tile_mode(
    keyboard_input: Res<ButtonInput<KeyCode>>,

    mut game: NonSendMut<Game>,
) -> Result<(), Box<dyn Error>> {
    if keyboard_input.just_pressed(KeyCode::F9) {
        let mut state = CONSOLE_STATE.lock()?;
        let tile_mode = state.tile_mode().toggle();
        state.set_tile_mode(tile_mode);

        game.game_state_mut().set_and_save_tile_mode(tile_mode)?;
        game.game_state().play_sound_effect_ui_select();
    }

    Ok(())
}

fn draw_console_text(
    mut console_text_characters: Query<(&mut Text2d, &mut TextColor, &mut TextBackgroundColor, &mut Visibility, &ConsoleTextCharacter), Without<ConsoleTileCharacter>>,
    mut console_tile_characters: Query<(&mut Sprite, &mut Visibility, &ConsoleTileCharacter), Without<ConsoleTextCharacter>>,

    current_color_scheme_index: Res<CurrentColorSchemeIndex>,
    asset_server: Res<AssetServer>,
) {
    //TODO optimize repaint logic

    let state = CONSOLE_STATE.lock().unwrap();

    let buffer = state.primary_buffer();
    let text_buffer = buffer.text_buffer();
    let text_color_buffer = buffer.text_color_buffer();

    let color_scheme = &COLOR_SCHEMES[current_color_scheme_index.0];

    for (
        mut text,
        mut fg_color,
        mut bg_color,
        mut visibility,
        ConsoleTextCharacter { x, y },
    ) in console_text_characters.iter_mut() {
        let character = text_buffer[x + y * 74];
        let (fg, bg) = text_color_buffer[x + y * 74];

        let char = character.get();
        match char {
            Ok(char) => {
                *visibility = Visibility::Visible;
                text.0 = String::from_utf8_lossy(&[char]).into();
                fg_color.0 = fg.into_bevy_color(color_scheme);
                bg_color.0 = bg.into_bevy_color(color_scheme);
            },
            Err(_) => {
                let inverted = bg == crate::io::bevy_abstraction::Color::Black;

                if inverted {
                    *visibility = Visibility::Visible;
                    text.0 = " ".to_string();
                    fg_color.0 = fg.into_bevy_color(color_scheme);
                    bg_color.0 = bg.into_bevy_color(color_scheme).with_alpha(0.9);
                }else {
                    *visibility = Visibility::Hidden;
                }
            },
        }
    }

    for (
        mut sprite,
        mut visibility,
        ConsoleTileCharacter { x, y },
    ) in console_tile_characters.iter_mut() {
        let character = text_buffer[x + y * 74];

        let char = character.get();
        match char {
            Ok(_) => {
                *visibility = Visibility::Hidden;
            },
            Err(tile) => {
                *visibility = Visibility::Visible;
                sprite.image = tile.into_image(&asset_server);
            },
        }
    }
}

fn calculate_character_scaling(
    window_width: f32,
    window_height: f32,

    columns: usize,
    rows: usize,
) -> CharacterScaling {
    let gameplay_width = window_width - 2.0 * BORDER_WIDTH as f32;
    let gameplay_height = window_height - 2.0 * BORDER_WIDTH as f32;

    let max_char_width = gameplay_width / columns as f32;
    let max_char_height = gameplay_height / rows as f32;

    let max_font_size_by_width = max_char_width / 60.0 * 100.0;
    let max_font_size_by_height = max_char_height / 120.0 * 100.0;

    let font_size = cmp::min((max_font_size_by_width * 100.0) as u32, (max_font_size_by_height * 100.0) as u32) as f32 * 0.01;

    let char_width = font_size * 60.0 / 100.0;
    let char_height = font_size * 120.0 / 100.0;

    let console_width = char_width * columns as f32;
    let console_height = char_height * rows as f32;

    let padding_x = (gameplay_width - console_width) * 0.5;
    let padding_y = (gameplay_height - console_height) * 0.5;

    let x_offset = BORDER_WIDTH as f32 + padding_x + char_width * 0.5;
    let y_offset = BORDER_WIDTH as f32 + padding_y + char_height * 0.5;

    CharacterScaling {
        font_size,

        char_width,
        char_height,

        x_offset,
        y_offset
    }
}
