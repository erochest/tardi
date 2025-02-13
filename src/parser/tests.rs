use std::{f64::consts::PI, iter::Iterator};

use pretty_assertions::assert_eq;

use super::*;

fn test_parse_token_types(input: &str, expected: Vec<TokenType>) {
    let result = parse(input).map(|t| t.into_iter().map(|t| t.token_type).collect::<Vec<_>>());

    assert!(result.is_ok(), "result error: {:?}", result);
    let result = result.unwrap();
    assert_eq!(result, expected);
}

#[test]
fn test_parse_skips_whitespace() {
    let input = "\n1\n";
    let expected = vec![TokenType::Integer(1)];
    test_parse_token_types(input, expected);
}

#[test]
fn test_parse_integer() {
    let input = "10 3";
    let expected = vec![TokenType::Integer(10), TokenType::Integer(3)];
    test_parse_token_types(input, expected);
}

#[test]
fn test_parse_divide() {
    let input = "10 3 /";
    let expected = vec![
        TokenType::Integer(10),
        TokenType::Integer(3),
        TokenType::Slash,
    ];
    test_parse_token_types(input, expected);
}

#[test]
fn test_parse_plus() {
    let input = "10 3 +";
    let expected = vec![
        TokenType::Integer(10),
        TokenType::Integer(3),
        TokenType::Plus,
    ];
    test_parse_token_types(input, expected);
}

#[test]
fn test_parse_minus() {
    let input = "10 3 -";
    let expected = vec![
        TokenType::Integer(10),
        TokenType::Integer(3),
        TokenType::Minus,
    ];
    test_parse_token_types(input, expected);
}

#[test]
fn test_parse_multiply() {
    let input = "10 3 *";
    let expected = vec![
        TokenType::Integer(10),
        TokenType::Integer(3),
        TokenType::Star,
    ];
    test_parse_token_types(input, expected);
}

#[test]
fn test_parse_string_empty() {
    let input = "\"\"";
    let expected = vec![TokenType::String(String::new())];
    test_parse_token_types(input, expected);
}

#[test]
fn test_parse_string_single_word() {
    let input = "\"hello\"";
    let expected = vec![TokenType::String("hello".to_string())];
    test_parse_token_types(input, expected);
}

#[test]
fn test_parse_string_with_escaped_whitespace() {
    let input = "\"\\tthis string contains\\nwhitespace\\n\\tlots of whitespace.\\r\\n\"";
    let expected = vec![TokenType::String(
        "\tthis string contains\nwhitespace\n\tlots of whitespace.\r\n".to_string(),
    )];
    test_parse_token_types(input, expected);
}

#[test]
fn test_parse_string_with_escaped_quotes() {
    let input = "\"the alien said, \\\"greetings, earthling.\\\"\"";
    let expected = vec![TokenType::String(
        "the alien said, \"greetings, earthling.\"".to_string(),
    )];
    test_parse_token_types(input, expected);
}

#[test]
fn test_parse_string_multiple_words() {
    let input = "\"hello world\"";
    let expected = vec![TokenType::String("hello world".to_string())];
    test_parse_token_types(input, expected);
}

#[test]
fn test_parse_string_with_emoji() {
    let input = "\"hello! \\u{1f642}\"";
    let expected = vec![TokenType::String("hello! 🙂".to_string())];
    test_parse_token_types(input, expected);
}

#[test]
fn test_parse_invalid_unicode_escape() {
    let input = "\"invalid unicode \\u!\"";
    let result = parse(input);
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), Error::InvalidUnicodeChar))
}

#[test]
fn test_parse_multiline_string() {
    let input = " \"\"\"
        This is a
        multiline string
        with \"quotes\" and \t tabs
        and \n newlines
    \"\"\"";
    let expected = vec![TokenType::String(
        "
        This is a
        multiline string
        with \"quotes\" and \t tabs
        and \n newlines
    "
        .to_string(),
    )];
    test_parse_token_types(input, expected);
}

#[test]
fn test_parse_hexadecimal() {
    let input = "0x1A 0XFF";
    let expected = vec![TokenType::Integer(26), TokenType::Integer(255)];
    test_parse_token_types(input, expected);
}

#[test]
fn test_parse_octal() {
    let input = "0o123 0O77";
    let expected = vec![TokenType::Integer(83), TokenType::Integer(63)];
    test_parse_token_types(input, expected);
}

#[test]
fn test_parse_binary() {
    let input = "0b1010 0B1111";
    let expected = vec![TokenType::Integer(10), TokenType::Integer(15)];
    test_parse_token_types(input, expected);
}

#[test]
fn test_parse_negative_decimal() {
    let input = "-10 -3";
    let expected = vec![TokenType::Integer(-10), TokenType::Integer(-3)];
    test_parse_token_types(input, expected);
}

#[test]
fn test_parse_negative_hexadecimal() {
    let input = "-0x1A -0XFF";
    let expected = vec![TokenType::Integer(-26), TokenType::Integer(-255)];
    test_parse_token_types(input, expected);
}

#[test]
fn test_parse_negative_octal() {
    let input = "-0o123 -0O77";
    let expected = vec![TokenType::Integer(-83), TokenType::Integer(-63)];
    test_parse_token_types(input, expected);
}

#[test]
fn test_parse_negative_binary() {
    let input = "-0b1010 -0B1111";
    let expected = vec![TokenType::Integer(-10), TokenType::Integer(-15)];
    test_parse_token_types(input, expected);
}

#[test]
fn test_parse_float_trailing_dot() {
    let input = "1.";
    let expected = vec![TokenType::Float(1.0)];
    test_parse_token_types(input, expected);
}

