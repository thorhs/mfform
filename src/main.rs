use std::{
    ffi::{OsStr, OsString},
    io::{self, BufRead},
};

mod parser;

use mfform_lib::{App, EventResult, Form, Pos};

pub fn form_from_textfile(input_file: impl AsRef<OsStr>, size: impl Into<Pos>) -> io::Result<Form> {
    let mut form = Form::new(size)?;

    let file = std::fs::File::open(input_file.as_ref())?;
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

fn main() -> io::Result<()> {
    let stdout = io::stdout();

    let mut args = std::env::args_os();
    let screen_name = if args.len() > 1 {
        let _ = args.next().unwrap();
        args.next().unwrap()
    } else {
        OsString::from("screen.mfform")
    };

    let mut app = App::with_writer(stdout);
    app.init()?;

    let mut form = form_from_textfile(screen_name, (82, 24))?;

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
