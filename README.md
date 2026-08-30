# SokoTerm Demo

This is the demo branch of SokoTerm. A puzzle game about pushing boxes.


Test your puzzle game with more than 150 brand-new puzzles across 5 built-in level packs with unique challenges which get
progressively harder. In the GUI version you can play this game with Graphical Tiles:
![Animated SokoTerm Gameplay - Pack 04 - Level 03 - Graphical](https://github.com/user-attachments/assets/3a5fbcbd-ac14-426f-a44a-71a5ef78e208)
or traditional ASCII tiles:
![Animated SokoTerm Gameplay - Pack 04 - Level 03 - ASCII](https://github.com/user-attachments/assets/264f4528-2a71-4db5-ae31-b6b393d9ad5a)

Test your puzzle solving skills with new gameplay mechanics like one-way doors and wraparound levels.
![SokoTerm Gameplay - Pack 03 - Level 20 - Graphical](https://github.com/user-attachments/assets/39f23102-d1da-431c-b4df-740238c4029a)
![SokoTerm Gameplay - Pack 03 - Level 34 - Graphical](https://github.com/user-attachments/assets/e58cbea9-7fa5-4e66-9374-94580b13a324)

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
