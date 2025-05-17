use super::*;

use crate::value::ValueData;

use pretty_assertions::assert_eq;

fn test_word<VD: Copy + Into<ValueData>>(script: &str, expected_stack: &[VD]) {
    let mut tardi = Tardi::new(None).unwrap();
    let result = tardi.execute_str(script);
    let expected_stack = expected_stack
        .iter()
        .map(|vd| (*vd).into())
        .collect::<Vec<_>>();

    assert!(result.is_ok(), "Expected Ok, got {:?}", result);

    let stack = tardi
        .stack()
        .into_iter()
        .map(|v| v.data)
        .collect::<Vec<_>>();
    assert_eq!(stack, expected_stack);
}

#[test]
fn test_dip() {
    test_word("1 [ 2 ] dip", &[2i64, 1]);
}

#[test]
fn test_2drop() {
    test_word("1 2 3 2drop", &[1i64]);
}

#[test]
fn test_3drop() {
    // env_logger::init();
    test_word("1 2 3 4 3drop", &[1i64]);
}

#[test]
fn test_4drop() {
    test_word("1 2 3 4 5 4drop", &[1i64]);
}

#[test]
fn test_5drop() {
    test_word("1 2 3 4 5 6 5drop", &[1i64]);
}

#[test]
fn test_nip() {
    test_word("1 2 3 nip", &[1i64, 3]);
}

#[test]
fn test_2nip() {
    test_word("1 2 3 4 2nip", &[1i64, 4]);
}

#[test]
fn test_3nip() {
    test_word("1 2 3 4 5 3nip", &[1i64, 5]);
}

#[test]
fn test_4nip() {
    test_word("1 2 3 4 5 6 4nip", &[1i64, 6]);
}

#[test]
fn test_5nip() {
    test_word("1 2 3 4 5 6 7 5nip", &[1i64, 7]);
}

#[test]
fn test_dupd() {
    test_word("1 2 3 dupd", &[1i64, 2, 2, 3]);
}

#[test]
fn test_swapd() {
    test_word("1 2 3 swapd", &[2i64, 1, 3]);
}

#[test]
fn test_over() {
    test_word("1 2 3 over", &[1i64, 2, 3, 2]);
}

#[test]
fn test_overd() {
    test_word("1 2 3 overd", &[1i64, 2, 1, 3]);
}

#[test]
fn test_rot() {
    test_word("1 2 3 rot", &[2i64, 3, 1]);
}

#[test]
fn test_rot_() {
    test_word("1 2 3 -rot", &[3i64, 1, 2]);
}

#[test]
fn test_spin() {
    test_word("1 2 3 spin", &[3i64, 2, 1]);
}

#[test]
fn test_4spin() {
    test_word("1 2 3 4 4spin", &[4i64, 3, 2, 1]);
}

#[test]
fn test_rotd() {
    test_word("1 2 3 4 rotd", &[2i64, 3, 1, 4]);
}

#[test]
fn test_rotd_() {
    test_word("1 2 3 4 -rotd", &[3i64, 1, 2, 4]);
}

#[test]
fn test_nipd() {
    test_word("1 2 3 nipd", &[2i64, 3]);
}

#[test]
fn test_2nipd() {
    test_word("1 2 3 4 2nipd", &[3i64, 4]);
}

#[test]
fn test_3nipd() {
    test_word("1 2 3 4 5 3nipd", &[4i64, 5]);
}

#[test]
fn test_2dup() {
    test_word("1 2 2dup", &[1i64, 2, 1, 2]);
}

#[test]
fn test_2dupd() {
    test_word("1 2 3 2dupd", &[1i64, 2, 1, 2, 3]);
}

#[test]
fn test_3dup() {
    test_word("1 2 3 3dup", &[1i64, 2, 3, 1, 2, 3]);
}

#[test]
fn test_2over() {
    test_word("1 2 3 2over", &[1i64, 2, 3, 1, 2]);
}

#[test]
fn test_pick() {
    test_word("1 2 3 pick", &[1i64, 2, 3, 1]);
}

#[test]
fn test_if() {
    test_word("#t [ 13 ] [ 42 ] if", &[13i64]);
    test_word("#f [ 13 ] [ 42 ] if", &[42i64]);
}
