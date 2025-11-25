#[cfg(feature = "cli")]
pub use console_lib::{Console, Key, Color};

#[cfg(feature = "gui")]
pub mod bevy_abstraction;
#[cfg(feature = "gui")]
pub use bevy_abstraction::{Console, Key, Color};
