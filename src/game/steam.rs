use std::error::Error;
use std::sync::atomic::{AtomicBool, Ordering};
use bevy::prelude::*;
use bevy_steamworks::{AppId, CallbackResult, Client, FileType, PublishedFileId, SteamError, SteamworksEvent};
use crate::game::Game;
use crate::game::level::LevelPack;

pub mod achievement;

pub const APP_ID: AppId = AppId(4160140);

static USER_STATS_RECEIVED: AtomicBool = AtomicBool::new(false);

pub fn steam_init(
    steam_client: Res<Client>,
) {
    steam_client.user_stats().request_user_stats(steam_client.user().steam_id().raw());
}

pub fn steam_callback(
    mut steamworks_event: MessageReader<SteamworksEvent>,
) {
    for event in steamworks_event.read() {
        let SteamworksEvent::CallbackResult(event) = event;

        info!("Received steam event: {event:?}");

        #[expect(clippy::single_match)]
        match event {
            CallbackResult::UserStatsReceived(user_stats_received) => {
                match user_stats_received.result {
                    Ok(_) => {
                        USER_STATS_RECEIVED.store(true, Ordering::Relaxed);
                    },
                    Err(err) => {
                        error!("{err}");
                    },
                }
            }

            _ => {},
        }
    }
}

pub fn prepare_workshop_upload_temp_data(level_pack: &LevelPack) -> Result<(), Box<dyn Error>> {
    let mut tmp_upload_path = Game::get_or_create_save_game_folder()?;
    tmp_upload_path.push("SteamWorkshop/UploadTemp");

    if std::fs::exists(&tmp_upload_path)? {
        std::fs::remove_dir_all(&tmp_upload_path)?;
    }

    tmp_upload_path.push("/Data");
    std::fs::create_dir_all(&tmp_upload_path)?;

    tmp_upload_path.push("/");

    tmp_upload_path.push("pack.lvl");

    level_pack.export_editor_level_pack_to_path(tmp_upload_path)?;

    Ok(())
}

pub fn crate_workshop_item<F>(
    steam_client: Client,
    callback: F,
) where F: FnOnce(std::result::Result<(PublishedFileId, bool), SteamError>) + 'static + Send {
    steam_client.ugc().create_item(APP_ID, FileType::Community, callback);
}
