use std::cmp;
use bevy::prelude::*;
use std::process::ExitCode;
use std::sync::{Arc, LazyLock, Mutex};
use bevy::asset::{embedded_asset, load_embedded_asset};
use bevy::input::ButtonState;
use bevy::input::keyboard::{KeyboardInput};
use bevy::input::mouse::MouseButtonInput;
use bevy::window::{PrimaryWindow, WindowMode, WindowResized};
use crate::game::Game;
use crate::io::bevy_abstraction::{ConsoleState, Key};
use crate::io::Console;

#[expect(clippy::type_complexity)]
static CONSOLE_STATE: LazyLock<Arc<Mutex<ConsoleState>>, fn() -> Arc<Mutex<ConsoleState>>> =
    LazyLock::new(|| Arc::new(Mutex::new(ConsoleState::new::<74, 23>())));

const BORDER_WIDTH: u32 = 5;

#[derive(Debug, Component)]
struct ConsoleTextCharacter {
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

pub fn run_game() -> ExitCode {
    let console = Box::leak(Box::new(Console::new(CONSOLE_STATE.clone())));
    let game = Game::new(console);
    let game = match game {
        Ok(game) => game,
        Err(err) => {
            eprintln!("{err}");

            return ExitCode::FAILURE;
        },
    };

    let mut app = App::new();

    app.
            add_plugins(DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    title: format!("Console Sokoban (v{})", Game::VERSION),
                    mode: WindowMode::BorderlessFullscreen(MonitorSelection::Primary),
                    ..default()
                }),
                ..default()
            })).
            insert_resource(Time::<Fixed>::from_seconds(0.040)). //Run FixedUpdate every 40ms
            insert_resource(ClearColor(Color::srgb_u8(23, 20, 33))).
            insert_resource(CharacterScaling::default()).

            add_systems(Startup, spawn_camera).
            add_systems(Startup, update_text_entities).
            insert_non_send_resource(game).

            add_systems(FixedUpdate, update_game).

            add_systems(Update, (on_resize, draw_game));

    embedded_asset!(app, "../../assets/font/JetBrainsMono-Bold.ttf");

    let exit_code = app.run();
    match exit_code {
        AppExit::Success => ExitCode::SUCCESS,
        AppExit::Error(code) => ExitCode::from(code.get()),
    }
}

fn spawn_camera(
    mut commands: Commands,
) {
    commands.spawn(Camera2d);
}

fn update_text_entities(
    mut commands: Commands,

    console_text_characters: Query<Entity, With<ConsoleTextCharacter>>,
    window_query: Query<&Window, With<PrimaryWindow>>,

    asset_server: Res<AssetServer>,
    mut character_scaling: ResMut<CharacterScaling>,
) {
    for entity in console_text_characters.iter() {
        commands.entity(entity).despawn();
    }

    let window = window_query.single().unwrap();
    let window_width = window.width();
    let window_height = window.height();

    let gameplay_width = window_width - 2.0 * BORDER_WIDTH as f32;
    let gameplay_height = window_height - 2.0 * BORDER_WIDTH as f32;

    let max_char_width = gameplay_width / 74.0;
    let max_char_height = gameplay_height / 23.0;

    let max_font_size_by_width = max_char_width / 60.0 * 100.0;
    let max_font_size_by_height = max_char_height / 120.0 * 100.0;

    let font_size = cmp::min((max_font_size_by_width * 100.0) as u32, (max_font_size_by_height * 100.0) as u32) as f32 * 0.01;
    character_scaling.font_size = font_size;

    let char_width = font_size * 60.0 / 100.0;
    let char_height = font_size * 120.0 / 100.0;
    character_scaling.char_width = char_width;
    character_scaling.char_height = char_height;

    let console_width = char_width * 74.0;
    let console_height = char_height * 23.0;

    let padding_x = (gameplay_width as f32 - console_width) * 0.5;
    let padding_y = (gameplay_height as f32 - console_height) * 0.5;

    let x_offset = BORDER_WIDTH as f32 + padding_x + char_width * 0.5;
    let y_offset = BORDER_WIDTH as f32 + padding_y + char_height * 0.5;
    character_scaling.x_offset = x_offset;
    character_scaling.y_offset = y_offset;

    let font = load_embedded_asset!(&*asset_server, "../../assets/font/JetBrainsMono-Bold.ttf");
    let text_font = TextFont {
        font: font.clone(),
        font_size,
        ..default()
    };

    let state = CONSOLE_STATE.lock().unwrap();
    let text_buffer = state.text_buffer();
    let text_color_buffer = state.text_color_buffer();

    let mut iter = text_buffer.iter().copied().zip(text_color_buffer.iter().copied());
    for y in 0..23 {
        for x in 0..74 {
            let (character, (fg, bg)) = iter.next().unwrap();

            let screen_x = x_offset + x as f32 * char_width - window_width as f32 * 0.5;
            let screen_y = window_height as f32 * 0.5 - (y_offset + y as f32 * char_height);

            commands.spawn((
                Text2d::new(String::from_utf8_lossy(&[character])),
                text_font.clone(),
                Transform::from_translation(Vec3::new(screen_x, screen_y, 0.0)),
                TextColor(fg.fg_color_into_bevy_color()),
                TextBackgroundColor(bg.bg_color_into_bevy_color()),
                ConsoleTextCharacter { x, y },
            ));
        }
    }
}

