use crate::game::{audio, GameState};
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
struct Section {
    layer: SectionLayer,
    name: String,
    page: u32,
}

impl Section {
    pub fn new(layer: SectionLayer, name: impl Into<String>, page: u32) -> Self {
        Self { layer, name: name.into(), page }
    }

    pub fn draw(&self, console: &Console, width: usize) {
        console.set_color(self.layer.get_heading_color(), Color::Default);

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

        console.draw_text(format!("{}{}{}", heading, ".".repeat(width - heading_len - page_len), page));
    }
}

struct TableOfContents {
    next_section: u32,
    next_sub_section: u32,
    next_sub_sub_section: u32,

    sections: Vec<Section>,
}

impl TableOfContents {
    pub fn new() -> Self {
        Self {
            next_section: Default::default(),
            next_sub_section: Default::default(),
            next_sub_sub_section: Default::default(),

            sections: Vec::new(),
        }
    }

    pub fn add_section(&mut self, name: impl Into<String>, page: u32) {
        self.next_section += 1;
        self.next_sub_section = 0;
        self.next_sub_sub_section = 0;

        self.sections.push(Section::new(
            SectionLayer::Section(
                self.next_section
            ), name, page
        ));
    }

    pub fn add_sub_section(&mut self, name: impl Into<String>, page: u32) {
        self.next_sub_section += 1;
        self.next_sub_sub_section = 0;

        self.sections.push(Section::new(
            SectionLayer::SubSection(
                self.next_section,
                self.next_sub_section
            ), name, page
        ));
    }

    pub fn add_sub_sub_section(&mut self, name: impl Into<String>, page: u32) {
        self.next_sub_sub_section += 1;

        self.sections.push(Section::new(
            SectionLayer::SubSubSection(
                self.next_section,
                self.next_sub_section,
                self.next_sub_sub_section
            ), name, page
        ));
    }

    pub fn draw(&self, console: &Console, x: usize, y: usize, width: usize, height: usize, page: u32) {
        for (i, section) in self.sections.iter().
                skip(height * page as usize).
                take(height).
                enumerate() {
            console.set_cursor_pos(x, y + i);
            section.draw(console, width);
        }

        console.reset_color();
    }

    pub fn get_page_mouse_clicked(&self, height: usize, page: u32, row: u32) -> Option<u32> {
        self.sections.get(height * page as usize + row as usize).map(|section| section.page)
    }
}

pub struct HelpPage {
    table_of_contents: TableOfContents,

    page: u32,
}

impl HelpPage {
    const PAGE_COUNT: u32 = 10;

    pub fn new() -> Self {
        let mut table_of_contents = TableOfContents::new();
        table_of_contents.add_section("Controls", 2);
        table_of_contents.add_sub_section("Keyboard", 2);
        table_of_contents.add_sub_sub_section("Help menu", 2);
        table_of_contents.add_sub_sub_section("Exit window", 2);
        table_of_contents.add_sub_sub_section("Start menu", 2);
        table_of_contents.add_sub_sub_section("Game controls", 2);
        table_of_contents.add_sub_section("Mouse input", 3);
        table_of_contents.add_sub_sub_section("Help menu", 3);
        table_of_contents.add_sub_sub_section("Exit window", 3);
        table_of_contents.add_sub_sub_section("Start menu", 4);
        table_of_contents.add_section("Console arguments", 4);
        table_of_contents.add_section("Gameplay", 5);
        table_of_contents.add_sub_section("Game screen", 5);
        table_of_contents.add_section("Editor", 6);
        table_of_contents.add_sub_section("Controls", 6);
        table_of_contents.add_sub_sub_section("Level Pack selection", 6);
        table_of_contents.add_sub_sub_section("Level selection", 6);
        table_of_contents.add_sub_sub_section("Level editor (Playing mode)", 7);
        table_of_contents.add_sub_sub_section("Level editor (Editing mode)", 7);
        table_of_contents.add_sub_sub_section("Level editor (Editing mode - Tiles)", 8);
        table_of_contents.add_sub_sub_section("Level editor (Editing mode - Decoration Tiles)", 9);

        Self {
            table_of_contents,
            page: Default::default(),
        }
    }

