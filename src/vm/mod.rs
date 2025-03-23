use std::collections::HashMap;

use crate::error::{Error, Result, VMError};

pub mod value;
use self::value::{shared, SharedValue, Value};

// TODO: Make this an enum with BuiltIn (like below),
// Lambda, and Fn
/// Function pointer type for VM operations
pub type OpFn = fn(&mut VM) -> Result<()>;

/// Trait for programs that can be executed by the VM
pub trait Program: 'static {
    fn get_instruction(&self, ip: usize) -> Option<usize>;
    fn get_constant(&self, index: usize) -> Option<&Value>;
    fn get_op(&self, index: usize) -> Option<&OpFn>;
    fn instructions_len(&self) -> usize;
}

/// The Virtual Machine implementation using Indirect Threaded Code (ITC)
pub struct VM {
    /// The program being executed
    program: Option<Box<dyn Program>>,

    /// Instruction pointer tracking the current position in the instruction stream
    ip: usize,

    /// Data stack for operation arguments and results
    stack: Vec<SharedValue>,

    /// Return stack for control flow
    return_stack: Vec<SharedValue>,
}

impl Default for VM {
    fn default() -> Self {
        Self::new()
    }
}

impl VM {
    /// Returns an iterator over stack values from bottom to top
    pub fn stack_iter(&self) -> impl Iterator<Item = Value> + '_ {
        self.stack.iter().map(|shared| shared.borrow().clone())
    }

    /// Creates a new VM instance
    pub fn new() -> Self {
        VM {
            program: None,
            ip: 0,
            stack: Vec::new(),
            return_stack: Vec::new(),
        }
    }

    /// Pushes a shared value onto the return stack
    pub fn push_return(&mut self, value: SharedValue) -> Result<()> {
        if self.return_stack.len() >= 1024 {
            return Err(VMError::ReturnStackOverflow.into());
        }
        self.return_stack.push(value);
        Ok(())
    }

    /// Pops a shared value from the return stack
    pub fn pop_return(&mut self) -> Result<SharedValue> {
        self.return_stack
            .pop()
            .ok_or(VMError::ReturnStackUnderflow.into())
    }

    /// Moves an item from the data stack to the return stack (>r operation)
    pub fn to_r(&mut self) -> Result<()> {
        let value = self.pop()?;
        self.push_return(value)
    }

    /// Moves an item from the return stack to the data stack (r> operation)
    pub fn r_from(&mut self) -> Result<()> {
        let value = self.pop_return()?;
        self.push(value)
    }

    /// Copies the top item from the return stack to the data stack (r@ operation)
    pub fn r_fetch(&mut self) -> Result<()> {
        if let Some(value) = self.return_stack.last() {
            self.push(value.clone())
        } else {
            Err(VMError::ReturnStackUnderflow.into())
        }
    }

    /// Loads a program into the VM
    pub fn load_program(&mut self, program: Box<dyn Program>) {
        self.program = Some(program);
        self.ip = 0;
    }

    /// Runs the VM, executing all instructions in the instruction stream
    pub fn run(&mut self) -> Result<()> {
        while let Some(program) = &self.program {
            if self.ip >= program.instructions_len() {
                break;
            }

            // Get the next instruction and operation
            let op_index = program
                .get_instruction(self.ip)
                .ok_or(Error::VMError(VMError::InvalidOpCode(self.ip)))?;
            let operation = program
                .get_op(op_index)
                .ok_or(Error::VMError(VMError::InvalidOpCode(op_index)))?;

            // Store the operation in a local variable
            let op = *operation;
            self.ip += 1;

            // Execute the operation
            op(self)?;
        }

        Ok(())
    }

    /// Pushes a shared value onto the data stack
    pub fn push(&mut self, value: SharedValue) -> Result<()> {
        if self.stack.len() >= 1024 {
            return Err(VMError::StackOverflow.into());
        }
        self.stack.push(value);
        Ok(())
    }

    /// Pops a shared value from the data stack
    pub fn pop(&mut self) -> Result<SharedValue> {
        self.stack.pop().ok_or(VMError::StackUnderflow.into())
    }

    /// Returns the current size of the data stack
    pub fn stack_size(&self) -> usize {
        self.stack.len()
    }

    /// Executes the lit operation - loads a constant onto the stack
    pub fn lit(&mut self) -> Result<()> {
        let program = self
            .program
            .as_ref()
            .ok_or(Error::VMError(VMError::NoProgram))?;

        let const_index = program
            .get_instruction(self.ip)
            .ok_or(Error::VMError(VMError::InvalidOpCode(self.ip)))?;
        self.ip += 1;

        if let Some(value) = program.get_constant(const_index) {
            self.push(shared(value.clone()))
        } else {
            Err(Error::VMError(VMError::InvalidConstantIndex(const_index)))
        }
    }

    /// Duplicates the top item on the stack
    pub fn dup(&mut self) -> Result<()> {
        let value = self.pop()?;
        self.push(value.clone())?;
        self.push(value)
    }

    /// Swaps the top two items on the stack
    pub fn swap(&mut self) -> Result<()> {
        let b = self.pop()?;
        let a = self.pop()?;
        self.push(b)?;
        self.push(a)
    }

    /// Rotates the top three items on the stack
    pub fn rot(&mut self) -> Result<()> {
        let c = self.pop()?;
        let b = self.pop()?;
        let a = self.pop()?;
        self.push(b)?;
        self.push(c)?;
        self.push(a)
    }

    /// Removes the top item from the stack
    pub fn drop_op(&mut self) -> Result<()> {
        self.pop().map(|_| ())
    }

    /// Adds the top two items on the stack
    pub fn add(&mut self) -> Result<()> {
        let b = self.pop()?.borrow().clone();
        let a = self.pop()?.borrow().clone();
        let result = (a + b)?;
        self.push(shared(result))
    }

    /// Subtracts the top item from the second item on the stack
    pub fn subtract(&mut self) -> Result<()> {
        let b = self.pop()?.borrow().clone();
        let a = self.pop()?.borrow().clone();
        let result = (a - b)?;
        self.push(shared(result))
    }

    /// Multiplies the top two items on the stack
    pub fn multiply(&mut self) -> Result<()> {
        let b = self.pop()?.borrow().clone();
        let a = self.pop()?.borrow().clone();
        let result = (a * b)?;
        self.push(shared(result))
    }

    /// Divides the second item by the top item on the stack
    pub fn divide(&mut self) -> Result<()> {
        let b = self.pop()?.borrow().clone();
        let a = self.pop()?.borrow().clone();
        let result = (a / b)?;
        self.push(shared(result))
    }

    /// Compares if two values are equal
    pub fn equal(&mut self) -> Result<()> {
        let b = self.pop()?.borrow().clone();
        let a = self.pop()?.borrow().clone();
        if a.partial_cmp(&b).is_none() {
            return Err(VMError::TypeMismatch("equality comparison".to_string()).into());
        }
        self.push(shared(Value::Boolean(a == b)))
    }

    /// Compares if two values are not equal
    pub fn not_equal(&mut self) -> Result<()> {
        let b = self.pop()?.borrow().clone();
        let a = self.pop()?.borrow().clone();
        if a.partial_cmp(&b).is_none() {
            return Err(VMError::TypeMismatch("inequality comparison".to_string()).into());
        }
        self.push(shared(Value::Boolean(a != b)))
    }

    /// Compares if a is less than b
    pub fn less(&mut self) -> Result<()> {
        let b = self.pop()?.borrow().clone();
        let a = self.pop()?.borrow().clone();
        match a.partial_cmp(&b) {
            Some(ordering) => self.push(shared(Value::Boolean(ordering.is_lt()))),
            None => Err(VMError::TypeMismatch("less than comparison".to_string()).into()),
        }
    }

    /// Compares if a is greater than b
    pub fn greater(&mut self) -> Result<()> {
        let b = self.pop()?.borrow().clone();
        let a = self.pop()?.borrow().clone();
        match a.partial_cmp(&b) {
            Some(ordering) => self.push(shared(Value::Boolean(ordering.is_gt()))),
            None => Err(VMError::TypeMismatch("greater than comparison".to_string()).into()),
        }
    }

    /// Compares if a is less than or equal to b
    pub fn less_equal(&mut self) -> Result<()> {
        let b = self.pop()?.borrow().clone();
        let a = self.pop()?.borrow().clone();
        match a.partial_cmp(&b) {
            Some(ordering) => self.push(shared(Value::Boolean(ordering.is_le()))),
            None => Err(VMError::TypeMismatch("less than or equal comparison".to_string()).into()),
        }
    }

    /// Compares if a is greater than or equal to b
    pub fn greater_equal(&mut self) -> Result<()> {
        let b = self.pop()?.borrow().clone();
        let a = self.pop()?.borrow().clone();
        match a.partial_cmp(&b) {
            Some(ordering) => self.push(shared(Value::Boolean(ordering.is_ge()))),
            None => {
                Err(VMError::TypeMismatch("greater than or equal comparison".to_string()).into())
            }
        }
    }

    /// Performs logical NOT operation on the top stack item
    pub fn not(&mut self) -> Result<()> {
        let value = self.pop()?.borrow().clone();
        match value {
            Value::Boolean(b) => self.push(shared(Value::Boolean(!b))),
            _ => Err(VMError::TypeMismatch("logical NOT".to_string()).into()),
        }
    }

    /// Creates a new empty list and pushes it onto the stack
    pub fn create_list(&mut self) -> Result<()> {
        self.push(shared(Value::List(Vec::new())))
    }

    /// Appends a value to the end of a list
    pub fn append(&mut self) -> Result<()> {
        let list = self.pop()?;
        let value = self.pop()?;

        {
            let mut list_ref = list.borrow_mut();
            if let Value::List(items) = &mut *list_ref {
                items.push(value);
                Ok(())
            } else {
                Err(VMError::TypeMismatch(format!("append to list: {}", list_ref)).into())
            }
        }
    }

    /// Prepends a value to the beginning of a list
    pub fn prepend(&mut self) -> Result<()> {
        let list = self.pop()?;
        let value = self.pop()?;

        {
            let mut list_ref = list.borrow_mut();
            if let Value::List(items) = &mut *list_ref {
                items.insert(0, value);
                Ok(())
            } else {
                Err(VMError::TypeMismatch(format!("prepend to list: {}", list_ref)).into())
            }
        }
    }

    /// Concatenates two lists
    pub fn concat(&mut self) -> Result<()> {
        let list2 = self.pop()?;
        let list1 = self.pop()?;

        let new_items = {
            let list1_ref = list1.borrow();
            let list2_ref = list2.borrow();
            match (&*list1_ref, &*list2_ref) {
                (Value::List(items1), Value::List(items2)) => {
                    let mut new_items = items1.clone();
                    new_items.extend(items2.iter().cloned());
                    Ok(new_items)
                }
                _ => Err(Error::from(VMError::TypeMismatch(
                    "concatenate lists".to_string(),
                ))),
            }
        }?;

        self.push(shared(Value::List(new_items)))
    }

    /// Removes and returns the first element of a list
    pub fn split_head(&mut self) -> Result<()> {
        let list = self.pop()?;

        let head = {
            let mut list_ref = list.borrow_mut();
            if let Value::List(items) = &mut *list_ref {
                if items.is_empty() {
                    return Err(VMError::EmptyList.into());
                }
                items.remove(0)
            } else {
                return Err(VMError::TypeMismatch("split head of list".to_string()).into());
            }
        };

        self.push(head)
    }

    /// Creates a new empty string and pushes it onto the stack
    pub fn create_string(&mut self) -> Result<()> {
        self.push(shared(Value::String(String::new())))
    }

    /// Converts a value to its string representation
    pub fn to_string(&mut self) -> Result<()> {
        let value = self.pop()?.borrow().clone();
        self.push(shared(Value::String(value.to_string())))
    }

    /// Converts a list of UTF-8 byte values to a string
    pub fn utf8_to_string(&mut self) -> Result<()> {
        let list = self.pop()?;
        let list_ref = list.borrow();

        if let Value::List(items) = &*list_ref {
            let mut bytes = Vec::new();
            for item in items {
                match &*item.borrow() {
                    Value::Integer(n) if *n >= 0 && *n <= 255 => bytes.push(*n as u8),
                    _ => return Err(VMError::TypeMismatch("UTF-8 byte value".to_string()).into()),
                }
            }

            match String::from_utf8(bytes) {
                Ok(s) => self.push(shared(Value::String(s))),
                Err(_) => Err(VMError::TypeMismatch("invalid UTF-8 sequence".to_string()).into()),
            }
        } else {
            Err(VMError::TypeMismatch("list of bytes".to_string()).into())
        }
    }

    /// Concatenates two strings
    pub fn string_concat(&mut self) -> Result<()> {
        let b = self.pop()?;
        let a = self.pop()?;

        let result = {
            let a_ref = a.borrow();
            let b_ref = b.borrow();
            match (&*a_ref, &*b_ref) {
                (Value::String(s1), Value::String(s2)) => {
                    let mut new_string = s1.clone();
                    new_string.push_str(s2);
                    Ok(new_string)
                }
                _ => Err(VMError::TypeMismatch("string concatenation".to_string())),
            }
        }?;

        self.push(shared(Value::String(result)))
    }
}

