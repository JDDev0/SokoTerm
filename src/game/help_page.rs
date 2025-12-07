use std::collections::HashMap;
use crate::game::{audio, GameState};
use crate::game::console_extension::ConsoleExtension;
use crate::game::level::Tile;
use crate::io::{Color, Console, Key};

#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
enum SectionLayer {
    Section(u32),
    SubSection(u32, u32),
    SubSubSection(u32, u32, u32),
}

impl SectionLayer {
    pub fn get_heading_color(&self) -> Color {
        match self {
            Self::Section(..) => Color::Blue,
            Self::SubSection(..) => Color::Green,
            Self::SubSubSection(..) => Color::Cyan,
        }
    }
}

#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
struct SectionLabel {
    layer: SectionLayer,
    name: Box<str>,
    page: u32,
}

impl SectionLabel {
    pub fn new(layer: SectionLayer, name: &str, page: u32) -> Self {
        Self { layer, name: Box::from(name), page }
    }

    pub fn draw_page_entry(&self, console: &Console, width: usize) {
        let heading = match self.layer {
            SectionLayer::Section(section) => {
                format!("{} {}", section, self.name)
            },
            SectionLayer::SubSection(section, sub_section) => {
                format!("  {}.{} {}", section, sub_section, self.name)
            },
            SectionLayer::SubSubSection(section, sub_section, sub_sub_section) => {
                format!("      {}.{}.{} {}", section, sub_section, sub_sub_section, self.name)
            },
        };
        let heading_len = heading.chars().count();

        let page = (self.page + 1).to_string();
        let page_len = page.chars().count();

        console.set_color(self.layer.get_heading_color(), Color::Default);
        console.draw_text(format!("{}{}{}", heading, ".".repeat(width - heading_len - page_len), page));
    }

    pub fn draw_reference(&self, console: &Console) {
        let heading = match self.layer {
            SectionLayer::Section(section) => {
                format!("{} {}", section, self.name)
            },
            SectionLayer::SubSection(section, sub_section) => {
                format!("{}.{} {}", section, sub_section, self.name)
            },
            SectionLayer::SubSubSection(section, sub_section, sub_sub_section) => {
                format!("{}.{}.{} {}", section, sub_section, sub_sub_section, self.name)
            },
        };

        console.set_underline(true);
        console.set_color(self.layer.get_heading_color(), Color::Default);
        console.draw_text(heading);
        console.set_underline(false);
    }
}

#[derive(Debug, Clone)]
struct SectionLabelBuilder {
    next_section: u32,
    next_sub_section: u32,
    next_sub_sub_section: u32,
}

impl SectionLabelBuilder {
    pub fn new() -> Self {
        Self {
            next_section: Default::default(),
            next_sub_section: Default::default(),
            next_sub_sub_section: Default::default(),
        }
    }

    pub fn new_section(&mut self, name: &str) -> SectionLabel {
        self.next_section += 1;
        self.next_sub_section = 0;
        self.next_sub_sub_section = 0;

        SectionLabel::new(
            SectionLayer::Section(
                self.next_section
            ), name, 0
        )
    }

    pub fn new_sub_section(&mut self, name: &str) -> SectionLabel {
        self.next_sub_section += 1;
        self.next_sub_sub_section = 0;

        SectionLabel::new(
            SectionLayer::SubSection(
                self.next_section,
                self.next_sub_section
            ), name, 0
        )
    }

    pub fn new_sub_sub_section(&mut self, name: &str) -> SectionLabel {
        self.next_sub_sub_section += 1;

        SectionLabel::new(
            SectionLayer::SubSubSection(
                self.next_section,
                self.next_sub_section,
                self.next_sub_sub_section
            ), name, 0
        )
    }
}

#[derive(Debug, Clone)]
struct TableOfContents {
    sections: Box<[SectionLabel]>,
}

impl TableOfContents {
    pub fn new(sections: Box<[SectionLabel]>) -> Self {
        Self {
            sections,
        }
    }

