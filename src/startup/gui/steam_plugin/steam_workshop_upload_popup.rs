use std::error::Error;
use std::path::Path;
use std::sync::{Arc, LazyLock, Mutex};
use bevy::input::ButtonState;
use bevy::input::keyboard::{Key, KeyboardInput};
use bevy::input_focus::{AutoFocus, InputDispatchPlugin, InputFocus};
use bevy::input_focus::tab_navigation::{TabGroup, TabIndex, TabNavigationPlugin};
use bevy::picking::hover::Hovered;
use bevy::ui_widgets::{checkbox_self_update, observe, Activate, Button, Checkbox, RadioButton, RadioGroup, UiWidgetsPlugins, ValueChange};
use bevy::prelude::*;
use bevy::text::LineHeight;
use bevy::ui::Checked;
use bevy_steamworks::*;
use crate::game::{audio, steam, Game, GameError};
use crate::game::steam::achievement::Achievement;
use crate::startup::gui::AppState;
use crate::startup::gui::steam_plugin::{handle_recoverable_error, on_resize_popup_text, PlaySoundEffect, ResizableNodeDimension, ResizableText};
use crate::utils;

pub struct SteamWorkshopUploadPopupPlugin;

impl Plugin for SteamWorkshopUploadPopupPlugin {
    fn build(&self, app: &mut App) {
        app.
                add_plugins((
                    UiWidgetsPlugins,
                    InputDispatchPlugin,
                    TabNavigationPlugin,
                )).

                insert_resource(DifficultyTag::Easy).

                add_message::<ValidateAndStartUpload>().
                add_message::<SetUploadProgressPopupTitle>().
                add_message::<SetUploadProgressPopupContent>().

                add_systems(Update, (
                    process_and_update_upload_progress.pipe(handle_recoverable_error),
                    process_update_progress_status.pipe(handle_recoverable_error),
                    update_text_input_fields,
                    update_radio_button_checked_state,
                    update_ui_styles,
                    update_hover_ui_styles,
                    update_focus_styles,
                    on_validate_and_start_upload,
                    on_set_upload_progress_title.pipe(handle_recoverable_error),
                    on_set_upload_progress_content.pipe(handle_recoverable_error),
                ).run_if(in_state(AppState::SteamWorkshopUploadPopup))).

                add_systems(OnEnter(AppState::SteamWorkshopUploadPopup), on_open_steam_workshop_upload_popup).
                add_systems(OnEnter(AppState::SteamWorkshopUploadPopup), on_resize_popup_text.after(on_open_steam_workshop_upload_popup)).

                add_systems(OnExit(AppState::SteamWorkshopUploadPopup), on_close_steam_workshop_upload_popup);
    }
}

#[derive(Debug, Default, Clone)]
enum SteamWorkshopUploadWorkingData {
    #[default]
    Idle,
    Waiting,
    CreateItemResult(Result<(PublishedFileId, bool), SteamError>),
    SubmitItemResult((PublishedFileId, Result<bool, SteamError>)),
}

#[expect(clippy::type_complexity)]
static STEAM_WORKSHOP_UPLOAD_WORKING_DATA: LazyLock<
    Arc<Mutex<SteamWorkshopUploadWorkingData>>,
    fn() -> Arc<Mutex<SteamWorkshopUploadWorkingData>>,
> = LazyLock::new(Default::default);

#[derive(Resource)]
struct UpdateWatchHandleWrapper(UpdateWatchHandle);

#[derive(Debug, Resource)]
struct PreviousUpdateStatus((UpdateStatus, u64, u64));

#[derive(Debug, Message)]
struct ValidateAndStartUpload;

#[derive(Debug, Component)]
struct SteamWorkshopUploadPopup;

#[derive(Debug, Component)]
struct UploadProgressPopup;

#[derive(Debug, Component)]
struct UploadProgressPopupTitle;

#[derive(Debug, Component)]
struct UploadProgressPopupContent;

#[derive(Debug, Component)]
struct UploadProgressPopupButtonContainer;

#[derive(Debug, Clone, Message)]
struct SetUploadProgressPopupTitle {
    title: String,
    error: bool,
}

#[derive(Debug, Clone, Message)]
struct SetUploadProgressPopupContent {
    text: String,
    error: bool,
}

#[derive(Debug, Component)]
struct LevelPackName;

#[derive(Debug, Component)]
struct LevelPackDescription;

#[derive(Debug, Component)]
struct TextCursor;

#[derive(Debug, Component)]
struct LinkText;

#[derive(Debug, Clone, Copy, Ord, PartialOrd, Eq, PartialEq, Hash, Resource, Component)]
enum DifficultyTag {
    Easy,
    Medium,
    Hard,
    Demon,
}

#[derive(Debug, Clone, Copy, Ord, PartialOrd, Eq, PartialEq, Hash, Resource, Component)]
enum GameplayTag {
    Fun,
    Tricky,
    Weird,
}

