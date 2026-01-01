use std::error::Error;
use std::process::ExitCode;
use std::time::Duration;
use bevy_app::prelude::*;
use bevy_app::ScheduleRunnerPlugin;
use bevy_ecs::prelude::*;
use bevy_time::prelude::*;
use bevy_time::TimePlugin;
use crate::game::Game;
use crate::io::Console;

#[cfg(unix)]
mod linux_terminal_helper;

pub fn run_game() -> ExitCode {
    #[cfg(unix)]
    if let Some(exit_code) = linux_terminal_helper::reopen_in_terminal_if_required() {
        return exit_code;
    }

    let console = Box::leak(Box::new(Console::new().unwrap()));

    let ret = run_game_internal(console);

    // Drop of Console must be called to restore the terminal mode
    //
    // SAFETY: The mutable reference &mut console is valid before calling "Box::from_raw()"
    // and is no longer used afterward
    drop(unsafe { Box::from_raw(console as *const _ as *mut Console) });

    ret.unwrap_or_else(|err| {
        // The terminal mode is restored in the Console's Drop implementation,
        // therefore it must be dropped before the error output is printed.

        eprintln!("{err}");

        ExitCode::FAILURE
    })
}

fn run_game_internal(console: &'static Console) -> Result<ExitCode, Box<dyn Error>> {
    let game = Game::new(console)?;

    let mut app = App::new();

    app.
            add_plugins(TaskPoolPlugin::default()).
            add_plugins(TimePlugin).
            add_plugins(ScheduleRunnerPlugin::run_loop(Duration::from_millis(40))).

            insert_resource(Time::<Fixed>::from_seconds(0.040)). //Run FixedUpdate every 40ms

            insert_non_send_resource(game).

            add_systems(FixedUpdate, update_game);

    let exit_code = app.run();
    let exit_code = match exit_code {
        AppExit::Success => ExitCode::SUCCESS,
        AppExit::Error(code) => ExitCode::from(code.get()),
    };

    Ok(exit_code)
}

fn update_game(
    mut game: NonSendMut<Game>,

    mut app_exit_event_writer: MessageWriter<AppExit>,
) {
    let should_stop = game.update();
    game.draw();

    if should_stop {
        app_exit_event_writer.write(AppExit::Success);
    }
}
