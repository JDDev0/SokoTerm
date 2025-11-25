#![allow(clippy::uninlined_format_args)]

//Disable windows console window for GUI build
#![cfg_attr(feature = "gui", windows_subsystem = "windows")]

#[cfg(not(any(feature = "cli", feature = "gui")))]
compile_error!("Either feature `cli` or `gui` must be enabled");

#[cfg(all(feature = "cli", feature = "gui"))]
compile_error!("Features `cli` and `gui` are mutually exclusively");

use std::process::ExitCode;
use ui::run_game;

pub mod game;
pub mod collections;
pub mod io;
pub mod utils;
mod ui;

fn main() -> ExitCode {
    run_game()
}
