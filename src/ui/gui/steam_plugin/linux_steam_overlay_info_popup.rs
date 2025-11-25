//! Temporary plugin for showing that steam overlay is active
//!
//! Will be removed if steam overlay works on linux with bevy

use bevy::input_focus::AutoFocus;
use bevy::input_focus::tab_navigation::{TabGroup, TabIndex};
use bevy::prelude::*;
use bevy::text::LineHeight;
use bevy_steamworks::{CallbackResult, GameOverlayActivated, SteamworksEvent};
use crate::ui::gui::CharacterScaling;
use crate::ui::gui::steam_plugin::{on_resize_popup_text, ResizableNodeDimension, ResizableText};

#[derive(Debug, Component)]
struct LinuxSteamOverlayInfoPopup;

pub struct LinuxSteamOverlayInfoPlugin;

impl Plugin for LinuxSteamOverlayInfoPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, on_overlay_activated);
    }
}

fn on_overlay_activated(
    mut commands: Commands,

    mut steamworks_event: MessageReader<SteamworksEvent>,

    resizable_text_query: Query<(&mut TextFont, &ResizableText), With<ResizableText>>,
    resizable_node_dimension_query: Query<(&mut Node, &ResizableNodeDimension), With<ResizableNodeDimension>>,
    linux_steam_overlay_info_popup_elements: Query<Entity, With<LinuxSteamOverlayInfoPopup>>,

    character_scaling: Res<CharacterScaling>,

    asset_server: Res<AssetServer>,
) {
    for event in steamworks_event.read() {
        let SteamworksEvent::CallbackResult(event) = event;

        if let CallbackResult::GameOverlayActivated(GameOverlayActivated {active}) = event {
            if *active {
                let font = asset_server.load("embedded://font/JetBrainsMonoNL-ExtraLight.ttf");
                let text_font = TextFont {
                    font: font.clone(),
                    line_height: LineHeight::RelativeToFont(1.1),
                    font_size: 1.0, //Dummy value
                    ..default()
                };

                let font = asset_server.load("embedded://font/JetBrainsMono-Bold.ttf");
                let bold_text_font = TextFont {
                    font: font.clone(),
                    font_size: 1.0, //Dummy value
                    ..default()
                };
                let heading_font = TextFont {
                    font: font.clone(),
                    font_size: 1.0, //Dummy value
                    ..default()
                };

                commands.spawn((
                    Node {
                        width: percent(100),
                        height: percent(100),
                        align_items: AlignItems::Center,
                        flex_direction: FlexDirection::Column,
                        justify_content: JustifyContent::Center,
                        ..default()
                    },
                    BackgroundColor(Color::BLACK.with_alpha(0.75)),
                    LinuxSteamOverlayInfoPopup,
                    TabGroup::modal(),
                    children![(
                        Node {
                            width: percent(100),
                            height: percent(100),
                            align_items: AlignItems::Center,
                            flex_direction: FlexDirection::Column,
                            justify_content: JustifyContent::Center,
                            ..default()
                        },
                        TabIndex::default(),
                        AutoFocus,
                        children![(
                            Node {
                                display: Display::Grid,
                                width: percent(80),
                                height: percent(80),
                                min_width: px(460),
                                min_height: px(340),
                                align_items: AlignItems::Center,
                                grid_template_rows: vec![GridTrack::auto(), GridTrack::fr(1.0), GridTrack::auto()],
                                row_gap: px(10),
                                padding: UiRect::all(px(30)),
                                ..default()
                            },
                            BackgroundColor(Color::srgb_u8(180, 180, 180)),
                            BorderRadius::all(percent(5)),
                            children![(
                                Text("Steam Overlay is currently broken on Linux...".to_string()),
                                heading_font.clone(),
                                TextColor(Color::BLACK),
                                TextLayout::new(Justify::Center, LineBreak::WordBoundary),
                                ResizableText::Heading,
                            ), (
                                Text(
                                    "The Steam Overlay is currently broken on Linux with the default windowing library of the Bevy game engine.\n\
                                    Even tough the Steam Overlay is invisible, it will still block all inputs from reaching the game.\n\n".to_string(),
                                ),
                                text_font.clone(),
                                TextColor(Color::BLACK),
                                TextLayout::new(Justify::Center, LineBreak::WordBoundary),
                                ResizableText::Paragraph,
                                children![(
                                    TextSpan(
                                        "Please press SHIFT + TAB (or your custom key combination) to close the invisible Steam Overlay to continue.\n".to_string(),
                                    ),
                                    bold_text_font.clone(),
                                    TextColor(Color::BLACK),
                                    ResizableText::Paragraph,
                                ), (
                                    TextSpan(
                                        "\n\
                                        If you desperately need it, you can play the Windows version of this game \
                                        through Proton which works just as good as the native Linux version!".to_string(),
                                    ),
                                    text_font.clone(),
                                    TextColor(Color::BLACK),
                                    ResizableText::Paragraph,
                                )],
                            ), (
                                Node {
                                    width: percent(100),
                                    flex_direction: FlexDirection::Column,
                                    row_gap: px(10),
                                    ..default()
                                },
                            )],
                        )],
                    )],
                ));
            }else {
                for entity in linux_steam_overlay_info_popup_elements.iter() {
                    commands.entity(entity).despawn();
                }
            }
        }
    }

    on_resize_popup_text(character_scaling, resizable_text_query, resizable_node_dimension_query);
}
