use pretty_assertions::assert_str_eq;

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
