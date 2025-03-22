use super::*;

struct TestProgram {
    instructions: Vec<usize>,
    constants: Vec<Value>,
    op_table: Vec<OpFn>,
}

impl Program for TestProgram {
    fn get_instruction(&self, ip: usize) -> Option<usize> {
        self.instructions.get(ip).copied()
    }

    fn get_constant(&self, index: usize) -> Option<&Value> {
        self.constants.get(index)
    }

    fn get_op(&self, index: usize) -> Option<&OpFn> {
        self.op_table.get(index)
    }

    fn instructions_len(&self) -> usize {
        self.instructions.len()
    }
}

#[test]
fn test_stack_operations() {
    let mut vm = VM::new();

    // Test push and pop
    vm.push(shared(Value::Integer(42))).unwrap();
    assert_eq!(vm.stack_size(), 1);
    let value = vm.pop().unwrap();
    assert!(matches!(*value.borrow(), Value::Integer(42)));
    assert!(matches!(
        vm.pop(),
        Err(Error::VMError(VMError::StackUnderflow))
    ));

    // Test dup
    vm.push(shared(Value::Integer(1))).unwrap();
    vm.dup().unwrap();
    assert_eq!(vm.stack_size(), 2);
    assert!(matches!(*vm.pop().unwrap().borrow(), Value::Integer(1)));
    assert!(matches!(*vm.pop().unwrap().borrow(), Value::Integer(1)));

    // Test swap
    vm.push(shared(Value::Integer(1))).unwrap();
    vm.push(shared(Value::Integer(2))).unwrap();
    vm.swap().unwrap();
    assert!(matches!(*vm.pop().unwrap().borrow(), Value::Integer(1)));
    assert!(matches!(*vm.pop().unwrap().borrow(), Value::Integer(2)));

    // Test rot
    vm.push(shared(Value::Integer(1))).unwrap();
    vm.push(shared(Value::Integer(2))).unwrap();
    vm.push(shared(Value::Integer(3))).unwrap();
    vm.rot().unwrap();
    assert!(matches!(*vm.pop().unwrap().borrow(), Value::Integer(1)));
    assert!(matches!(*vm.pop().unwrap().borrow(), Value::Integer(3)));
    assert!(matches!(*vm.pop().unwrap().borrow(), Value::Integer(2)));

    // Test drop_op
    vm.push(shared(Value::Integer(42))).unwrap();
    vm.drop_op().unwrap();
    assert_eq!(vm.stack_size(), 0);
}

#[test]
fn test_basic_vm_execution() {
    let mut vm = VM::new();
    let (op_table, _) = create_op_table();

    let program = Box::new(TestProgram {
        instructions: vec![0, 0], // lit operation index followed by constant index
        constants: vec![Value::Integer(123)],
        op_table,
    });

    vm.load_program(program);
    vm.run().unwrap();

    // Verify the result
    let value = vm.pop().unwrap();
    assert!(matches!(*value.borrow(), Value::Integer(123)));
}

#[test]
fn test_invalid_opcode() {
    let mut vm = VM::new();
    let program = Box::new(TestProgram {
        instructions: vec![999], // Invalid opcode
        constants: vec![],
        op_table: vec![],
    });

    vm.load_program(program);
    assert!(matches!(
        vm.run(),
        Err(Error::VMError(VMError::InvalidOpCode(_)))
    ));
}

#[test]
fn test_return_stack_operations() {
    let mut vm = VM::new();

    // Test >r (to_r)
    vm.push(shared(Value::Integer(42))).unwrap();
    vm.to_r().unwrap();
    assert_eq!(vm.stack_size(), 0);

    // Test r> (r_from)
    vm.r_from().unwrap();
    assert_eq!(vm.stack_size(), 1);
    assert!(matches!(*vm.pop().unwrap().borrow(), Value::Integer(42)));

    // Test r@ (r_fetch)
    vm.push(shared(Value::Integer(10))).unwrap();
    vm.to_r().unwrap();
    vm.r_fetch().unwrap();
    assert_eq!(vm.stack_size(), 1);
    assert!(matches!(*vm.pop().unwrap().borrow(), Value::Integer(10)));
    vm.r_from().unwrap(); // Clear the return stack

    // Test return stack overflow
    for i in 0..1024 {
        vm.push(shared(Value::Integer(1))).unwrap();
        match vm.to_r() {
            Ok(_) => continue,
            Err(Error::VMError(VMError::ReturnStackOverflow)) => {
                // We've hit the overflow, verify we pushed the expected number of values
                assert!(i > 0);
                break;
            }
            Err(e) => panic!("Unexpected error: {:?}", e),
        }
    }

    // Clear the return stack (only pop what we successfully pushed)
    while let Ok(_) = vm.r_from() {}

    // Test return stack underflow
    assert!(matches!(
        vm.r_from(),
        Err(Error::VMError(VMError::ReturnStackUnderflow))
    ));
    assert!(matches!(
        vm.r_fetch(),
        Err(Error::VMError(VMError::ReturnStackUnderflow))
    ));
}

