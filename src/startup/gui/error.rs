use bevy::asset::{embedded_asset, load_embedded_asset};
use bevy::prelude::*;
use crate::game::Game;
use crate::startup::gui::spawn_camera;

#[derive(Debug, Default, Resource)]
struct ErrorTextResource {
    error_text: Box<str>,
}

pub fn show_error_dialog(app: &mut App, error_message: &str) {
    app.
            add_plugins(DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    title: format!("Console Sokoban (v{})", Game::VERSION),
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
            add_systems(Startup, spawn_camera);

    embedded_asset!(app, "../../../assets/font/JetBrainsMono-Bold.ttf");

    app.
            insert_resource(ErrorTextResource {
                error_text: Box::from(error_message),
            }).

            add_systems(Startup, create_error_dialog_menu).

            add_systems(Update, button_update).
            //TODO allow closing window with "Enter"

            run();
}

fn create_error_dialog_menu(
    mut commands: Commands,

    error_text: Res<ErrorTextResource>,
    asset_server: Res<AssetServer>,
) {
    error!(target: "ConsoleSokoban", "{}", &error_text.error_text);

    let font = load_embedded_asset!(&*asset_server, "../../../assets/font/JetBrainsMono-Bold.ttf");
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
            Button,
            Node {
                width: px(100),
                height: px(50),
                border: UiRect::all(px(2)),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BorderColor::all(crate::io::bevy_abstraction::Color::White),
            BorderRadius::all(px(10)),
            BackgroundColor(crate::io::bevy_abstraction::Color::Black.into()),
            children![(
                Text::new("Ok"),
                text_font.clone(),
                TextColor(crate::io::bevy_abstraction::Color::White.into()),
            )],
        )],
    ));
}

fn button_update(
    mut interaction_query: Query<
        (
            &Interaction,
            &mut BackgroundColor,
        ),
        Changed<Interaction>,
    >,

    mut app_exit_event_writer: MessageWriter<AppExit>,
) {
    for (interaction, mut color) in interaction_query.iter_mut() {
        match interaction {
            Interaction::Pressed => {
                app_exit_event_writer.write(AppExit::Success);
            },

            Interaction::Hovered => {
                color.0 = crate::io::bevy_abstraction::Color::LightBlack.into();
            },

            Interaction::None => {
                color.0 = crate::io::bevy_abstraction::Color::Black.into();
            },
        }
    }
}
