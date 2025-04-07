use crate::Tardi;

use super::*;

// TODO: better tests for errors

fn scan_raw(input: &str) -> Vec<Result<Token>> {
    let mut tardi = Tardi::default();
    tardi.scan(input).unwrap()
}

fn scan(input: &str) -> Vec<Token> {
    let mut tardi = Tardi::default();
    let tokens = tardi.scan(input);
    assert!(tokens.is_ok());
    let tokens = tokens.unwrap();
    let tokens: Result<Vec<Token>> = tokens.into_iter().collect();
    assert!(tokens.is_ok());
    tokens.unwrap()
}

fn top<T>(vector: &mut Vec<T>) -> T {
    let item = vector.remove(0);
    item
}

fn assert_top(
    tokens: &mut Vec<Token>,
    line: usize,
    column: usize,
    length: usize,
    lexeme: Option<&str>,
) -> Token {
    let token = top(tokens);
    assert_eq!(token.line, line);
    assert_eq!(token.column, column);
    assert_eq!(token.length, length);
    if let Some(lexeme) = lexeme {
        assert_eq!(lexeme, token.lexeme);
    }
    token.clone()
}

#[test]
fn test_scanner_position_tracking() {
    let mut tokens = scan("abc\ndef");

    // Initial position
    let token = assert_top(&mut tokens, 1, 1, 3, None);
    assert_eq!(token.offset, 0);
    let token = assert_top(&mut tokens, 2, 1, 3, None);
    assert_eq!(token.offset, 4);
}

#[test]
fn test_scanner_whitespace_handling() {
    let mut tokens = scan("   abc   \n   def");

    // Test initial whitespace skipping
    let token = assert_top(&mut tokens, 1, 4, 3, None);
    assert_eq!(token.offset, 3);

    // Test skipping spaces and newline
    let token = assert_top(&mut tokens, 2, 4, 3, None);
    assert_eq!(token.offset, 13);
}

#[test]
fn test_scan_integers() {
    let mut tokens = scan("42 123 0 -1");

    // Test "42"
    let token = assert_top(&mut tokens, 1, 1, 2, Some("42"));
    assert!(matches!(token.token_type, TokenType::Integer(42)));

    // Test "123"
    let token = top(&mut tokens);
    assert!(matches!(token.token_type, TokenType::Integer(123)));
    assert_eq!(token.lexeme, "123");
}

#[test]
fn test_scan_character_literals() {
    let mut tokens = scan("'a' '\\n' '\\t' '\\r' '\\'' '\\\\' '🦀' '\\u41' '\\u{1F600}'");

    // Test 'a'
    let token = top(&mut tokens);
    assert!(
        matches!(token.token_type, TokenType::Char('a')),
        "mismatched token type: {:?}",
        token.token_type
    );
    assert_eq!(token.lexeme, "'a'");

    // Test '\n'
    let token = top(&mut tokens);
    assert!(
        matches!(token.token_type, TokenType::Char('\n')),
        "mismatched token type: {:?}",
        token.token_type
    );
    assert_eq!(token.lexeme, "'\\n'");

    // Test '\t'
    let token = top(&mut tokens);
    assert!(
        matches!(token.token_type, TokenType::Char('\t')),
        "mismatched token type: {:?}",
        token.token_type
    );
    assert_eq!(token.lexeme, "'\\t'");

    // Test '\r'
    let token = top(&mut tokens);
    assert!(
        matches!(token.token_type, TokenType::Char('\r')),
        "mismatched token type: {:?}",
        token.token_type
    );
    assert_eq!(token.lexeme, "'\\r'");

    // Test '\''
    let token = top(&mut tokens);
    assert!(
        matches!(token.token_type, TokenType::Char('\'')),
        "mismatched token type: {:?}",
        token.token_type
    );
    assert_eq!(token.lexeme, "'\\''");

    // Test '\\'
    let token = top(&mut tokens);
    assert!(
        matches!(token.token_type, TokenType::Char('\\')),
        "mismatched token type: {:?}",
        token.token_type
    );
    assert_eq!(token.lexeme, "'\\\\'");

    // Test '🦀'
    let token = top(&mut tokens);
    assert!(
        matches!(token.token_type, TokenType::Char('🦀')),
        "mismatched token type: {:?}",
        token.token_type
    );
    assert_eq!(token.lexeme, "'🦀'");

    // Test '\u41' (ASCII 'A')
    let token = top(&mut tokens);
    assert!(
        matches!(token.token_type, TokenType::Char('A')),
        "mismatched token type: {:?}",
        token.token_type
    );
    assert_eq!(token.lexeme, "'\\u41'");

    // Test '\u{1F600}' (Unicode emoji 😀)
    let token = top(&mut tokens);
    assert!(
        matches!(token.token_type, TokenType::Char('😀')),
        "mismatched token type: {:?}",
        token.token_type
    );
    assert_eq!(token.lexeme, "'\\u{1F600}'");
}

