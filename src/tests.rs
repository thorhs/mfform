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
