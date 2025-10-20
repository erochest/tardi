use pretty_assertions::{assert_eq, assert_str_eq};

use super::*;

#[test]
fn test_scanner_when_empty_input_then_empty_output() {
    let scanner = Scanner::from_string("");
    let output = scanner.map(|v| v.text).collect::<Vec<_>>();
    assert!(output.is_empty());
}

#[test]
fn test_scanner_scans_words() {
    let scanner = Scanner::from_string("a bb ccc");
    let output = scanner.map(|v| v.text).collect::<Vec<_>>().join(",");
    assert_str_eq!("a,bb,ccc", output);
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
    assert_eq!(Some(1), output[0].index);
    assert_eq!(Some(2), output[0].length);
    // 1
    assert_str_eq!("\"one\"", output[1].text);
    assert_eq!(Some(4), output[1].index);
    assert_eq!(Some(5), output[1].length);
    // 0
    assert_str_eq!("\"two words\"", output[2].text);
    assert_eq!(Some(10), output[2].index);
    assert_eq!(Some(11), output[2].length);
    // 0
    assert_str_eq!("\"three words full\"", output[3].text);
    assert_eq!(Some(22), output[3].index);
    assert_eq!(Some(18), output[3].length);
}

// TODO: string with quotes