#[test]
fn test_scan_return_stack_operations() {
    let mut tokens = scan(">r r> r@");

    // Test ">r"
    let token = assert_top(&mut tokens, 1, 1, 2, Some(">r"));
    assert!(matches!(token.token_type, TokenType::ToR));

    // Test "r>"
    let token = top(&mut tokens);
    assert!(matches!(token.token_type, TokenType::RFrom));
    assert_eq!(token.lexeme, "r>");

    // Test "r@"
    let token = top(&mut tokens);
    assert!(matches!(token.token_type, TokenType::RFetch));
    assert_eq!(token.lexeme, "r@");
}

#[test]
fn test_scan_floats() {
    let mut tokens = scan("3.14 2.0 0.123");

    // Test "3.14"
    let token = assert_top(&mut tokens, 1, 1, 4, Some("3.14"));
    assert!(matches!(token.token_type, TokenType::Float(3.14)));

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
    assert_eq!(token.line, 1);
    assert_eq!(token.column, 1);
    assert_eq!(token.length, 2);
    assert_eq!(token.lexeme, "#t".to_string());
    assert!(matches!(token.token_type, TokenType::Boolean(true)));

    // Test "#f"
    let token = top(&mut tokens);
    assert!(token.is_ok());
    let token = token.unwrap();
    assert!(matches!(token.token_type, TokenType::Boolean(false)));
    assert_eq!(token.lexeme, "#f");

    // Test error "#x"
    let token = top(&mut tokens);
    assert!(token.is_err());
}

#[test]
fn test_scan_stack_operations() {
    let mut tokens = scan("dup swap rot drop");

    // Test "dup"
    let token = assert_top(&mut tokens, 1, 1, 3, Some("dup"));
    assert!(matches!(token.token_type, TokenType::Dup));

    // Test "swap"
    let token = top(&mut tokens);
    assert!(matches!(token.token_type, TokenType::Swap));
    assert_eq!(token.lexeme, "swap");

    // Test "rot"
    let token = top(&mut tokens);
    assert!(matches!(token.token_type, TokenType::Rot));
    assert_eq!(token.lexeme, "rot");

    // Test "drop"
    let token = top(&mut tokens);
    assert!(matches!(token.token_type, TokenType::Drop));
    assert_eq!(token.lexeme, "drop");
}

#[test]
fn test_scan_arithmetic_operators() {
    let mut tokens = scan("+ - * /");

    // Test "+"
    let token = assert_top(&mut tokens, 1, 1, 1, Some("+"));
    if !matches!(token.token_type, TokenType::Plus) {
        panic!("Expected Plus token, got {:?}", token.token_type);
    }

    // Test "-"
    let token = top(&mut tokens);
    assert!(matches!(token.token_type, TokenType::Dash));
    assert_eq!(token.lexeme, "-");

    // Test "*"
    let token = top(&mut tokens);
    assert!(matches!(token.token_type, TokenType::Star));
    assert_eq!(token.lexeme, "*");

    // Test "/"
    let token = top(&mut tokens);
    assert!(matches!(token.token_type, TokenType::Slash));
    assert_eq!(token.lexeme, "/");
}