    pub fn draw(&self, console: &Console, x: usize, y: usize, width: usize, height: usize, page: u32) {
        for (i, section) in self.sections.iter().
                skip(height * page as usize).
                take(height).
                enumerate() {
            console.set_cursor_pos(x, y + i);
            section.draw_page_entry(console, width);
        }

        console.reset_color();
    }

    pub fn page_count(&self, entries_per_page: usize) -> usize {
        if self.sections.is_empty() {
            0
        }else {
            (self.sections.len() - 1) / entries_per_page + 1
        }
    }

    pub fn get_page_mouse_clicked(&self, height: usize, page: u32, row: u32) -> Option<u32> {
        self.sections.get(height * page as usize + row as usize).map(|section| section.page)
    }
}

struct Section {
    section_label: SectionLabel,

    lines: Box<[fn(&Console)]>,
}

impl Section {
    pub fn new(section_label: SectionLabel, lines: &[fn(&Console)]) -> Self {
        Self { section_label, lines: Box::from(lines) }
    }

    pub fn draw(&self, console: &Console, x_offset: usize, y_offset: usize) {
        console.set_cursor_pos(x_offset, y_offset);
        self.section_label.draw_reference(console);

        for (i, line_draw_function) in self.lines.iter().enumerate() {
            console.set_cursor_pos(x_offset, y_offset + i + 1);
            line_draw_function(console);
        }
    }

    pub fn section_height(&self) -> usize {
        if self.lines.is_empty() {
            1
        }else {
            //If text is present: padding of 1 line
            2 + self.lines.len()
        }
    }
}

pub struct HelpPage {
    table_of_contents: TableOfContents,
    sections: Box<[Section]>,
    page_to_section_start_index_and_count: HashMap<u32, (usize, usize)>,

    width: usize,
    height: usize,

    page_count: u32,
    page: u32,
}

impl HelpPage {
    fn build_sections() -> Box<[Section]> {
        let mut section_label_builder = SectionLabelBuilder::new();

        fn empty_line(_: &Console) {}

        vec![
            Section::new(
                section_label_builder.new_section("Basic controls"), &[],
            ),
            Section::new(
                section_label_builder.new_sub_section("Keyboard"), &[|console| {
                    console.draw_key_input_text("F1");
                    console.reset_color();
                    console.draw_text(": Open/close help menu");
                }, |console| {
                    console.draw_key_input_text("F8");
                    console.reset_color();
                    console.draw_text(": Change animation speed");
                }, |console| {
                    console.draw_key_input_text("F9");
                    console.reset_color();
                    console.draw_text(": Enable/Disable background music");
                }, #[cfg(feature = "gui")] |console| {
                    console.draw_key_input_text("F10");
                    console.reset_color();
                    console.draw_text(": Cycle through color scheme");
                }, #[cfg(feature = "gui")] |console| {
                    console.draw_key_input_text("F11");
                    console.reset_color();
                    console.draw_text(": Toggle Fullscreen");
                }, empty_line, |console| {
                    console.draw_key_input_text("UP");
                    console.reset_color();
                    console.draw_text("/");
                    console.draw_key_input_text("DOWN");
                    console.reset_color();
                    console.draw_text(": Scroll up/down");
                }, |console| {
                    console.draw_key_input_text("ENTER");
                    console.reset_color();
                    console.draw_text("/");
                    console.draw_key_input_text("SPACEBAR");
                    console.reset_color();
                    console.draw_text(": Accept/Continue");
                }, |console| {
                    console.draw_key_input_text("ESC");
                    console.reset_color();
                    console.draw_text(": Cancel/Exit/Go back to last screen");
                }],
            ),
            Section::new(
                section_label_builder.new_sub_sub_section("Level (pack) selection"), &[|console| {
                    console.draw_key_input_text("Arrow keys");
                    console.reset_color();
                    console.draw_text(": Move level (pack) selection cursor");
                }, |console| {
                    console.draw_key_input_text("p");
                    console.reset_color();
                    console.draw_text(": Level preview");
                }],
            ),
            Section::new(
                section_label_builder.new_sub_sub_section("Game controls"), &[|console| {
                    console.draw_key_input_text("Arrow keys");
                    console.reset_color();
                    console.draw_text("/");
                    console.draw_key_input_text("WASD");
                    console.reset_color();
                    console.draw_text(": Move player");
                }, |console| {
                    console.draw_key_input_text("r");
                    console.reset_color();
                    console.draw_text(": Reset level");
                }, |console| {
                    console.draw_key_input_text("z");
                    console.reset_color();
                    console.draw_text("/");
                    console.draw_key_input_text("u");
                    console.reset_color();
                    console.draw_text(": Undo");
                }, |console| {
                    console.draw_key_input_text("y");
                    console.reset_color();
                    console.draw_text(": Redo");
                }, |console| {
                    console.draw_key_input_text("q");
                    console.reset_color();
                    console.draw_text(": Show/Hide floor tiles");
                }],
            ),

