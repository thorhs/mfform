use crate::{
    form::Form,
    widget::{Widget, WidgetType},
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

    let (rest, (_, x, _, y, _, length, _, name, _)) = tuple((
        tag("INPUT "),
        u16,
        multispace1,
        u16,
        multispace1,
        u16,
        multispace1,
        identifier,
        multispace0,
    ))(input)?;

    Ok((
        "",
        Widget {
            pos: (x, y).into(),
            widget_type: WidgetType::Input {
                length,
                name: name.to_string(),
                value: rest.to_string(),
                default_value: rest.to_string(),
            },
        },
    ))
}

pub fn parse_widget(input: &str) -> Result<Widget, String> {
    let (_, widget) = alt((parse_label, parse_input))(input).map_err(|e| e.to_string())?;

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
            WidgetType::Input {
                length: 10,
                name: "nafn".to_string(),
                value: "texti hér".to_string(),
                default_value: "texti hér".to_string(),
            }
        );
    }
}
