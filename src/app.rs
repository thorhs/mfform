use std::io::{self, Stdout};

use crossterm::{
    //event::{self, Event, KeyCode},
    terminal::{self, disable_raw_mode, enable_raw_mode},
    ExecutableCommand,
};

pub struct App {
    stdout: Stdout,
}

impl App {
    pub fn with_writer(stdout: Stdout) -> Self {
        Self { stdout }
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
}

impl Drop for App {
    fn drop(&mut self) {
        let _ = Self::restore_terminal();
    }
}