            Section::new(
                section_label_builder.new_sub_section("Mouse input"), &[|console| {
                    console.reset_color();
                    console.draw_text("Left click: [");
                    console.set_color(Color::Default, Color::Yellow);
                    console.draw_text("L");
                    console.reset_color();
                    console.draw_text("] \"Position\"");
                }, |console| {
                    console.reset_color();
                    console.draw_text("Right click: [");
                    console.set_color(Color::Default, Color::Yellow);
                    console.draw_text("R");
                    console.reset_color();
                    console.draw_text("] \"Position\"");
                }, |console| {
                    console.draw_text("Middle click: [");
                    console.set_color(Color::Default, Color::Yellow);
                    console.draw_text("M");
                    console.reset_color();
                    console.draw_text("] \"Position\"");
                }, empty_line, |console| {
                    console.reset_color();
                    console.draw_text("[");
                    console.set_color(Color::Default, Color::Yellow);
                    console.draw_text("L");
                    console.reset_color();
                    console.draw_text("] Almost anywhere with ");
                    console.draw_key_input_text("key input label");
                    console.reset_color();
                    console.draw_text(" text");
                }],
            ),
            Section::new(
                section_label_builder.new_sub_sub_section("Level (pack) selection"), &[|console| {
                    console.reset_color();
                    console.draw_text("[");
                    console.set_color(Color::Default, Color::Yellow);
                    console.draw_text("L");
                    console.reset_color();
                    console.draw_text("] Level pack selection number tiles");
                }, |console| {
                    console.reset_color();
                    console.draw_text("[");
                    console.set_color(Color::Default, Color::Yellow);
                    console.draw_text("L");
                    console.reset_color();
                    console.draw_text("] Level selection number tiles");
                }, |console| {
                    console.reset_color();
                    console.draw_text("[");
                    console.set_color(Color::Default, Color::Yellow);
                    console.draw_text("L");
                    console.reset_color();
                    console.draw_text("] Level pack editor selection number tiles");
                }, |console| {
                    console.reset_color();
                    console.draw_text("[");
                    console.set_color(Color::Default, Color::Yellow);
                    console.draw_text("L");
                    console.reset_color();
                    console.draw_text("] Level editor selection number tiles");
                }, |console| {
                    console.reset_color();
                    console.draw_text("[");
                    console.set_color(Color::Default, Color::Yellow);
                    console.draw_text("L");
                    console.reset_color();
                    console.draw_text("] Level pack editor background music selection");
                }],
            ),
            Section::new(
                section_label_builder.new_sub_sub_section("Help menu"), &[|console| {
                    console.reset_color();
                    console.draw_text("[");
                    console.set_color(Color::Default, Color::Yellow);
                    console.draw_text("L");
                    console.reset_color();
                    console.draw_text("] \"Page: 00\": Switch page (The same as ");
                    console.draw_key_input_text("DOWN");
                    console.reset_color();
                    console.draw_text(")");
                }, |console| {
                    console.reset_color();
                    console.draw_text("[");
                    console.set_color(Color::Default, Color::Yellow);
                    console.draw_text("L");
                    console.reset_color();
                    console.draw_text("] Table of contents");
                }],
            ),

