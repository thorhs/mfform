use crossterm::{
    cursor,
    event::KeyCode,
    style,
    terminal::{self, ClearType},
    QueueableCommand,
};
use log::debug;
use std::{
    cmp::Ordering,
    io::{self, Stdout, Write},
};

use crate::pos::Pos;

#[derive(Debug, Clone)]
enum Choice {
    None,
    Select,
    Exclude,
}

#[derive(Debug, Clone)]
struct Item {
    choice: char,
    id: String,
    text: String,
}

impl From<&(String, String)> for Item {
    fn from(value: &(String, String)) -> Self {
        Self {
            choice: ' ',
            id: value.0.to_string(),
            text: value.1.to_string(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct SelectForm {
    pub(crate) items: Vec<Item>,
    pub(crate) current_pos: Pos,
    pub(crate) size: Pos,
}

impl SelectForm {
    pub fn new(items: &[(String, String)], size: impl Into<Pos>) -> io::Result<Self> {
        Ok(Self {
            items: items.iter().map(|i| Item::from(i)).collect(),
            current_pos: (0, 0).into(),
            size: size.into(),
        })
    }

    fn display_choice(stdout: &mut Stdout, pos: impl Into<Pos>, item: &Item) -> io::Result<()> {
        let pos = pos.into();

        stdout
            .queue(cursor::MoveTo(pos.x, pos.y))?
            .queue(style::SetAttribute(style::Attribute::Underlined))?
            .queue(style::Print(item.choice))?
            .queue(style::SetAttribute(style::Attribute::NoUnderline))?
            .queue(style::Print(' '))?
            .queue(style::Print(&item.text))?;

        Ok(())
    }

    pub fn display(&mut self, stdout: &mut Stdout) -> io::Result<()> {
        // Clear dialog
        stdout
            .queue(cursor::MoveTo(self.size.x, self.size.y))?
            .queue(terminal::Clear(ClearType::FromCursorUp))?;

        // Border
        stdout.queue(style::SetForegroundColor(style::Color::DarkGreen))?;
        for y in 0..24 {
            stdout
                .queue(cursor::MoveTo(80, y))?
                .queue(style::Print('│'))?;
        }
        stdout
            .queue(cursor::MoveTo(0, 24))?
            .queue(style::Print("─".repeat(80)))?
            .queue(style::Print('┘'))?;

        stdout
            .queue(cursor::MoveTo(2, 24))?
            .queue(style::Print(" Esc=Abort "))?
            .queue(style::Print('─'))?
            .queue(style::Print(" Enter=Submit "))?;

        for (i, item) in self.items.clone().into_iter().enumerate() {
            Self::display_choice(stdout, (20, 5 + (i as u16 * 2)), &item)?;
        }

        stdout.queue(cursor::MoveTo(self.current_pos.x, self.current_pos.y))?;
        stdout.queue(cursor::SetCursorStyle::SteadyUnderScore)?;

        stdout.flush()
    }
}

impl SelectForm {
    pub fn move_event(&mut self, code: KeyCode) {
        self.current_pos = match code {
            KeyCode::Left => Pos {
                x: self.current_pos.x.checked_sub(1).unwrap_or_default(),
                y: self.current_pos.y,
            },
            KeyCode::Right => Pos {
                x: self.current_pos.x + 1,
                y: self.current_pos.y,
            },
            KeyCode::Up => Pos {
                x: self.current_pos.x,
                y: self.current_pos.y.checked_sub(1).unwrap_or_default(),
            },
            KeyCode::Down => Pos {
                x: self.current_pos.x,
                y: self.current_pos.y + 1,
            },
            _ => self.current_pos,
        }
        .constrain(self.size)
    }

    pub fn key(&mut self, key: char) {
        if self.current_pos.x != 20 {
            debug!("Not correct column: {:?}", self.current_pos);
            return;
        }

        let i = self.current_pos.y - 5;

        if i % 2 != 0 {
            debug!("Not in event row: {:?}", self.current_pos);
            return;
        }

        let choice = i / 2;

        if choice > self.items.len() as u16 {
            debug!("Below lowest item: {:?}", self.current_pos);
            return;
        }

        self.items.get_mut(choice as usize).map(|c| c.choice = key);
    }

    pub fn key_backspace(&mut self) -> io::Result<()> {
        self.key(' ');
        Ok(())
    }

    pub fn key_delete(&mut self) -> io::Result<()> {
        self.key(' ');
        Ok(())
    }

    pub(crate) fn find_next_input(&mut self) -> Option<Pos> {
        let flattened = self.current_pos.y * 82 + self.current_pos.x;
        let flattened = flattened - (82 * 7) - 19;
        let choice = flattened / 82;
        if choice > self.items.len() as u16 {
            return Some((20, 5).into());
        } else {
            return None; // TODO
        }
    }

    pub(crate) fn find_prev_input(&mut self) -> Option<Pos> {
        let flattened = self.current_pos.y * 82 + self.current_pos.x;
        let flattened = flattened - (82 * 7) - 19;
        let choice = flattened / 82;
        if choice > self.items.len() as u16 {
            return Some((20, 5).into());
        } else {
            return None; // TODO
        }
    }

    pub fn next_input(&mut self) {
        if let Some(pos) = self.find_next_input() {
            self.current_pos = pos;
        }
    }

    pub fn prev_input(&mut self) {
        if let Some(pos) = self.find_prev_input() {
            self.current_pos = pos;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
}
