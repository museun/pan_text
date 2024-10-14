use std::io::Read as _;

use too::{
    math::{pos2, Pos2},
    App, AppRunner as _, Event, Key, Keybind, Modifiers,
};
use too_crossterm::{Config, Term};
use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

fn main() -> std::io::Result<()> {
    let input = match std::env::args().nth(1).as_deref() {
        Some(path) => std::fs::read_to_string(path)?,
        None => {
            let mut buf = String::new();
            std::io::stdin().read_to_string(&mut buf)?;
            buf
        }
    };

    let term = Term::setup(Config::default())?;
    Pan::new(input.lines()).run(term)
}

impl App for Pan {
    fn event(&mut self, event: Event, _ctx: too::Context<'_>) {
        const FASTER: i32 = 5;
        const SLOWER: i32 = 1;

        let modifiers = modifiers(&event);
        let scale = if modifiers.is_shift() || modifiers.is_alt() {
            FASTER
        } else {
            SLOWER
        };

        if let Event::MouseDragHeld { delta, .. } = event {
            self.offset += pos2(delta.x * scale, delta.y * scale);
        }

        if let Event::MouseScroll { delta, .. } = event {
            if modifiers.is_ctrl() {
                self.offset += pos2(delta.y * scale, delta.x * scale)
            } else {
                self.offset += pos2(-delta.x * scale, -delta.y * scale)
            }
        }

        if event.is_keybind_pressed('r') {
            self.offset = Pos2::ZERO
        }

        if is_key_pressed(&event, Key::Left)
            || is_key_pressed(&event, 'h')
            || is_key_pressed(&event, 'a')
        {
            self.offset += pos2(-scale, 0)
        }

        if is_key_pressed(&event, Key::Right)
            || is_key_pressed(&event, 'l')
            || is_key_pressed(&event, 'd')
        {
            self.offset += pos2(scale, 0)
        }

        if is_key_pressed(&event, Key::Up)
            || is_key_pressed(&event, 'j')
            || is_key_pressed(&event, 'w')
        {
            self.offset += pos2(0, -scale)
        }

        if is_key_pressed(&event, Key::Down)
            || is_key_pressed(&event, 'k')
            || is_key_pressed(&event, 's')
        {
            self.offset += pos2(0, scale)
        }

        if let Some(pos) = event.mouse_pos() {
            self.cursor = pos
        }
    }

    fn render(&mut self, surface: &mut impl too::Canvas, _ctx: too::Context<'_>) {
        let size = surface.rect().size();
        let mut start = self.offset;
        for line in &self.lines {
            if start.y >= size.y {
                break;
            }

            for grapheme in line.graphemes(true) {
                if start.x >= size.x {
                    break;
                }

                surface.set(start, grapheme);
                start.x += grapheme.width() as i32;
            }

            start.x = self.offset.x;
            start.y += 1;
        }
    }
}

// TODO these aren't implemented in this commit
fn is_key_pressed(event: &Event, key: impl Into<Keybind>) -> bool {
    let keybind = key.into();
    let Event::KeyPressed { key, .. } = event else {
        return false;
    };

    let want = match keybind.key {
        Key::Char(ch) => Key::Char(ch.to_ascii_lowercase()),
        key => key,
    };

    let got = match *key {
        Key::Char(ch) => Key::Char(ch.to_ascii_lowercase()),
        key => key,
    };

    want == got
}

// TODO these aren't implemented in this commit
const fn modifiers(event: &Event) -> Modifiers {
    match event {
        Event::KeyPressed { modifiers, .. }
        | Event::MouseDragStart { modifiers, .. }
        | Event::MouseDragHeld { modifiers, .. }
        | Event::MouseDragRelease { modifiers, .. }
        | Event::MouseScroll { modifiers, .. } => *modifiers,
        _ => Modifiers::NONE,
    }
}

struct Pan {
    lines: Vec<String>,
    offset: Pos2,
    cursor: Pos2,
}

impl Pan {
    fn new(lines: impl IntoIterator<Item = impl ToString>) -> Self {
        let lines: Vec<_> = lines.into_iter().map(|s| s.to_string()).collect();
        Self {
            lines,
            offset: Pos2::ZERO,
            cursor: Pos2::ZERO,
        }
    }
}
