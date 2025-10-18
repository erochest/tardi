use super::*;

#[test]
fn test_scanner_when_empty_input_then_empty_output() {
    let scanner = Scanner::from_string("");
    let output = scanner.map(|v| v.text).collect::<Vec<_>>();
    assert!(output.is_empty());
}
