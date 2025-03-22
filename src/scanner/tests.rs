use super::*;

#[test]
fn test_scanner_position_tracking() {
    let mut scanner = Scanner::new("abc\ndef");

    // Initial position
    assert_eq!(scanner.line, 1);
    assert_eq!(scanner.column, 1);
    assert_eq!(scanner.offset, 0);

    // Advance through first line
    scanner.next_char(); // 'a'
    assert_eq!(scanner.line, 1);
    assert_eq!(scanner.column, 2);
    assert_eq!(scanner.offset, 1);

    scanner.next_char(); // 'b'
    assert_eq!(scanner.line, 1);
    assert_eq!(scanner.column, 3);
    assert_eq!(scanner.offset, 2);

    scanner.next_char(); // 'c'
    assert_eq!(scanner.line, 1);
    assert_eq!(scanner.column, 4);
    assert_eq!(scanner.offset, 3);

    scanner.next_char(); // '\n'
    assert_eq!(scanner.line, 2);
    assert_eq!(scanner.column, 1);
    assert_eq!(scanner.offset, 4);

    // First character of second line
    scanner.next_char(); // 'd'
    assert_eq!(scanner.line, 2);
    assert_eq!(scanner.column, 2);
    assert_eq!(scanner.offset, 5);
}

#[test]
fn test_scanner_whitespace_handling() {
    let mut scanner = Scanner::new("   abc   \n   def");

    // Test initial whitespace skipping
    scanner.skip_whitespace();
    assert_eq!(scanner.offset, 3); // Skipped 3 spaces
    assert_eq!(scanner.column, 4); // Column should be 4 (1-based indexing)
    assert_eq!(scanner.line, 1); // Still on the first line

    // Test reading non-whitespace
    assert_eq!(scanner.next_char(), Some('a')); // 'a'
    assert_eq!(scanner.column, 5); // 'a' is at column 5
    assert_eq!(scanner.next_char(), Some('b')); // 'b'
    assert_eq!(scanner.next_char(), Some('c')); // 'c'

    // Test skipping spaces and newline
    scanner.skip_whitespace();
    assert_eq!(scanner.offset, 13); // Skipped 3 more spaces
    assert_eq!(scanner.column, 4); // Column 4 on the second line
    assert_eq!(scanner.line, 2); // Still on the second line

    // Test skipping when there is no whitespace
    scanner.skip_whitespace();
    assert_eq!(scanner.offset, 13); // Skipped 3 more spaces
    assert_eq!(scanner.column, 4); // Column 4 on the second line
    assert_eq!(scanner.line, 2); // Still on the second line

    // Test reading after whitespace
    assert_eq!(scanner.next_char(), Some('d')); // 'd'
    assert_eq!(scanner.column, 5); // 'd' is at column 5 of the second line
}

#[test]
fn test_scan_integers() {
    let mut scanner = Scanner::new("42 123 0 -1");

    // Test "42"
    // TODO: refactor away from this pattern that uses an explicit `panic!`
    if let Some(Ok(token)) = scanner.next() {
        assert!(matches!(token.token_type, TokenType::Integer(42)));
        assert_eq!(token.line, 1);
        assert_eq!(token.column, 1);
        assert_eq!(token.length, 2);
        assert_eq!(token.lexeme, "42");
    } else {
        panic!("Failed to scan integer");
    }

    // Test "123"
    if let Some(Ok(token)) = scanner.next() {
        assert!(matches!(token.token_type, TokenType::Integer(123)));
        assert_eq!(token.lexeme, "123");
    } else {
        panic!("Failed to scan integer");
    }
}

