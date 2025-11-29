# SokoTerm

A sokoban game for Linux terminal/TTY and with the Windows CMD and PowerShell.
There is also an GUI version available for Linux and Windows.

Download on itch.io: [SokoTerm](https://jddev0.itch.io/sokoterm)

## Gameplay
![image](https://github.com/user-attachments/assets/4772dc3d-3258-4bf9-8ca5-86df9296a852)

## Compile & Run

### Requirements
Linux:
- Required packages: `cmake`, `make`, `gcc`, `libncurses-dev`
- Rust compiler must be installed [Rust installation](https://www.rust-lang.org/tools/install)

Windows:
- Install cmake and add it to $PATH
- Install MinGW and add it to $PATH
- Rust compiler must be installed [Rust installation](https://www.rust-lang.org/tools/install)

### Compile & Run

CLI:

1. `cargo build --features cli`
2. `cargo run --features cli`

GUI:

1. `cargo build --features gui`
2. `cargo run --features gui`
