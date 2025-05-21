use std::iter::from_fn;

use super::*;
use crate::module::SANDBOX;
use crate::scanner::error::ScannerError;

// TODO: better tests for errors

fn scan_raw(input: &str) -> Vec<ScannerResult<Value>> {
    let mut scanner = Scanner::from_input_string(input);
    from_fn(|| scanner.scan_value()).collect()
}

fn scan(input: &str) -> Vec<Value> {
    let mut scanner = Scanner::from_input_string(input);
    let tokens = from_fn(|| scanner.scan_value());
    let tokens: ScannerResult<Vec<Value>> = tokens.collect();
    assert!(tokens.is_ok());
    tokens.unwrap()
}

fn top<T>(vector: &mut Vec<T>) -> T {
    vector.remove(0)
}

fn assert_top(
    tokens: &mut Vec<Value>,
    line: usize,
    column: usize,
    length: usize,
    lexeme: Option<&str>,
) -> Value {
    let value = top(tokens);

    assert!(value.pos.is_some());
    let pos = value.pos.as_ref().unwrap();
    assert_eq!(pos.line, line);
    assert_eq!(pos.column, column);
    assert_eq!(pos.length, length);

    let lexeme = lexeme.map(|s| s.to_string());
    assert_eq!(lexeme, value.lexeme, "{:?} != {:?}", lexeme, value);

    value.clone()
}

#[test]
fn test_scanner_position_tracking() {
    let mut tokens = scan("abc\ndef");

    // Initial position
    let token = assert_top(&mut tokens, 1, 1, 3, Some("abc"));
    let pos = token.pos.as_ref().unwrap();
    assert_eq!(pos.offset, 0);
    let token = assert_top(&mut tokens, 2, 1, 3, Some("def"));
    let pos = token.pos.as_ref().unwrap();
    assert_eq!(pos.offset, 4);
}

#[test]
fn test_scanner_whitespace_handling() {
    let mut tokens = scan("   abc   \n   def");

    // Test initial whitespace skipping
    let token = assert_top(&mut tokens, 1, 4, 3, Some("abc"));
    let pos = token.pos.as_ref().unwrap();
    assert_eq!(pos.offset, 3);

    // Test skipping spaces and newline
    let token = assert_top(&mut tokens, 2, 4, 3, Some("def"));
    let pos = token.pos.as_ref().unwrap();
    assert_eq!(pos.offset, 13);
}

#[test]
fn test_scan_integers() {
    let mut tokens = scan("42 123 0 -1");

    // Test "42"
    let token = assert_top(&mut tokens, 1, 1, 2, Some("42"));
    assert!(
        matches!(token.data, ValueData::Integer(42)),
        "not an int {:?}",
        token.data
    );

    // Test "123"
    let token = top(&mut tokens);
    assert!(matches!(token.data, ValueData::Integer(123)));
    assert_eq!(token.lexeme, Some("123".to_string()));
}

#[test]
fn test_scan_character_literals() {
    let mut tokens = scan("'a' '\\n' '\\t' '\\r' '\\'' '\\\\' '🦀' '\\u41' '\\u{1F600}'");

    // Test 'a'
    let token = top(&mut tokens);
    assert!(
        matches!(token.data, ValueData::Char('a')),
        "mismatched token type: {:?}",
        token.data
    );
    assert_eq!(token.lexeme, Some("'a'".to_string()));

    // Test '\n'
    let token = top(&mut tokens);
    assert!(
        matches!(token.data, ValueData::Char('\n')),
        "mismatched token type: {:?}",
        token.data
    );
    assert_eq!(token.lexeme, Some("'\\n'".to_string()));

    // Test '\t'
    let token = top(&mut tokens);
    assert!(
        matches!(token.data, ValueData::Char('\t')),
        "mismatched token type: {:?}",
        token.data
    );
    assert_eq!(token.lexeme, Some("'\\t'".to_string()));

    // Test '\r'
    let token = top(&mut tokens);
    assert!(
        matches!(token.data, ValueData::Char('\r')),
        "mismatched token type: {:?}",
        token.data
    );
    assert_eq!(token.lexeme, Some("'\\r'".to_string()));

    // Test '\''
    let token = top(&mut tokens);
    assert!(
        matches!(token.data, ValueData::Char('\'')),
        "mismatched token type: {:?}",
        token.data
    );
    assert_eq!(token.lexeme, Some("'\\''".to_string()));

    // Test '\\'
    let token = top(&mut tokens);
    assert!(
        matches!(token.data, ValueData::Char('\\')),
        "mismatched token type: {:?}",
        token.data
    );
    assert_eq!(token.lexeme, Some("'\\\\'".to_string()));

    // Test '🦀'
    let token = top(&mut tokens);
    assert!(
        matches!(token.data, ValueData::Char('🦀')),
        "mismatched token type: {:?}",
        token.data
    );
    assert_eq!(token.lexeme, Some("'🦀'".to_string()));

    // Test '\u41' (ASCII 'A')
    let token = top(&mut tokens);
    assert!(
        matches!(token.data, ValueData::Char('A')),
        "mismatched token type: {:?}",
        token.data
    );
    assert_eq!(token.lexeme, Some("'\\u41'".to_string()));

    // Test '\u{1F600}' (Unicode emoji 😀)
    let token = top(&mut tokens);
    assert!(
        matches!(token.data, ValueData::Char('😀')),
        "mismatched token type: {:?}",
        token.data
    );
    assert_eq!(token.lexeme, Some("'\\u{1F600}'".to_string()));
}