#[expect(clippy::too_many_arguments)]
fn process_and_update_upload_progress(
    mut commands: Commands,

    level_pack_name_text_input_field_query: Query<
        &Children,
        With<LevelPackName>,
    >,

    level_pack_description_text_input_field_query: Query<
        &Children,
        With<LevelPackDescription>,
    >,

    gameplay_tag_checkboxes_query: Query<
        (Has<Checked>, &GameplayTag),
    >,

    text_query: Query<&Text>,

    upload_progress_popup_button_container_query: Query<Entity, With<UploadProgressPopupButtonContainer>>,

    steam_client: Res<Client>,
    difficulty_tag_resource: Res<DifficultyTag>,
    asset_server: Res<AssetServer>,

    mut set_upload_progress_popup_title: MessageWriter<SetUploadProgressPopupTitle>,
    mut set_upload_progress_popup_content: MessageWriter<SetUploadProgressPopupContent>,

    mut play_sound_effect: MessageWriter<PlaySoundEffect>,
) -> Result<(), Box<dyn Error>> {
    let current_data = STEAM_WORKSHOP_UPLOAD_WORKING_DATA.lock().unwrap().clone();
    match current_data {
        SteamWorkshopUploadWorkingData::CreateItemResult(Ok((id, _needs_to_accept_workshop_terms))) => {
            let level_pack_name = {
                let Some(children) = level_pack_name_text_input_field_query.iter().next() else {
                    return Err(Box::new(GameError::new("Level pack name input field invalid")));
                };

                let Some(text_entity_id) = children.first() else {
                    return Err(Box::new(GameError::new("Level pack name input field is invalid")));
                };

                let Ok(level_pack_name) = text_query.get(*text_entity_id) else {
                    return Err(Box::new(GameError::new("Level pack name input field is invalid")));
                };

                level_pack_name
            };
            let level_pack_description = {
                let Some(children) = level_pack_description_text_input_field_query.iter().next() else {
                    return Err(Box::new(GameError::new("Level pack description input field invalid")));
                };

                let Some(text_entity_id) = children.first() else {
                    return Err(Box::new(GameError::new("Level pack description input field is invalid")));
                };

                let Ok(level_pack_description) = text_query.get(*text_entity_id) else {
                    return Err(Box::new(GameError::new("Level pack description input field is invalid")));
                };

                level_pack_description
            };

            let difficulty_tag = match &*difficulty_tag_resource {
                DifficultyTag::Easy => "Easy",
                DifficultyTag::Medium => "Medium",
                DifficultyTag::Hard => "Hard",
                DifficultyTag::Demon => "Demon",
            };

            let mut gameplay_tags = Vec::new();
            for (checked, gameplay_tag) in gameplay_tag_checkboxes_query {
                if checked {
                    gameplay_tags.push(match gameplay_tag {
                        GameplayTag::Fun => "Fun",
                        GameplayTag::Tricky => "Tricky",
                        GameplayTag::Weird => "Weird",
                    });
                }
            }

            let mut tags = gameplay_tags;
            tags.push(difficulty_tag);

            let mut tmp_upload_path = Game::get_or_create_save_game_folder()?;
            tmp_upload_path.push("SteamWorkshop/UploadTemp/");

            let handle = steam_client.ugc().start_item_update(steam::APP_ID, id).
                    visibility(PublishedFileVisibility::Private).
                    title(level_pack_name).
                    description(level_pack_description).
                    content_path(Path::new(&tmp_upload_path)).
                    //TODO preview
                    tags(tags, false).
                    submit(Some("<Initial Release>"), move |ret| {
                        *STEAM_WORKSHOP_UPLOAD_WORKING_DATA.lock().unwrap() = SteamWorkshopUploadWorkingData::SubmitItemResult(match ret {
                            Ok((id, needs_to_accept_workshop_terms)) => {
                                (id, Ok(needs_to_accept_workshop_terms))
                            },

                            Err(err) => {
                                (id, Err(err))
                            },
                        });
                    });

            commands.insert_resource(UpdateWatchHandleWrapper(handle));
            commands.insert_resource(PreviousUpdateStatus((UpdateStatus::Invalid, 0, 0)));

            *STEAM_WORKSHOP_UPLOAD_WORKING_DATA.lock().unwrap() = SteamWorkshopUploadWorkingData::Waiting;
        },
        SteamWorkshopUploadWorkingData::CreateItemResult(Err(err)) => {
            *STEAM_WORKSHOP_UPLOAD_WORKING_DATA.lock().unwrap() = SteamWorkshopUploadWorkingData::Waiting;

            play_sound_effect.write(PlaySoundEffect {
                sound_effect: audio::UI_ERROR_EFFECT,
            });

            set_upload_progress_popup_title.write(SetUploadProgressPopupTitle {
                title: "Upload failed!".to_string(),
                error: true,
            });

            set_upload_progress_popup_content.write(SetUploadProgressPopupContent {
                text: format!("An error occurred during item creation: {err}"),
                error: true,
            });

            let Some(popup_button_container_id) = upload_progress_popup_button_container_query.into_iter().next() else {
                return Err(Box::new(GameError::new("Invalid popup status")));
            };

            let font = asset_server.load("embedded://font/JetBrainsMonoNL-ExtraLight.ttf");
            let text_font = TextFont {
                font: font.clone(),
                line_height: LineHeight::RelativeToFont(1.1),
                font_size: 1.0, //Dummy value
                ..default()
            };

            commands.entity(popup_button_container_id).with_child((
                Node {
                    width: percent(100),
                    border: UiRect::all(px(2)),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                Button,
                Hovered::default(),
                TabIndex::default(),
                BorderColor::all(crate::io::bevy_abstraction::Color::White),
                BorderRadius::all(px(10)),
                BackgroundColor(crate::io::bevy_abstraction::Color::Black.into()),
                children![(
                    Text::new("Close"),
                    text_font.clone(),
                    TextColor(crate::io::bevy_abstraction::Color::White.into()),
                    ResizableText::Paragraph,
                )],
                observe(
                    |_: On<Activate>,

                     commands: Commands,

                     upload_progress_popup_elements: Query<Entity, With<UploadProgressPopup>>,

                     mut play_sound_effect: MessageWriter<PlaySoundEffect>| {
                        *STEAM_WORKSHOP_UPLOAD_WORKING_DATA.lock().unwrap() = SteamWorkshopUploadWorkingData::Idle;

                        play_sound_effect.write(PlaySoundEffect {
                            sound_effect: audio::UI_SELECT_EFFECT,
                        });

                        close_upload_progress_popup(commands, upload_progress_popup_elements);
                    },
                ),
            ));

            return Ok(());
        },

        SteamWorkshopUploadWorkingData::SubmitItemResult((id, Ok(needs_to_accept_workshop_terms))) => {
            steam_client.friends().activate_game_overlay_to_web_page(&format!("steam://url/CommunityFilePage/{}", id.0));

            Achievement::STEAM_WORKSHOP_LEVEL_PACK_CREATED.unlock(steam_client.clone());

            commands.remove_resource::<UpdateWatchHandleWrapper>();
            commands.remove_resource::<PreviousUpdateStatus>();
            *STEAM_WORKSHOP_UPLOAD_WORKING_DATA.lock().unwrap() = SteamWorkshopUploadWorkingData::Waiting;

            play_sound_effect.write(PlaySoundEffect {
                sound_effect: audio::LEVEL_COMPLETE_EFFECT,
            });

            set_upload_progress_popup_title.write(SetUploadProgressPopupTitle {
                title: "Upload completed!".to_string(),
                error: false,
            });

            set_upload_progress_popup_content.write(SetUploadProgressPopupContent {
                text:

                "To make this level pack public, you must change the visibility in the Steam Workshop to public!\n\
\n\
                You can also change the title and description there.\n\
                You can also upload additional images.".to_string(),

                error: false,
            });

            let Some(popup_button_container_id) = upload_progress_popup_button_container_query.into_iter().next() else {
                return Err(Box::new(GameError::new("Invalid popup status")));
            };

            let font = asset_server.load("embedded://font/JetBrainsMonoNL-ExtraLight.ttf");
            let bold_text_font = TextFont {
                font: font.clone(),
                font_size: 1.0, //Dummy value
                ..default()
            };
            let text_font = TextFont {
                font: font.clone(),
                line_height: LineHeight::RelativeToFont(1.1),
                font_size: 1.0, //Dummy value
                ..default()
            };

            if needs_to_accept_workshop_terms {
                commands.entity(popup_button_container_id).with_child((
                    Node {
                        align_items: AlignItems::Center,
                        flex_direction: FlexDirection::Column,
                        justify_content: JustifyContent::Center,
                        ..default()
                    },
                    children![(
                        Text("To make this level pack public, you need to agree to the".to_string()),
                        bold_text_font.clone(),
                        TextColor(crate::io::bevy_abstraction::Color::Black.into()),
                        TextLayout::new(Justify::Center, LineBreak::NoWrap),
                        ResizableText::Paragraph,
                    ), (
                        Node {
                            border: UiRect::bottom(px(5)),
                            box_sizing: BoxSizing::BorderBox,
                            ..default()
                        },
                        BorderColor::all(crate::io::bevy_abstraction::Color::LightBlue),
                        children![(
                            Text("workshop terms of service".to_string()),
                            bold_text_font.clone(),
                            Button,
                            LinkText,
                            Hovered::default(),
                            TabIndex::default(),
                            TextColor(crate::io::bevy_abstraction::Color::LightBlue.into()),
                            TextLayout::new(Justify::Center, LineBreak::NoWrap),
                            ResizableText::Paragraph,
                            observe(|_: On<Activate>, steam_client: Res<Client>| {
                                steam_client.friends().activate_game_overlay_to_web_page("steam://openurl/https://steamcommunity.com/sharedfiles/workshoplegalagreement");
                            }),
                        )],
                    )],
                ));
            }

            commands.entity(popup_button_container_id).with_child((
                Node {
                    width: percent(100),
                    border: UiRect::all(px(2)),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                Button,
                Hovered::default(),
                TabIndex::default(),
                BorderColor::all(crate::io::bevy_abstraction::Color::White),
                BorderRadius::all(px(10)),
                BackgroundColor(crate::io::bevy_abstraction::Color::Black.into()),
                children![(
                    Text::new("Close"),
                    text_font.clone(),
                    TextColor(crate::io::bevy_abstraction::Color::White.into()),
                    ResizableText::Paragraph,
                )],
                observe(
                    |_: On<Activate>,

                     commands: Commands,

                     upload_progress_popup_elements: Query<Entity, With<UploadProgressPopup>>,

                     mut app_state_next_state: ResMut<NextState<AppState>>,

                     mut play_sound_effect: MessageWriter<PlaySoundEffect>| {
                        play_sound_effect.write(PlaySoundEffect {
                            sound_effect: audio::UI_SELECT_EFFECT,
                        });

                        close_upload_progress_popup(commands, upload_progress_popup_elements);
                        app_state_next_state.set(AppState::InGame);
                    },
                ),
            ));
        },

        SteamWorkshopUploadWorkingData::SubmitItemResult((id, Err(err))) => {
            commands.remove_resource::<UpdateWatchHandleWrapper>();
            commands.remove_resource::<PreviousUpdateStatus>();
            *STEAM_WORKSHOP_UPLOAD_WORKING_DATA.lock().unwrap() = SteamWorkshopUploadWorkingData::Waiting;

            steam_client.ugc().delete_item(id, |ret| {
                info!("Workshop item upload failed: Cleanup status: {ret:?}");
            });

            play_sound_effect.write(PlaySoundEffect {
                sound_effect: audio::UI_ERROR_EFFECT,
            });

            set_upload_progress_popup_title.write(SetUploadProgressPopupTitle {
                title: "Upload failed!".to_string(),
                error: true,
            });

            set_upload_progress_popup_content.write(SetUploadProgressPopupContent {
                text: format!("An error occurred during item submission: {err}"),
                error: true,
            });

            let Some(popup_button_container_id) = upload_progress_popup_button_container_query.into_iter().next() else {
                return Err(Box::new(GameError::new("Invalid popup status")));
            };

            let font = asset_server.load("embedded://font/JetBrainsMonoNL-ExtraLight.ttf");
            let text_font = TextFont {
                font: font.clone(),
                line_height: LineHeight::RelativeToFont(1.1),
                font_size: 1.0, //Dummy value
                ..default()
            };

            commands.entity(popup_button_container_id).with_child((
                Node {
                    width: percent(100),
                    border: UiRect::all(px(2)),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                Button,
                Hovered::default(),
                TabIndex::default(),
                BorderColor::all(crate::io::bevy_abstraction::Color::White),
                BorderRadius::all(px(10)),
                BackgroundColor(crate::io::bevy_abstraction::Color::Black.into()),
                children![(
                    Text::new("Close"),
                    text_font.clone(),
                    TextColor(crate::io::bevy_abstraction::Color::White.into()),
                    ResizableText::Paragraph,
                )],
                observe(
                    |_: On<Activate>,

                     commands: Commands,

                     upload_progress_popup_elements: Query<Entity, With<UploadProgressPopup>>,

                     mut play_sound_effect: MessageWriter<PlaySoundEffect>| {
                        *STEAM_WORKSHOP_UPLOAD_WORKING_DATA.lock().unwrap() = SteamWorkshopUploadWorkingData::Idle;

                        play_sound_effect.write(PlaySoundEffect {
                            sound_effect: audio::UI_SELECT_EFFECT,
                        });

                        close_upload_progress_popup(commands, upload_progress_popup_elements);
                    },
                ),
            ));

            return Ok(());
        },

        SteamWorkshopUploadWorkingData::Waiting |
        SteamWorkshopUploadWorkingData::Idle => {},
    }

    Ok(())
}

fn process_update_progress_status(
    upload_progress_popup_content_text_query: Query<(&mut Text, &mut TextColor), With<UploadProgressPopupContent>>,

    update_watch_handle: Option<Res<UpdateWatchHandleWrapper>>,
    previous_update_status: Option<ResMut<PreviousUpdateStatus>>,
) -> Result<(), Box<dyn Error>> {
    'update_check: {
        if let Some(update_watch_handle) = update_watch_handle &&
                let Some(mut previous_update_status) = previous_update_status {
            let current_update_status = update_watch_handle.0.progress();
            if previous_update_status.0 == current_update_status {
                break 'update_check;
            };
            previous_update_status.0 = current_update_status;

            let (status, progress, max_progress) = current_update_status;

            let update_status_text = match status {
                UpdateStatus::Invalid => {
                    "Invalid".to_string()
                }
                UpdateStatus::PreparingConfig => {
                    "Preparing config...".to_string()
                }
                UpdateStatus::PreparingContent => {
                    "Preparing content...".to_string()
                }
                UpdateStatus::UploadingContent => {
                    format!(
                        "Uploading content... ({} / {})",
                        utils::byte_count_to_string_with_binary_prefix(progress),
                        utils::byte_count_to_string_with_binary_prefix(max_progress),
                    )
                }
                UpdateStatus::UploadingPreviewFile => {
                    format!(
                        "Uploading preview image... ({} / {})",
                        utils::byte_count_to_string_with_binary_prefix(progress),
                        utils::byte_count_to_string_with_binary_prefix(max_progress),
                    )
                }
                UpdateStatus::CommittingChanges => {
                    "Committing changes...".to_string()
                }
            };
            info!("Workshop Update Item status: {update_status_text}");

            let Some((mut popup_text, _)) = upload_progress_popup_content_text_query.into_iter().next() else {
                return Err(Box::new(GameError::new("Invalid popup status")));
            };

            if !matches!(status, UpdateStatus::Invalid) {
                popup_text.0 = update_status_text;
            }
        }
    }

    Ok(())
}

#[expect(clippy::too_many_arguments)]
fn update_text_input_fields(
    focus: Res<InputFocus>,
    time: Res<Time>,

    level_pack_name_text_input_field_query: Query<
        &Children,
        With<LevelPackName>,
    >,

    level_pack_description_text_input_field_query: Query<
        &Children,
        With<LevelPackDescription>,
    >,

    children_query: Query<&Children>,
    mut text_query: Query<&mut Text>,
    mut text_color_query: Query<&mut TextColor>,

    mut keyboard_event: MessageReader<KeyboardInput>,
) {
    let Some(entity_id) = focus.0 else {
        return;
    };

    let (children, is_level_pack_name) = if let Ok(children) = level_pack_name_text_input_field_query.get(entity_id) {
        (children, true)
    }else if let Ok(children) = level_pack_description_text_input_field_query.get(entity_id) {
        (children, false)
    }else {
        return;
    };

    let Some(text_entity_id) = children.first() else {
        warn!("Invalid text input field");
        return;
    };

    let Ok(mut text) = text_query.get_mut(*text_entity_id) else {
        return;
    };

    let show_cursor = (time.elapsed_secs_wrapped() * 2.0) as u32 & 1 == 1;
    if let Ok(children) = children_query.get(*text_entity_id) {
        let Some(text_span_entity_id) = children.first() else {
            warn!("Invalid text input field");
            return;
        };

        if let Ok(mut text_color) = text_color_query.get_mut(*text_span_entity_id) {
            if show_cursor {
                text_color.0 = Color::BLACK;
            }else {
                text_color.0 = Color::NONE;
            }
        }
    }

    for event in keyboard_event.read() {
        if event.state == ButtonState::Released {
            continue;
        }

        if event.logical_key == Key::Tab {
            //Focus changed
            return;
        }

        if event.logical_key == Key::Backspace {
            if !text.is_empty() {
                text.pop();
            }

            continue;
        }

        if matches!(event.logical_key, Key::Delete | Key::Escape) {
            continue;
        }

        if is_level_pack_name  && event.logical_key == Key::Enter {
            continue;
        }

        //TODO check for control key

        if let Some(key) = &event.text {
            if key == "\r" {
                text.push('\n');
            }else {
                text.push_str(key);
            }
        }
    }
}

fn update_radio_button_checked_state(
    mut commands: Commands,

    difficulty_tag_radio_input_query: Query<(Entity, &DifficultyTag, Has<Checked>)>,

    difficulty_tag_resource: Res<DifficultyTag>,
) {
    for (entity_id, value, checked) in difficulty_tag_radio_input_query.iter() {
        let checked_new = *value == *difficulty_tag_resource;
        if checked_new != checked {
            if checked_new {
                commands.entity(entity_id).insert(Checked);
            }else {
                commands.entity(entity_id).remove::<Checked>();
            }
        }
    }
}

#[expect(clippy::type_complexity)]
fn update_ui_styles(
    radio_and_checkbox_query: Query<
        (Has<Checked>, &Hovered, &Children),
        (
            Or<(With<RadioButton>, With<Checkbox>)>,
            Or<(Added<Checked>, Changed<Hovered>, Changed<DifficultyTag>)>
        ),
    >,

    mut radio_or_checkbox_unticked: RemovedComponents<Checked>,
    radio_or_checkbox_unticked_query: Query<
        (&Hovered, &Children),
        (
            Or<(With<RadioButton>, With<Checkbox>)>,
        ),
    >,

    children_query: Query<&Children>,
    mut background_color_query: Query<&mut BackgroundColor, Without<Children>>,
    mut border_color_query: Query<&mut BorderColor, With<Children>>,
) {
    for (checked, Hovered(hovered), children) in radio_and_checkbox_query.into_iter() {
        let Some(radio_or_checkbox_node_id) = children.get(1) else {
            warn!("Invalid radio button or checkbox");
            continue;
        };

        let Some(radio_or_checkbox_inner_node_id) = children_query.get(*radio_or_checkbox_node_id).ok().and_then(|children| children.first()) else {
            warn!("Invalid radio button or checkbox");
            continue;
        };

        let Ok(mut border_color) = border_color_query.get_mut(*radio_or_checkbox_inner_node_id) else {
            warn!("Invalid radio button or checkbox");
            continue;
        };

        let Some(radio_or_checkbox_inner_inner_node_id) = children_query.get(*radio_or_checkbox_inner_node_id).ok().and_then(|children| children.first()) else {
            warn!("Invalid radio button or checkbox");
            continue;
        };

        let Ok(mut background_color) = background_color_query.get_mut(*radio_or_checkbox_inner_inner_node_id) else {
            warn!("Invalid radio button or checkbox");
            continue;
        };

        if checked {
            background_color.0 = crate::io::bevy_abstraction::Color::Green.into();
        }else {
            background_color.0 = Srgba::NONE.into();
        }

        if *hovered {
            *border_color = BorderColor::all(crate::io::bevy_abstraction::Color::LightBlack);
        }else {
            *border_color = BorderColor::all(Color::BLACK);
        }
    }

    for radio_id in radio_or_checkbox_unticked.read() {
        if let Ok((Hovered(hovered), children)) = radio_or_checkbox_unticked_query.get(radio_id) {
            let Some(radio_or_checkbox_node_id) = children.get(1) else {
                warn!("Invalid radio button or checkbox");
                continue;
            };

            let Some(radio_or_checkbox_inner_node_id) = children_query.get(*radio_or_checkbox_node_id).ok().and_then(|children| children.first()) else {
                warn!("Invalid radio button or checkbox");
                continue;
            };

            let Ok(mut border_color) = border_color_query.get_mut(*radio_or_checkbox_inner_node_id) else {
                warn!("Invalid radio button or checkbox");
                continue;
            };

            let Some(radio_or_checkbox_inner_inner_node_id) = children_query.get(*radio_or_checkbox_inner_node_id).ok().and_then(|children| children.first()) else {
                warn!("Invalid radio button or checkbox");
                continue;
            };

            let Ok(mut background_color) = background_color_query.get_mut(*radio_or_checkbox_inner_inner_node_id) else {
                warn!("Invalid radio button or checkbox");
                continue;
            };

            background_color.0 = Srgba::NONE.into();

            if *hovered {
                *border_color = BorderColor::all(crate::io::bevy_abstraction::Color::LightBlack);
            }else {
                *border_color = BorderColor::all(Color::BLACK);
            }
        }
    }
}

#[expect(clippy::type_complexity)]
fn update_hover_ui_styles(
    button_query: Query<
        (&Hovered, &mut BackgroundColor),
        (
            With<Button>,
            Changed<Hovered>,
            Without<LinkText>,
        ),
    >,

    link_query: Query<
        (&Hovered, &mut TextColor, &ChildOf),
        (
            With<Button>,
            Changed<Hovered>,
            With<LinkText>,
        ),
    >,

    mut border_color_query: Query<&mut BorderColor>,
) {
    for (Hovered(hovered), mut background_color) in button_query {
        if *hovered {
            background_color.0 = crate::io::bevy_abstraction::Color::LightBlack.into();
        }else {
            background_color.0 = crate::io::bevy_abstraction::Color::Black.into();
        }
    }

    for (Hovered(hovered), mut text_color, child_of) in link_query {
        if *hovered {
            text_color.0 = crate::io::bevy_abstraction::Color::Blue.into();
        }else {
            text_color.0 = crate::io::bevy_abstraction::Color::LightBlue.into();
        }

        let parent = child_of.parent();
        if let Ok(mut border_color) = border_color_query.get_mut(parent) {
            if *hovered {
                (border_color).set_all(crate::io::bevy_abstraction::Color::Blue);
            }else {
                border_color.set_all(crate::io::bevy_abstraction::Color::LightBlue);
            }
        }
    }
}

#[expect(clippy::type_complexity)]
fn update_focus_styles(
    mut commands: Commands,

    focus: Res<InputFocus>,

    ui_element_query: Query<
        Entity,
        Or<(With<LevelPackName>, With<LevelPackDescription>, With<Button>, With<RadioGroup>, With<Checkbox>)>,
    >,

    text_cursor_query: Query<
        &mut TextColor,
        With<TextCursor>,
    >,
) {
    if focus.is_changed() {
        for mut text_color in text_cursor_query {
            text_color.0 = Color::NONE;
        }

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

fn on_validate_and_start_upload(
    mut commands: Commands,

    mut event_reader: MessageReader<ValidateAndStartUpload>,

    asset_server: Res<AssetServer>,
    steam_client: Res<Client>,
) {
    for _ in event_reader.read() {
        let font = asset_server.load("embedded://font/JetBrainsMonoNL-ExtraLight.ttf");
        let text_font = TextFont {
            font: font.clone(),
            line_height: LineHeight::RelativeToFont(1.1),
            font_size: 1.0, //Dummy value
            ..default()
        };
        let font = asset_server.load("embedded://font/JetBrainsMono-Bold.ttf");
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
            UploadProgressPopup,
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
                        width: percent(70),
                        height: percent(60),
                        min_width: px(380),
                        min_height: px(240),
                        align_items: AlignItems::Center,
                        grid_template_rows: vec![GridTrack::auto(), GridTrack::fr(1.0), GridTrack::auto()],
                        row_gap: px(10),
                        padding: UiRect::all(px(30)),
                        ..default()
                    },
                    BackgroundColor(Color::srgb_u8(180, 180, 180)),
                    BorderRadius::all(percent(5)),
                    children![(
                        Text("Upload in progress".to_string()),
                        UploadProgressPopupTitle,
                        heading_font.clone(),
                        TextColor(Color::BLACK),
                        TextLayout::new(Justify::Center, LineBreak::WordBoundary),
                        ResizableText::Heading,
                    ), (
                        Text("Validating...".to_string()),
                        UploadProgressPopupContent,
                        text_font.clone(),
                        TextColor(Color::BLACK),
                        TextLayout::new(Justify::Center, LineBreak::WordBoundary),
                        ResizableText::Paragraph,
                    ), (
                        Node {
                            width: percent(100),
                            flex_direction: FlexDirection::Column,
                            row_gap: px(10),
                            ..default()
                        },
                        UploadProgressPopupButtonContainer,
                        children![()],
                    )],
                )],
            )],
        ));

        //TODO validate inputs (title, description) [Not empty {title with warning: should be english characters only}]

        *STEAM_WORKSHOP_UPLOAD_WORKING_DATA.lock().unwrap() = SteamWorkshopUploadWorkingData::Waiting;

        steam::crate_workshop_item(steam_client.clone(), |ret| {
            *STEAM_WORKSHOP_UPLOAD_WORKING_DATA.lock().unwrap() = SteamWorkshopUploadWorkingData::CreateItemResult(ret);
        });
    }
}

