use crossterm::{
    cursor,
    event::{Event, KeyCode},
    style,
    terminal::{self, ClearType},
    QueueableCommand,
};
use log::debug;
use std::io::{self, Stdout, Write};

use crate::{
    app::{EventHandlerResult, EventResult},
    input::Select,
    pos::Pos,
};

#[derive(Debug, Clone)]
pub(crate) struct Item {
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

static FIRST_FIELD_POS: Pos = Pos { x: 20, y: 5 };

#[derive(Debug, Clone)]
pub struct SelectForm {
    pub(crate) items: Vec<Item>,
    pub(crate) current_pos: Pos,
    pub(crate) size: Pos,
    pub(crate) select_type: Select,
}

impl SelectForm {
    pub fn new(
        items: &[(String, String)],
        size: impl Into<Pos>,
        select_type: Select,
    ) -> io::Result<Self> {
        Ok(Self {
            items: items.iter().map(Item::from).collect(),
            current_pos: FIRST_FIELD_POS,
            size: size.into(),
            select_type,
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

    pub(crate) fn get_selection(&self) -> Vec<String> {
        self.items
            .iter()
            .filter(|i| i.choice == 's')
            .map(|i| i.id.to_string())
            .collect()
    }
}

impl SelectForm {
    pub fn event_handler(&mut self, event: &Event) -> io::Result<EventHandlerResult> {
        match event {
            Event::Key(k) if k.code == KeyCode::Esc => {
                return Ok(EventHandlerResult::Handled(EventResult::Abort));
            }
            Event::Key(k) if k.code == KeyCode::Enter => {
                if self.select_type == Select::Single && self.get_selection().len() <= 1 {
                    return Ok(EventHandlerResult::Handled(EventResult::Submit));
                }
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
            Event::Key(k) if k.code == KeyCode::Backspace => {
                self.key_backspace()?;
            }
            Event::Key(k) if k.code == KeyCode::Delete => {
                self.key_delete()?;
            }
            Event::Key(k) if k.modifiers.is_empty() => {
                if let KeyCode::Char(c) = k.code {
                    self.key(c);
                } else {
                    return Ok(EventHandlerResult::NotHandled);
                }
            }
            _ => return Ok(EventHandlerResult::NotHandled),
        }

        Ok(EventHandlerResult::Handled(EventResult::None))
    }

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

        if let Some(c) = self.items.get_mut(choice as usize) {
            c.choice = key;
        }
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
        fn pos_to_flat(size: Pos, pos: Pos) -> u16 {
            (size.x * pos.y) + pos.x
        }

        let first_field = pos_to_flat(self.size, FIRST_FIELD_POS);
        let current_pos = pos_to_flat(self.size, self.current_pos);

        if current_pos < first_field {
            return Some(FIRST_FIELD_POS);
        }

        let difference = current_pos - first_field;

        let diff_and_some = difference + (self.size.x * 2);

        let choice = diff_and_some / self.size.x / 2;

        if choice >= self.items.len() as u16 {
            Some(FIRST_FIELD_POS)
        } else {
            Some(Pos {
                x: FIRST_FIELD_POS.x,
                y: FIRST_FIELD_POS.y + (2 * (choice)),
            })
        }
    }

    pub(crate) fn find_prev_input(&mut self) -> Option<Pos> {
        fn pos_to_flat(size: Pos, pos: Pos) -> u16 {
            (size.x * pos.y) + pos.x
        }

        let first_field = pos_to_flat(self.size, FIRST_FIELD_POS);
        let current_pos = pos_to_flat(self.size, self.current_pos);

        if current_pos < first_field {
            return Some(Pos {
                x: FIRST_FIELD_POS.x,
                y: FIRST_FIELD_POS.y + (self.items.len() as u16 * 2) - 2,
            });
        }

        let difference = current_pos.wrapping_sub(first_field).wrapping_sub(1);

        let choice = difference / self.size.x / 2;

        if choice >= self.items.len() as u16 {
            Some(Pos {
                x: FIRST_FIELD_POS.x,
                y: FIRST_FIELD_POS.y + (self.items.len() as u16 * 2) - 2,
            })
        } else {
            Some(Pos {
                x: FIRST_FIELD_POS.x,
                y: FIRST_FIELD_POS.y + (2 * (choice)),
            })
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
    use crate::input::Select;
    use crate::pos::Pos;

    use super::{SelectForm, FIRST_FIELD_POS};

    #[test]
    fn test_next_before_first() {
        let mut form = SelectForm::new(
            &[
                ("1".to_string(), "item1".to_string()),
                ("2".to_string(), "item2".to_string()),
                ("3".to_string(), "item3".to_string()),
            ],
            (80, 24),
            Select::Single,
        )
        .unwrap();

        form.current_pos = (0, 0).into();

        assert_eq!(form.find_next_input(), Some(FIRST_FIELD_POS));
    }

    #[test]
    fn test_next_first() {
        let mut form = SelectForm::new(
            &[
                ("1".to_string(), "item1".to_string()),
                ("2".to_string(), "item2".to_string()),
                ("3".to_string(), "item3".to_string()),
                ("4".to_string(), "item4".to_string()),
            ],
            (80, 24),
            Select::Single,
        )
        .unwrap();

        form.current_pos = FIRST_FIELD_POS;

        assert_eq!(
            form.find_next_input(),
            Some(Pos {
                x: FIRST_FIELD_POS.x,
                y: FIRST_FIELD_POS.y + 2
            })
        );
    }

    #[test]
    fn test_next_second() {
        let mut form = SelectForm::new(
            &[
                ("1".to_string(), "item1".to_string()),
                ("2".to_string(), "item2".to_string()),
                ("3".to_string(), "item3".to_string()),
                ("4".to_string(), "item4".to_string()),
            ],
            (80, 24),
            Select::Single,
        )
        .unwrap();

        form.current_pos = Pos {
            x: FIRST_FIELD_POS.x,
            y: FIRST_FIELD_POS.y + 2,
        };

        assert_eq!(
            form.find_next_input(),
            Some(Pos {
                x: FIRST_FIELD_POS.x,
                y: FIRST_FIELD_POS.y + 4
            })
        );
    }

    #[test]
    fn test_next_between_second_and_third() {
        let mut form = SelectForm::new(
            &[
                ("1".to_string(), "item1".to_string()),
                ("2".to_string(), "item2".to_string()),
                ("3".to_string(), "item3".to_string()),
                ("4".to_string(), "item4".to_string()),
            ],
            (80, 24),
            Select::Single,
        )
        .unwrap();

        form.current_pos = Pos {
            x: FIRST_FIELD_POS.x + 3,
            y: FIRST_FIELD_POS.y + 3,
        };

        assert_eq!(
            form.find_next_input(),
            Some(Pos {
                x: FIRST_FIELD_POS.x,
                y: FIRST_FIELD_POS.y + 4
            })
        );
    }

    #[test]
    fn test_next_third() {
        let mut form = SelectForm::new(
            &[
                ("1".to_string(), "item1".to_string()),
                ("2".to_string(), "item2".to_string()),
                ("3".to_string(), "item3".to_string()),
                ("4".to_string(), "item4".to_string()),
            ],
            (80, 24),
            Select::Single,
        )
        .unwrap();

        form.current_pos = Pos {
            x: FIRST_FIELD_POS.x,
            y: FIRST_FIELD_POS.y + 2,
        };

        assert_eq!(
            form.find_next_input(),
            Some(Pos {
                x: FIRST_FIELD_POS.x,
                y: FIRST_FIELD_POS.y + 4
            })
        );
    }

    #[test]
    fn test_next_last() {
        let mut form = SelectForm::new(
            &[
                ("1".to_string(), "item1".to_string()),
                ("2".to_string(), "item2".to_string()),
                ("3".to_string(), "item3".to_string()),
                ("4".to_string(), "item4".to_string()),
            ],
            (80, 24),
            Select::Single,
        )
        .unwrap();

        form.current_pos = Pos {
            x: FIRST_FIELD_POS.x,
            y: FIRST_FIELD_POS.y + 6,
        };

        assert_eq!(
            form.find_next_input(),
            Some(Pos {
                x: FIRST_FIELD_POS.x,
                y: FIRST_FIELD_POS.y
            })
        );
    }

    #[test]
    fn test_next_after_last() {
        let mut form = SelectForm::new(
            &[
                ("1".to_string(), "item1".to_string()),
                ("2".to_string(), "item2".to_string()),
                ("3".to_string(), "item3".to_string()),
                ("4".to_string(), "item4".to_string()),
            ],
            (80, 24),
            Select::Single,
        )
        .unwrap();

        form.current_pos = Pos {
            x: FIRST_FIELD_POS.x,
            y: FIRST_FIELD_POS.y + 9,
        };

        assert_eq!(
            form.find_next_input(),
            Some(Pos {
                x: FIRST_FIELD_POS.x,
                y: FIRST_FIELD_POS.y
            })
        );
    }

    #[test]
    fn test_prev_before_first() {
        let mut form = SelectForm::new(
            &[
                ("1".to_string(), "item1".to_string()),
                ("2".to_string(), "item2".to_string()),
                ("3".to_string(), "item3".to_string()),
                ("4".to_string(), "item4".to_string()),
            ],
            (80, 24),
            Select::Single,
        )
        .unwrap();

        form.current_pos = (0, 0).into();

        assert_eq!(
            form.find_prev_input(),
            Some(Pos {
                x: FIRST_FIELD_POS.x,
                y: FIRST_FIELD_POS.y + 6
            })
        );
    }

    #[test]
    fn test_prev_first() {
        let mut form = SelectForm::new(
            &[
                ("1".to_string(), "item1".to_string()),
                ("2".to_string(), "item2".to_string()),
                ("3".to_string(), "item3".to_string()),
                ("4".to_string(), "item4".to_string()),
            ],
            (80, 24),
            Select::Single,
        )
        .unwrap();

        form.current_pos = FIRST_FIELD_POS;

        assert_eq!(
            form.find_prev_input(),
            Some(Pos {
                x: FIRST_FIELD_POS.x,
                y: FIRST_FIELD_POS.y + 6
            })
        );
    }

    #[test]
    fn test_prev_second() {
        let mut form = SelectForm::new(
            &[
                ("1".to_string(), "item1".to_string()),
                ("2".to_string(), "item2".to_string()),
                ("3".to_string(), "item3".to_string()),
                ("4".to_string(), "item4".to_string()),
            ],
            (80, 24),
            Select::Single,
        )
        .unwrap();

        form.current_pos = Pos {
            x: FIRST_FIELD_POS.x,
            y: FIRST_FIELD_POS.y + 2,
        };

        assert_eq!(
            form.find_prev_input(),
            Some(Pos {
                x: FIRST_FIELD_POS.x,
                y: FIRST_FIELD_POS.y
            })
        );
    }

    #[test]
    fn test_prev_between_second_and_third() {
        let mut form = SelectForm::new(
            &[
                ("1".to_string(), "item1".to_string()),
                ("2".to_string(), "item2".to_string()),
                ("3".to_string(), "item3".to_string()),
                ("4".to_string(), "item4".to_string()),
            ],
            (80, 24),
            Select::Single,
        )
        .unwrap();

        form.current_pos = Pos {
            x: FIRST_FIELD_POS.x + 2,
            y: FIRST_FIELD_POS.y + 3,
        };

        assert_eq!(
            form.find_prev_input(),
            Some(Pos {
                x: FIRST_FIELD_POS.x,
                y: FIRST_FIELD_POS.y + 2
            })
        );
    }

    #[test]
    fn test_prev_third() {
        let mut form = SelectForm::new(
            &[
                ("1".to_string(), "item1".to_string()),
                ("2".to_string(), "item2".to_string()),
                ("3".to_string(), "item3".to_string()),
                ("4".to_string(), "item4".to_string()),
            ],
            (80, 24),
            Select::Single,
        )
        .unwrap();

        form.current_pos = Pos {
            x: FIRST_FIELD_POS.x,
            y: FIRST_FIELD_POS.y + 4,
        };

        assert_eq!(
            form.find_prev_input(),
            Some(Pos {
                x: FIRST_FIELD_POS.x,
                y: FIRST_FIELD_POS.y + 2
            })
        );
    }

    #[test]
    fn test_prev_last() {
        let mut form = SelectForm::new(
            &[
                ("1".to_string(), "item1".to_string()),
                ("2".to_string(), "item2".to_string()),
                ("3".to_string(), "item3".to_string()),
                ("4".to_string(), "item4".to_string()),
            ],
            (80, 24),
            Select::Single,
        )
        .unwrap();

        form.current_pos = Pos {
            x: FIRST_FIELD_POS.x,
            y: FIRST_FIELD_POS.y + 6,
        };

        assert_eq!(
            form.find_prev_input(),
            Some(Pos {
                x: FIRST_FIELD_POS.x,
                y: FIRST_FIELD_POS.y + 4
            })
        );
    }

    #[test]
    fn test_prev_after_last() {
        let mut form = SelectForm::new(
            &[
                ("1".to_string(), "item1".to_string()),
                ("2".to_string(), "item2".to_string()),
                ("3".to_string(), "item3".to_string()),
                ("4".to_string(), "item4".to_string()),
            ],
            (80, 24),
            Select::Single,
        )
        .unwrap();

        form.current_pos = Pos {
            x: FIRST_FIELD_POS.x,
            y: FIRST_FIELD_POS.y + 9,
        };

        assert_eq!(
            form.find_prev_input(),
            Some(Pos {
                x: FIRST_FIELD_POS.x,
                y: FIRST_FIELD_POS.y + 6
            })
        );
    }
}