#[allow(clippy::approx_constant)]
#[test]
fn test_scan_floats() {
    let mut tokens = scan("3.14 2.0 0.123");

    // Test "3.14"
    let token = assert_top(&mut tokens, 1, 1, 4, Some("3.14"));
    assert!(matches!(token.data, ValueData::Float(3.14)));

    // TODO: These tests are commented out since we now parse these as words,
    // and we plan to support this notation in the future
    // Test invalid float formats
    // let mut tokens = scan("3. .14");
    // assert!(scanner.next().unwrap().is_err()); // "3." is invalid
    // assert!(scanner.next().unwrap().is_err()); // ".14" is invalid (no leading digit)
}

#[test]
fn test_scan_booleans() {
    let mut tokens = scan_raw("#t #f #x");

    // Test "#t"
    let token = top(&mut tokens);
    assert!(token.is_ok());
    let token = token.unwrap();
    let pos = token.pos.unwrap();
    assert_eq!(pos.line, 1);
    assert_eq!(pos.column, 1);
    assert_eq!(pos.length, 2);
    assert_eq!(token.lexeme, Some("#t".to_string()));
    assert!(matches!(token.data, ValueData::Boolean(true)));

    // Test "#f"
    let token = top(&mut tokens);
    assert!(token.is_ok());
    let token = token.unwrap();
    assert!(matches!(token.data, ValueData::Boolean(false)));
    assert_eq!(token.lexeme, Some("#f".to_string()));

    // Test error "#x"
    let token = top(&mut tokens);
    assert!(token.is_ok_and(|token| matches!(token.data, ValueData::Symbol { .. })));
}

#[test]
fn test_scan_comments() {
    let mut tokens =
        scan("42 // This is a comment\n<vector> // Another comment\ndup // Final comment");

    // Test "42"
    let token = top(&mut tokens);
    assert!(matches!(token.data, ValueData::Integer(42)));
    assert_eq!(token.lexeme, Some("42".to_string()));

    // Test "<vector>"
    let token = top(&mut tokens);
    assert!(
        matches!(token.data, ValueData::Symbol { word: ref w, .. } if w == "<vector>"),
        "not a symbol: {:?}",
        token.data
    );
    assert_eq!(token.lexeme, Some("<vector>".to_string()));

    // Test "dup"
    let token = top(&mut tokens);
    assert!(matches!(token.data, ValueData::Symbol { word: w, .. } if w == "dup"));
    assert_eq!(token.lexeme, Some("dup".to_string()));

    // Ensure no more tokens were read.
    assert!(tokens.is_empty());
}

#[test]
fn test_set_source() {
    let scanner = Scanner::from_input_string("something something here");
    assert_eq!(scanner.input, "something something here".to_string());
    assert_eq!(scanner.index, 0);
    assert_eq!(scanner.line, 1);
    assert_eq!(scanner.column, 1);
    assert_eq!(scanner.offset, 0);
}

#[test]
fn test_scan_token() {
    let mut scanner = Scanner::from_input_string("24 42 * word");

    let token = scanner.scan_value();
    assert!(matches!(token, Some(Ok(_))));
    let token = token.unwrap().unwrap();
    assert_eq!(
        token,
        Value::from_parts(ValueData::Integer(24), "24", 1, 1, 0, 2),
    );
    let token = scanner.scan_value();
    assert!(matches!(token, Some(Ok(_))));
    let token = token.unwrap().unwrap();
    assert_eq!(
        token,
        Value::from_parts(ValueData::Integer(42), "42", 1, 4, 3, 2),
    );
    let token = scanner.scan_value();
    assert!(matches!(token, Some(Ok(_))));
    let token = token.unwrap().unwrap();
    assert_eq!(
        token,
        Value::from_parts(
            ValueData::Symbol {
                module: SANDBOX.to_string(),
                word: "*".to_string()
            },
            "*",
            1,
            7,
            6,
            1
        ),
    );
    let token = scanner.scan_value();
    assert!(matches!(token, Some(Ok(_))));
    let token = token.unwrap().unwrap();
    assert_eq!(
        token,
        Value::from_parts(
            ValueData::Symbol {
                module: SANDBOX.to_string(),
                word: "word".to_string()
            },
            "word",
            1,
            9,
            8,
            4
        ),
    );
    let token = scanner.scan_value();
    assert!(token.is_none());
}