#[test]
fn test_arithmetic_operations() {
    let mut vm = VM::new();

    // Test integer addition
    vm.push(shared(Value::Integer(3))).unwrap();
    vm.push(shared(Value::Integer(4))).unwrap();
    vm.add().unwrap();
    assert!(matches!(*vm.pop().unwrap().borrow(), Value::Integer(7)));

    // Test float addition
    vm.push(shared(Value::Float(3.5))).unwrap();
    vm.push(shared(Value::Float(1.5))).unwrap();
    vm.add().unwrap();
    assert!(matches!(*vm.pop().unwrap().borrow(), Value::Float(5.0)));

    // Test mixed addition (integer + float)
    vm.push(shared(Value::Integer(2))).unwrap();
    vm.push(shared(Value::Float(1.5))).unwrap();
    vm.add().unwrap();
    assert!(matches!(*vm.pop().unwrap().borrow(), Value::Float(3.5)));

    // Test subtraction
    vm.push(shared(Value::Integer(5))).unwrap();
    vm.push(shared(Value::Integer(3))).unwrap();
    vm.subtract().unwrap();
    assert!(matches!(*vm.pop().unwrap().borrow(), Value::Integer(2)));

    // Test multiplication
    vm.push(shared(Value::Integer(4))).unwrap();
    vm.push(shared(Value::Integer(3))).unwrap();
    vm.multiply().unwrap();
    assert!(matches!(*vm.pop().unwrap().borrow(), Value::Integer(12)));

    // Test division
    vm.push(shared(Value::Integer(10))).unwrap();
    vm.push(shared(Value::Integer(2))).unwrap();
    vm.divide().unwrap();
    assert!(matches!(*vm.pop().unwrap().borrow(), Value::Integer(5)));
}

#[test]
fn test_arithmetic_errors() {
    let mut vm = VM::new();

    // Test division by zero (integer)
    vm.push(shared(Value::Integer(10))).unwrap();
    vm.push(shared(Value::Integer(0))).unwrap();
    assert!(matches!(
        vm.divide(),
        Err(Error::VMError(VMError::DivisionByZero))
    ));

    // Test division by zero (float)
    vm.push(shared(Value::Float(10.0))).unwrap();
    vm.push(shared(Value::Float(0.0))).unwrap();
    assert!(matches!(
        vm.divide(),
        Err(Error::VMError(VMError::DivisionByZero))
    ));

    // Test type mismatch
    vm.push(shared(Value::Integer(1))).unwrap();
    vm.push(shared(Value::Boolean(true))).unwrap();
    assert!(matches!(
        vm.add(),
        Err(Error::VMError(VMError::TypeMismatch(_)))
    ));

    // Test stack underflow
    assert!(matches!(
        VM::new().add(),
        Err(Error::VMError(VMError::StackUnderflow))
    ));
}

#[test]
fn test_comparison_operations() {
    let mut vm = VM::new();

    // Test equal
    vm.push(shared(Value::Integer(5))).unwrap();
    vm.push(shared(Value::Integer(5))).unwrap();
    vm.equal().unwrap();
    assert!(matches!(*vm.pop().unwrap().borrow(), Value::Boolean(true)));

    vm.push(shared(Value::Integer(5))).unwrap();
    vm.push(shared(Value::Integer(6))).unwrap();
    vm.equal().unwrap();
    assert!(matches!(*vm.pop().unwrap().borrow(), Value::Boolean(false)));

    // Test not equal
    vm.push(shared(Value::Integer(5))).unwrap();
    vm.push(shared(Value::Integer(6))).unwrap();
    vm.not_equal().unwrap();
    assert!(matches!(*vm.pop().unwrap().borrow(), Value::Boolean(true)));

    // Test less than
    vm.push(shared(Value::Integer(5))).unwrap();
    vm.push(shared(Value::Integer(6))).unwrap();
    vm.less().unwrap();
    assert!(matches!(*vm.pop().unwrap().borrow(), Value::Boolean(true)));

    // Test greater than
    vm.push(shared(Value::Integer(6))).unwrap();
    vm.push(shared(Value::Integer(5))).unwrap();
    vm.greater().unwrap();
    assert!(matches!(*vm.pop().unwrap().borrow(), Value::Boolean(true)));

    // Test less than or equal
    vm.push(shared(Value::Integer(5))).unwrap();
    vm.push(shared(Value::Integer(5))).unwrap();
    vm.less_equal().unwrap();
    assert!(matches!(*vm.pop().unwrap().borrow(), Value::Boolean(true)));

    // Test greater than or equal
    vm.push(shared(Value::Integer(6))).unwrap();
    vm.push(shared(Value::Integer(5))).unwrap();
    vm.greater_equal().unwrap();
    assert!(matches!(*vm.pop().unwrap().borrow(), Value::Boolean(true)));

    // Test comparison with different types
    vm.push(shared(Value::Integer(5))).unwrap();
    vm.push(shared(Value::Float(5.0))).unwrap();
    vm.equal().unwrap();
    assert!(matches!(*vm.pop().unwrap().borrow(), Value::Boolean(true)));

    // Test comparison error
    vm.push(shared(Value::Integer(5))).unwrap();
    vm.push(shared(Value::Boolean(true))).unwrap();
    assert!(matches!(
        vm.equal(),
        Err(Error::VMError(VMError::TypeMismatch(_)))
    ));

    // Test NOT operation
    vm.push(shared(Value::Boolean(true))).unwrap();
    vm.not().unwrap();
    assert!(matches!(*vm.pop().unwrap().borrow(), Value::Boolean(false)));

    vm.push(shared(Value::Boolean(false))).unwrap();
    vm.not().unwrap();
    assert!(matches!(*vm.pop().unwrap().borrow(), Value::Boolean(true)));

    // Test NOT operation error
    vm.push(shared(Value::Integer(5))).unwrap();
    assert!(matches!(
        vm.not(),
        Err(Error::VMError(VMError::TypeMismatch(_)))
    ));
}
