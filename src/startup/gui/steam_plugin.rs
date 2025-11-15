use std::collections::VecDeque;
use std::error::Error;
use std::sync::{Arc, LazyLock, Mutex};
use bevy::prelude::*;
use bevy_steamworks::*;
use crate::game::Game;
use crate::startup::gui::{on_resize, CharacterScaling};
use crate::startup::gui::steam_plugin::steam_workshop_upload_popup::SteamWorkshopUploadPopupPlugin;

mod steam_workshop_upload_popup;

pub struct SteamPlugin;

impl Plugin for SteamPlugin {
    fn build(&self, app: &mut App) {
        app.
                add_plugins(SteamWorkshopUploadPopupPlugin).

                add_message::<PlaySoundEffect>().

                add_systems(PostStartup, load_steam_workshop_items.pipe(handle_recoverable_error)).

                add_systems(Update, handle_workshop_item_loading_queue.pipe(handle_recoverable_error)).
                add_systems(Update, on_resize_popup_text.after(on_resize)).
                add_systems(Update, on_play_sound_effect);
    }
}

#[expect(clippy::type_complexity)]
static STEAM_WORKSHOP_ITEM_LOADING_QUEUE: LazyLock<
    Arc<Mutex<VecDeque<Result<QueryResult, SteamError>>>>,
    fn() -> Arc<Mutex<VecDeque<Result<QueryResult, SteamError>>>>,
> = LazyLock::new(Default::default);

#[derive(Debug, Component)]
enum ResizableText {
    Paragraph,
    Heading,
}

#[derive(Debug, Component)]
enum ResizableNodeDimension {
    Width(f32),
    Height(f32),
    Both(f32, f32),
}

#[derive(Debug, Message)]
struct PlaySoundEffect {
    sound_effect: &'static [u8],
}

fn handle_recoverable_error(
    In(result): In<Result<(), Box<dyn Error>>>,
) {
    let Err(err) = result else {
        return;
    };

    //TODO show popup with ok button
    error!("An error occurred: {err}");
}

fn on_resize_popup_text(
    character_scaling: Res<CharacterScaling>,

    resizable_text_query: Query<(&mut TextFont, &ResizableText), With<ResizableText>>,

    resizable_node_dimension_query: Query<(&mut Node, &ResizableNodeDimension), With<ResizableNodeDimension>>,
) {
    for (mut font, resizeable_text) in resizable_text_query {
        font.font_size = match resizeable_text {
            ResizableText::Paragraph => character_scaling.font_size * 0.9,
            ResizableText::Heading => character_scaling.font_size * 1.2,
        };
    }

    for (mut node, resizable_node_dimension) in resizable_node_dimension_query {
        match resizable_node_dimension {
            ResizableNodeDimension::Width(width) => node.width = px(width * character_scaling.font_size),
            ResizableNodeDimension::Height(height) => node.height = px(height * character_scaling.font_size),
            ResizableNodeDimension::Both(width, height) => {
                node.width = px(width * character_scaling.font_size);
                node.height = px(height * character_scaling.font_size);
            },
        }
    }
}

fn on_play_sound_effect(
    mut sound_effect_event: MessageReader<PlaySoundEffect>,

    game: NonSend<Game>,
) {
    for event in sound_effect_event.read() {
        game.game_state().play_sound_effect(event.sound_effect);
    }
}

fn load_steam_workshop_items(
    steam_client: Res<Client>,
) -> Result<(), Box<dyn Error>> {
    let subscribed_items = steam_client.ugc().subscribed_items(false);

    let mut to_be_loaded_level_pack_ids = Vec::new();

    for item_id in subscribed_items.iter() {
        let state = steam_client.ugc().item_state(*item_id);

        if state.contains(ItemState::NEEDS_UPDATE) {
            let _download_started_successfully = steam_client.ugc().download_item(*item_id, true);

            //TODO popup (if not successfully: show warning, that not all items are present)
        }else if state.contains(ItemState::DOWNLOADING) || state.contains(ItemState::DOWNLOAD_PENDING) {
            //TODO popup
        }else {
            to_be_loaded_level_pack_ids.push(*item_id);
        }

        //TODO register download listener and install listeners to install level packs [!!!CHECK APP ID!!!]
    }

    if !to_be_loaded_level_pack_ids.is_empty() {
        steam_client.ugc().query_items(to_be_loaded_level_pack_ids)?.fetch(|ret| {
            match ret {
                Ok(query_results) => {
                    for item in query_results.iter() {
                        if let Some(item) = item {
                            STEAM_WORKSHOP_ITEM_LOADING_QUEUE.lock().unwrap().push_back(Ok(item));
                        }else {
                            warn!("Invalid workshop item after query.");
                        }
                    }
                },

                Err(err) => STEAM_WORKSHOP_ITEM_LOADING_QUEUE.lock().unwrap().push_back(Err(err)),
            }
        });
    }

    Ok(())
}

fn handle_workshop_item_loading_queue(
    mut game: NonSendMut<Game>,
) -> Result<(), Box<dyn Error>> {
    let mut loading_queue = STEAM_WORKSHOP_ITEM_LOADING_QUEUE.lock().unwrap();
    while let Some(item) = loading_queue.pop_front() {
        let item = item?;

        game.load_steam_workshop_level_pack(item)?;
    }

    Ok(())
}
