use pretty_assertions::assert_eq;

use super::super::*;

fn assert_i8(expected: i8, value: &Value) {
    assert!(
        matches!(value.data, ValueData::I8(_)),
        "value.data not i8 {:?}",
        value.data
    );
    if let ValueData::I8(ref actual) = value.data {
        assert_eq!(&expected, actual);
    }
}
fn assert_i16(expected: i16, value: &Value) {
    assert!(
        matches!(value.data, ValueData::I16(_)),
        "value.data not i16 {:?}",
        value.data
    );
    if let ValueData::I16(ref actual) = value.data {
        assert_eq!(&expected, actual);
    }
}
fn assert_i32(expected: i32, value: &Value) {
    assert!(
        matches!(value.data, ValueData::I32(_)),
        "value.data not i32 {:?}",
        value.data
    );
    if let ValueData::I32(ref actual) = value.data {
        assert_eq!(&expected, actual);
    }
}
fn assert_i64(expected: i64, value: &Value) {
    assert!(
        matches!(value.data, ValueData::I64(_)),
        "value.data not i64 {:?}",
        value.data
    );
    if let ValueData::I64(ref actual) = value.data {
        assert_eq!(&expected, actual);
    }
}
fn assert_i128(expected: i128, value: &Value) {
    assert!(
        matches!(value.data, ValueData::I128(_)),
        "value.data not i128 {:?}",
        value.data
    );
    if let ValueData::I128(ref actual) = value.data {
        assert_eq!(&expected, actual);
    }
}
fn assert_isize(expected: isize, value: &Value) {
    assert!(matches!(value.data, ValueData::Isize(_)));
    if let ValueData::Isize(ref actual) = value.data {
        assert_eq!(&expected, actual);
    }
}
fn assert_u8(expected: u8, value: &Value) {
    assert!(
        matches!(value.data, ValueData::U8(_)),
        "value.data not u8 {:?}",
        value.data
    );
    if let ValueData::U8(ref actual) = value.data {
        assert_eq!(&expected, actual);
    }
}
fn assert_u16(expected: u16, value: &Value) {
    assert!(
        matches!(value.data, ValueData::U16(_)),
        "value.data not u16 {:?}",
        value.data
    );
    if let ValueData::U16(ref actual) = value.data {
        assert_eq!(&expected, actual);
    }
}
fn assert_u32(expected: u32, value: &Value) {
    assert!(
        matches!(value.data, ValueData::U32(_)),
        "value.data not u32 {:?}",
        value.data
    );
    if let ValueData::U32(ref actual) = value.data {
        assert_eq!(&expected, actual);
    }
}
fn assert_u64(expected: u64, value: &Value) {
    assert!(
        matches!(value.data, ValueData::U64(_)),
        "value.data not u64 {:?}",
        value.data
    );
    if let ValueData::U64(ref actual) = value.data {
        assert_eq!(&expected, actual);
    }
}
fn assert_u128(expected: u128, value: &Value) {
    assert!(
        matches!(value.data, ValueData::U128(_)),
        "value.data not u128 {:?}",
        value.data
    );
    if let ValueData::U128(ref actual) = value.data {
        assert_eq!(&expected, actual);
    }
}
fn assert_usize(expected: usize, value: &Value) {
    assert!(
        matches!(value.data, ValueData::Usize(_)),
        "value.data not usize {:?}",
        value.data
    );
    if let ValueData::Usize(ref actual) = value.data {
        assert_eq!(&expected, actual);
    }
}

#[test]
fn test_scans_positive_integers() {
    let scanner = Scanner::from_string("1 20 350 4524 50110");
    let values = scanner.collect::<Vec<_>>();
    assert_isize(1, &values[0]);
    assert_isize(20, &values[1]);
    assert_isize(350, &values[2]);
    assert_isize(4524, &values[3]);
    assert_isize(50110, &values[4]);
}

#[test]
fn test_scans_negative_integers() {
    let scanner = Scanner::from_string("-1 -20 -350 -4255 -50101");
    let values = scanner.collect::<Vec<_>>();
    assert_isize(-1, &values[0]);
    assert_isize(-20, &values[1]);
    assert_isize(-350, &values[2]);
    assert_isize(-4255, &values[3]);
    assert_isize(-50101, &values[4]);
}

#[test]
#[ignore]
fn test_scans_positive_signs() {
    let scanner = Scanner::from_string("+1 +20 +350 +4255 +50101");
    let values = scanner.collect::<Vec<_>>();
    assert_isize(1, &values[0]);
    assert_isize(20, &values[1]);
    assert_isize(350, &values[2]);
    assert_isize(4255, &values[3]);
    assert_isize(50101, &values[4]);
}

// TODO: placeholders: 1_000_000 1,000,001
#[test]
#[ignore]
fn test_scans_delimited_integers() {
    let scanner = Scanner::from_string("1_000_000 1,000,001");
    let values = scanner.collect::<Vec<_>>();
    assert_isize(1000000, &values[0]);
    assert_isize(1000001, &values[1]);
}

// TODO: hexadecimal: 0x66 0xff
#[test]
#[ignore]
fn test_scans_hexadecimal_integers() {
    let scanner = Scanner::from_string("0x66 0xff");
    let values = scanner.collect::<Vec<_>>();
    assert_isize(102, &values[0]);
    assert_isize(255, &values[1]);
}

// TODO: octal: 0o66 0o32
#[test]
#[ignore]
fn test_scans_octal_integers() {
    let scanner = Scanner::from_string("0o66 0o32");
    let values = scanner.collect::<Vec<_>>();
    assert_isize(54, &values[0]);
    assert_isize(26, &values[1]);
}

// TODO: binary: 0b10 0b1111
#[test]
#[ignore]
fn test_scans_binary_integers() {
    let scanner = Scanner::from_string("0b10 0b1111");
    let values = scanner.collect::<Vec<_>>();
    assert_isize(2, &values[0]);
    assert_isize(15, &values[1]);
}

// TODO: type markers, positive, negative, and placeholders: 1:u8 20:u32 4524:usize -1:i8 -20:i32 -4524:i64 1_000_000:i64 1,000,001:u64
#[test]
#[ignore]
fn test_scans_typed_numbers() {
    let scanner = Scanner::from_string(
        "1:u8 20:u32 4524:usize -1:i8 -20:i32 -4524:i64 1_000_000:i64 1,000,001:u64",
    );
    let values = scanner.collect::<Vec<_>>();
    assert_u8(1, &values[0]);
    assert_u32(20, &values[1]);
    assert_usize(4524, &values[2]);
    assert_i8(-1, &values[3]);
    assert_i32(-20, &values[4]);
    assert_i64(-4524, &values[5]);
    assert_i64(1000000, &values[6]);
    assert_u64(1000001, &values[7]);
}
