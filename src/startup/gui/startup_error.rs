use std::io::Cursor;
use std::mem;
use std::path::{Path, PathBuf};
use bevy::asset::io::embedded::EmbeddedAssetRegistry;
use bevy::input_focus::{AutoFocus, InputDispatchPlugin, InputFocus};
use bevy::input_focus::tab_navigation::{TabGroup, TabIndex, TabNavigationPlugin};
use bevy::picking::hover::Hovered;
use bevy::prelude::*;
use bevy::ui_widgets::{observe, Activate, Button, Checkbox, RadioGroup, UiWidgetsPlugins};
use bevy::window::{CursorIcon, PrimaryWindow, SystemCursorIcon};
use rodio::{Decoder, OutputStream, Source};
use crate::game::{audio, Game};
use crate::startup::gui::{assets, spawn_camera};

#[derive(Debug, Default, Resource)]
struct ErrorTextResource {
    error_text: Box<str>,
}

pub fn show_startup_error_dialog(app: &mut App, error_message: &str) {
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
                    ..default()
                }),
                ..default()
            })).
            add_plugins((
                UiWidgetsPlugins,
                InputDispatchPlugin,
                TabNavigationPlugin,
            )).

            add_systems(Startup, (
                spawn_camera,
                play_error_sound
            ));

    let embedded = app.world_mut().resource_mut::<EmbeddedAssetRegistry>();

    embedded.insert_asset(
        PathBuf::from("../../../assets/font/JetBrainsMono-Bold.ttf"),
        Path::new("font/JetBrainsMono-Bold.ttf"),
        assets::font::JETBRAINS_MONO_BOLD_BYTES,
    );

    app.
            insert_resource(ErrorTextResource {
                error_text: Box::from(error_message),
            }).

            add_systems(Startup, create_error_dialog_menu).

            add_systems(Update, (
                update_hover_ui_styles,
                update_focus_styles,
                update_mouse_cursor_style,
            )).

            run();
}

fn play_error_sound() {
    let output_stream = OutputStream::try_default();
    if let Ok((stream, stream_handle)) = output_stream {
        let cursor = Cursor::new(audio::UI_ERROR_EFFECT);
        if let Ok(source) = Decoder::new(cursor) {
            let _ = stream_handle.play_raw(source.convert_samples());
        }

        //Keep OutputStream and OutputStreamHandle around to finish playing sound effect
        mem::forget(stream);
        mem::forget(stream_handle);
    }
}

fn create_error_dialog_menu(
    mut commands: Commands,

    error_text: Res<ErrorTextResource>,
    asset_server: Res<AssetServer>,
) {
    error!(target: "SokoTerm", "{}", &error_text.error_text);

    let font = asset_server.load("embedded://font/JetBrainsMono-Bold.ttf");
    let text_font = TextFont {
        font: font.clone(),
        font_size: 25.0,
        ..default()
    };
    let heading_font = TextFont {
        font: font.clone(),
        font_size: 40.0,
        ..default()
    };

    commands.spawn((
        Node {
            width: percent(100),
            height: percent(100),
            align_items: AlignItems::Center,
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::Center,
            row_gap: px(10),
            ..default()
        },
        TabGroup::default(),
        children![(
            Text::new("An error occurred"),
            heading_font.clone(),
            TextColor(crate::io::bevy_abstraction::Color::Red.into()),
            TextLayout::new(Justify::Center, LineBreak::WordBoundary),
        ), (
            Text::new(&*error_text.error_text),
            text_font.clone(),
            TextColor(crate::io::bevy_abstraction::Color::Red.into()),
            TextLayout::new(Justify::Center, LineBreak::WordBoundary),
        ), (
            Node {
                width: px(100),
                height: px(50),
                border: UiRect::all(px(2)),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            Button,
            AutoFocus,
            Hovered::default(),
            TabIndex::default(),
            BorderColor::all(crate::io::bevy_abstraction::Color::White),
            BorderRadius::all(px(10)),
            BackgroundColor(crate::io::bevy_abstraction::Color::Black.into()),
            children![(
                Text::new("Ok"),
                text_font.clone(),
                TextColor(crate::io::bevy_abstraction::Color::White.into()),
            )],
            observe(|_: On<Activate>, mut app_exit_event_writer: MessageWriter<AppExit>| {
                app_exit_event_writer.write(AppExit::Success);
            }),
        )],
    ));
}

#[expect(clippy::type_complexity)]
fn update_hover_ui_styles(
    button_query: Query<
        (&Hovered, &mut BackgroundColor),
        (
            With<Button>,
            Changed<Hovered>,
        ),
    >,
) {
    for (Hovered(hovered), mut background_color) in button_query {
        if *hovered {
            background_color.0 = crate::io::bevy_abstraction::Color::LightBlack.into();
        }else {
            background_color.0 = crate::io::bevy_abstraction::Color::Black.into();
        }
    }
}

#[expect(clippy::type_complexity)]
fn update_focus_styles(
    mut commands: Commands,

    focus: Res<InputFocus>,

    ui_element_query: Query<
        Entity,
        Or<(With<Button>, With<RadioGroup>, With<Checkbox>)>,
    >,
) {
    if focus.is_changed() {
        for ui_element_id in ui_element_query {
            if focus.0 == Some(ui_element_id) {
                commands.entity(ui_element_id).insert(Outline {
                    color: Color::WHITE,
                    width: px(5),
                    offset: px(5),
                });
            }else {
                commands.entity(ui_element_id).remove::<Outline>();
            }
        }
    }
}

fn update_mouse_cursor_style(
    mut commands: Commands,

    hovering_changed_query: Query<&Hovered, Changed<Hovered>>,
    hovered_query: Query<&Hovered>,

    window_query: Query<Entity, With<PrimaryWindow>>,
    cursor_icon_query: Query<&CursorIcon, With<PrimaryWindow>>,
) {
    if cursor_icon_query.iter().any(|cursor_icon| *cursor_icon == CursorIcon::System(SystemCursorIcon::Wait)) {
        return;
    }

    let any_hovering_changed = hovering_changed_query.iter().any(|_| true);
    if !any_hovering_changed {
        return;
    }

    let hovering_any = hovered_query.iter().
            any(|hovering| hovering.0);

    if let Ok(window_id) = window_query.single() {
        if hovering_any {
            commands.entity(window_id).insert(CursorIcon::System(SystemCursorIcon::Pointer));
        }else {
            commands.entity(window_id).insert(CursorIcon::System(SystemCursorIcon::Default));
        }
    }
}
