use crossterm::{
    cursor,
    event::{Event, KeyCode},
    style,
    terminal::{self, ClearType},
    QueueableCommand,
};
use log::debug;
use std::{
    cmp::Ordering,
    io::{self, Stdout, Write},
};

use crate::{
    app::{EventHandlerResult, EventResult},
    input::{Input, Select},
    label::Label,
    pos::Pos,
    select_form::SelectForm,
};

#[derive(Debug, Clone)]
pub struct Form {
    pub(crate) labels: Vec<Label>,
    pub(crate) inputs: Vec<Input>,
    pub(crate) current_pos: Pos,
    pub(crate) size: Pos,
    pub(crate) select_form: Option<SelectForm>,
}

impl Form {
    pub fn new(size: impl Into<Pos>) -> io::Result<Self> {
        Ok(Self {
            labels: Default::default(),
            inputs: Default::default(),
            current_pos: (0, 0).into(),
            size: size.into(),
            select_form: None,
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

// Input handling
impl Form {
    pub fn event_handler(&mut self, event: &Event) -> io::Result<EventHandlerResult> {
        // Popup input handling
        if let Some(select_form) = self.select_form.as_mut() {
            let result = select_form.event_handler(event)?;
            match result {
                EventHandlerResult::Handled(EventResult::Submit) => {
                    let selected = select_form.get_selection();
                    if select_form.select_type == Select::Single {
                        debug!("Selected: {:?}", selected);
                        assert!(
                            selected.len() <= 1,
                            "Single should only return none or one item"
                        );
                    }
                    self.select_form = None;
                    if let Some(f) = self.current_field() {
                        f.value = selected
                            .first()
                            .map(|s| s.as_str())
                            .unwrap_or_default()
                            .to_string();
                    };
                    self.current_pos = self
                        .current_field()
                        .map(|f| f.pos)
                        .unwrap_or(self.current_pos);
                    return Ok(EventHandlerResult::Handled(EventResult::None));
                }
                EventHandlerResult::Handled(EventResult::Abort) => {
                    debug!("Aborted");
                    self.select_form = None;
                    return Ok(EventHandlerResult::Handled(EventResult::None));
                }
                _ => return Ok(result),
            };
        }

        let mut current_pos = self.current_pos;

        if let Some(current_field) = self.current_field() {
            if let EventHandlerResult::Handled(result) =
                current_field.event_handler(event, &mut current_pos)?
            {
                self.current_pos = current_pos;
                return Ok(EventHandlerResult::Handled(result));
            }
        }

        match event {
            Event::Key(k) if k.code == KeyCode::Esc => {
                return Ok(EventHandlerResult::Handled(EventResult::Abort));
            }
            Event::Key(k) if k.code == KeyCode::Enter => {
                return Ok(EventHandlerResult::Handled(EventResult::Submit));
            }
            Event::Key(k) if k.code == KeyCode::Left => {
                self.move_event(k.code);
            }
            Event::Key(k) if k.code == KeyCode::Right => {
                self.move_event(k.code);
            }
            Event::Key(k) if k.code == KeyCode::Up => {
                self.move_event(k.code);
            }
            Event::Key(k) if k.code == KeyCode::Down => {
                self.move_event(k.code);
            }
            Event::Key(k) if k.code == KeyCode::Tab => {
                self.next_input();
            }
            Event::Key(k) if k.code == KeyCode::BackTab => {
                self.prev_input();
            }
            Event::Key(k) if k.code == KeyCode::F(4) => {
                debug!("Display select form");
                let Some(current_field) = self.current_field() else {
                    return Ok(EventHandlerResult::Handled(EventResult::None));
                };

                let mut select_form =
                    SelectForm::new(&current_field.select_static, (80, 24), Select::Single)?;
                select_form.display(&mut std::io::stdout())?;

                self.select_form = Some(select_form);
            }
            _ => return Ok(EventHandlerResult::NotHandled),
        }

        Ok(EventHandlerResult::Handled(EventResult::None))
    }
}

impl Form {
    pub fn display(&mut self, stdout: &mut Stdout) -> io::Result<()> {
        if let Some(select_form) = self.select_form.as_mut() {
            return select_form.display(stdout);
        }

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
                debug!("Returning last");
                return inputs.last().map(|w| w.pos);
            };

            debug!("Comparing {:?} to {:?}", self.current_pos, input.pos);

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

    fn current_field(&mut self) -> Option<&mut Input> {
        self.inputs
            .iter_mut()
            .find(|f| f.has_focus(self.current_pos))
    }
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

    pub(crate) fn add_select(&mut self, input: String, id: String, value: String) {
        let input = self.inputs.iter_mut().find(|i| i.name == input);

        if let Some(input) = input {
            if input.select == Select::None {
                input.select = Select::Single;
            }

            input.select_static.push((id, value));
            debug!("List: {:?}", input.select_static);
        } else {
            panic!("Input not found");
        }
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