// Define the operations
pub fn lit(vm: &mut VM) -> Result<()> {
    vm.lit()
}

pub fn dup(vm: &mut VM) -> Result<()> {
    vm.dup()
}

pub fn swap(vm: &mut VM) -> Result<()> {
    vm.swap()
}

pub fn rot(vm: &mut VM) -> Result<()> {
    vm.rot()
}

pub fn drop_op(vm: &mut VM) -> Result<()> {
    vm.drop_op()
}

pub fn add(vm: &mut VM) -> Result<()> {
    vm.add()
}

pub fn subtract(vm: &mut VM) -> Result<()> {
    vm.subtract()
}

pub fn multiply(vm: &mut VM) -> Result<()> {
    vm.multiply()
}

pub fn divide(vm: &mut VM) -> Result<()> {
    vm.divide()
}

// Helper function to add an operation to the table and map
// TODO: rename to `add_word`
fn add_op(op_table: &mut Vec<OpFn>, op_map: &mut HashMap<String, usize>, op: OpFn, name: &str) {
    let index = op_table.len();
    op_table.push(op);
    op_map.insert(name.to_string(), index);
}

// Create the default operation table
pub fn create_op_table() -> (Vec<OpFn>, HashMap<String, usize>) {
    let mut op_table = Vec::new();
    let mut op_map = HashMap::new();

    add_op(&mut op_table, &mut op_map, lit, "lit");
    add_op(&mut op_table, &mut op_map, dup, "dup");
    add_op(&mut op_table, &mut op_map, swap, "swap");
    add_op(&mut op_table, &mut op_map, rot, "rot");
    add_op(&mut op_table, &mut op_map, drop_op, "drop");

    // Add arithmetic operations
    add_op(&mut op_table, &mut op_map, add, "+");
    add_op(&mut op_table, &mut op_map, subtract, "-");
    add_op(&mut op_table, &mut op_map, multiply, "*");
    add_op(&mut op_table, &mut op_map, divide, "/");

    // Add comparison operations
    add_op(&mut op_table, &mut op_map, equal, "==");
    add_op(&mut op_table, &mut op_map, not_equal, "!=");
    add_op(&mut op_table, &mut op_map, less, "<");
    add_op(&mut op_table, &mut op_map, greater, ">");
    add_op(&mut op_table, &mut op_map, less_equal, "<=");
    add_op(&mut op_table, &mut op_map, greater_equal, ">=");
    add_op(&mut op_table, &mut op_map, not, "!");

    // Add return stack operations
    add_op(&mut op_table, &mut op_map, to_r, ">r");
    add_op(&mut op_table, &mut op_map, r_from, "r>");
    add_op(&mut op_table, &mut op_map, r_fetch, "r@");

    // Add list operations
    add_op(&mut op_table, &mut op_map, create_list, "<list>");
    add_op(&mut op_table, &mut op_map, append, "append");
    add_op(&mut op_table, &mut op_map, prepend, "prepend");
    add_op(&mut op_table, &mut op_map, concat, "concat");
    add_op(&mut op_table, &mut op_map, split_head, "split-head!");

    // Add string operations
    add_op(&mut op_table, &mut op_map, create_string, "<string>");
    add_op(&mut op_table, &mut op_map, to_string, ">string");
    add_op(&mut op_table, &mut op_map, utf8_to_string, "utf8>string");
    add_op(&mut op_table, &mut op_map, string_concat, "string-concat");

    (op_table, op_map)
}

