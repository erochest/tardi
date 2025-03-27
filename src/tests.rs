use pretty_assertions::assert_eq;

use super::*;


#[test]
fn test_tardi_execute_str() {
    let mut tardi = Tardi::default();

    let result = tardi.execute_str("42".to_string());
    assert!(result.is_ok());
    
    let stack = tardi.stack();
    assert_eq!(vec![Value::Integer(42)], stack);
}
