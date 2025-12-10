#![allow(unused)]

use crate::game::GameState;
use crate::io::{Color, Console, Key};

pub struct Rect {
    pub x: usize,
    pub y: usize,
    pub width: usize,
    pub height: usize,
}

impl Rect {
    pub fn new(x: usize, y: usize, width: usize, height: usize) -> Self {
        Self { x, y, width, height }
    }
}

pub struct UIListElement {
    display_text: Box<str>,
    fg_color: Color,
    bg_color: Color,
}

impl UIListElement {
    pub fn new(display_text: impl Into<Box<str>>, fg_color: Color, bg_color: Color) -> Self {
        Self { display_text: display_text.into(), fg_color, bg_color }
    }

    pub fn display_text(&self) -> &str {
        &self.display_text
    }

    pub fn fg_color(&self) -> Color {
        self.fg_color
    }

    pub fn bg_color(&self) -> Color {
        self.bg_color
    }
}

pub struct UIList {
    rect: Rect,
    cursor_index: usize,
    elements: Vec<UIListElement>,
    #[expect(clippy::type_complexity)]
    on_select: Box<dyn FnMut(&mut GameState, usize)>,
}

impl UIList {
    #[expect(clippy::type_complexity)]
    pub fn new(rect: Rect, elements: Vec<UIListElement>, on_select: Box<dyn FnMut(&mut GameState, usize)>) -> Self {
        Self { rect, cursor_index: 0, elements, on_select }
    }

    pub fn cursor_index(&self) -> usize {
        self.cursor_index
    }

    pub fn set_cursor_index(&mut self, cursor_index: usize) {
        self.cursor_index = cursor_index;
    }

    pub fn elements(&self) -> &[UIListElement] {
        &self.elements
    }

    pub fn elements_mut(&mut self) -> &mut Vec<UIListElement> {
        &mut self.elements
    }

    pub fn draw(&self, console: &Console) {
        let elements_per_row = (self.rect.width - 1) / 3;

        //Draw first line of border
        console.set_cursor_pos(self.rect.x, self.rect.y);
        console.draw_text("-");

        let element_count_first_row = self.elements.len().min(24);
        for i in 0..element_count_first_row  {
            let x = self.rect.x + 1 + i*3;

            console.set_cursor_pos(x, self.rect.y);
            console.draw_text("---");
        }

        for (i, ele) in self.elements.iter().enumerate() {
            let x = self.rect.x + 1 + (i%elements_per_row)*3;
            let y = self.rect.y + 1 + (i/elements_per_row)*2;

            //First box
            if i%elements_per_row == 0 {
                console.set_cursor_pos(x - 1, y);
                console.draw_text("|");

                console.set_cursor_pos(x - 1, y + 1);
                console.draw_text("-");
            }

            console.set_cursor_pos(x, y);
            console.set_color(ele.fg_color, ele.bg_color);
            console.draw_text(&*ele.display_text);

            console.reset_color();
            console.draw_text("|");

            console.set_cursor_pos(x, y + 1);
            console.draw_text("---");
        }

        if self.cursor_index < self.elements.len() {
            //Mark selected element
            let x = self.rect.x + (self.cursor_index%elements_per_row)*3;
            let y = self.rect.y + (self.cursor_index/elements_per_row)*2;

            console.set_color(Color::Cyan, Color::Default);
            console.set_cursor_pos(x, y);
            console.draw_text("----");
            console.set_cursor_pos(x, y + 1);
            console.draw_text("|");
            console.set_cursor_pos(x + 3, y + 1);
            console.draw_text("|");
            console.set_cursor_pos(x, y + 2);
            console.draw_text("----");
        }
    }

    pub fn on_key_press(&mut self, game_state: &mut GameState, key: Key) {
        let elements_per_row = (self.rect.width - 1) / 3;

        match key {
            Key::LEFT|Key::A => {
                if self.cursor_index == 0 {
                    return;
                }

                self.cursor_index -= 1;
            },
            Key::UP|Key::W => {
                if self.cursor_index <= elements_per_row {
                    return;
                }

                self.cursor_index -= elements_per_row;
            },
            Key::RIGHT|Key::D => {
                if self.cursor_index + 1 >= self.elements.len() {
                    return;
                }

                self.cursor_index += 1;
            },
            Key::DOWN|Key::S => {
                if self.cursor_index + elements_per_row >= self.elements.len() {
                    return;
                }

                self.cursor_index += elements_per_row;
            },

            Key::ENTER|Key::SPACE => {
                if self.cursor_index < self.elements.len() {
                    (self.on_select)(game_state, self.cursor_index);
                }
            },

            _ => {},
        }
    }

    pub fn on_mouse_pressed(&mut self, game_state: &mut GameState, column: usize, row: usize) {
        if column < self.rect.x || row < self.rect.y {
            return;
        }

        let column = column - self.rect.x;
        let row = row - self.rect.y;

        let elements_per_row = (self.rect.width - 1) / 3;

        let element_index = column/3 + row/2 * elements_per_row;
        if element_index < self.elements().len() {
            self.cursor_index = element_index;
            (self.on_select)(game_state, element_index);
        }
    }
}