#[test]
fn test_scan_character_literals() {
    let mut scanner = Scanner::new("'a' '\\n' '\\t' '\\r' '\\'' '\\\\' '🦀' '\\u41' '\\u{1F600}'");

    // Test 'a'
    if let Some(Ok(token)) = scanner.next() {
        assert!(matches!(token.token_type, TokenType::Char('a')));
        assert_eq!(token.lexeme, "'a'");
    } else {
        panic!("Failed to scan character 'a'");
    }

    // Test '\n'
    if let Some(Ok(token)) = scanner.next() {
        assert!(matches!(token.token_type, TokenType::Char('\n')));
        assert_eq!(token.lexeme, "'\\n'");
    } else {
        panic!("Failed to scan character '\\n'");
    }

    // Test '\t'
    if let Some(Ok(token)) = scanner.next() {
        assert!(matches!(token.token_type, TokenType::Char('\t')));
        assert_eq!(token.lexeme, "'\\t'");
    } else {
        panic!("Failed to scan character '\\t'");
    }

    // Test '\r'
    if let Some(Ok(token)) = scanner.next() {
        assert!(matches!(token.token_type, TokenType::Char('\r')));
        assert_eq!(token.lexeme, "'\\r'");
    } else {
        panic!("Failed to scan character '\\r'");
    }

    // Test '\''
    if let Some(Ok(token)) = scanner.next() {
        assert!(matches!(token.token_type, TokenType::Char('\'')));
        assert_eq!(token.lexeme, "'\\''");
    } else {
        panic!("Failed to scan character '\\''");
    }

    // Test '\\'
    if let Some(Ok(token)) = scanner.next() {
        assert!(matches!(token.token_type, TokenType::Char('\\')));
        assert_eq!(token.lexeme, "'\\\\'");
    } else {
        panic!("Failed to scan character '\\\\'");
    }

    // Test '🦀'
    if let Some(Ok(token)) = scanner.next() {
        assert!(matches!(token.token_type, TokenType::Char('🦀')));
        assert_eq!(token.lexeme, "'🦀'");
    } else {
        panic!("Failed to scan character '🦀'");
    }

    // Test '\u41' (ASCII 'A')
    if let Some(Ok(token)) = scanner.next() {
        assert!(matches!(token.token_type, TokenType::Char('A')));
        assert_eq!(token.lexeme, "'\\u41'");
    } else {
        panic!("Failed to scan character '\\u41'");
    }

    // Test '\u{1F600}' (Unicode emoji 😀)
    if let Some(Ok(token)) = scanner.next() {
        assert!(matches!(token.token_type, TokenType::Char('😀')));
        assert_eq!(token.lexeme, "'\\u{1F600}'");
    } else {
        panic!("{}", "Failed to scan character '\\u{1F600}'");
    }
}

#[test]
fn test_invalid_character_literals() {
    // Test unterminated character
    let mut scanner = Scanner::new("'a");
    assert!(scanner.next().unwrap().is_err());

    // Test empty character literal
    let mut scanner = Scanner::new("''");
    assert!(scanner.next().unwrap().is_err());

    // Test invalid escape sequence
    let mut scanner = Scanner::new("'\\x'");
    assert!(scanner.next().unwrap().is_err());

    // Test invalid Unicode escape
    let mut scanner = Scanner::new("'\\u{FFFFFFFF}'");
    assert!(scanner.next().unwrap().is_err());

    // Test invalid ASCII escape
    let mut scanner = Scanner::new("'\\u80'");
    assert!(scanner.next().unwrap().is_err());
}

#[test]
fn test_scan_return_stack_operations() {
    let mut scanner = Scanner::new(">r r> r@");

    // Test ">r"
    if let Some(Ok(token)) = scanner.next() {
        assert!(matches!(token.token_type, TokenType::ToR));
        assert_eq!(token.line, 1);
        assert_eq!(token.column, 1);
        assert_eq!(token.length, 2);
        assert_eq!(token.lexeme, ">r");
    } else {
        panic!("Failed to scan >r");
    }

    // Test "r>"
    if let Some(Ok(token)) = scanner.next() {
        assert!(matches!(token.token_type, TokenType::RFrom));
        assert_eq!(token.lexeme, "r>");
    } else {
        panic!("Failed to scan r>");
    }

    // Test "r@"
    if let Some(Ok(token)) = scanner.next() {
        assert!(matches!(token.token_type, TokenType::RFetch));
        assert_eq!(token.lexeme, "r@");
    } else {
        panic!("Failed to scan r@");
    }
}

#[test]
fn test_scan_floats() {
    let mut scanner = Scanner::new("3.14 2.0 0.123");

    // Test "3.14"
    if let Some(Ok(token)) = scanner.next() {
        assert!(matches!(token.token_type, TokenType::Float(3.14)));
        assert_eq!(token.line, 1);
        assert_eq!(token.column, 1);
        assert_eq!(token.length, 4);
        assert_eq!(token.lexeme, "3.14");
    } else {
        panic!("Failed to scan float");
    }

    // TODO: These tests are commented out since we now parse these as words,
    // and we plan to support this notation in the future
    // Test invalid float formats
    // let mut scanner = Scanner::new("3. .14");
    // assert!(scanner.next().unwrap().is_err()); // "3." is invalid
    // assert!(scanner.next().unwrap().is_err()); // ".14" is invalid (no leading digit)
}

#[test]
fn test_scan_booleans() {
    let mut scanner = Scanner::new("#t #f #x");

    // Test "#t"
    if let Some(Ok(token)) = scanner.next() {
        assert!(matches!(token.token_type, TokenType::Boolean(true)));
        assert_eq!(token.line, 1);
        assert_eq!(token.column, 1);
        assert_eq!(token.length, 2);
        assert_eq!(token.lexeme, "#t");
    } else {
        panic!("Failed to scan boolean");
    }

    // Test "#f"
    if let Some(Ok(token)) = scanner.next() {
        assert!(matches!(token.token_type, TokenType::Boolean(false)));
        assert_eq!(token.lexeme, "#f");
    } else {
        panic!("Failed to scan boolean");
    }

    // Test invalid boolean
    assert!(scanner.next().unwrap().is_err()); // "#x" is not a valid boolean
}

