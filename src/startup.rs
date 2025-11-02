#[cfg(feature = "cli")]
mod cli;
#[cfg(feature = "gui")]
mod gui;

#[cfg(feature = "cli")]
pub use cli::run_game;

#[cfg(feature = "gui")]
pub use gui::run_game;
