use crossterm::{cursor, event::KeyCode, style, QueueableCommand};
use std::{
    cmp::Ordering,
    io::{self, Stdout, Write},
};

use crate::{
    pos::Pos,
    widget::{Widget, WidgetType},
};

#[derive(Debug, Clone)]
pub struct Form {
    pub(crate) widgets: Vec<Widget>,
    pub(crate) current_pos: Pos,
    pub(crate) size: Pos,
}

impl Form {
    pub fn new(size: impl Into<Pos>) -> io::Result<Self> {
        Ok(Self {
            widgets: Default::default(),
            current_pos: (0, 0).into(),
            size: size.into(),
        })
    }
}

fn display_string(
    stdout: &mut Stdout,
    pos: Pos,
    s: &str,
    default: &str,
    length: u16,
) -> io::Result<()> {
    stdout
        .queue(cursor::MoveTo(pos.x, pos.y))?
        .queue(style::SetAttribute(style::Attribute::Underlined))?;
    for i in 0..(length as usize) {
        match (s.chars().nth(i), default.chars().nth(i)) {
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

fn display_password(
    stdout: &mut Stdout,
    pos: Pos,
    s: &str,
    length: u16,
    mask_char: char,
) -> io::Result<()> {
    let pass_len = s.chars().count();
    stdout
        .queue(cursor::MoveTo(pos.x, pos.y))?
        .queue(style::SetAttribute(style::Attribute::Underlined))?
        .queue(style::SetForegroundColor(style::Color::DarkRed))?
        .queue(style::Print(mask_char.to_string().repeat(pass_len)))?
        .queue(style::SetForegroundColor(style::Color::DarkGreen))?
        .queue(style::Print(" ".repeat(length as usize - pass_len)))?
        .queue(style::SetAttribute(style::Attribute::NoUnderline))?;

    Ok(())
}

fn display_generic(stdout: &mut Stdout, pos: Pos, widget: &WidgetType) -> io::Result<()> {
    let WidgetType::Generic {
        length,
        name: _,
        value,
        default_value,
        allowed_characters: _,
        mask_char,
        select,
    } = widget
    else {
        unimplemented!();
    };

    if let Some(mask_char) = mask_char {
        display_password(stdout, pos, value, *length, *mask_char)?;
    } else {
        display_string(stdout, pos, value, default_value, *length)?;
    }

    if *select == crate::widget::Select::None {
        stdout
            .queue(cursor::MoveTo(82 - 6 - 10, 24))?
            .queue(style::SetForegroundColor(style::Color::DarkGreen))?
            .queue(style::Print(" F4 - Select "))?;
        /*
        } else {
            stdout
                .queue(cursor::MoveTo(82 - 3 - 10, 24))?
                .queue(style::SetForegroundColor(style::Color::DarkGreen))?
                .queue(style::Print("             "))?;
        */
    };

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

        for widget in self.widgets.clone() {
            match widget.widget_type {
                WidgetType::Text { value } => {
                    stdout
                        .queue(cursor::MoveTo(widget.pos.x, widget.pos.y))?
                        .queue(style::SetForegroundColor(style::Color::White))?
                        .queue(style::Print(value))?;
                }
                WidgetType::Generic { .. } => {
                    display_generic(stdout, widget.pos, &widget.widget_type)?;
                }
            }
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
        for (i, widget) in self.widgets.clone().iter_mut().enumerate() {
            match &widget.widget_type {
                WidgetType::Text { .. } => (),
                WidgetType::Generic {
                    length,
                    name,
                    value,
                    default_value,
                    allowed_characters,
                    mask_char,
                    select,
                } => {
                    if let Some(str_pos) = self.current_pos.within(&widget.pos, *length) {
                        if let Some(ac) = allowed_characters {
                            if !ac.contains(&key) {
                                return;
                            }
                        }

                        self.current_pos = self.current_pos.move_x(1, widget.pos.x + length);

                        let _ = std::mem::replace(
                            &mut self.widgets[i],
                            Widget::new_generic(
                                widget.pos,
                                *length,
                                name,
                                set_char_in_string(value, str_pos, key),
                                default_value,
                                allowed_characters.clone(),
                                *mask_char,
                                *select,
                            ),
                        );
                    }
                }
            }
        }
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
        for (i, widget) in self.widgets.clone().iter_mut().enumerate() {
            if let WidgetType::Generic {
                length,
                name,
                value,
                default_value,
                allowed_characters,
                mask_char,
                select,
            } = &widget.widget_type
            {
                if let Some(str_pos) = self.current_pos.within(&widget.pos, *length) {
                    log::debug!("Backspace, at pos: {}", str_pos);
                    // Backspace on first character does nothing
                    if self.current_pos.x == widget.pos.x {
                        return Ok(());
                    }

                    self.current_pos = self.current_pos.move_x(-1, widget.pos.x + length);

                    let _ = std::mem::replace(
                        &mut self.widgets[i],
                        Widget::new_generic(
                            widget.pos,
                            *length,
                            name,
                            Self::delete_in_string(value, str_pos - 1),
                            default_value,
                            allowed_characters.clone(),
                            *mask_char,
                            *select,
                        ),
                    );
                }
            }
        }
        Ok(())
    }

    pub fn key_delete(&mut self) -> io::Result<()> {
        for (i, widget) in self.widgets.clone().iter_mut().enumerate() {
            if let WidgetType::Generic {
                length,
                name,
                value,
                default_value,
                allowed_characters,
                mask_char,
                select,
            } = &widget.widget_type
            {
                if let Some(str_pos) = self.current_pos.within(&widget.pos, *length) {
                    let _ = std::mem::replace(
                        &mut self.widgets[i],
                        Widget::new_generic(
                            widget.pos,
                            *length,
                            name,
                            Self::delete_in_string(value, str_pos),
                            default_value,
                            allowed_characters.clone(),
                            *mask_char,
                            *select,
                        ),
                    );
                }
            }
        }
        Ok(())
    }

    pub(crate) fn find_next_input(&mut self) -> Option<Pos> {
        let mut widgets = self.widgets.clone();
        widgets.sort();

        let mut i = widgets.iter().filter(|w| w.is_input());
        let mut first_pos = None;

        loop {
            // Return None if no input fields
            let Some(widget) = i.next() else {
                return first_pos;
            };

            // Set first pos if none set
            if first_pos.is_none() {
                first_pos = Some(widget.pos);
            }

            match self.current_pos.cmp(&widget.pos) {
                // Current pos is before the widget pos
                Ordering::Less => {
                    return Some(widget.pos);
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
        let mut widgets = self.widgets.clone();
        widgets.sort();

        let mut i = widgets.iter().filter(|w| w.is_input()).rev();

        loop {
            // Return None if no input fields
            let Some(widget) = i.next() else {
                return widgets.last().map(|w| w.pos);
            };

            match self.current_pos.cmp(&widget.pos) {
                // Current pos is before the widget pos
                Ordering::Greater => {
                    return Some(widget.pos);
                }

                // We are on the first character if an input field
                // We should return the next input field
                Ordering::Equal => {
                    if let Some(next) = i.next() {
                        // There is a next field available, return it's pos
                        return Some(next.pos);
                    } else {
                        // No next field, wrap around to first_pos
                        return widgets
                            .iter()
                            .filter(|w| w.is_input())
                            .map(|w| w.pos)
                            .last();
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
        self.widgets.push(Widget::new_label(pos, text));

        self
    }

    /*
    #[allow(dead_code)]
    pub fn add_input(
        mut self,
        pos: impl Into<Pos>,
        length: u16,
        name: &'static str,
        default_value: impl Into<String>,
    ) -> Self {
        let value = default_value.into();
        self.widgets
            .push(Widget::new_input(pos, length, name, value.clone(), value));

        self
    }
    */

    pub(crate) fn add_widget(&mut self, widget: Widget) {
        self.widgets.push(widget);
    }

    pub fn place_cursor(mut self) -> Self {
        self.current_pos = self.find_next_input().unwrap_or((0, 0).into());

        self
    }

    #[allow(dead_code)]
    fn get_input(&self, field_name: &'static str) -> Option<String> {
        self.widgets
            .iter()
            .find_map(|widget| match &widget.widget_type {
                WidgetType::Generic { name, value, .. } => {
                    if name == field_name {
                        Some(value.to_string())
                    } else {
                        None
                    }
                }
                _ => None,
            })
    }

    pub fn get_field_and_data(&self) -> Vec<(&str, &str)> {
        let mut output = Vec::new();

        for widget in &self.widgets {
            if let WidgetType::Generic { name, value, .. } = &widget.widget_type {
                output.push((name.as_str(), value.as_str()));
            }
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