            Section::new(
                section_label_builder.new_section("Gameplay"), &[|console| {
                    console.reset_color();
                    console.draw_text("Play the tutorial level pack for instructions.");
                }],
            ),
            Section::new(
                section_label_builder.new_sub_section("Gameplay elements"), &[|console| {
                    Tile::Empty.draw(console, false, false);

                    console.reset_color();
                    console.draw_text(": Empty");
                }, |console| {
                    Tile::FragileFloor.draw(console, false, false);

                    console.reset_color();
                    console.draw_text(": Fragile Floor");
                }, |console| {
                    Tile::Ice.draw(console, false, false);

                    console.reset_color();
                    console.draw_text(": Ice");
                }, |console| {
                    Tile::OneWayLeft.draw(console, false, false);

                    console.reset_color();
                    console.draw_text(" ");

                    Tile::OneWayUp.draw(console, false, false);

                    console.reset_color();
                    console.draw_text(" ");

                    Tile::OneWayRight.draw(console, false, false);

                    console.reset_color();
                    console.draw_text(" ");

                    Tile::OneWayDown.draw(console, false, false);

                    console.reset_color();
                    console.draw_text(": One-way doors");
                }, |console| {
                    Tile::Wall.draw(console, false, false);

                    console.reset_color();
                    console.draw_text(": Wall");
                }, |console| {
                    Tile::Player.draw(console, false, false);

                    console.reset_color();
                    console.draw_text(": Player");
                }, |console| {
                    Tile::Box.draw(console, false, false);

                    console.reset_color();
                    console.draw_text(" ");

                    Tile::BoxInGoal.draw(console, false, false);

                    console.reset_color();
                    console.draw_text(" ");

                    Tile::BoxInHole.draw(console, false, false);

                    console.reset_color();
                    console.draw_text(": Box");
                }, |console| {
                    Tile::Goal.draw(console, false, false);

                    console.reset_color();
                    console.draw_text(": Goal");
                }, |console| {
                    Tile::Hole.draw(console, false, false);

                    console.reset_color();
                    console.draw_text(": Hole");
                }, |console| {
                    Tile::Key.draw(console, false, false);

                    console.reset_color();
                    console.draw_text(" ");

                    Tile::KeyInGoal.draw(console, false, false);

                    console.reset_color();
                    console.draw_text(": Key");
                }, |console| {
                    Tile::LockedDoor.draw(console, false, false);

                    console.reset_color();
                    console.draw_text(": Locked Door");
                }, |console| {
                    Tile::DecorationBlank.draw(console, false, false);

                    console.reset_color();
                    console.draw_text(": Decoration");
                }],
            ),

