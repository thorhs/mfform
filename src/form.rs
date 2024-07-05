use crossterm::{cursor, event::KeyCode, style, QueueableCommand};
use std::{
    cmp::Ordering,
    io::{self, Stdout, Write},
};

use crate::{
    input::{Input, Select},
    label::Label,
    pos::Pos,
};

#[derive(Debug, Clone)]
pub struct Form {
    pub(crate) labels: Vec<Label>,
    pub(crate) inputs: Vec<Input>,
    pub(crate) current_pos: Pos,
    pub(crate) size: Pos,
}

impl Form {
    pub fn new(size: impl Into<Pos>) -> io::Result<Self> {
        Ok(Self {
            labels: Default::default(),
            inputs: Default::default(),
            current_pos: (0, 0).into(),
            size: size.into(),
        })
    }
}

fn display_string(stdout: &mut Stdout, input: &Input) -> io::Result<()> {
    stdout
        .queue(cursor::MoveTo(input.pos.x, input.pos.y))?
        .queue(style::SetAttribute(style::Attribute::Underlined))?;
    for i in 0..(input.length as usize) {
        match (
            input.value.chars().nth(i),
            input.default_value.chars().nth(i),
        ) {
            (Some(s), Some(d)) if s != d => {
                stdout
                    .queue(style::SetForegroundColor(style::Color::DarkRed))?
                    .queue(style::Print(s))?;
            }
            (Some(s), Some(_)) => {
                stdout
                    .queue(style::SetForegroundColor(style::Color::DarkGreen))?
                    .queue(style::Print(s))?;
            }
            (Some(s), None) => {
                stdout
                    .queue(style::SetForegroundColor(style::Color::DarkRed))?
                    .queue(style::Print(s))?;
            }
            _ => {
                stdout
                    .queue(style::SetForegroundColor(style::Color::DarkGreen))?
                    .queue(style::Print(" "))?;
            }
        }
    }

    stdout.queue(style::SetAttribute(style::Attribute::NoUnderline))?;

    Ok(())
}

fn display_password(stdout: &mut Stdout, input: &Input) -> io::Result<()> {
    let pass_len = input.value.chars().count();

    // We only get called if there is a mask_char
    let mask_char = input.mask_char.unwrap();
    stdout
        .queue(cursor::MoveTo(input.pos.x, input.pos.y))?
        .queue(style::SetAttribute(style::Attribute::Underlined))?
        .queue(style::SetForegroundColor(style::Color::DarkRed))?
        .queue(style::Print(mask_char.to_string().repeat(pass_len)))?
        .queue(style::SetForegroundColor(style::Color::DarkGreen))?
        .queue(style::Print(" ".repeat(input.length as usize - pass_len)))?
        .queue(style::SetAttribute(style::Attribute::NoUnderline))?;

    Ok(())
}

fn display_generic(stdout: &mut Stdout, input: &Input) -> io::Result<()> {
    if input.mask_char.is_some() {
        display_password(stdout, input)?;
    } else {
        display_string(stdout, input)?;
    }

    Ok(())
}

impl Form {
    pub fn display(&mut self, stdout: &mut Stdout) -> io::Result<()> {
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

        if self
            .inputs
            .iter()
            .find(|i| i.has_focus(self.current_pos))
            .filter(|i| i.select != Select::None)
            .is_some()
        {
            stdout
                .queue(cursor::MoveTo(82 - 6 - 10, 24))?
                .queue(style::SetForegroundColor(style::Color::DarkGreen))?
                .queue(style::Print(" F4 - Select "))?;
        }

        for label in self.labels.clone() {
            stdout
                .queue(cursor::MoveTo(label.pos.x, label.pos.y))?
                .queue(style::SetForegroundColor(style::Color::White))?
                .queue(style::Print(label.text))?;
        }

        for input in self.inputs.clone() {
            display_generic(stdout, &input)?;
        }

        stdout.queue(cursor::MoveTo(self.current_pos.x, self.current_pos.y))?;
        stdout.queue(cursor::SetCursorStyle::SteadyUnderScore)?;

        stdout.flush()
    }
}