#[test]
fn test_parse_float_with_digits() {
    let input = "1.23";
    let expected = vec![TokenType::Float(1.23)];
    test_parse_token_types(input, expected);
}

#[test]
fn test_parse_float_scientific_notation() {
    let input = "1e7";
    let expected = vec![TokenType::Float(1e7)];
    test_parse_token_types(input, expected);
}

#[test]
fn test_parse_float_scientific_notation_with_dot() {
    let input = "1.23e12";
    let expected = vec![TokenType::Float(1.23e12)];
    test_parse_token_types(input, expected);
}

#[test]
fn test_parse_float_scientific_notation_negative_exponent() {
    let input = "1e-7 1.23e-12";
    let expected = vec![TokenType::Float(1e-7), TokenType::Float(1.23e-12)];
    test_parse_token_types(input, expected);
}

#[test]
fn test_parse_float_scientific_notation_positive_exponent() {
    let input = "1e+7 1.23e+12";
    let expected = vec![TokenType::Float(1e7), TokenType::Float(1.23e12)];
    test_parse_token_types(input, expected);
}

#[test]
fn test_parse_float_scientific_notation_capital_e() {
    let input = "1E7";
    let expected = vec![TokenType::Float(1e7)];
    test_parse_token_types(input, expected);
}

#[test]
fn test_parse_rational_simple() {
    let input = "1/3";
    let expected = vec![TokenType::Rational(1, 3)];
    test_parse_token_types(input, expected);
}

#[test]
fn test_parse_rational_mixed_positive() {
    let input = "1+1/3";
    let expected = vec![TokenType::Rational(4, 3)];
    test_parse_token_types(input, expected);
}

#[test]
fn test_parse_rational_mixed_negative() {
    let input = "1-1/3";
    let expected = vec![TokenType::Rational(2, 3)];
    test_parse_token_types(input, expected);
}

#[test]
fn test_parse_rational_signed() {
    let input = "+1/3 -1/3 1/-3 1/+3 +1+1/3 -1+1/3";
    let expected = vec![
        TokenType::Rational(1, 3),
        TokenType::Rational(-1, 3),
        TokenType::Rational(-1, 3),
        TokenType::Rational(1, 3),
        TokenType::Rational(4, 3),
        TokenType::Rational(-2, 3),
    ];
    test_parse_token_types(input, expected);
}

#[test]
fn test_from_f64() {
    assert_eq!(TokenType::from(PI), TokenType::Float(PI));
}

#[test]
fn test_from_i64() {
    assert_eq!(TokenType::from(42), TokenType::Integer(42));
}

#[test]
fn test_from_string() {
    assert_eq!(
        TokenType::from("greetings".to_string()),
        TokenType::String("greetings".to_string())
    );
}

#[test]
fn test_parse_booleans() {
    let input = "true false";
    let expected = vec![TokenType::Boolean(true), TokenType::Boolean(false)];
    test_parse_token_types(input, expected);
}

#[test]
fn test_parse_boolean_operators() {
    let input = "== != < > <= >= !";
    let expected = vec![
        TokenType::Equal,
        TokenType::BangEqual,
        TokenType::Less,
        TokenType::Greater,
        TokenType::LessEqual,
        TokenType::GreaterEqual,
        TokenType::Bang,
    ];
    test_parse_token_types(input, expected);
}

#[test]
fn test_read_until() {
    let input = " 1 2 3 \"something with spaces\" } end";
    let expected = vec![
        TokenType::Integer(1),
        TokenType::Integer(2),
        TokenType::Integer(3),
        TokenType::String("something with spaces".to_string()),
    ];

    let mut scanner = Scanner::from_string(input);
    let result = scanner.scan_until(&TokenType::CloseBrace);
    assert!(result.is_ok());
    let tokens = result.unwrap();
    let token_types = tokens.into_iter().map(|t| t.token_type).collect::<Vec<_>>();

    assert_eq!(scanner.index, input.len() - 3);
    assert_eq!(token_types, expected);
}

#[test]
// TODO: This one's broken, and it's because the scanner is trying to build an AST.
fn test_parse_vector() {
    let input = "4 { 5 6 { 7 8 } } 9";
    let tokens = vec![
        Token {
            token_type: 7.into(),
            line_no: 1,
            column: 10,
            length: 1,
        },
        Token {
            token_type: 8.into(),
            line_no: 1,
            column: 12,
            length: 1,
        },
    ];
    let tokens = vec![
        Token {
            token_type: 5.into(),
            line_no: 1,
            column: 4,
            length: 1,
        },
        Token {
            token_type: 6.into(),
            line_no: 1,
            column: 6,
            length: 1,
        },
        Token {
            token_type: tokens.into(),
            line_no: 1,
            column: 1,
            length: 6,
        },
    ];
    let expected = vec![TokenType::Integer(4), tokens.into(), 9.into()];

    test_parse_token_types(input, expected);
}

#[test]
#[ignore = "working out scanner"]
fn test_function() {
    let input = ": double ( x -- x ) 2 * ;";
    let expected_scan = vec![
        TokenType::Colon,
        TokenType::Word("double".to_string()),
        TokenType::OpenParen,
        TokenType::Word("x".to_string()),
        TokenType::LongDash,
        TokenType::Word("x".to_string()),
        TokenType::CloseParen,
        TokenType::Integer(2),
        TokenType::Star,
        TokenType::Semicolon,
    ];

    let actual = scan(&input);

    assert!(actual.is_ok());
    let token_types: Vec<_> = actual.unwrap().into_iter().map(|t| t.token_type).collect();
    assert_eq!(token_types, expected_scan);
}