            Section::new(
                section_label_builder.new_section("Editor"), &[],
            ),
            Section::new(
                section_label_builder.new_sub_section("Controls"), &[],
            ),
            Section::new(
                section_label_builder.new_sub_sub_section("Level pack selection"), &[|console| {
                    console.draw_key_input_text("ENTER");
                    console.reset_color();
                    console.draw_text("/");
                    console.draw_key_input_text("SPACEBAR");
                    console.reset_color();
                    console.draw_text(": Select or create a level pack");
                }, |console| {
                    console.draw_key_input_text("s");
                    console.reset_color();
                    console.draw_text(": Select the background music for the selected level pack");
                }, |console| {
                    console.draw_key_input_text("e");
                    console.reset_color();
                    console.draw_text(": Export the selected level pack to the current directory");
                }, #[cfg(feature = "steam")] |console| {
                    console.draw_key_input_text("u");
                    console.reset_color();
                    console.draw_text(": Upload the selected level pack to the steam workshop");
                }, |console| {
                    console.draw_key_input_text("DELETE");
                    console.reset_color();
                    console.draw_text(": Delete the selected level pack");
                }, |console| {
                    console.draw_key_input_text("ESC");
                    console.reset_color();
                    console.draw_text(": Cancel the creation of a new level pack");
                }],
            ),
            Section::new(
                section_label_builder.new_sub_sub_section("Level pack editor / Level selection"), &[|console| {
                    console.draw_key_input_text("ENTER");
                    console.reset_color();
                    console.draw_text("/");
                    console.draw_key_input_text("SPACEBAR");
                    console.reset_color();
                    console.draw_text(": Select or creates a level");
                }, |console| {
                    console.draw_key_input_text("t");
                    console.reset_color();
                    console.draw_text(": Set or unset level as level pack thumbnail");
                }, |console| {
                    console.draw_key_input_text("c");
                    console.reset_color();
                    console.draw_text(": Copy the selected level");
                }, |console| {
                    console.draw_key_input_text("x");
                    console.reset_color();
                    console.draw_text(": Cut the selected level");
                }, |console| {
                    console.draw_key_input_text("v");
                    console.reset_color();
                    console.draw_text(": Paste the copied level at the current cursor position");
                }, |console| {
                    console.reset_color();
                    console.draw_text("   (Levels can also be pasted from one level pack to another level pack)");
                }, |console| {
                    console.draw_key_input_text("DELETE");
                    console.reset_color();
                    console.draw_text(": Delete the selected level");
                }, |console| {
                    console.draw_key_input_text("ESC");
                    console.reset_color();
                    console.draw_text(": Cancel the creation of a new level");
                }],
            ),
            Section::new(
                section_label_builder.new_sub_sub_section("Level editor (Playing mode)"), &[|console| {
                    console.draw_key_input_text("Arrow keys");
                    console.reset_color();
                    console.draw_text("/");
                    console.draw_key_input_text("WASD");
                    console.reset_color();
                    console.draw_text(": Move player");
                }, |console| {
                    console.draw_key_input_text("z");
                    console.reset_color();
                    console.draw_text("/");
                    console.draw_key_input_text("u");
                    console.reset_color();
                    console.draw_text(": Undo");
                }, |console| {
                    console.draw_key_input_text("y");
                    console.reset_color();
                    console.draw_text(": Redo");
                }, |console| {
                    console.draw_key_input_text("r");
                    console.reset_color();
                    console.draw_text(": Switch to editing mode");
                }, |console| {
                    console.draw_key_input_text("q");
                    console.reset_color();
                    console.draw_text(": Show/Hide floor tiles");
                }],
            ),
            Section::new(
                section_label_builder.new_sub_sub_section("Level editor (Editing mode)"), &[|console| {
                    console.draw_key_input_text("ESC");
                    console.reset_color();
                    console.draw_text(": Save and exit editor");
                }, |console| {
                    console.draw_key_input_text("ENTER");
                    console.reset_color();
                    console.draw_text(": Save without exiting editor");
                }, |console| {
                    console.draw_key_input_text("Arrow keys");
                    console.reset_color();
                    console.draw_text(": Move cursor");
                }, |console| {
                    console.draw_key_input_text("w a s d");
                    console.reset_color();
                    console.draw_text(": Set the direction of the cursor");
                }, |console| {
                    console.draw_key_input_text("i");
                    console.reset_color();
                    console.draw_text(": Insert a row or column in the cursor direction");
                }, |console| {
                    console.draw_key_input_text("c");
                    console.reset_color();
                    console.draw_text(": Copy the current row or column in the cursor direction");
                }, |console| {
                    console.draw_key_input_text("z");
                    console.reset_color();
                    console.draw_text("/");
                    console.draw_key_input_text("u");
                    console.reset_color();
                    console.draw_text(": Undo");
                }, |console| {
                    console.draw_key_input_text("y");
                    console.reset_color();
                    console.draw_text(": Redo");
                }, |console| {
                    console.draw_key_input_text("r");
                    console.reset_color();
                    console.draw_text(": Switch to playing mode");
                }, |console| {
                    console.draw_key_input_text("q");
                    console.reset_color();
                    console.draw_text(": Show/Hide floor tiles");
                }, empty_line, |console| {
                    console.reset_color();
                    console.draw_text("[");
                    console.set_color(Color::Default, Color::Yellow);
                    console.draw_text("L");
                    console.reset_color();
                    console.draw_text("] Press on any tile to set the cursor position");
                }],
            ),
            Section::new(
                section_label_builder.new_sub_sub_section("Level editor (Editing mode - Tiles)"), &[|console| {
                    console.draw_key_input_text("SPACEBAR");
                    console.reset_color();
                    console.draw_text(": Move the cursor in cursor direction");
                }, |console| {
                    console.draw_key_input_text("-");
                    console.reset_color();
                    console.draw_text(": Insert an empty tile");
                }, |console| {
                    console.draw_key_input_text("~");
                    console.reset_color();
                    console.draw_text(": Insert a fragile floor tile");
                }, |console| {
                    console.draw_key_input_text("%");
                    console.reset_color();
                    console.draw_text(": Insert an ice tile");
                }, |console| {
                    console.draw_key_input_text("< ^ > v");
                    console.reset_color();
                    console.draw_text(": Insert an one-way door tile");
                }, |console| {
                    console.draw_key_input_text("#");
                    console.reset_color();
                    console.draw_text(": Insert a wall tile");
                }, |console| {
                    console.draw_key_input_text("p");
                    console.reset_color();
                    console.draw_text(": Insert a player tile");
                }, |console| {
                    console.draw_key_input_text(",");
                    console.reset_color();
                    console.draw_text(": Insert a player on fragile floor tile");
                }, |console| {
                    console.draw_key_input_text("&");
                    console.reset_color();
                    console.draw_text(": Insert a player on ice tile");
                }, |console| {
                    console.draw_key_input_text("*");
                    console.reset_color();
                    console.draw_text(": Insert a key tile");
                }, |console| {
                    console.draw_key_input_text(":");
                    console.reset_color();
                    console.draw_text(": Insert a key in goal tile");
                }, |console| {
                    console.draw_key_input_text(";");
                    console.reset_color();
                    console.draw_text(": Insert a key on fragile floor tile");
                }, |console| {
                    console.draw_key_input_text("\\");
                    console.reset_color();
                    console.draw_text(": Insert a key on ice tile");
                }, |console| {
                    console.draw_key_input_text("=");
                    console.reset_color();
                    console.draw_text(": Insert a locked door tile");
                }, |console| {
                    console.draw_key_input_text("@");
                    console.reset_color();
                    console.draw_text(": Insert a box tile");
                }, |console| {
                    console.draw_key_input_text("+");
                    console.reset_color();
                    console.draw_text(": Insert a box in goal tile");
                }, |console| {
                    console.draw_key_input_text("!");
                    console.reset_color();
                    console.draw_text(": Insert a box on fragile floor tile");
                }, |console| {
                    console.draw_key_input_text("/");
                    console.reset_color();
                    console.draw_text(": Insert a box on ice tile");
                }],
            ),
            Section::new(
                section_label_builder.new_sub_sub_section("Level editor (Editing mode - Tiles) [continuation]"), &[|console| {
                    console.draw_key_input_text("x");
                    console.reset_color();
                    console.draw_text(": Insert a goal tile");
                }, |console| {
                    console.draw_key_input_text("o");
                    console.reset_color();
                    console.draw_text(": Insert a hole tile");
                }, |console| {
                    console.draw_key_input_text(".");
                    console.reset_color();
                    console.draw_text(": Insert a box in hole tile");
                }],
            ),
            Section::new(
                section_label_builder.new_sub_sub_section("Level editor (Editing mode - Decoration tiles)"), &[|console| {
                    console.reset_color();
                    console.draw_text("Decoration tiles act like wall tiles.");
                }, empty_line, |console| {
                    console.draw_key_input_text("b");
                    console.reset_color();
                    console.draw_text(": Insert a blank decoration tile");
                }],
            ),

            Section::new(
                section_label_builder.new_section("Command-line arguments"), &[|console| {
                    console.reset_color();
                    console.draw_text("1) No arguments");
                }, |console| {
                    console.reset_color();
                    console.draw_text("2) \"Path to level pack 1\" \"Path to level pack 2\" ...");
                }],
            ),
        ].into_boxed_slice()
    }

    pub fn new(width: usize, height: usize) -> Self {
        let mut sections = Self::build_sections();

        let page_height = height - 4;
        let table_of_contents_page_count = (sections.len() - 1) / page_height + 1;

        let mut page_to_section_start_index_and_count = HashMap::new();
        let mut current_page = table_of_contents_page_count as u32 - 1;
        let mut start_index = 0;
        let mut section_count = 0;
        let mut content_height = 0;

        for (i, section) in sections.iter_mut().enumerate() {
            let section_height = section.section_height();
            if matches!(section.section_label.layer, SectionLayer::Section(..)) || content_height + section_height > page_height {
                page_to_section_start_index_and_count.insert(current_page, (start_index, section_count));

                current_page += 1;
                start_index = i;
                section_count = 0;
                content_height = 0;
            }

            section.section_label.page = current_page;
            section_count += 1;
            content_height += section_height;
        }

        //Insert last page to section mapping
        page_to_section_start_index_and_count.insert(current_page, (start_index, section_count));

        let table_of_contents = TableOfContents::new(sections.iter().
                map(|section| &section.section_label).
                cloned().
                collect::<Box<[_]>>());

        Self {
            table_of_contents,
            sections,
            page_to_section_start_index_and_count,

            width, height,

            page_count: current_page + 1,
            page: Default::default(),
        }
    }

    pub fn draw(&self, console: &Console) {
        console.set_color(Color::Yellow, Color::Default);
        console.set_underline(true);
        console.draw_text("Help menu");
        console.set_underline(false);

        if (self.page as usize) < self.table_of_contents.page_count(self.height - 4) {
            self.table_of_contents.draw(console, 0, 2, self.width, self.height - 4, self.page);
        }else {
            let mut row = 2;

            let (start_index, count) = self.page_to_section_start_index_and_count[&self.page];

            for section in self.sections.iter().
                    skip(start_index).
                    take(count) {
                section.draw(console, 0, row);

                row += section.section_height();
            }
        }

        console.set_cursor_pos(0, self.height - 1);
        console.reset_color();
        console.draw_text("Page: ");
        console.set_color(Color::Cyan, Color::Default);
        console.draw_text(format!("{}", self.page + 1));
        console.reset_color();
        console.draw_text(" of ");
        console.set_color(Color::Cyan, Color::Default);
        console.draw_text(format!("{}", self.page_count));
    }

    pub fn on_key_pressed(&mut self, game_state: &mut GameState, key: Key) {
        if key == Key::UP {
            game_state.play_sound_effect(audio::BOOK_FLIP_EFFECT);

            self.page = if self.page == 0 {
                self.page_count - 1
            }else {
                self.page - 1
            };
        }else if key == Key::DOWN {
            game_state.play_sound_effect(audio::BOOK_FLIP_EFFECT);

            self.page = if self.page == self.page_count - 1 {
                0
            }else {
                self.page + 1
            };
        }
    }

    pub fn on_mouse_pressed(&mut self, _width: usize, height: usize, game_state: &mut GameState, column: usize, row: usize) {
        if row >= 2 && row < height - 2 &&  let Some(page_clicked) = self.table_of_contents.get_page_mouse_clicked(height, self.page, row as u32 - 2) {
            game_state.play_sound_effect(audio::BOOK_FLIP_EFFECT);

            self.page = page_clicked;
        }

        if row == height - 1 && column < 8 {
            self.on_key_pressed(game_state, Key::DOWN);
        }
    }
}
