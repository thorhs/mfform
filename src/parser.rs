use crate::{
    form::Form,
    widget::{Select, Widget, WidgetType},
};
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

    Ok((
        "",
        Widget {
            pos: (x, y).into(),
            widget_type: WidgetType::Text {
                value: rest.to_string(),
            },
        },
    ))
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
            Widget::new_generic(
                (x, y),
                length,
                name,
                rest,
                "",
                None::<Vec<char>>,
                None,
                Select::None,
            ),
            /*
                Widget {
                    pos: (x, y).into(),
                    widget_type: WidgetType::Generic {
                        length,
                        name: name.to_string(),
                        value: rest.to_string(),
                        default_value: rest.to_string(),
                        allowed_characters: None,
                        mask_char: None,
                    },
                },
            */
        )),
        "PASSWORD" => Ok((
            "",
            Widget::new_generic(
                (x, y),
                length,
                name,
                rest,
                "",
                None::<Vec<char>>,
                Some('*'),
                Select::None,
            ), /*
               Widget {
                   pos: (x, y).into(),
                   widget_type: WidgetType::Generic {
                       length,
                       name: name.to_string(),
                       value: rest.to_string(),
                       default_value: "".to_string(),
                       allowed_characters: None,
                       mask_char: Some('X'),
                   },
               },
                   */
        )),
        _ => unimplemented!(),
    }
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
            Widget::new_generic(
                (x, y),
                length,
                name,
                &default_value,
                &default_value,
                Some(('0'..='9').collect::<Vec<char>>()),
                None,
                Select::None,
            ),
            /*
            Widget {
                pos: (x, y).into(),
                widget_type: WidgetType::Generic {
                    length,
                    name: name.to_string(),
                    value: default_value.clone(),
                    default_value,
                    allowed_characters: Some(('0'..='9').collect::<Vec<char>>()),
                    mask_char: None,
                },
            },
            */
        )),
        _ => unimplemented!(),
    }
}

pub fn parse_widget(input: &str) -> Result<Widget, String> {
    let (_, widget) =
        alt((parse_label, parse_input, parse_number))(input).map_err(|e| e.to_string())?;

    Ok(widget)
}

pub fn parse_str(form: &mut Form, input: &str) -> Result<(), String> {
    let widget = parse_widget(input)?;

    form.add_widget(widget);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_label() {
        let widget = parse_widget("LABEL 1 2 texti hér").unwrap();

        assert_eq!(widget.pos, (1, 2).into());
        assert_eq!(
            widget.widget_type,
            WidgetType::Text {
                value: "texti hér".to_string()
            }
        );
    }

    #[test]
    fn test_parse_input() {
        let widget = parse_widget("INPUT 5 111 10 nafn texti hér").unwrap();

        assert_eq!(widget.pos, (5, 111).into());
        assert_eq!(
            widget.widget_type,
            WidgetType::Generic {
                length: 10,
                name: "nafn".to_string(),
                value: "texti hér".to_string(),
                default_value: "texti hér".to_string(),
                allowed_characters: None,
                mask_char: None,
                select: Select::None,
            }
        );
    }
}