#[test]
fn test_scan_stack_operations() {
    let mut scanner = Scanner::new("dup swap rot drop");

    // Test "dup"
    if let Some(Ok(token)) = scanner.next() {
        assert!(matches!(token.token_type, TokenType::Dup));
        assert_eq!(token.line, 1);
        assert_eq!(token.column, 1);
        assert_eq!(token.length, 3);
        assert_eq!(token.lexeme, "dup");
    } else {
        panic!("Failed to scan dup");
    }

    // Test "swap"
    if let Some(Ok(token)) = scanner.next() {
        assert!(matches!(token.token_type, TokenType::Swap));
        assert_eq!(token.lexeme, "swap");
    } else {
        panic!("Failed to scan swap");
    }

    // Test "rot"
    if let Some(Ok(token)) = scanner.next() {
        assert!(matches!(token.token_type, TokenType::Rot));
        assert_eq!(token.lexeme, "rot");
    } else {
        panic!("Failed to scan rot");
    }

    // Test "drop"
    if let Some(Ok(token)) = scanner.next() {
        assert!(matches!(token.token_type, TokenType::Drop));
        assert_eq!(token.lexeme, "drop");
    } else {
        panic!("Failed to scan drop");
    }
}

#[test]
fn test_scan_arithmetic_operators() {
    let mut scanner = Scanner::new("+ - * /");

    // Test "+"
    if let Some(Ok(token)) = scanner.next() {
        assert!(matches!(token.token_type, TokenType::Plus));
        assert_eq!(token.line, 1);
        assert_eq!(token.column, 1);
        assert_eq!(token.length, 1);
        assert_eq!(token.lexeme, "+");
    } else {
        panic!("Failed to scan plus");
    }

    // Test "-"
    if let Some(Ok(token)) = scanner.next() {
        assert!(matches!(token.token_type, TokenType::Dash));
        assert_eq!(token.lexeme, "-");
    } else {
        panic!("Failed to scan dash");
    }

    // Test "*"
    if let Some(Ok(token)) = scanner.next() {
        assert!(matches!(token.token_type, TokenType::Star));
        assert_eq!(token.lexeme, "*");
    } else {
        panic!("Failed to scan star");
    }

    // Test "/"
    if let Some(Ok(token)) = scanner.next() {
        assert!(matches!(token.token_type, TokenType::Slash));
        assert_eq!(token.lexeme, "/");
    } else {
        panic!("Failed to scan slash");
    }
}

#[test]
fn test_scan_comparison_operators_and_words() {
    let mut scanner = Scanner::new("== != < > <= >= custom_word");

    // Test "=="
    if let Some(Ok(token)) = scanner.next() {
        assert!(matches!(token.token_type, TokenType::EqualEqual));
        assert_eq!(token.lexeme, "==");
    } else {
        panic!("Failed to scan ==");
    }

    // Test "!="
    if let Some(Ok(token)) = scanner.next() {
        assert!(matches!(token.token_type, TokenType::BangEqual));
        assert_eq!(token.lexeme, "!=");
    } else {
        panic!("Failed to scan !=");
    }

    // Test "<"
    if let Some(Ok(token)) = scanner.next() {
        assert!(matches!(token.token_type, TokenType::Less));
        assert_eq!(token.lexeme, "<");
    } else {
        panic!("Failed to scan <");
    }

    // Test ">"
    if let Some(Ok(token)) = scanner.next() {
        assert!(matches!(token.token_type, TokenType::Greater));
        assert_eq!(token.lexeme, ">");
    } else {
        panic!("Failed to scan >");
    }

    // Test "<="
    if let Some(Ok(token)) = scanner.next() {
        assert!(matches!(token.token_type, TokenType::LessEqual));
        assert_eq!(token.lexeme, "<=");
    } else {
        panic!("Failed to scan <=");
    }

    // Test ">="
    if let Some(Ok(token)) = scanner.next() {
        assert!(matches!(token.token_type, TokenType::GreaterEqual));
        assert_eq!(token.lexeme, ">=");
    } else {
        panic!("Failed to scan >=");
    }

    // Test custom word
    if let Some(Ok(token)) = scanner.next() {
        assert!(matches!(token.token_type, TokenType::Word(word) if word == "custom_word"));
        assert_eq!(token.lexeme, "custom_word");
    } else {
        panic!("Failed to scan custom word");
    }
}
