use crossterm::{
    cursor,
    event::{self, Event, KeyCode},
    style,
    terminal::{self, disable_raw_mode, enable_raw_mode},
    ExecutableCommand, QueueableCommand,
};
use log4rs::Handle;
use std::{
    cmp::Ordering,
    io::{self, Stdout, Write},
};

mod pos;
use pos::Pos;

#[derive(Debug, Clone, Eq)]
struct Widget {
    pos: Pos,
    widget_type: WidgetType,
}

#[derive(Debug, Clone, Eq, PartialEq, PartialOrd)]
enum WidgetType {
    Text {
        value: String,
    },
    Input {
        length: u16,
        name: &'static str,
        value: String,
        default_value: String,
    },
}

impl Widget {
    fn is_input(&self) -> bool {
        if let WidgetType::Input { .. } = self.widget_type {
            true
        } else {
            false
        }
    }
}

#[derive(Debug, Clone)]
struct Form {
    widgets: Vec<Widget>,
    current_pos: Pos,
    size: Pos,
}

impl Ord for Widget {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.pos.cmp(&other.pos)
    }
}

impl PartialEq for Widget {
    fn eq(&self, other: &Self) -> bool {
        self.pos == other.pos // TODO: && self.widget_type == other.widget_type
    }
}

impl PartialOrd for Widget {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.pos.partial_cmp(&other.pos)
    }
}

