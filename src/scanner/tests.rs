use super::*;

fn scan_raw(input: &str) -> Vec<Result<Token>> {
    let mut scanner = Scanner::new();

    Scan::scan(&mut scanner, input)
}

fn scan(input: &str) -> Vec<Token> {
    let tokens = scan_raw(input);
    let tokens = tokens.into_iter().collect::<Result<Vec<_>>>();

    assert!(tokens.is_ok());

    tokens.unwrap()
}

#[test]
fn test_scanner_position_tracking() {
    let mut tokens = scan("abc\ndef");

    // Initial position
    let token = tokens.remove(0);
    assert_eq!(token.line, 1);
    assert_eq!(token.column, 1);
    assert_eq!(token.offset, 0);

    let token = tokens.remove(0);
    assert_eq!(token.line, 2);
    assert_eq!(token.column, 1);
    assert_eq!(token.offset, 4);
}

#[test]
fn test_scanner_whitespace_handling() {
    let mut tokens = scan("   abc   \n   def");

    // Test initial whitespace skipping
    let token = tokens.remove(0);
    assert_eq!(token.offset, 3); // Skipped 3 spaces
    assert_eq!(token.column, 4); // Column should be 4 (1-based indexing)
    assert_eq!(token.line, 1); // Still on the first line

    // Test skipping spaces and newline
    let token = tokens.remove(0);
    assert_eq!(token.offset, 13); // Skipped 3 more spaces
    assert_eq!(token.column, 4); // Column 4 on the second line
    assert_eq!(token.line, 2); // Still on the second line
}

#[test]
fn test_scan_integers() {
    let mut tokens = scan("42 123 0 -1");

    // Test "42"
    let token = tokens.remove(0);
    assert!(matches!(token.token_type, TokenType::Integer(42)));
    assert_eq!(token.line, 1);
    assert_eq!(token.column, 1);
    assert_eq!(token.length, 2);
    assert_eq!(token.lexeme, "42");

    // Test "123"
    let token = tokens.remove(0);
    assert!(matches!(token.token_type, TokenType::Integer(123)));
    assert_eq!(token.lexeme, "123");
}

#[test]
fn test_scan_character_literals() {
    let mut tokens = scan("'a' '\\n' '\\t' '\\r' '\\'' '\\\\' '🦀' '\\u41' '\\u{1F600}'");

    // Test 'a'
    let token = tokens.remove(0);
    assert!(
        matches!(token.token_type, TokenType::Char('a')),
        "mismatched token type: {:?}",
        token.token_type
    );
    assert_eq!(token.lexeme, "'a'");

    // Test '\n'
    let token = tokens.remove(0);
    assert!(
        matches!(token.token_type, TokenType::Char('\n')),
        "mismatched token type: {:?}",
        token.token_type
    );
    assert_eq!(token.lexeme, "'\\n'");

    // Test '\t'
    let token = tokens.remove(0);
    assert!(
        matches!(token.token_type, TokenType::Char('\t')),
        "mismatched token type: {:?}",
        token.token_type
    );
    assert_eq!(token.lexeme, "'\\t'");

    // Test '\r'
    let token = tokens.remove(0);
    assert!(
        matches!(token.token_type, TokenType::Char('\r')),
        "mismatched token type: {:?}",
        token.token_type
    );
    assert_eq!(token.lexeme, "'\\r'");

    // Test '\''
    let token = tokens.remove(0);
    assert!(
        matches!(token.token_type, TokenType::Char('\'')),
        "mismatched token type: {:?}",
        token.token_type
    );
    assert_eq!(token.lexeme, "'\\''");

    // Test '\\'
    let token = tokens.remove(0);
    assert!(
        matches!(token.token_type, TokenType::Char('\\')),
        "mismatched token type: {:?}",
        token.token_type
    );
    assert_eq!(token.lexeme, "'\\\\'");

    // Test '🦀'
    let token = tokens.remove(0);
    assert!(
        matches!(token.token_type, TokenType::Char('🦀')),
        "mismatched token type: {:?}",
        token.token_type
    );
    assert_eq!(token.lexeme, "'🦀'");

    // Test '\u41' (ASCII 'A')
    let token = tokens.remove(0);
    assert!(
        matches!(token.token_type, TokenType::Char('A')),
        "mismatched token type: {:?}",
        token.token_type
    );
    assert_eq!(token.lexeme, "'\\u41'");

    // Test '\u{1F600}' (Unicode emoji 😀)
    let token = tokens.remove(0);
    assert!(
        matches!(token.token_type, TokenType::Char('😀')),
        "mismatched token type: {:?}",
        token.token_type
    );
    assert_eq!(token.lexeme, "'\\u{1F600}'");
}

#[test]
fn test_invalid_character_literals() {
    // Test unterminated character
    let mut tokens = scan_raw("'a").into_iter();
    assert!(tokens.next().unwrap().is_err());

    // Test empty character literal
    let mut tokens = scan_raw("''").into_iter();
    assert!(tokens.next().unwrap().is_err());

    // Test invalid escape sequence
    let mut tokens = scan_raw("'\\x'").into_iter();
    assert!(tokens.next().unwrap().is_err());

    // Test invalid Unicode escape
    let mut tokens = scan_raw("'\\u{FFFFFFFF}'").into_iter();
    assert!(tokens.next().unwrap().is_err());

    // Test invalid ASCII escape
    let mut tokens = scan_raw("'\\u80'").into_iter();
    assert!(tokens.next().unwrap().is_err());
}

