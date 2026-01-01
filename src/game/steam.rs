use std::sync::atomic::{AtomicBool, Ordering};
use bevy::prelude::*;
use bevy_steamworks::{AppId, CallbackResult, Client, SteamworksEvent};

pub mod stats;

pub const APP_ID: AppId = AppId(4254220);

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