impl Form {
    fn new(size: impl Into<Pos>) -> io::Result<Self> {
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
        match (s.get(i..i + 1), default.get(i..i + 1)) {
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
            /*
            (_, Some(d)) => {
                stdout
                    .queue(style::SetForegroundColor(style::Color::DarkGreen))?
                    .queue(style::Print(d))?;
            }
            */
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

impl Form {
    fn display(&mut self, stdout: &mut Stdout) -> io::Result<()> {
        // Border
        for y in 0..24 {
            stdout
                .queue(cursor::MoveTo(80, y))?
                .queue(style::Print('│'))?;
        }
        stdout
            .queue(cursor::MoveTo(0, 24))?
            .queue(style::Print("─".repeat(80)))?
            .queue(style::Print('┘'))?;

        for widget in self.widgets.clone() {
            match widget.widget_type {
                WidgetType::Text { value } => {
                    stdout
                        .queue(cursor::MoveTo(widget.pos.x, widget.pos.y))?
                        .queue(style::SetForegroundColor(style::Color::White))?
                        .queue(style::Print(value))?;
                }
                WidgetType::Input {
                    length,
                    name: _,
                    value,
                    default_value,
                } => {
                    display_string(stdout, widget.pos, &value, &default_value, length)?;
                } // _ => unimplemented!(),
            }
        }

        stdout.queue(cursor::MoveTo(self.current_pos.x, self.current_pos.y))?;
        stdout.queue(cursor::SetCursorStyle::SteadyUnderScore)?;

        stdout.flush()
    }
}

impl Form {
    fn move_event(&mut self, code: KeyCode) {
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

    fn key(&mut self, key: char) {
        for (i, widget) in self.widgets.clone().iter_mut().enumerate() {
            if let WidgetType::Input {
                length,
                name,
                value,
                default_value,
            } = &widget.widget_type
            {
                if let Some(str_pos) = self.current_pos.within(&widget.pos, *length) {
                    self.current_pos = self.current_pos.move_x(1, widget.pos.x + length);

                    let _ = std::mem::replace(
                        &mut self.widgets[i],
                        Widget {
                            pos: widget.pos,

                            widget_type: WidgetType::Input {
                                length: *length,
                                name,
                                value: set_char_in_string(value, str_pos, key),
                                default_value: default_value.to_string(),
                            },
                        },
                    );
                }
            }
        }
    }

    fn backspace_in_string(input: &str, default: &str, pos: usize) -> String {
        if pos > input.len() {
            let mut output = input.to_string();
            output.truncate(input.len() - 1);
            return output;
        }

        input
            .chars()
            .zip(default.chars().chain(std::iter::repeat(' ')))
            .enumerate()
            .map(|(i, (inp, def))| if i == pos { def } else { inp })
            .collect()
    }

    fn delete_in_string(input: &str, pos: usize) -> String {
        if pos > input.len() {
            return input.to_string();
        }

        let mut output: String = input.chars().take(pos).collect();

        let rest: String = input.chars().skip(pos + 1).collect();

        output.push_str(&rest);

        output
    }

    fn key_backspace(&mut self) -> io::Result<()> {
        for (i, widget) in self.widgets.clone().iter_mut().enumerate() {
            if let WidgetType::Input {
                length,
                name,
                value,
                default_value,
            } = &widget.widget_type
            {
                if let Some(str_pos) = self.current_pos.within(&widget.pos, *length) {
                    // Backspace on first character does nothing
                    if self.current_pos.x == widget.pos.x {
                        return Ok(());
                    }

                    self.current_pos = self.current_pos.move_x(-1, widget.pos.x + length);

                    let _ = std::mem::replace(
                        &mut self.widgets[i],
                        Widget {
                            pos: widget.pos,

                            widget_type: WidgetType::Input {
                                length: *length,
                                name,
                                value: Self::backspace_in_string(value, default_value, str_pos + 1),
                                default_value: default_value.to_string(),
                            },
                        },
                    );
                }
            }
        }
        Ok(())
    }

    fn key_delete(&mut self) -> io::Result<()> {
        for (i, widget) in self.widgets.clone().iter_mut().enumerate() {
            if let WidgetType::Input {
                length,
                name,
                value,
                default_value,
            } = &widget.widget_type
            {
                if let Some(str_pos) = self.current_pos.within(&widget.pos, *length) {
                    let _ = std::mem::replace(
                        &mut self.widgets[i],
                        Widget {
                            pos: widget.pos,

                            widget_type: WidgetType::Input {
                                length: *length,
                                name,
                                value: Self::delete_in_string(value, str_pos),
                                default_value: default_value.to_string(),
                            },
                        },
                    );
                }
            }
        }
        Ok(())
    }

    fn find_next_input(&mut self) -> Option<Pos> {
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
            if first_pos == None {
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

    fn find_prev_input(&mut self) -> Option<Pos> {
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
}

fn set_char_in_string(s: &str, pos: usize, ch: char) -> String {
    let mut s = s.to_string();

    if s.len() <= pos {
        let rep = " ".repeat(pos - s.len() + 1);
        s.push_str(&rep);
    }

    s.replace_range(pos..pos + 1, &ch.to_string());

    s
}

impl Form {
    fn add_text(mut self, pos: impl Into<Pos>, text: impl Into<String>) -> Self {
        self.widgets.push(Widget {
            pos: pos.into(),
            widget_type: WidgetType::Text { value: text.into() },
        });

        self
    }

    fn add_input(
        mut self,
        pos: impl Into<Pos>,
        len: u16,
        name: &'static str,
        default_value: impl Into<String>,
    ) -> Self {
        let value = default_value.into();
        self.widgets.push(Widget {
            pos: pos.into(),
            widget_type: WidgetType::Input {
                length: len,
                name,
                value: value.clone(),
                default_value: value,
            },
        });

        self
    }

    fn place_cursor(mut self) -> Self {
        self.current_pos = self.find_next_input().unwrap_or((0, 0).into());

        self
    }

    #[allow(dead_code)]
    fn get_input(&self, field_name: &'static str) -> Option<String> {
        self.widgets.iter().find_map(|widget| {
            if let WidgetType::Input {
                length: _,
                name,
                value,
                default_value: _default_value,
            } = &widget.widget_type
            {
                if *name == field_name {
                    Some(value.to_string())
                } else {
                    None
                }
            } else {
                None
            }
        })
    }
}

fn enable_logging() -> Handle {
    use log::LevelFilter;
    use log4rs::append::file::FileAppender;
    use log4rs::config::{Appender, Config, Root};
    use log4rs::encode::pattern::PatternEncoder;

    let debug_log = FileAppender::builder()
        .encoder(Box::new(PatternEncoder::new("{l} - {m}{n}\n")))
        .build("log/debug.log")
        .unwrap();

    let config = Config::builder()
        .appender(Appender::builder().build("debug_log", Box::new(debug_log)))
        .build(
            Root::builder()
                .appender("debug_log")
                .build(LevelFilter::Trace),
        )
        .unwrap();

    let handle = log4rs::init_config(config).unwrap();

    handle
}

fn main() -> io::Result<()> {
    let _logger_handle = enable_logging();

    let mut stdout = io::stdout();

    stdout.execute(terminal::EnterAlternateScreen)?;
    stdout.execute(terminal::Clear(terminal::ClearType::All))?;

    enable_raw_mode()?;

    let mut form = Form::new((80, 24))?
        .add_text((0, 0), "Hello world")
        .add_input((12, 0), 10, "hello", "hello")
        .add_input((12, 2), 10, "hello2", "hello2")
        .add_input((25, 0), 10, "hello3", "hello3")
        .add_text((10, 5), "YoYo")
        .place_cursor();

    let mut submit = false;
    loop {
        form.display(&mut stdout)?;

        let ev = event::read()?;

        match ev {
            Event::Key(k) if k.code == KeyCode::Esc => {
                break;
            }
            Event::Key(k) if k.code == KeyCode::Enter => {
                submit = true;
                break;
            }
            Event::Key(k) if k.code == KeyCode::Left => {
                form.move_event(k.code);
            }
            Event::Key(k) if k.code == KeyCode::Right => {
                form.move_event(k.code);
            }
            Event::Key(k) if k.code == KeyCode::Up => {
                form.move_event(k.code);
            }
            Event::Key(k) if k.code == KeyCode::Down => {
                form.move_event(k.code);
            }
            Event::Key(k) if k.code == KeyCode::Tab => {
                if let Some(pos) = form.find_next_input() {
                    form.current_pos = pos;
                }
            }
            Event::Key(k) if k.code == KeyCode::BackTab => {
                if let Some(pos) = form.find_prev_input() {
                    form.current_pos = pos;
                }
            }
            Event::Key(k) if k.code == KeyCode::Backspace => {
                form.key_backspace()?;
            }
            Event::Key(k) if k.code == KeyCode::Delete => {
                form.key_delete()?;
            }
            Event::Key(k) => {
                if let KeyCode::Char(c) = k.code {
                    form.key(c);
                }
            }
            _ => unimplemented!(),
        }
    }

    disable_raw_mode()?;

    stdout.execute(terminal::LeaveAlternateScreen)?;
    //.execute(cursor::MoveToNextLine(1))?
    //.execute(terminal::Clear(ClearType::FromCursorDown))?
    //.execute(style::ResetColor)?;

    if submit {
        for widget in form.widgets {
            if let WidgetType::Input {
                length: _,
                name,
                value,
                default_value: _,
            } = widget.widget_type
            {
                println!(r#"{}="{}""#, name, snailquote::escape(&value));
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::iter::empty;

    use super::*;

    #[test]
    fn find_next() {
        log4rs_test_utils::test_logging::init_logging_once_for(empty(), None, None);

        let mut form = Form::new((80, 24))
            .unwrap()
            .add_text((0, 0), "Hello world")
            .add_input((12, 0), 10, "hello", "hello")
            .add_input((12, 2), 10, "hello2", "hello2")
            .add_input((25, 0), 10, "hello3", "hello3")
            .add_text((10, 5), "YoYo");

        // Before first field
        form.current_pos = (0, 0).into();
        assert_eq!(
            form.find_next_input().unwrap(),
            (12, 0).into(),
            "Before first field"
        );

        form.current_pos = (12, 0).into();
        assert_eq!(
            form.find_next_input().unwrap(),
            (25, 0).into(),
            "On first field"
        );

        form.current_pos = (25, 0).into();
        assert_eq!(
            form.find_next_input().unwrap(),
            (12, 2).into(),
            "On second field"
        );

        // Last field -> First field
        form.current_pos = (12, 2).into();
        assert_eq!(
            form.find_next_input().unwrap(),
            (12, 0).into(),
            "Last field -> First field"
        );

        // After last field
        form.current_pos = (25, 8).into();
        assert_eq!(
            form.find_next_input().unwrap(),
            (12, 0).into(),
            "After last field"
        );

        // Middle of fields
        form.current_pos = (25, 1).into();
        assert_eq!(
            form.find_next_input().unwrap(),
            (12, 2).into(),
            "Middle of fields"
        );
    }

    #[test]
    fn backspace() {
        let input = "12345678901";

        let output = Form::backspace_in_string(input, "abcdefhijkl", 1);
        assert_eq!(output, "1b345678901");

        let output = Form::backspace_in_string(input, "abcdefhijkl", 0);
        assert_eq!(output, "a2345678901");

        let output = Form::backspace_in_string(input, "abcdefhijkl", 10);
        assert_eq!(output, "1234567890l");

        let output = Form::backspace_in_string(input, "abcdefhijkl", 11);
        assert_eq!(output, "12345678901", "Delete after input string");

        let output = Form::backspace_in_string(input, "abcdefh", 7);
        assert_eq!(output, "1234567 901");
    }

    #[test]
    fn delete() {
        let input = "12345678901";

        let output = Form::delete_in_string(input, 1);
        assert_eq!(output, "1345678901");

        let output = Form::delete_in_string(input, 0);
        assert_eq!(output, "2345678901");

        let output = Form::delete_in_string(input, 10);
        assert_eq!(output, "1234567890");

        let output = Form::delete_in_string(input, 11);
        assert_eq!(output, "12345678901", "Delete after input string");

        let output = Form::delete_in_string(input, 7);
        assert_eq!(output, "1234567901");
    }
}
