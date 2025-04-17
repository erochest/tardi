use pretty_assertions::assert_eq;
use value::ValueData;

use super::*;

#[test]
fn test_tardi_execute_str() {
    let mut tardi = Tardi::default();

    let result = tardi.execute_str("42");
    assert!(result.is_ok());

    let stack = tardi.stack();
    assert_eq!(stack, vec![ValueData::Integer(42).into()]);
}
