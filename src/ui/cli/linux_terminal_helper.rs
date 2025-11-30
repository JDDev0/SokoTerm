use std::ffi::{OsStr, OsString};
use std::io::IsTerminal;
use std::iter;
use std::process::{Command, ExitCode};

const TERMINALS: [&str; 29] = [
    //Terminal selection taken from https://github.com/i3/i3/blob/next/i3-sensible-terminal (Public Domain)
    "x-terminal-emulator", "mate-terminal", "gnome-terminal", "terminator", "xfce4-terminal",
    "urxvt", "rxvt", "termit", "Eterm", "aterm", "uxterm", "xterm", "roxterm", "termite",
    "lxterminal", "terminology", "st", "qterminal", "lilyterm", "tilix", "terminix",
    "konsole", "kitty", "guake", "tilda", "alacritty", "hyper", "wezterm", "rio",
];

///Returns Some(ExitCode) if a terminal was opened and the program should exit
pub fn reopen_in_terminal_if_required() -> Option<ExitCode> {
    if std::io::stdout().is_terminal() {
        //No need to reopen in a terminal
        return None;
    }

    println!("Game is not running in a terminal: Trying to reopen in a terminal...");

    let current_exe = std::env::current_exe();
    let current_exe = match current_exe {
        Ok(current_exe) => current_exe,
        Err(err) => {
            eprintln!("An error occurred during reading of current exe: {err}!");

            return Some(ExitCode::FAILURE);
        },
    };

    let terminal_env = std::env::var_os("TERMINAL");

    let terminal_iter = iter::chain(
        terminal_env, TERMINALS.iter().map(OsString::from),
    );

    for terminal in terminal_iter {
        let terminal_exists = terminal_exists(&*terminal);
        match terminal_exists {
            Ok(terminal_exists) => {
                if !terminal_exists {
                    continue;
                }
            },

            Err(err) => {
                eprintln!("An error occurred during checking of presence of terminal \"{}\": {err}!", terminal.to_string_lossy());
                continue;
            },
        }

        let command_output = Command::new(&terminal).
                arg("-e").
                arg(&current_exe).
                output();

        match command_output {
            Ok(_command_output) => {
                return Some(ExitCode::SUCCESS);
            },

            Err(err) => {
                eprintln!("An error occurred during reopening in terminal \"{}\": {err}!", terminal.to_string_lossy());
            },
        }
    }

    eprintln!("No suitable terminal emulator was found: Please install one or specify it with the environment variable \"TERMINAL\".");

    Some(ExitCode::FAILURE)
}

fn terminal_exists<'a>(terminal: impl Into<&'a OsStr>) -> Result<bool, std::io::Error> {
    let mut command_v_terminal = OsString::new();
    command_v_terminal.push("command -v ");
    command_v_terminal.push(terminal.into());

    let command_output = Command::new("sh").
            arg("-c").
            arg(command_v_terminal).
            output()?;

    Ok(command_output.status.success())
}
