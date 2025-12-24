# SokoTerm Demo

This is the demo branch of SokoTerm. A Sokoban inspired puzzle game.

Test your puzzle game with more than 200 brand-new puzzles across 5 built-in level packs with unique challenges which get
progressively harder. In the GUI version you can play this game with Graphical Tiles:
![Animated SokoTerm Gameplay - Pack 04 - Level 03 - Graphical](https://github.com/user-attachments/assets/3a5fbcbd-ac14-426f-a44a-71a5ef78e208)
or traditional ASCII tiles:
![Animated SokoTerm Gameplay - Pack 04 - Level 03 - ASCII](https://github.com/user-attachments/assets/264f4528-2a71-4db5-ae31-b6b393d9ad5a)

Test your puzzle solving skills with new gameplay mechanics like one-way doors and wraparound levels.
![SokoTerm Gameplay - Pack 03 - Level 19 - Graphical](https://github.com/user-attachments/assets/75d1edea-4149-46b7-8434-f9b851412723)
![SokoTerm Gameplay - Pack 03 - Level 31 - Graphical](https://github.com/user-attachments/assets/e4baea50-800f-4a4f-b8d0-89d7027b7b5d)

Get it on Steam: [SokoTerm](https://store.steampowered.com/app/4160140/SokoTerm/)<br>
Get it on itch.io: [SokoTerm](https://jddev0.itch.io/sokoterm)

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
