use std::io;

mod app;
mod dialog_appender;
mod form;
mod input;
mod label;
mod parser;
mod pos;
mod vec_appender;

use app::{App, EventResult};

fn main() -> io::Result<()> {
    let stdout = io::stdout();

    let mut app = App::with_writer(stdout);
    app.init()?;

    let mut form = app.form_from_textfile((82, 24))?;

    let result = app.execute(&mut form)?;

    drop(app);

    if result == EventResult::Submit {
        let fields = form.get_field_and_data();

        for (name, value) in fields {
            println!("{}={}", name, snailquote::escape(value));
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests;
