use std::sync::atomic::Ordering;
use bevy::prelude::*;
use bevy_steamworks::Client;
use crate::game::steam::USER_STATS_RECEIVED;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Stat {
    id: &'static str,
}

macro_rules! stat {
    ( $id:ident$(,)? ) => {
        pub const $id: Stat = Stat {
            id: stringify!($id),
        };
    };
}

impl Stat {
    stat! { MAX_COMPLETED_LEVEL }

    pub fn get(&self, steam_client: Client) -> i32 {
        if !USER_STATS_RECEIVED.load(Ordering::Relaxed) {
            error!("Steam stats were not received yet!");

            return -1;
        }

        steam_client.user_stats().get_stat_i32(self.id).unwrap_or(-1)
    }

    pub fn set(&self, steam_client: Client, val: i32) {
        info!("Steam stat set: {} => {}", self.id, val);

        if !USER_STATS_RECEIVED.load(Ordering::Relaxed) {
            error!("Steam stats were not received yet!");

            return;
        }

        let ret = steam_client.user_stats().set_stat_i32(self.id, val);
        if ret.is_err() {
            error!("Could not set stat \"{}\"!", self.id);

            return;
        }

        let ret = steam_client.user_stats().store_stats();
        if ret.is_err() {
            error!("Could not save stats!");
        }
    }
}
