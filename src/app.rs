use std::{
    io::{self, BufRead, Stdout},
    sync::{Arc, RwLock},
};

use crossterm::{
    event::{self, Event, KeyCode, KeyModifiers},
    terminal::{self, disable_raw_mode, enable_raw_mode},
    ExecutableCommand, QueueableCommand,
};
use log::debug;
use log4rs::Handle;

use crate::{form::Form, pos::Pos};

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum EventResult {
    Submit,
    Abort,
    ToggleDebug,
    None,
}

pub struct App {
    stdout: Stdout,
    _logger_handle: log4rs::Handle,
    logging_enabled: Arc<RwLock<bool>>,
}

impl App {
    pub fn with_writer(stdout: Stdout) -> Self {
        let logging_enabled = Arc::new(RwLock::new(false));

        let logger_handle = Self::configure_logging(logging_enabled.clone());

        Self {
            stdout,
            _logger_handle: logger_handle,
            logging_enabled,
        }
    }

    pub fn init(&mut self) -> io::Result<()> {
        self.init_terminal()?;
        self.install_panic_handler();

        Ok(())
    }

    fn install_panic_handler(&self) {
        let original_hook = std::panic::take_hook();
        std::panic::set_hook(Box::new(move |panic_info| {
            let _ = Self::restore_terminal();
            original_hook(panic_info);
        }))
    }

    fn init_terminal(&mut self) -> io::Result<()> {
        self.stdout.execute(terminal::EnterAlternateScreen)?;
        self.stdout
            .execute(terminal::Clear(terminal::ClearType::All))?;

        enable_raw_mode()?;

        Ok(())
    }

    fn restore_terminal() -> io::Result<()> {
        disable_raw_mode()?;

        io::stdout().execute(terminal::LeaveAlternateScreen)?;

        Ok(())
    }

    pub fn toggle_log_output(&self) -> io::Result<()> {
        if Self::is_logging_enabled(self.logging_enabled.clone()) {
            self.disable_log_output()?;
        } else {
            self.enable_log_output()?;
        }

        Ok(())
    }

    fn enable_log_output(&self) -> io::Result<()> {
        let mut logging_enabled = self
            .logging_enabled
            .write()
            .expect("Unable to get write lock on log output toggle");
        *logging_enabled = true;

        // Drop write lock before using again in logging
        drop(logging_enabled);

        debug!("Log output enabled");

        Ok(())
    }
    fn disable_log_output(&self) -> io::Result<()> {
        let mut logging_enabled = self
            .logging_enabled
            .write()
            .expect("Unable to get write lock on log output toggle");
        *logging_enabled = false;

        // Drop write lock before using again in logging
        drop(logging_enabled);

        std::io::stdout().queue(crossterm::terminal::Clear(
            crossterm::terminal::ClearType::All,
        ))?;

        debug!("Log output disabled");

        Ok(())
    }
}

// Static functions
impl App {
    fn configure_logging(logging_enabled: Arc<RwLock<bool>>) -> Handle {
        use log::LevelFilter;
        use log4rs::append::file::FileAppender;
        use log4rs::config::{Appender, Config, Root};
        use log4rs::encode::pattern::PatternEncoder;

        let log_buffer = crate::vec_appender::Appender::with_capacity(100);
        let log_dialog = crate::dialog_appender::Appender::new((0, 26), 5, logging_enabled);

        let debug_log = FileAppender::builder()
            .encoder(Box::new(PatternEncoder::new("{l} - {m}{n}\n")))
            .build("log/debug.log")
            .unwrap();

        let config = Config::builder()
            .appender(Appender::builder().build("debug_log", Box::new(debug_log)))
            .appender(Appender::builder().build("log_buffer", Box::new(log_buffer)))
            .appender(Appender::builder().build("log_dialog", Box::new(log_dialog)))
            .build(
                Root::builder()
                    .appender("log_dialog")
                    .build(LevelFilter::Trace),
            )
            .unwrap();

        log4rs::init_config(config).unwrap()
    }

    fn is_logging_enabled(logging_enabled: Arc<RwLock<bool>>) -> bool {
        logging_enabled.read().map(|e| *e).unwrap_or(false)
    }

    pub fn keyboard_event(&mut self, form: &mut Form, ev: Event) -> io::Result<EventResult> {
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
            Event::Key(k)
                if k.code == KeyCode::Char('d') && k.modifiers.contains(KeyModifiers::CONTROL) =>
            {
                return Ok(EventResult::ToggleDebug);
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

    pub fn execute(&mut self, form: &mut Form) -> io::Result<EventResult> {
        let mut result = EventResult::None;
        loop {
            form.display(&mut io::stdout())?;

            let ev = event::read()?;

            debug!("Key event: {:?}", ev);

            match self.keyboard_event(form, ev)? {
                EventResult::Abort => break,
                EventResult::Submit => {
                    result = EventResult::Submit;
                    break;
                }
                EventResult::ToggleDebug => {
                    self.toggle_log_output()?;
                }
                EventResult::None => (),
            };
        }

        Ok(result)
    }
}

// Form creation fucctions
impl App {
    pub fn form_from_textfile(&self, size: impl Into<Pos>) -> io::Result<Form> {
        let mut form = Form::new(size)?;

        let file = std::fs::File::open("screen.mfform")?;
        let mut reader = std::io::BufReader::new(file);

        let mut line = String::new();
        while let Ok(read_bytes) = reader.read_line(&mut line) {
            if read_bytes == 0 {
                break;
            }

            if line.len() > 5 {
                crate::parser::parse_str(&mut form, line.trim())
                    .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
            }
            line.clear();
        }

        let form = form.place_cursor();

        Ok(form)
    }
}

impl Drop for App {
    fn drop(&mut self) {
        let _ = Self::restore_terminal();
    }
}
