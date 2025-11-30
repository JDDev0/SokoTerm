use std::process::ExitCode;
use std::thread::sleep;
use std::time::Duration;
use crate::game::Game;
use crate::io::Console;

#[cfg(unix)]
mod linux_terminal_helper;

pub fn run_game() -> ExitCode {
    #[cfg(unix)]
    if let Some(exit_code) = linux_terminal_helper::reopen_in_terminal_if_required() {
        return exit_code;
    }

    let console = Console::new().unwrap();

    let game = Game::new(&console);
    let mut game = match game {
        Ok(game) => game,
        Err(err) => {
            drop(console);

            eprintln!("{err}");

            return ExitCode::FAILURE;
        },
    };

    loop {
        let should_stop = game.update();
        game.draw();

        if should_stop {
            return ExitCode::SUCCESS;
        }

        sleep(Duration::from_millis(40));
    }
}