impl Form {
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
        self.inputs
            .iter_mut()
            .find(|i| i.has_focus(self.current_pos))
            .map(|i| {
                if let Some(str_pos) = self.current_pos.within(i.pos, i.length) {
                    if let Some(ac) = &i.allowed_characters {
                        if !ac.contains(&key) {
                            return;
                        }
                    }

                    self.current_pos = self.current_pos.move_x(1, i.pos.x + i.length);

                    i.value = set_char_in_string(&i.value, str_pos, key);
                }
            });
    }

    pub(crate) fn delete_in_string(input: &str, pos: usize) -> String {
        let input_len = input.chars().count();
        if pos > input_len {
            return input.to_string();
        }

        let mut output: String = input.chars().take(pos).collect();
        output.extend(input.chars().skip(pos + 1));

        output
    }

    pub fn key_backspace(&mut self) -> io::Result<()> {
        self.inputs
            .iter_mut()
            .find(|i| i.has_focus(self.current_pos))
            .map(|i| {
                if let Some(str_pos) = self.current_pos.within(i.pos, i.length) {
                    self.current_pos = self.current_pos.move_x(-1, i.pos.x + i.length);

                    i.value = Self::delete_in_string(&i.value, str_pos - 1);
                }
            });
        Ok(())
    }

    pub fn key_delete(&mut self) -> io::Result<()> {
        self.inputs
            .iter_mut()
            .find(|i| i.has_focus(self.current_pos))
            .map(|i| {
                if let Some(str_pos) = self.current_pos.within(i.pos, i.length) {
                    i.value = Self::delete_in_string(&i.value, str_pos);
                }
            });
        Ok(())
    }

    pub(crate) fn find_next_input(&mut self) -> Option<Pos> {
        let mut inputs = self.inputs.clone();
        inputs.sort();

        let mut i = inputs.iter();
        let mut first_pos = None;

        loop {
            // Return None if no input fields
            let Some(input) = i.next() else {
                return first_pos;
            };

            // Set first pos if none set
            if first_pos.is_none() {
                first_pos = Some(input.pos);
            }

            match self.current_pos.cmp(&input.pos) {
                // Current pos is before the input pos
                Ordering::Less => {
                    return Some(input.pos);
                }

                // We are on the first character if an input field
                // We should return the next input field
                Ordering::Equal => {
                    if let Some(next) = i.next() {
                        // There is a next field available, return it's pos
                        return Some(next.pos);
                    } else {
                        // No next field, wrap around to first_pos
                        return first_pos;
                    }
                }
                _ => (),
            }
        }
    }

    pub(crate) fn find_prev_input(&mut self) -> Option<Pos> {
        let mut inputs = self.inputs.clone();
        inputs.sort();

        let mut i = inputs.iter().rev();

        loop {
            // Return None if no input fields
            let Some(input) = i.next() else {
                return inputs.last().map(|w| w.pos);
            };

            match self.current_pos.cmp(&input.pos) {
                // Current pos is before the input pos
                Ordering::Greater => {
                    return Some(input.pos);
                }

                // We are on the first character if an input field
                // We should return the next input field
                Ordering::Equal => {
                    if let Some(next) = i.next() {
                        // There is a next field available, return it's pos
                        return Some(next.pos);
                    } else {
                        // No next field, wrap around to first_pos
                        return inputs.iter().map(|w| w.pos).last();
                    }
                }
                _ => (),
            }
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

fn set_char_in_string(s: &str, pos: usize, ch: char) -> String {
    let mut s = s.to_string();

    let len = s.chars().count();

    if len <= pos {
        let rep = " ".repeat(pos - len + 1);
        s.push_str(&rep);
    }

    let mut output: String = s.chars().take(pos).collect();

    output.push(ch);
    output.extend(s.chars().skip(pos + 1));

    output
}

impl Form {
    #[allow(dead_code)]
    pub fn add_text(mut self, pos: impl Into<Pos>, text: impl Into<String>) -> Self {
        self.labels.push(Label::new_label(pos, text));

        self
    }

    pub(crate) fn add_label(&mut self, label: Label) {
        self.labels.push(label);
    }

    pub(crate) fn add_input(&mut self, input: Input) {
        self.inputs.push(input);
    }

    pub fn place_cursor(mut self) -> Self {
        self.current_pos = self.find_next_input().unwrap_or((0, 0).into());

        self
    }

    #[allow(dead_code)]
    fn get_input(&self, field_name: &'static str) -> Option<String> {
        self.inputs.iter().find_map(|input| {
            if input.name == field_name {
                Some(input.value.to_string())
            } else {
                None
            }
        })
    }

    pub fn get_field_and_data(&self) -> Vec<(&str, &str)> {
        let mut output = Vec::new();

        for input in &self.inputs {
            output.push((input.name.as_str(), input.value.as_str()));
        }

        output
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn set_char_in_string_plain() {
        let input = "1234567890";
        let pos = 2;
        let ch = 'a';
        let expected = "12a4567890";

        let output = set_char_in_string(input, pos, ch);

        assert_eq!(output, expected);
    }

    #[test]
    fn set_char_in_string_nls() {
        let input = "1æ34567890";
        let pos = 2;
        let ch = 'ö';
        let expected = "1æö4567890";

        let output = set_char_in_string(input, pos, ch);

        assert_eq!(output, expected);
    }
}
