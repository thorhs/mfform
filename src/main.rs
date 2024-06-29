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
    io::{self, BufRead, Error, Stdout, Write},
};

mod parser;
mod pos;

use parser::parse_str;
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
        name: String,
        value: String,
        default_value: String,
    },
}

impl Widget {
    fn is_input(&self) -> bool {
        matches!(self.widget_type, WidgetType::Input { .. })
    }

    fn new_label(pos: impl Into<Pos>, text: impl Into<String>) -> Self {
        Self {
            pos: pos.into(),
            widget_type: WidgetType::Text { value: text.into() },
        }
    }

    fn new_input(
        pos: impl Into<Pos>,
        length: u16,
        name: impl Into<String>,
        value: impl Into<String>,
        default_value: impl Into<String>,
    ) -> Self {
        Self {
            pos: pos.into(),
            widget_type: WidgetType::Input {
                length,
                name: name.into(),
                value: value.into(),
                default_value: default_value.into(),
            },
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
        Some(self.cmp(other))
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
                        Widget::new_input(
                            widget.pos,
                            *length,
                            name,
                            set_char_in_string(value, str_pos, key),
                            default_value,
                        ),
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
                        Widget::new_input(
                            widget.pos,
                            *length,
                            name,
                            Self::backspace_in_string(value, default_value, str_pos + 1),
                            default_value,
                        ),
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
                        Widget::new_input(
                            widget.pos,
                            *length,
                            name,
                            Self::delete_in_string(value, str_pos),
                            default_value,
                        ),
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
        self.widgets.push(Widget::new_label(pos, text));

        self
    }

    fn add_input(
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

    log4rs::init_config(config).unwrap()
}

fn create_form(size: impl Into<Pos>) -> io::Result<Form> {
    /*
    let form = Form::new(size)?
        .add_text((0, 0), "Hello world")
        .add_input((12, 0), 10, "hello", "hello")
        .add_input((12, 2), 10, "hello2", "hello2")
        .add_input((25, 0), 10, "hello3", "hello3")
        .add_text((10, 5), "YoYo")
        .place_cursor();
    */

    let mut form = Form::new(size)?;

    let file = std::fs::File::open("screen.mfform")?;
    let mut reader = std::io::BufReader::new(file);

    let mut line = String::new();
    while let Ok(read_bytes) = reader.read_line(&mut line) {
        if read_bytes == 0 {
            break;
        }

        if line.len() > 5 {
            parser::parse_str(&mut form, line.trim())
                .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
        }
        line.clear();
    }

    let form = form.place_cursor();

    Ok(form)
}

enum EventResult {
    Submit,
    Abort,
    None,
}
fn keyboard_event(form: &mut Form, ev: Event) -> io::Result<EventResult> {
    match ev {
        Event::Key(k) if k.code == KeyCode::Esc => {
            return Ok(EventResult::Abort);
        }
        Event::Key(k) if k.code == KeyCode::Enter => {
            return Ok(EventResult::Submit);
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

    Ok(EventResult::None)
}

fn main() -> io::Result<()> {
    let _logger_handle = enable_logging();

    let mut stdout = io::stdout();

    stdout.execute(terminal::EnterAlternateScreen)?;
    stdout.execute(terminal::Clear(terminal::ClearType::All))?;

    enable_raw_mode()?;

    let mut form = create_form((82, 24))?;

    let mut submit = false;
    loop {
        form.display(&mut stdout)?;

        let ev = event::read()?;

        match keyboard_event(&mut form, ev)? {
            EventResult::Abort => break,
            EventResult::Submit => {
                submit = true;
                break;
            }
            EventResult::None => (),
        };
    }

    disable_raw_mode()?;

    stdout.execute(terminal::LeaveAlternateScreen)?;

    if submit {
        for widget in &form.widgets {
            if let WidgetType::Input {
                length: _,
                name,
                value,
                default_value: _,
            } = &widget.widget_type
            {
                println!("{}={}", name, snailquote::escape(value));
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests;