fn on_open_steam_workshop_upload_popup(
    mut commands: Commands,

    game: NonSend<Game>,

    asset_server: Res<AssetServer>,
) {
    *STEAM_WORKSHOP_UPLOAD_WORKING_DATA.lock().unwrap() = SteamWorkshopUploadWorkingData::Idle;

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
        TabGroup::default(),
        BackgroundColor(Color::BLACK.with_alpha(0.75)),
        SteamWorkshopUploadPopup,
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
                Node {
                    flex_direction: FlexDirection::Column,
                    row_gap: px(10),
                    ..default()
                },
                children![(
                    Text("Level pack upload".to_string()),
                    heading_font.clone(),
                    TextColor(Color::BLACK),
                    TextLayout::new(Justify::Center, LineBreak::WordBoundary),
                    ResizableText::Heading,
                ), (
                    Text(format!("You are about to upload level pack \"{}\".", game.game_state().editor_state().get_current_level_pack().unwrap().id())),
                    text_font.clone(),
                    TextColor(Color::BLACK),
                    TextLayout::new(Justify::Center, LineBreak::WordBoundary),
                    ResizableText::Paragraph,
                ), (
                    Node {
                        min_height: percent(2),
                        ..default()
                    },
                )],
            ), (
                Node {
                    width: percent(100),
                    align_items: AlignItems::FlexStart,
                    flex_direction: FlexDirection::Column,
                    justify_content: JustifyContent::FlexStart,
                    ..default()
                },
                children![(
                    //TODO Mark with red color if invalid

                    Text("Level pack name:".to_string()),
                    bold_text_font.clone(),
                    TextColor(Color::BLACK),
                    TextLayout::new(Justify::Left, LineBreak::WordBoundary),
                    ResizableText::Paragraph,
                ), (
                    Node {
                        width: percent(100),
                        align_items: AlignItems::FlexStart,
                        flex_direction: FlexDirection::Row,
                        justify_content: JustifyContent::FlexStart,
                        padding: UiRect::all(px(10)),
                        overflow: Overflow::scroll(),
                        ..default()
                    },
                    LevelPackName,
                    TabIndex::default(),
                    BackgroundColor(Color::srgb_u8(120, 120, 120)),
                    ResizableNodeDimension::Height(1.2),
                    children![(
                        Text("".to_string()),
                        text_font.clone(),
                        TextColor(Color::BLACK),
                        ResizableText::Paragraph,
                        children![(
                            TextSpan("|".to_string()),
                            TextCursor,
                            text_font.clone(),
                            TextColor(Color::NONE),
                            ResizableText::Paragraph,
                        )],
                    )],
                ), (
                    //TODO Mark with red color if invalid

                    Text("Level pack description:".to_string()),
                    bold_text_font.clone(),
                    TextColor(Color::BLACK),
                    TextLayout::new(Justify::Left, LineBreak::WordBoundary),
                    ResizableText::Paragraph,
                ), (
                    Node {
                        width: percent(100),
                        align_items: AlignItems::FlexStart,
                        flex_direction: FlexDirection::Row,
                        justify_content: JustifyContent::FlexStart,
                        padding: UiRect::all(px(10)),
                        overflow: Overflow::scroll(),
                        ..default()
                    },
                    LevelPackDescription,
                    TabIndex::default(),
                    BackgroundColor(Color::srgb_u8(120, 120, 120)),
                    ResizableNodeDimension::Height(3.2),
                    children![(
                        Text("".to_string()),
                        text_font.clone(),
                        TextColor(Color::BLACK),
                        ResizableText::Paragraph,
                        children![(
                            TextSpan("|".to_string()),
                            TextCursor,
                            text_font.clone(),
                            TextColor(Color::NONE),
                            ResizableText::Paragraph,
                        )],
                    )],
                ), (
                    two_column_layout(
                         children![(
                            Text("Difficulty tag (Cannot be changed after upload):".to_string()),
                            bold_text_font.clone(),
                            TextColor(Color::BLACK),
                            TextLayout::new(Justify::Left, LineBreak::WordBoundary),
                            ResizableText::Paragraph,
                        ), (
                            Node {
                                flex_direction: FlexDirection::Column,
                                align_items: AlignItems::Start,
                                column_gap: px(4),
                                ..default()
                            },
                            RadioGroup,
                            TabIndex::default(),
                            children![(
                                radio(text_font.clone(), DifficultyTag::Easy, "Easy"),
                            ), (
                                radio(text_font.clone(), DifficultyTag::Medium, "Medium"),
                            ), (
                                radio(text_font.clone(), DifficultyTag::Hard, "Hard"),
                            ), (
                                radio(text_font.clone(), DifficultyTag::Demon, "Demon"),
                            )],
                            observe(
                                |entity_id: On<ValueChange<Entity>>,
                                mut difficulty_tag_resource: ResMut<DifficultyTag>,
                                value_query: Query<&DifficultyTag>| {
                                    if let Ok(value) = value_query.get(entity_id.value) {
                                        *difficulty_tag_resource = *value;
                                    }
                                },
                            ),
                        )],

                        children![(
                            Text("Gameplay tags (Cannot be changed after upload):".to_string()),
                            bold_text_font.clone(),
                            TextColor(Color::BLACK),
                            TextLayout::new(Justify::Left, LineBreak::WordBoundary),
                            ResizableText::Paragraph,
                        ), (
                            checkbox(text_font.clone(), GameplayTag::Fun, "Fun"),
                            observe(checkbox_self_update),
                        ), (
                            checkbox(text_font.clone(), GameplayTag::Tricky, "Tricky"),
                            observe(checkbox_self_update),
                        ), (
                            checkbox(text_font.clone(), GameplayTag::Weird, "Weird"),
                            observe(checkbox_self_update),
                        )],
                    ),
                )],
            ), (
                Node {
                    flex_direction: FlexDirection::Column,
                    row_gap: px(10),
                    ..default()
                },
                children![(
                    Node {
                        align_items: AlignItems::Center,
                        flex_direction: FlexDirection::Column,
                        justify_content: JustifyContent::Center,
                        ..default()
                    },
                    children![(
                        Text("By submitting this level pack, you agree to the".to_string()),
                        bold_text_font.clone(),
                        TextColor(crate::io::bevy_abstraction::Color::Black.into()),
                        TextLayout::new(Justify::Center, LineBreak::NoWrap),
                        ResizableText::Paragraph,
                    ), (
                        Node {
                            border: UiRect::bottom(px(5)),
                            box_sizing: BoxSizing::BorderBox,
                            ..default()
                        },
                        BorderColor::all(crate::io::bevy_abstraction::Color::LightBlue),
                        children![(
                            Text("workshop terms of service".to_string()),
                            bold_text_font.clone(),
                            Button,
                            LinkText,
                            Hovered::default(),
                            TabIndex::default(),
                            TextColor(crate::io::bevy_abstraction::Color::LightBlue.into()),
                            TextLayout::new(Justify::Center, LineBreak::NoWrap),
                            ResizableText::Paragraph,
                            observe(|_: On<Activate>, steam_client: Res<Client>| {
                                steam_client.friends().activate_game_overlay_to_web_page("steam://openurl/https://steamcommunity.com/sharedfiles/workshoplegalagreement");
                            }),
                        )],
                    )],
                ), (
                    two_column_layout(
                        children![(
                            Node {
                                width: percent(100),
                                border: UiRect::all(px(2)),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                ..default()
                            },
                            Button,
                            Hovered::default(),
                            TabIndex::default(),
                            BorderColor::all(crate::io::bevy_abstraction::Color::White),
                            BorderRadius::all(px(10)),
                            BackgroundColor(crate::io::bevy_abstraction::Color::Black.into()),
                            children![(
                                Text::new("Ok"),
                                text_font.clone(),
                                TextColor(crate::io::bevy_abstraction::Color::White.into()),
                                ResizableText::Paragraph,
                            )],
                            observe(|_: On<Activate>, mut validate_and_start_upload: MessageWriter<ValidateAndStartUpload>, mut play_sound_effect: MessageWriter<PlaySoundEffect>| {
                                if let SteamWorkshopUploadWorkingData::Idle = *STEAM_WORKSHOP_UPLOAD_WORKING_DATA.lock().unwrap() {
                                    play_sound_effect.write(PlaySoundEffect {
                                        sound_effect: audio::UI_SELECT_EFFECT,
                                    });

                                    validate_and_start_upload.write(ValidateAndStartUpload);
                                }
                            }),
                        )],

                        children![(
                            Node {
                                width: percent(100),
                                border: UiRect::all(px(2)),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                ..default()
                            },
                            Button,
                            Hovered::default(),
                            TabIndex::default(),
                            BorderColor::all(crate::io::bevy_abstraction::Color::White),
                            BorderRadius::all(px(10)),
                            BackgroundColor(crate::io::bevy_abstraction::Color::Black.into()),
                            children![(
                                Text::new("Cancel"),
                                text_font.clone(),
                                TextColor(crate::io::bevy_abstraction::Color::White.into()),
                                ResizableText::Paragraph,
                            )],
                            observe(|_: On<Activate>, mut app_state_next_state: ResMut<NextState<AppState>>, mut play_sound_effect: MessageWriter<PlaySoundEffect>| {
                                if let SteamWorkshopUploadWorkingData::Idle = *STEAM_WORKSHOP_UPLOAD_WORKING_DATA.lock().unwrap() {
                                    play_sound_effect.write(PlaySoundEffect {
                                        sound_effect: audio::UI_SELECT_EFFECT,
                                    });

                                    //TODO Fix input on click will be send to console wrapper directly after state change
                                    app_state_next_state.set(AppState::InGame);
                                }
                            }),
                        )],
                    ),
                )],
            )],
        )],
    ));
}