fn update_game(
    mut window_query: Query<&mut Window, With<PrimaryWindow>>,

    mut game: NonSendMut<Game>,

    character_scaling: Res<CharacterScaling>,

    mut keyboard_event: MessageReader<KeyboardInput>,
    mut mouse_event: MessageReader<MouseButtonInput>,

    mut app_exit_event_writer: MessageWriter<AppExit>,
) {
    {
        let mut window = window_query.single_mut().unwrap();

        let mut state = CONSOLE_STATE.lock().unwrap();
        for event in keyboard_event.read() {
            if event.state == ButtonState::Released {
                continue;
            }

            if event.logical_key == bevy::input::keyboard::Key::F11 {
                if window.mode == WindowMode::Windowed {
                    window.mode = WindowMode::BorderlessFullscreen(MonitorSelection::Primary);
                }else {
                    window.mode = WindowMode::Windowed;
                    window.position = WindowPosition::Centered(MonitorSelection::Current);
                }

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

fn on_resize(
    commands: Commands,

    console_text_characters: Query<Entity, With<ConsoleTextCharacter>>,
    window_query: Query<&Window, With<PrimaryWindow>>,

    asset_server: Res<AssetServer>,
    character_scaling: ResMut<CharacterScaling>,

    mut resize_reader: MessageReader<WindowResized>,
) {
    let event = resize_reader.read().last();
    if event.is_some() {
        update_text_entities(
            commands,

            console_text_characters,
            window_query,

            asset_server,
            character_scaling,
        );
    }
}

fn draw_game(
    mut console_text_characters: Query<(&mut Text2d, &mut TextColor, &mut TextBackgroundColor, &ConsoleTextCharacter)>,
) {
    //TODO optimize repaint logic

    let mut state = CONSOLE_STATE.lock().unwrap();
    //TODO check if dirty does any performance improvement
    if !state.dirty() {
        return;
    }

    let text_buffer = state.text_buffer();
    let text_color_buffer = state.text_color_buffer();

    for (
        ref mut text,
        ref mut fg_color,
        ref mut bg_color,
        ConsoleTextCharacter { x, y },
    ) in console_text_characters.iter_mut() {
        let character = text_buffer[x + y * 74];
        let (fg, bg) = text_color_buffer[x + y * 74];

        text.0 = String::from_utf8_lossy(&[character]).into();
        fg_color.0 = fg.fg_color_into_bevy_color();
        bg_color.0 = bg.bg_color_into_bevy_color();
    }

    state.mark_not_dirty();
}

