#![allow(clippy::uninlined_format_args)]

//Disable windows console window for GUI build
#![cfg_attr(feature = "gui", windows_subsystem = "windows")]

#[cfg(not(any(feature = "cli", feature = "gui")))]
compile_error!("Either feature `startup` or `gui` must be enabled");

#[cfg(all(feature = "cli", feature = "gui"))]
compile_error!("Features `cli` and `gui` are mutually exclusively");

use std::process::ExitCode;
use crate::startup::run_game;

pub mod game;
pub mod collections;
pub mod io;

mod startup;

fn main() -> ExitCode {
    run_game()
}
