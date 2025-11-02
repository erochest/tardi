use pretty_assertions::{assert_eq, assert_str_eq};

use super::*;

#[test]
fn test_scanner_when_empty_input_then_empty_output() {
    let scanner = Scanner::from_string("");
    let output = scanner.collect::<Vec<_>>();
    assert!(output.is_empty());
}

fn assert_word(expected: &str, value: &Value) {
    assert!(matches!(value.data, ValueData::Word(_)));
    if let ValueData::Word(ref actual) = value.data {
        assert_str_eq!(expected, actual);
    }
}

fn assert_string(expected: &str, value: &Value) {
    assert!(matches!(value.data, ValueData::String(_)));
    if let ValueData::String(ref actual) = value.data {
        assert_str_eq!(expected, actual);
    }
}

// TODO: wrap `text` in `Option` and make the in-memory object 1st-class/non-optional
#[test]
fn test_scanner_scans_words() {
    let scanner = Scanner::from_string("a bb ccc");
    let output = scanner.collect::<Vec<_>>();
    let output_text = output
        .iter()
        .map(|v| v.text.clone())
        .collect::<Vec<_>>()
        .join(",");
    assert_str_eq!("a,bb,ccc", output_text);
    assert_word("a", &output[0]);
    assert_word("bb", &output[1]);
    assert_word("ccc", &output[2]);
}

#[test]
fn test_scanner_includes_position() {
    let scanner = Scanner::from_string("a bb ccc");
    let output = scanner
        .map(|v| v.index.map(|i| i.to_string()).unwrap_or_default())
        .collect::<Vec<_>>()
        .join(",");
    assert_str_eq!("0,2,5", output);
}

#[test]
fn test_scanner_includes_length() {
    let scanner = Scanner::from_string("a bb ccc");
    let output = scanner
        .map(|v| v.length.map(|i| i.to_string()).unwrap_or_default())
        .collect::<Vec<_>>()
        .join(",");
    assert_str_eq!("1,2,3", output);
}

#[test]
fn test_scanner_scans_strings() {
    let scanner = Scanner::from_string(r#" "" "one" "two words" "three words full" "#);
    let output = scanner.collect::<Vec<_>>();
    assert_eq!(4, output.len());
    // 0
    assert_str_eq!("\"\"", output[0].text);
    assert_string("", &output[0]);
    assert_eq!(Some(1), output[0].index);
    assert_eq!(Some(2), output[0].length);
    // 1
    assert_str_eq!("\"one\"", output[1].text);
    assert_string("one", &output[1]);
    assert_eq!(Some(4), output[1].index);
    assert_eq!(Some(5), output[1].length);
    // 0
    assert_str_eq!("\"two words\"", output[2].text);
    assert_string("two words", &output[2]);
    assert_eq!(Some(10), output[2].index);
    assert_eq!(Some(11), output[2].length);
    // 0
    assert_str_eq!("\"three words full\"", output[3].text);
    assert_string("three words full", &output[3]);
    assert_eq!(Some(22), output[3].index);
    assert_eq!(Some(18), output[3].length);
}

#[test]
fn test_scanner_scan_strings_with_quotes() {
    let scanner = Scanner::from_string(
        r#"
        "I say, \"old man,\"" he chuckled. "\"Jolly\" good time."
        The clerk squinted at the other man, sucked in his teeth, and
        spat back, "\"no.\" It isn't."
        "#,
    );
    let output = scanner.collect::<Vec<_>>();

    assert_eq!(19, output.len());
    assert_str_eq!("\"I say, \\\"old man,\\\"\"", output[0].text);
    assert_string("I say, \"old man,\"", &output[0]);
    assert_str_eq!("\"\\\"Jolly\\\" good time.\"", output[3].text);
    assert_string("\"Jolly\" good time.", &output[3]);
    assert_str_eq!("\"\\\"no.\\\" It isn't.\"", output[18].text);
    assert_string("\"no.\" It isn't.", &output[18]);
}