fn on_close_steam_workshop_upload_popup(
    mut commands: Commands,

    steam_workshop_upload_popup_elements: Query<Entity, With<SteamWorkshopUploadPopup>>,

    mut game: NonSendMut<Game>,
) {
    *STEAM_WORKSHOP_UPLOAD_WORKING_DATA.lock().unwrap() = SteamWorkshopUploadWorkingData::Idle;
    commands.remove_resource::<UpdateWatchHandleWrapper>();
    commands.remove_resource::<PreviousUpdateStatus>();

    for entity in steam_workshop_upload_popup_elements.iter() {
        commands.entity(entity).despawn();
    }

    game.game_state_mut().show_workshop_upload_popup = false;
}

fn on_set_upload_progress_title(
    mut set_upload_progress_popup_title_event: MessageReader<SetUploadProgressPopupTitle>,

    upload_progress_popup_title_text_query: Query<(&mut Text, &mut TextColor), With<UploadProgressPopupTitle>>,
) -> Result<(), Box<dyn Error>> {
    if let Some(event) = set_upload_progress_popup_title_event.read().next() {
        let Some((mut popup_text, mut popup_text_color)) = upload_progress_popup_title_text_query.into_iter().next() else {
            return Err(Box::new(GameError::new("Invalid popup status")));
        };

        popup_text.0 = event.title.clone();
        popup_text_color.0 = if event.error {
            crate::io::bevy_abstraction::Color::Red.into()
        }else {
            Color::BLACK
        };
    }

    Ok(())
}

