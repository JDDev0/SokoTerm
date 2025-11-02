#[cfg(feature = "gui")]
pub mod bevy_abstraction;

#[cfg(feature = "cli")]
pub type Console<'a> = console_lib::Console<'a>;
#[cfg(feature = "cli")]
pub type Key = console_lib::Key;
#[cfg(feature = "cli")]
pub type Color = console_lib::Color;

#[cfg(feature = "gui")]
pub type Console<'a> = bevy_abstraction::Console<'a>;
#[cfg(feature = "gui")]
pub type Key = bevy_abstraction::Key;
#[cfg(feature = "gui")]
pub type Color = bevy_abstraction::Color;