#[test]
fn test_scan_value_list() {
    let mut scanner = Scanner::from_input_string("\n: double 2 * ;\n7 double\n");

    let token = scanner.scan_value();
    assert!(token.is_some_and(|r| r.is_ok_and(|t| t.data
        == ValueData::Symbol {
            module: SANDBOX.to_string(),
            word: ":".to_string()
        })));

    let tokens = scanner.scan_value_list(ValueData::Word(";".to_string()));
    assert!(tokens.is_ok());
    let tokens = tokens.unwrap();
    let tokens = tokens.into_iter().collect::<ScannerResult<Vec<_>>>();
    assert!(tokens.is_ok());
    let tokens = tokens.unwrap();
    assert_eq!(tokens.len(), 3);
    assert_eq!(
        tokens[0],
        Value::from_parts(
            ValueData::Symbol {
                module: SANDBOX.to_string(),
                word: "double".to_string()
            },
            "double",
            2,
            3,
            3,
            6
        )
    );
    assert_eq!(
        tokens[1],
        Value::from_parts(ValueData::Integer(2), "2", 2, 10, 10, 1)
    );
    assert_eq!(
        tokens[2],
        Value::from_parts(
            ValueData::Symbol {
                module: SANDBOX.to_string(),
                word: "*".to_string()
            },
            "*",
            2,
            12,
            12,
            1
        )
    );

    let token = scanner.scan_value();
    assert!(token.is_some_and(|r| r.is_ok_and(|t| t.data == ValueData::Integer(7))));
}

#[test]
fn test_read_string_until() {
    let mut scanner = Scanner::from_input_string("\n<< double 2 * >>\n7 double\n");

    let token = scanner.scan_value();
    assert!(token.is_some_and(|r| r.is_ok_and(|t| t.data
        == ValueData::Symbol {
            module: SANDBOX.to_string(),
            word: "<<".to_string()
        })));

    let result = scanner.read_string_until(">>");
    assert!(result.is_ok(), "error on {:?}", result);
    let text_range = result.unwrap();
    assert_eq!(text_range, " double 2 * ".to_string());

    let token = scanner.scan_value();
    assert!(token.is_some_and(|r| r.is_ok_and(|t| t.data == ValueData::Integer(7))));

    let result = scanner.read_string_until(">>");
    assert!(matches!(result, Err(ScannerError::UnexpectedEndOfInput)));
}

#[test]
fn test_read_string_until_overlapping_delimiters() {
    let mut scanner = Scanner::from_input_string("bcababa");

    let result = scanner.read_string_until("aba");
    assert!(result.is_ok(), "error on {:?}", result);
    let text_range = result.unwrap();
    assert_eq!(text_range, "bc".to_string());
}

#[test]
fn test_words_starting_with_numbers() {
    let mut scanner = Scanner::from_input_string("123abc");
    let token = scanner.scan_value();
    assert!(token.is_some_and(|r| r.is_ok_and(|t| t.data
        == ValueData::Symbol {
            module: SANDBOX.to_string(),
            word: "123abc".to_string()
        })));
}

#[test]
fn test_multi_byte_utf8_characters() {
    let mut scanner = Scanner::from_input_string("こんにちは world");
    let token = scanner.scan_value();
    assert!(token.is_some_and(|r| r.is_ok_and(|t| t.data
        == ValueData::Symbol {
            module: SANDBOX.to_string(),
            word: "こんにちは".to_string()
        })));

    let token = scanner.scan_value();
    assert!(token.is_some_and(|r| r.is_ok_and(|t| t.data
        == ValueData::Symbol {
            module: SANDBOX.to_string(),
            word: "world".to_string()
        })));

    assert_eq!(scanner.line, 1);
    assert_eq!(scanner.column, 12); // 'world' starts at column 12 (1-based)
    assert_eq!(scanner.offset, 21); // 'こんにちは' is 15 bytes + 1 space + 5 bytes for 'world'
}