fn on_set_upload_progress_content(
    mut set_upload_progress_popup_content_event: MessageReader<SetUploadProgressPopupContent>,

    upload_progress_popup_title_text_query: Query<(&mut Text, &mut TextColor), With<UploadProgressPopupContent>>,
) -> Result<(), Box<dyn Error>> {
    if let Some(event) = set_upload_progress_popup_content_event.read().next() {
        let Some((mut popup_text, mut popup_text_color)) = upload_progress_popup_title_text_query.into_iter().next() else {
            return Err(Box::new(GameError::new("Invalid popup status")));
        };

        popup_text.0 = event.text.clone();
        popup_text_color.0 = if event.error {
            crate::io::bevy_abstraction::Color::Red.into()
        }else {
            Color::BLACK
        };
    }

    Ok(())
}

fn close_upload_progress_popup(
    mut commands: Commands,

    upload_progress_popup_elements: Query<Entity, With<UploadProgressPopup>>,
) {
    for entity in upload_progress_popup_elements.iter() {
        commands.entity(entity).despawn();
    }
}

fn two_column_layout(left_hand_side_children: impl Bundle, right_hand_side_children: impl Bundle) -> impl Bundle {
    (
        Node {
            display: Display::Grid,
            width: percent(100),
            grid_template_columns: vec![GridTrack::fr(1.0), GridTrack::fr(1.0)],
            column_gap: px(20),
            ..default()
        },
        children![(
            Node {
                align_items: AlignItems::FlexStart,
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::FlexStart,
                ..default()
            },
            left_hand_side_children,
        ), (
            Node {
                align_items: AlignItems::FlexStart,
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::FlexStart,
                ..default()
            },
            right_hand_side_children,
        )],
    )
}

