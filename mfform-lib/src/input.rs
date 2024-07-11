use std::io;

use crossterm::event::{Event, KeyCode};
use log::debug;

use crate::{
    app::{EventHandlerResult, EventResult},
    pos::Pos,
};

#[allow(dead_code)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Select {
    None,
    Single,
    Multi,
}

/// Generic input field, supporting masked input (password) and number fields.
///
/// Also supports 'select'able fields, where the user can press F4 to get a list
/// of predefined values.
#[derive(Debug, Clone)]
pub struct Input {
    pub pos: Pos,
    pub length: u16,
    pub name: String,
    pub value: String,
    pub default_value: String,
    pub allowed_characters: Option<Vec<char>>,
    pub mask_char: Option<char>,
    pub select: Select,
    pub select_static: Vec<(String, String)>,
}

impl Ord for Input {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.pos.cmp(&other.pos)
    }
}

impl Eq for Input {}

impl PartialEq for Input {
    fn eq(&self, other: &Self) -> bool {
        self.pos == other.pos // TODO: && self.widget_type == other.widget_type
    }
}

impl PartialOrd for Input {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Input {
    pub(crate) fn has_focus(&self, cursor: Pos) -> bool {
        cursor.within(self.pos, self.length).is_some()
    }

    /// Create a InputBuilder for creating a new Input field.
    pub fn builder(pos: impl Into<Pos>, length: u16, name: impl Into<String>) -> InputBuilder {
        InputBuilder {
            pos: pos.into(),
            length,
            name: name.into(),
            value: Default::default(),
            default_value: Default::default(),
            allowed_characters: Default::default(),
            mask_char: Default::default(),
            select: Select::None,
            select_static: Default::default(),
        }
    }

    pub(crate) fn event_handler(
        &mut self,
        event: &Event,
        current_pos: &mut Pos,
    ) -> std::io::Result<EventHandlerResult> {
        match event {
            Event::Key(k) if k.code == KeyCode::Backspace => {
                self.key_backspace(current_pos)?;
            }
            Event::Key(k) if k.code == KeyCode::Delete => {
                self.key_delete(current_pos)?;
            }
            Event::Key(k) if k.modifiers.is_empty() => {
                if let KeyCode::Char(c) = k.code {
                    if let Some(ac) = &self.allowed_characters {
                        if !ac.contains(&c) {
                            debug!("{} is not an allowed character for input {}", c, self.name);
                            return Ok(EventHandlerResult::Handled(EventResult::None));
                        }
                    }

                    self.key(c, current_pos);
                } else {
                    return Ok(EventHandlerResult::NotHandled);
                }
            }
            _ => return Ok(EventHandlerResult::NotHandled),
        }

        Ok(EventHandlerResult::Handled(EventResult::None))
    }

    pub(crate) fn key(&mut self, key: char, current_pos: &mut Pos) {
        let str_pos = current_pos.x - self.pos.x;
        *current_pos = current_pos.move_x(1, self.pos.x + self.length);

        self.value = Self::set_char_in_string(&self.value, str_pos as usize, key);
    }

    pub(crate) fn set_char_in_string(s: &str, pos: usize, ch: char) -> String {
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

    pub(crate) fn delete_in_string(input: &str, pos: usize) -> String {
        let input_len = input.chars().count();
        if pos > input_len {
            return input.to_string();
        }

        let mut output: String = input.chars().take(pos).collect();
        output.extend(input.chars().skip(pos + 1));

        output
    }

    pub(crate) fn key_backspace(&mut self, current_pos: &mut Pos) -> io::Result<()> {
        let str_pos = current_pos.x - self.pos.x;

        // No backspace at start of field
        if str_pos == 0 {
            return Ok(());
        }

        *current_pos = current_pos.move_x(-1, self.pos.x + self.length);

        self.value = Self::delete_in_string(&self.value, (str_pos - 1) as usize);
        Ok(())
    }

    fn key_delete(&mut self, current_pos: &Pos) -> io::Result<()> {
        let str_pos = current_pos.x - self.pos.x;

        self.value = Self::delete_in_string(&self.value, str_pos as usize);
        Ok(())
    }
}

pub struct InputBuilder {
    pub pos: Pos,
    pub length: u16,
    pub name: String,
    pub value: String,
    pub default_value: String,
    pub allowed_characters: Option<Vec<char>>,
    pub mask_char: Option<char>,
    pub select: Select,
    pub select_static: Vec<(String, String)>,
}

impl InputBuilder {
    pub fn with_value(mut self, value: impl Into<String>) -> Self {
        self.value = value.into();

        self
    }

    pub fn with_default_value(mut self, default_value: impl Into<String>) -> Self {
        self.default_value = default_value.into();

        self
    }

    pub fn with_allowed_characters(
        mut self,
        allowed_characters: impl IntoIterator<Item = char>,
    ) -> Self {
        self.allowed_characters = Some(allowed_characters.into_iter().collect());

        self
    }

    pub fn with_mask_char(mut self, mask_char: char) -> Self {
        self.mask_char = Some(mask_char);

        self
    }

    #[allow(dead_code)]
    pub fn with_select_static(mut self, select_static: &[(String, String)]) -> Self {
        self.select_static = select_static.into();

        self
    }

    pub fn build(self) -> Input {
        Input {
            pos: self.pos,
            length: self.length,
            name: self.name,
            value: self.value,
            default_value: self.default_value,
            allowed_characters: self.allowed_characters,
            mask_char: self.mask_char,
            select: self.select,
            select_static: self.select_static,
        }
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

        let output = Input::set_char_in_string(input, pos, ch);

        assert_eq!(output, expected);
    }

    #[test]
    fn set_char_in_string_nls() {
        let input = "1æ34567890";
        let pos = 2;
        let ch = 'ö';
        let expected = "1æö4567890";

        let output = Input::set_char_in_string(input, pos, ch);

        assert_eq!(output, expected);
    }
}