    pub fn draw(&self, console: &Console, width: usize, height: usize) {
        console.set_color(Color::Yellow, Color::Default);
        console.set_underline(true);
        console.draw_text("Help menu");

        console.set_cursor_pos(0, 2);
        match self.page {
            page @ 0..=1 => {
                console.set_underline(false);
                self.table_of_contents.draw(console, 0, 2, width, height - 4, page);
            },
            2 => {
                console.set_color(Color::Blue, Color::Default);
                console.draw_text("1 Controls\n");

                console.set_color(Color::Green, Color::Default);
                console.draw_text("1.1 Keyboard\n");

                console.set_underline(false);
                console.set_color(Color::LightRed, Color::Default);
                console.draw_text("F1");
                console.reset_color();
                console.draw_text(": Open help menu");

                #[cfg(feature = "gui")] {
                    console.draw_text(", ");
                    console.set_color(Color::LightRed, Color::Default);
                    console.draw_text("F11");
                    console.reset_color();
                    console.draw_text(": Toggle Fullscreen");
                }

                console.set_underline(true);
                console.set_color(Color::Cyan, Color::Default);
                console.set_cursor_pos(0, 6);
                console.draw_text("1.1.1 Help menu\n");

                console.set_underline(false);
                console.set_color(Color::LightRed, Color::Default);
                console.draw_text("ESC");
                console.reset_color();
                console.draw_text("/");
                console.set_color(Color::LightRed, Color::Default);
                console.draw_text("F1");
                console.reset_color();
                console.draw_text(": Exit help menu\n");

                console.set_color(Color::LightRed, Color::Default);
                console.draw_text("UP");
                console.reset_color();
                console.draw_text("/");
                console.set_color(Color::LightRed, Color::Default);
                console.draw_text("DOWN");
                console.reset_color();
                console.draw_text(": Switch page");

                console.set_underline(true);
                console.set_color(Color::Cyan, Color::Default);
                console.set_cursor_pos(0, 10);
                console.draw_text("1.1.2 Exit window\n");

                console.set_underline(false);
                console.set_color(Color::LightRed, Color::Default);
                console.draw_text("y");
                console.reset_color();
                console.draw_text("/");
                console.set_color(Color::LightRed, Color::Default);
                console.draw_text("n");
                console.reset_color();
                console.draw_text(": Yes (Exit)/No (Not exit)");

                console.set_underline(true);
                console.set_color(Color::Cyan, Color::Default);
                console.set_cursor_pos(0, 13);
                console.draw_text("1.1.3 Start menu\n");

                console.set_underline(false);
                console.set_color(Color::LightRed, Color::Default);
                console.draw_text("ENTER");
                console.reset_color();
                console.draw_text(": Start game/Next Level\n");

                console.set_color(Color::LightRed, Color::Default);
                console.draw_text("ESC");
                console.reset_color();
                console.draw_text(": Exit window");

                console.set_underline(true);
                console.set_color(Color::Cyan, Color::Default);
                console.set_cursor_pos(0, 17);
                console.draw_text("1.1.4 Game controls\n");

                console.set_underline(false);
                console.set_color(Color::LightRed, Color::Default);
                console.draw_text("Arrow keys");
                console.reset_color();
                console.draw_text(": Move position\n");
                console.set_color(Color::LightRed, Color::Default);
                console.draw_text("r");
                console.reset_color();
                console.draw_text(": Reset level\n");
                console.set_color(Color::LightRed, Color::Default);
                console.draw_text("z");
                console.reset_color();
                console.draw_text(" / ");
                console.set_color(Color::LightRed, Color::Default);
                console.draw_text("y");
                console.reset_color();
                console.draw_text(": Undo / Redo");
            },
            3 => {
                console.set_color(Color::Green, Color::Default);
                console.draw_text("1.2 Mouse input\n");

                console.set_underline(false);
                console.reset_color();
                console.draw_text("Left click: [");
                console.set_color(Color::Default, Color::Yellow);
                console.draw_text("L");
                console.reset_color();
                console.draw_text("] \"Position\"\n");
                console.draw_text("Right click: [");
                console.set_color(Color::Default, Color::Yellow);
                console.draw_text("R");
                console.reset_color();
                console.draw_text("] \"Position\"\n");
                console.draw_text("Middle click: [");
                console.set_color(Color::Default, Color::Yellow);
                console.draw_text("M");
                console.reset_color();
                console.draw_text("] \"Position\"");

                console.set_underline(true);
                console.set_color(Color::Cyan, Color::Default);
                console.set_cursor_pos(0, 7);
                console.draw_text("1.2.1 Help menu\n");

                console.set_underline(false);
                console.reset_color();
                console.draw_text("[");
                console.set_color(Color::Default, Color::Yellow);
                console.draw_text("L");
                console.reset_color();
                console.draw_text("] \"Page: 00\": Switch page (The same as ");
                console.set_color(Color::LightRed, Color::Default);
                console.draw_text("DOWN");
                console.reset_color();
                console.draw_text(")\n[");
                console.set_color(Color::Default, Color::Yellow);
                console.draw_text("L");
                console.reset_color();
                console.draw_text("] Chapter at first pages: Goto page");

                console.set_underline(true);
                console.set_color(Color::Cyan, Color::Default);
                console.set_cursor_pos(0, 11);
                console.draw_text("1.2.2 Exit window\n");

                console.set_underline(false);
                console.reset_color();
                console.draw_text("[");
                console.set_color(Color::Default, Color::Yellow);
                console.draw_text("L");
                console.reset_color();
                console.draw_text("] \"[y]es\": Yes (The same as ");
                console.set_color(Color::LightRed, Color::Default);
                console.draw_text("y");
                console.reset_color();
                console.draw_text(")\n[");
                console.set_color(Color::Default, Color::Yellow);
                console.draw_text("L");
                console.reset_color();
                console.draw_text("] \"[n]o\": No (The same as ");
                console.set_color(Color::LightRed, Color::Default);
                console.draw_text("n");
                console.reset_color();
                console.draw_text(")");

                console.set_underline(true);
                console.set_color(Color::Cyan, Color::Default);
                console.set_cursor_pos(0, 15);
                console.draw_text("1.2.3 Start menu\n");

                console.set_underline(false);
                console.reset_color();
                console.draw_text("[");
                console.set_color(Color::Default, Color::Yellow);
                console.draw_text("L");
                console.reset_color();
                console.draw_text("] \"ENTER\": Start game (The same as ");
                console.set_color(Color::LightRed, Color::Default);
                console.draw_text("ENTER");
                console.reset_color();
                console.draw_text(")\n[");
                console.set_color(Color::Default, Color::Yellow);
                console.draw_text("L");
                console.reset_color();
                console.draw_text("] \"Help: F1\": Open help menu (The same as ");
                console.set_color(Color::LightRed, Color::Default);
                console.draw_text("F1");
                console.reset_color();
                console.draw_text(")");
            },
            4 => {
                console.set_color(Color::Blue, Color::Default);
                console.draw_text("2 Console arguments\n");

                console.set_underline(false);
                console.reset_color();
                console.draw_text("1) No arguments\n2) \"Path to level pack 1\" \"Path to level pack 2\" ...");
            },
            5 => {
                console.set_color(Color::Blue, Color::Default);
                console.draw_text("3 Gameplay\n");

                console.set_underline(false);

                console.reset_color();
                console.draw_text("Play the tutorial level pack for instructions.");

                console.set_underline(true);

                console.set_color(Color::Green, Color::Default);
                console.set_cursor_pos(0, 5);
                console.draw_text("3.1 Game screen\n");

                console.set_underline(false);

                console.reset_color();
                console.set_cursor_pos(1, 6);
                console.draw_text(
                    ": Empty\n       : One way doors\n : Wall\n : Player\n     : Box\n \
                    : Goal\n : Hole\n   : Key\n : Locked Door\n : Decoration"
                );

                console.set_color(Color::LightBlue, Color::Default);
                console.set_cursor_pos(0, 6);
                console.draw_text("-\n< ^ > v");
                console.set_color(Color::LightGreen, Color::Default);
                console.set_cursor_pos(0, 8);
                console.draw_text("#");
                console.set_color(Color::Yellow, Color::Default);
                console.set_cursor_pos(0, 9);
                console.draw_text("P");
                console.set_color(Color::LightCyan, Color::Default);
                console.set_cursor_pos(0, 10);
                console.draw_text("@");
                console.set_color(Color::Pink, Color::Default);
                console.set_cursor_pos(2, 10);
                console.draw_text("@");
                console.set_color(Color::Default, Color::LightBlue);
                console.set_cursor_pos(4, 10);
                console.draw_text("@");
                console.set_color(Color::LightRed, Color::Default);
                console.set_cursor_pos(0, 11);
                console.draw_text("x");
                console.set_color(Color::LightBlue, Color::Default);
                console.set_cursor_pos(0, 12);
                console.draw_text("O");
                console.set_color(Color::LightCyan, Color::Default);
                console.set_cursor_pos(0, 13);
                console.draw_text("*");
                console.set_color(Color::Pink, Color::Default);
                console.set_cursor_pos(2, 13);
                console.draw_text("*");
                console.set_color(Color::LightRed, Color::Default);
                console.set_cursor_pos(0, 14);
                console.draw_text("=");
                console.set_color(Color::LightBlue, Color::Default);
                console.set_cursor_pos(0, 15);
                console.draw_text(" ");
            },
            6 => {
                console.set_color(Color::Blue, Color::Default);
                console.draw_text("4 Editor\n");
                console.set_color(Color::Green, Color::Default);
                console.draw_text("4.1 Controls\n");
                console.set_color(Color::Cyan, Color::Default);
                console.draw_text("4.1.1 Level Pack selection\n");

                console.set_underline(false);

                console.set_cursor_pos(0, 5);
                console.set_color(Color::LightRed, Color::Default);
                console.draw_text("ENTER");
                console.reset_color();
                console.draw_text(": Selects or creates a level pack");

                console.set_cursor_pos(0, 6);
                console.set_color(Color::LightRed, Color::Default);
                console.draw_text("e");
                console.reset_color();
                console.draw_text(": Exports the selected level pack to the current directory");

                console.set_cursor_pos(0, 7);
                console.set_color(Color::LightRed, Color::Default);
                console.draw_text("DELETE");
                console.reset_color();
                console.draw_text(": Deletes the selected level pack");

                console.set_cursor_pos(0, 8);
                console.set_color(Color::LightRed, Color::Default);
                console.draw_text("ESC");
                console.reset_color();
                console.draw_text(": Cancels the creation of a new level pack");

                console.set_underline(true);

                console.set_cursor_pos(0, 10);
                console.set_color(Color::Cyan, Color::Default);
                console.draw_text("4.1.2 Level selection\n");

                console.set_underline(false);

                console.set_cursor_pos(0, 11);
                console.set_color(Color::LightRed, Color::Default);
                console.draw_text("ENTER");
                console.reset_color();
                console.draw_text(": Selects or create a level");

                console.set_cursor_pos(0, 12);
                console.set_color(Color::LightRed, Color::Default);
                console.draw_text("DELETE");
                console.reset_color();
                console.draw_text(": Deletes the selected level");

                console.set_cursor_pos(0, 13);
                console.set_color(Color::LightRed, Color::Default);
                console.draw_text("ESC");
                console.reset_color();
                console.draw_text(": Cancels the creation of a new level");
            },
            7 => {

                console.set_color(Color::Cyan, Color::Default);
                console.draw_text("4.1.3 Level editor (Playing mode)\n");

                console.set_underline(false);

                console.set_cursor_pos(0, 3);
                console.set_color(Color::LightRed, Color::Default);
                console.draw_text("Arrow keys");
                console.reset_color();
                console.draw_text(": Moves the player\n");

                console.set_cursor_pos(0, 4);
                console.set_color(Color::LightRed, Color::Default);
                console.draw_text("z");
                console.reset_color();
                console.draw_text(": Undo");

                console.set_cursor_pos(0, 5);
                console.set_color(Color::LightRed, Color::Default);
                console.draw_text("y");
                console.reset_color();
                console.draw_text(": Redo");

                console.set_cursor_pos(0, 6);
                console.set_color(Color::LightRed, Color::Default);
                console.draw_text("r");
                console.reset_color();
                console.draw_text(": Goes into the editing mode");

                console.set_underline(true);

                console.set_cursor_pos(0, 8);
                console.set_color(Color::Cyan, Color::Default);
                console.draw_text("4.1.4 Level editor (Editing mode)\n");

                console.set_underline(false);

                console.set_cursor_pos(0, 9);
                console.set_color(Color::LightRed, Color::Default);
                console.draw_text("Arrow keys");
                console.reset_color();
                console.draw_text(": Moves cursor position\n");

                console.set_cursor_pos(0, 10);
                console.set_color(Color::LightRed, Color::Default);
                console.draw_text("w a s d");
                console.reset_color();
                console.draw_text(": Sets the direction of the cursor");

                console.set_cursor_pos(0, 11);
                console.set_color(Color::LightRed, Color::Default);
                console.draw_text("i");
                console.reset_color();
                console.draw_text(": Inserts a row or column in the cursor direction");

                console.set_cursor_pos(0, 12);
                console.set_color(Color::LightRed, Color::Default);
                console.draw_text("c");
                console.reset_color();
                console.draw_text(": Copies the current row or column in the cursor direction");

                console.set_cursor_pos(0, 13);
                console.set_color(Color::LightRed, Color::Default);
                console.draw_text("z");
                console.reset_color();
                console.draw_text(": Undo");

                console.set_cursor_pos(0, 14);
                console.set_color(Color::LightRed, Color::Default);
                console.draw_text("y");
                console.reset_color();
                console.draw_text(": Redo");

                console.set_cursor_pos(0, 15);
                console.set_color(Color::LightRed, Color::Default);
                console.draw_text("r");
                console.reset_color();
                console.draw_text(": Goes into the playing mode");
            },
            8 => {
                console.set_color(Color::Cyan, Color::Default);
                console.draw_text("4.1.5 Level editor (Editing mode - Tiles)\n");

                console.set_underline(false);

                console.set_cursor_pos(0, 3);
                console.set_color(Color::LightRed, Color::Default);
                console.draw_text("SPACE");
                console.reset_color();
                console.draw_text(": Moves the cursor in cursor direction");

                console.set_cursor_pos(0, 4);
                console.set_color(Color::LightRed, Color::Default);
                console.draw_text("-");
                console.reset_color();
                console.draw_text(": Inserts an empty tile");

                console.set_cursor_pos(0, 5);
                console.set_color(Color::LightRed, Color::Default);
                console.draw_text("< ^ > v");
                console.reset_color();
                console.draw_text(": Inserts an one-way door tile");

                console.set_cursor_pos(0, 6);
                console.set_color(Color::LightRed, Color::Default);
                console.draw_text("#");
                console.reset_color();
                console.draw_text(": Inserts a wall tile");

                console.set_cursor_pos(0, 7);
                console.set_color(Color::LightRed, Color::Default);
                console.draw_text("p");
                console.reset_color();
                console.draw_text(": Inserts a player tile");

                console.set_cursor_pos(0, 8);
                console.set_color(Color::LightRed, Color::Default);
                console.draw_text("*");
                console.reset_color();
                console.draw_text(": Inserts a key tile");

                console.set_cursor_pos(0, 9);
                console.set_color(Color::LightRed, Color::Default);
                console.draw_text("~");
                console.reset_color();
                console.draw_text(": Inserts a key in goal tile");

                console.set_cursor_pos(0, 10);
                console.set_color(Color::LightRed, Color::Default);
                console.draw_text("=");
                console.reset_color();
                console.draw_text(": Inserts a closed door tile");

                console.set_cursor_pos(0, 11);
                console.set_color(Color::LightRed, Color::Default);
                console.draw_text("@");
                console.reset_color();
                console.draw_text(": Inserts a box tile");

                console.set_cursor_pos(0, 12);
                console.set_color(Color::LightRed, Color::Default);
                console.draw_text("+");
                console.reset_color();
                console.draw_text(": Inserts a box in goal tile");

                console.set_cursor_pos(0, 13);
                console.set_color(Color::LightRed, Color::Default);
                console.draw_text("x");
                console.reset_color();
                console.draw_text(": Inserts a goal tile");

                console.set_cursor_pos(0, 14);
                console.set_color(Color::LightRed, Color::Default);
                console.draw_text("o");
                console.reset_color();
                console.draw_text(": Inserts a hole tile");

                console.set_cursor_pos(0, 15);
                console.set_color(Color::LightRed, Color::Default);
                console.draw_text(".");
                console.reset_color();
                console.draw_text(": Inserts a box in hole tile");
            },
            9 => {
                console.set_color(Color::Cyan, Color::Default);
                console.draw_text("4.1.6 Level editor (Editing mode - Decoration Tiles)\n");

                console.set_underline(false);

                console.set_cursor_pos(0, 3);
                console.reset_color();
                console.draw_text("Decoration tiles act like wall tiles.");

                console.set_cursor_pos(0, 5);
                console.set_color(Color::LightRed, Color::Default);
                console.draw_text("b");
                console.reset_color();
                console.draw_text(": Inserts a blank decoration tile");
            },
            _ => {},
        }

        console.set_cursor_pos(0, height - 1);
        console.reset_color();
        console.draw_text("Page: ");
        console.set_color(Color::Cyan, Color::Default);
        console.draw_text(format!("{}", self.page + 1));
        console.reset_color();
        console.draw_text(" of ");
        console.set_color(Color::Cyan, Color::Default);
        console.draw_text(format!("{}", Self::PAGE_COUNT));
    }

    pub fn on_key_pressed(&mut self, game_state: &mut GameState, key: Key) {
        if key == Key::UP {
            game_state.play_sound_effect(audio::BOOK_FLIP_EFFECT);

            self.page = if self.page == 0 {
                Self::PAGE_COUNT - 1
            }else {
                self.page - 1
            };
        }else if key == Key::DOWN {
            game_state.play_sound_effect(audio::BOOK_FLIP_EFFECT);

            self.page = if self.page == Self::PAGE_COUNT - 1 {
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
