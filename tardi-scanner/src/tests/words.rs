use pretty_assertions::assert_str_eq;

use super::super::*;

fn assert_word(expected: &str, value: &Value) {
    assert!(matches!(value.data, ValueData::Word(_)));
    if let ValueData::Word(ref actual) = value.data {
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