fn checkbox(text_font: TextFont, value: impl Component, label: &str) -> impl Bundle {
    (
        Node {
            flex_direction: FlexDirection::Row,
            justify_content: JustifyContent::FlexStart,
            align_items: AlignItems::Center,
            align_content: AlignContent::Center,
            ..default()
        },
        value,
        Checkbox,
        Hovered::default(),
        TabIndex::default(),
        children![(
            Node {
                ..default()
            },
            ResizableNodeDimension::Width(0.1),
        ), (
            Node {
                ..default()
            },
            children![(
                Node {
                    border: UiRect::all(percent(10)),
                    box_sizing: BoxSizing::BorderBox,
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                ResizableNodeDimension::Both(0.5, 0.5),
                BorderColor::all(Color::BLACK),
                BorderRadius::all(px(3)),
                children![(
                    Node {
                        position_type: PositionType::Absolute,
                        margin: UiRect::all(percent(10)),
                        border: UiRect::all(percent(20)),
                        ..default()
                    },
                    ResizableNodeDimension::Both(0.5, 0.5),
                    BackgroundColor(crate::io::bevy_abstraction::Color::Green.into()),
                )],
            )],
        ), (
            Node {
                ..default()
            },
            ResizableNodeDimension::Width(0.1),
        ), (
            Text::new(label),
            text_font,
            TextColor(Color::BLACK),
            ResizableText::Paragraph,
        )],
    )
}

fn radio(text_font: TextFont, value: impl Component, label: &str) -> impl Bundle {
    (
        Node {
            flex_direction: FlexDirection::Row,
            justify_content: JustifyContent::FlexStart,
            align_items: AlignItems::Center,
            align_content: AlignContent::Center,
            ..default()
        },
        value,
        RadioButton,
        Hovered::default(),
        children![(
            Node {
                ..default()
            },
            ResizableNodeDimension::Width(0.1),
        ), (
            Node {
                ..default()
            },
            children![(
                Node {
                    border: UiRect::all(percent(10)),
                    box_sizing: BoxSizing::BorderBox,
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                ResizableNodeDimension::Both(0.5, 0.5),
                BorderColor::all(Color::BLACK),
                BorderRadius::MAX,
                children![(
                    Node {
                        position_type: PositionType::Absolute,
                        margin: UiRect::all(percent(10)),
                        border: UiRect::all(percent(20)),
                        ..default()
                    },
                    ResizableNodeDimension::Both(0.5, 0.5),
                    BorderRadius::MAX,
                    BackgroundColor(crate::io::bevy_abstraction::Color::Green.into()),
                )],
            )],
        ), (
            Node {
                ..default()
            },
            ResizableNodeDimension::Width(0.1),
        ), (
            Text::new(label),
            text_font,
            TextColor(Color::BLACK),
            ResizableText::Paragraph,
        )],
    )
}
