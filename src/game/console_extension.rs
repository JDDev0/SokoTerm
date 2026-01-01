use crate::game::level::Tile;
use crate::io::{Color, Console};

pub trait ConsoleExtension {
    fn draw_key_input_text(&self, input_text: &str);

    fn draw_tile(&self, tile: Tile, is_player_background: bool, inverted: bool);
}

impl<'a> ConsoleExtension for Console<'a> {
    fn draw_key_input_text(&self, input_text: &str) {
        self.set_color(Color::LightRed, Color::Default);
        self.draw_text(input_text);
    }

    #[cfg(feature = "cli")]
    fn draw_tile(&self, tile: Tile, is_player_background: bool, inverted: bool) {
        tile.draw_raw(self, is_player_background, inverted);
    }

    #[cfg(feature = "gui")]
    fn draw_tile(&self, tile: Tile, is_player_background: bool, inverted: bool) {
        self.draw_tile_internal(tile, is_player_background, inverted);
    }
}
