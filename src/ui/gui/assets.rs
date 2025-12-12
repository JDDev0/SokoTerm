#[derive(Debug)]
pub struct Asset {
    path: &'static str,
    data: &'static [u8],
}

impl Asset {
    pub fn path(&self) -> &'static str {
        self.path
    }

    pub fn data(&self) -> &'static [u8] {
        self.data
    }
}

macro_rules! asset {
    ( $id:ident, $path:literal $(,)? ) => {
        pub const $id: crate::ui::gui::assets::Asset = crate::ui::gui::assets::Asset {
            path: $path,
            data: include_bytes!(concat!("../../../assets/", $path)),
        };
    };
}

pub mod font {
    asset! { JETBRAINS_MONO_BOLD_BYTES, "font/JetBrainsMono-Bold.ttf" }

    #[cfg(feature = "steam")]
    asset! { JETBRAINS_MONO_NL_EXTRA_LIGHT_BYTES, "font/JetBrainsMonoNL-ExtraLight.ttf" }
}

pub mod textures {
    pub mod tiles {
        asset! { EMPTY, "textures/tiles/empty.png" }
        asset! { FRAGILE_FLOOR, "textures/tiles/fragile_floor.png" }

        asset! { ICE, "textures/tiles/ice.png" }

        asset! { ONE_WAY_LEFT, "textures/tiles/one_way_left.png" }
        asset! { ONE_WAY_UP, "textures/tiles/one_way_up.png" }
        asset! { ONE_WAY_RIGHT, "textures/tiles/one_way_right.png" }
        asset! { ONE_WAY_DOWN, "textures/tiles/one_way_down.png" }

        asset! { WALL, "textures/tiles/wall.png" }

        asset! { KEY, "textures/tiles/key.png" }
        asset! { KEY_IN_GOAL, "textures/tiles/key_in_goal.png" }
        asset! { KEY_ON_FRAGILE_FLOOR, "textures/tiles/key_on_fragile_floor.png" }
        asset! { KEY_ON_ICE, "textures/tiles/key_on_ice.png" }
        asset! { LOCKED_DOOR, "textures/tiles/locked_door.png" }

        asset! { BOX, "textures/tiles/box.png" }
        asset! { BOX_IN_GOAL, "textures/tiles/box_in_goal.png" }
        asset! { GOAL, "textures/tiles/goal.png" }

        asset! { HOLE, "textures/tiles/hole.png" }
        asset! { BOX_IN_HOLE, "textures/tiles/box_in_hole.png" }
    }
}