#[test]
fn test_scan_return_stack_operations() {
    let mut tokens = scan(">r r> r@");

    // Test ">r"
    let token = tokens.remove(0);
    assert!(matches!(token.token_type, TokenType::ToR));
    assert_eq!(token.line, 1);
    assert_eq!(token.column, 1);
    assert_eq!(token.length, 2);
    assert_eq!(token.lexeme, ">r");

    // Test "r>"
    let token = tokens.remove(0);
    assert!(matches!(token.token_type, TokenType::RFrom));
    assert_eq!(token.lexeme, "r>");

    // Test "r@"
    let token = tokens.remove(0);
    assert!(matches!(token.token_type, TokenType::RFetch));
    assert_eq!(token.lexeme, "r@");
}

#[test]
fn test_scan_floats() {
    let mut tokens = scan("3.14 2.0 0.123");

    // Test "3.14"
    let token = tokens.remove(0);
    assert!(matches!(token.token_type, TokenType::Float(3.14)));
    assert_eq!(token.line, 1);
    assert_eq!(token.column, 1);
    assert_eq!(token.length, 4);
    assert_eq!(token.lexeme, "3.14");

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
    let token = tokens.remove(0);
    assert!(token.is_ok(), "token error {:?}", token);
    let token = token.unwrap();
    assert!(matches!(token.token_type, TokenType::Boolean(true)));
    assert_eq!(token.line, 1);
    assert_eq!(token.column, 1);
    assert_eq!(token.length, 2);
    assert_eq!(token.lexeme, "#t");

    // Test "#f"
    let token = tokens.remove(0);
    assert!(token.is_ok());
    let token = token.unwrap();
    assert!(matches!(token.token_type, TokenType::Boolean(false)));
    assert_eq!(token.lexeme, "#f");

    // Test invalid boolean
    let token = tokens.remove(0);
    assert!(token.is_err()); // "#x" is not a valid boolean
}

#[test]
fn test_scan_stack_operations() {
    let mut tokens = scan("dup swap rot drop");

    // Test "dup"
    let token = tokens.remove(0);
    assert!(matches!(token.token_type, TokenType::Dup));
    assert_eq!(token.line, 1);
    assert_eq!(token.column, 1);
    assert_eq!(token.length, 3);
    assert_eq!(token.lexeme, "dup");

    // Test "swap"
    let token = tokens.remove(0);
    assert!(matches!(token.token_type, TokenType::Swap));
    assert_eq!(token.lexeme, "swap");

    // Test "rot"
    let token = tokens.remove(0);
    assert!(matches!(token.token_type, TokenType::Rot));
    assert_eq!(token.lexeme, "rot");

    // Test "drop"
    let token = tokens.remove(0);
    assert!(matches!(token.token_type, TokenType::Drop));
    assert_eq!(token.lexeme, "drop");
}

#[test]
fn test_scan_arithmetic_operators() {
    let mut tokens = scan("+ - * /");

    // Test "+"
    let token = tokens.remove(0);
    if !matches!(token.token_type, TokenType::Plus) {
        panic!("Expected Plus token, got {:?}", token.token_type);
    }
    assert_eq!(token.line, 1);
    assert_eq!(token.column, 1);
    assert_eq!(token.length, 1);
    assert_eq!(token.lexeme, "+");

    // Test "-"
    let token = tokens.remove(0);
    assert!(matches!(token.token_type, TokenType::Dash));
    assert_eq!(token.lexeme, "-");

    // Test "*"
    let token = tokens.remove(0);
    assert!(matches!(token.token_type, TokenType::Star));
    assert_eq!(token.lexeme, "*");

    // Test "/"
    let token = tokens.remove(0);
    if !matches!(token.token_type, TokenType::Slash) {
        panic!("Expected Slash token, got {:?}", token.token_type);
    }
    assert_eq!(token.lexeme, "/");
}

#[test]
fn test_scan_comparison_operators_and_words() {
    let mut tokens = scan("== != < > <= >= custom_word");

    // Test "=="
    let token = tokens.remove(0);
    assert!(matches!(token.token_type, TokenType::EqualEqual));
    assert_eq!(token.lexeme, "==");

    // Test "!="
    let token = tokens.remove(0);
    assert!(matches!(token.token_type, TokenType::BangEqual));
    assert_eq!(token.lexeme, "!=");

    // Test "<"
    let token = tokens.remove(0);
    assert!(matches!(token.token_type, TokenType::Less));
    assert_eq!(token.lexeme, "<");

    // Test ">"
    let token = tokens.remove(0);
    assert!(matches!(token.token_type, TokenType::Greater));
    assert_eq!(token.lexeme, ">");

    // Test "<="
    let token = tokens.remove(0);
    assert!(matches!(token.token_type, TokenType::LessEqual));
    assert_eq!(token.lexeme, "<=");

    // Test ">="
    let token = tokens.remove(0);
    assert!(matches!(token.token_type, TokenType::GreaterEqual));
    assert_eq!(token.lexeme, ">=");

    // Test custom word
    let token = tokens.remove(0);
    assert!(matches!(token.token_type, TokenType::Word(word) if word == "custom_word"));
    assert_eq!(token.lexeme, "custom_word");
}

#[test]
fn test_scan_comments() {
    let mut tokens =
        scan("42 // This is a comment\n<list> // Another comment\ndup // Final comment");

    // Test "42"
    let token = tokens.remove(0);
    assert!(matches!(token.token_type, TokenType::Integer(42)));
    assert_eq!(token.lexeme, "42");

    // Test "<list>"
    let token = tokens.remove(0);
    assert!(matches!(token.token_type, TokenType::CreateList));
    assert_eq!(token.lexeme, "<list>");

    // Test "dup"
    let token = tokens.remove(0);
    assert!(matches!(token.token_type, TokenType::Dup));
    assert_eq!(token.lexeme, "dup");

    // Ensure there are no more tokens
    assert!(tokens.is_empty());
}