#[test]
fn test_scan_comparison_operators_and_words() {
    let mut tokens = scan("== != < > <= >= custom_word");

    // Test "=="
    let token = top(&mut tokens);
    assert!(matches!(token.token_type, TokenType::EqualEqual));
    assert_eq!(token.lexeme, "==");

    // Test "!="
    let token = top(&mut tokens);
    assert!(matches!(token.token_type, TokenType::BangEqual));
    assert_eq!(token.lexeme, "!=");

    // Test "<"
    let token = top(&mut tokens);
    assert!(matches!(token.token_type, TokenType::Less));
    assert_eq!(token.lexeme, "<");

    // Test ">"
    let token = top(&mut tokens);
    assert!(matches!(token.token_type, TokenType::Greater));
    assert_eq!(token.lexeme, ">");

    // Test "<="
    let token = top(&mut tokens);
    assert!(matches!(token.token_type, TokenType::LessEqual));
    assert_eq!(token.lexeme, "<=");

    // Test ">="
    let token = top(&mut tokens);
    assert!(matches!(token.token_type, TokenType::GreaterEqual));
    assert_eq!(token.lexeme, ">=");

    // Test custom word
    let token = top(&mut tokens);
    assert!(matches!(token.token_type, TokenType::Word(word) if word == "custom_word"));
    assert_eq!(token.lexeme, "custom_word");
}

#[test]
fn test_scan_comments() {
    let mut tokens =
        scan("42 // This is a comment\n<list> // Another comment\ndup // Final comment");

    // Test "42"
    let token = top(&mut tokens);
    assert!(matches!(token.token_type, TokenType::Integer(42)));
    assert_eq!(token.lexeme, "42");

    // Test "<list>"
    let token = top(&mut tokens);
    assert!(matches!(token.token_type, TokenType::CreateList));
    assert_eq!(token.lexeme, "<list>");

    // Test "dup"
    let token = top(&mut tokens);
    assert!(matches!(token.token_type, TokenType::Dup));
    assert_eq!(token.lexeme, "dup");

    // Ensure there is only the EndOfInput token
    assert_eq!(tokens.len(), 1);
    assert_eq!(tokens[0].token_type, TokenType::EndOfInput);
}

#[test]
fn test_set_source() {
    let mut scanner = Scanner::new();
    scanner.set_source("something something here");
    assert_eq!(scanner.source, "something something here".to_string());
    assert_eq!(scanner.index, 0);
    assert_eq!(scanner.line, 1);
    assert_eq!(scanner.column, 1);
    assert_eq!(scanner.offset, 0);
}

#[test]
fn test_scan_token() {
    let mut scanner = Scanner::new();
    scanner.set_source("24 42 * word");

    let token = scanner.scan_token();
    assert!(matches!(token, Some(Ok(_))));
    let token = token.unwrap().unwrap();
    assert_eq!(
        token,
        Token {
            token_type: TokenType::Integer(24),
            line: 1,
            column: 1,
            offset: 0,
            length: 2,
            lexeme: "24".to_string()
        }
    );
    let token = scanner.scan_token();
    assert!(matches!(token, Some(Ok(_))));
    let token = token.unwrap().unwrap();
    assert_eq!(
        token,
        Token {
            token_type: TokenType::Integer(42),
            line: 1,
            column: 4,
            offset: 3,
            length: 2,
            lexeme: "42".to_string()
        }
    );
    let token = scanner.scan_token();
    assert!(matches!(token, Some(Ok(_))));
    let token = token.unwrap().unwrap();
    assert_eq!(
        token,
        Token {
            token_type: TokenType::Star,
            line: 1,
            column: 7,
            offset: 6,
            length: 1,
            lexeme: "*".to_string()
        }
    );
    let token = scanner.scan_token();
    assert!(matches!(token, Some(Ok(_))));
    let token = token.unwrap().unwrap();
    assert_eq!(
        token,
        Token {
            token_type: TokenType::Word("word".to_string()),
            line: 1,
            column: 9,
            offset: 8,
            length: 4,
            lexeme: "word".to_string()
        }
    );
    let token = scanner.scan_token();
    assert!(matches!(token, Some(Ok(_))));
    let token = token.unwrap().unwrap();
    assert_eq!(
        token,
        Token {
            token_type: TokenType::EndOfInput,
            line: 1,
            column: 13,
            offset: 12,
            length: 0,
            lexeme: "".to_string()
        }
    );
    let token = scanner.scan_token();
    assert!(token.is_none());
}
