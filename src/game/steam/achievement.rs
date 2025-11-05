use std::sync::atomic::Ordering;
use bevy::prelude::*;
use bevy_steamworks::Client;
use crate::game::steam::USER_STATS_RECEIVED;

macro_rules! achievement {
    ( $id:ident$(,)? ) => {
        pub const $id: Achievement = Achievement {
            id: stringify!($id),
        };
    };
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Achievement {
    id: &'static str,
}

impl Achievement {
    achievement! { LEVEL_PACK_TUTORIAL_COMPLETED }
    achievement! { LEVEL_PACK_MAIN_COMPLETED }
    achievement! { LEVEL_PACK_SPECIAL_COMPLETED }
    achievement! { LEVEL_PACK_DEMON_COMPLETED }
    achievement! { LEVEL_PACK_SECRET_COMPLETED }
    achievement! { LEVEL_PACK_TUTORIAL_FAST }
    achievement! { LEVEL_PACK_SECRET_DISCOVERED }
    achievement! { LEVEL_PACK_MAIN_LEVEL_96_COMPLETED }
    achievement! { STEAM_WORKSHOP_LEVEL_PACK_PLAYED }
    achievement! { STEAM_WORKSHOP_LEVEL_PACK_COMPLETED }
    achievement! { STEAM_WORKSHOP_LEVEL_PACK_CREATED }

    pub fn unlock(&self, steam_client: Client) {
        if !USER_STATS_RECEIVED.load(Ordering::Relaxed) {
            error!("Steam stats were not received yet!");

            return;
        }

        let ret = steam_client.user_stats().achievement(self.id).set();
        if ret.is_err() {
            error!("Could not set achievement \"{}\"!", self.id);

            return;
        }

        let ret = steam_client.user_stats().store_stats();
        if ret.is_err() {
            error!("Could not save achievement progress!");
        }
    }
}
