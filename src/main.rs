use crossterm::event::{self, Event, KeyCode};
use log4rs::Handle;
use std::io::{self, BufRead};

mod app;
mod form;
mod parser;
mod pos;
mod widget;

use app::App;
use form::Form;
use pos::Pos;

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
            form.next_input();
        }
        Event::Key(k) if k.code == KeyCode::BackTab => {
            form.prev_input();
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

    let stdout = io::stdout();

    let mut app = App::with_writer(stdout);
    app.init()?;

    let mut form = create_form((82, 24))?;

    let mut submit = false;
    loop {
        form.display(&mut io::stdout())?;

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

    drop(app);

    if submit {
        let fields = form.get_field_and_data();

        for (name, value) in fields {
            println!("{}={}", name, snailquote::escape(value));
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests;