pub fn to_r(vm: &mut VM) -> Result<()> {
    vm.to_r()
}

pub fn r_from(vm: &mut VM) -> Result<()> {
    vm.r_from()
}

pub fn r_fetch(vm: &mut VM) -> Result<()> {
    vm.r_fetch()
}

pub fn not(vm: &mut VM) -> Result<()> {
    vm.not()
}

pub fn equal(vm: &mut VM) -> Result<()> {
    vm.equal()
}

pub fn not_equal(vm: &mut VM) -> Result<()> {
    vm.not_equal()
}

pub fn less(vm: &mut VM) -> Result<()> {
    vm.less()
}

pub fn greater(vm: &mut VM) -> Result<()> {
    vm.greater()
}

pub fn less_equal(vm: &mut VM) -> Result<()> {
    vm.less_equal()
}

pub fn greater_equal(vm: &mut VM) -> Result<()> {
    vm.greater_equal()
}

// List operations
pub fn create_list(vm: &mut VM) -> Result<()> {
    vm.create_list()
}

pub fn append(vm: &mut VM) -> Result<()> {
    vm.append()
}

pub fn prepend(vm: &mut VM) -> Result<()> {
    vm.prepend()
}

pub fn concat(vm: &mut VM) -> Result<()> {
    vm.concat()
}

pub fn split_head(vm: &mut VM) -> Result<()> {
    vm.split_head()
}

// String operations
pub fn create_string(vm: &mut VM) -> Result<()> {
    vm.create_string()
}

pub fn to_string(vm: &mut VM) -> Result<()> {
    vm.to_string()
}

pub fn utf8_to_string(vm: &mut VM) -> Result<()> {
    vm.utf8_to_string()
}

pub fn string_concat(vm: &mut VM) -> Result<()> {
    vm.string_concat()
}

#[cfg(test)]
mod tests;
