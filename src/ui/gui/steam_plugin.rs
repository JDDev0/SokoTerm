use bevy::prelude::*;
use bevy_steamworks::*;
use crate::game::{steam, Game, GameError};
use crate::ui::gui::{on_resize, CharacterScaling};

#[cfg(unix)]
mod linux_steam_overlay_info_popup;

pub fn init(app: &mut App) -> Result<(), GameError> {
    let steamworks_plugin = SteamworksPlugin::init_app(steam::APP_ID);
    let steamworks_plugin = match steamworks_plugin {
        Ok(steamworks_plugin) => steamworks_plugin,
        Err(err) => {
            return Err(GameError::new(format!("Could not initialize Steam Client: {err}")));
        },
    };
    app.add_plugins(steamworks_plugin);

    Ok(())
}

pub struct SteamPlugin;

impl Plugin for SteamPlugin {
    fn build(&self, app: &mut App) {
        #[cfg(unix)]
        {
            app.add_plugins(linux_steam_overlay_info_popup::LinuxSteamOverlayInfoPlugin);
        }

        app.
                add_message::<PlaySoundEffect>().

                add_systems(Startup, steam::steam_init).

                add_systems(Update, steam::steam_callback).
                add_systems(Update, on_resize_popup_text.after(on_resize)).
                add_systems(Update, on_play_sound_effect);
    }
}

#[derive(Debug, Component)]
enum ResizableText {
    Paragraph,
    Heading,
}

#[derive(Debug, Message)]
struct PlaySoundEffect {
    sound_effect: &'static [u8],
}

fn on_resize_popup_text(
    character_scaling: Res<CharacterScaling>,

    resizable_text_query: Query<(&mut TextFont, &ResizableText), With<ResizableText>>,
) {
    for (mut font, resizeable_text) in resizable_text_query {
        font.font_size = match resizeable_text {
            ResizableText::Paragraph => character_scaling.font_size * 0.9,
            ResizableText::Heading => character_scaling.font_size * 1.2,
        };
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
