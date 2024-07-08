use crate::{form::Form, input::Input, label::Label};
use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{alpha1, alphanumeric1, multispace0, multispace1, u16},
    combinator::recognize,
    multi::many0_count,
    sequence::{pair, tuple},
    IResult,
};

pub fn identifier(input: &str) -> IResult<&str, &str> {
    recognize(pair(
        alt((alpha1, tag("_"))),
        many0_count(alt((alphanumeric1, tag("_")))),
    ))(input)
}

fn parse_label(input: &str) -> IResult<&str, Widget> {
    let (rest, (_, x, _, y, _)) =
        tuple((tag("LABEL "), u16, multispace1, u16, multispace1))(input)?;

    Ok(("", Widget::Label(Label::new_label((x, y), rest))))
}

fn parse_input(input: &str) -> IResult<&str, Widget> {
    // INPUT 5 111 10 nafn texti hér

    let (rest, (widget_type, _, x, _, y, _, length, _, name, _)) = tuple((
        alt((tag("INPUT"), tag("PASSWORD"))),
        multispace1,
        u16,
        multispace1,
        u16,
        multispace1,
        u16,
        multispace1,
        identifier,
        multispace0,
    ))(input)?;

    match widget_type {
        "INPUT" => Ok((
            "",
            Widget::Input(
                Input::builder((x, y), length, name)
                    .with_value(rest)
                    .build(),
            ),
        )),
        "PASSWORD" => Ok((
            "",
            Widget::Input(
                Input::builder((x, y), length, name)
                    .with_value(rest)
                    .with_mask_char('*')
                    .build(),
            ),
        )),
        _ => unimplemented!(),
    }
}

fn parse_select(input: &str) -> IResult<&str, Widget> {
    // SELECT input id display

    let (rest, (_widget_type, _, input, _, id, _)) = tuple((
        tag("SELECT"),
        multispace1,
        identifier,
        multispace1,
        identifier,
        multispace1,
    ))(input)?;

    Ok((
        "",
        Widget::Select(input.to_string(), id.to_string(), rest.to_string()),
    ))
}

fn parse_number(input: &str) -> IResult<&str, Widget> {
    // INPUT 5 111 10 nafn texti hér

    let (rest, (widget_type, _, x, _, y, _, length, _, name, _)) = tuple((
        tag("NUMBER"),
        multispace1,
        u16,
        multispace1,
        u16,
        multispace1,
        u16,
        multispace1,
        identifier,
        multispace0,
    ))(input)?;

    let default_value = if !rest.is_empty() {
        match str::parse::<i64>(rest) {
            Ok(i) => i.to_string(),
            Err(_) => {
                return Err(nom::Err::Error(nom::error::Error::new(
                    rest,
                    nom::error::ErrorKind::Digit,
                )));
            }
        }
    } else {
        String::new()
    };

    match widget_type {
        "NUMBER" => Ok((
            "",
            Widget::Input(
                Input::builder((x, y), length, name)
                    .with_value(&default_value)
                    .with_default_value(default_value)
                    .with_allowed_characters('0'..='9')
                    .build(),
            ),
        )),
        _ => unimplemented!(),
    }
}

enum Widget {
    Label(Label),
    Input(Input),
    Select(String, String, String),
}

fn parse_widget(input: &str) -> Result<Widget, String> {
    let (_, widget) = alt((parse_label, parse_input, parse_number, parse_select))(input)
        .map_err(|e| e.to_string())?;

    Ok(widget)
}

pub fn parse_str(form: &mut Form, input: &str) -> Result<(), String> {
    let widget = parse_widget(input)?;

    match widget {
        Widget::Label(l) => form.add_label(l),
        Widget::Input(i) => form.add_input(i),
        Widget::Select(input, id, text) => form.add_select(input, id, text),
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::input::Select;

    #[test]
    fn test_parse_label() {
        let Widget::Label(label) = parse_widget("LABEL 1 2 texti hér").unwrap() else {
            panic!("Parsed value is not a label");
        };

        assert_eq!(label.pos, (1, 2).into());
        assert_eq!(label.text, "texti hér");
    }

    #[test]
    fn test_parse_input() {
        let Widget::Input(input) = parse_widget("INPUT 5 111 10 nafn texti hér").unwrap() else {
            panic!("Parsed value is not an input");
        };

        assert_eq!(input.pos, (5, 111).into());
        assert_eq!(input.length, 10);
        assert_eq!(input.name, "nafn".to_string());
        assert_eq!(input.value, "texti hér".to_string());
        assert_eq!(input.default_value, "".to_string());
        assert_eq!(input.allowed_characters, None);
        assert_eq!(input.mask_char, None);
        assert_eq!(input.select, Select::None);
    }

    #[test]
    fn test_parse_select() {
        let Widget::Select(input, id, value) = parse_widget("SELECT inp id langur texti").unwrap()
        else {
            panic!("Parsed value is not a select");
        };

        assert_eq!(input, "inp".to_string());
        assert_eq!(id, "id".to_string());
        assert_eq!(value, "langur texti".to_string());
    }
}
