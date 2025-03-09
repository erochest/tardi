use std::cell::RefCell;
use std::rc::Rc;

use ahash::{HashMap, HashMapExt};

use crate::chunk::{Chunk, TardiFn};
use crate::error::{Error, Result};
use crate::pop_unwrap;
use crate::value::Value;
use crate::vm::{shared, VM};

macro_rules! builtin {
    ($vec:expr, $idx:expr, $name:expr, $fn:expr) => {
        insert_builtin($vec, $idx, $name, Rc::new(RefCell::new($fn)));
    };
}
macro_rules! builtin_mut {
    ($vec:expr, $idx:expr, $name:expr, $fn:expr) => {
        insert_builtin(&mut $vec, &mut $idx, $name, Rc::new(RefCell::new($fn)));
    };
}

pub fn define_builtins() -> (Vec<TardiFn>, HashMap<String, usize>) {
    let mut builtins = Vec::new();
    let mut index = HashMap::new();

    flow_builtins(&mut builtins, &mut index);
    stack_builtins(&mut builtins, &mut index);
    op_code_builtins(&mut builtins, &mut index);

    (builtins, index)
}

fn call_lambda(vm: &mut VM, lambda: &Value) -> Result<()> {
    if let Value::Lambda(_, ip) = lambda {
        vm.call_stack.push(shared(Value::from(vm.ip + 1)));
        vm.ip = ip - 1;
        Ok(())
    } else {
        Err(Error::UncallableObject(lambda.clone()))
    }
}

fn flow_builtins(builtins: &mut Vec<TardiFn>, index: &mut HashMap<String, usize>) {
    builtin!(
        builtins,
        index,
        "call",
        |vm: &mut VM, _chunk: &mut Chunk| {
            let top = pop_unwrap!(vm.stack);
            call_lambda(vm, &top)
        }
    );

    builtin!(builtins, index, "if", |vm: &mut VM, _chunk: &mut Chunk| {
        let else_clause = pop_unwrap!(vm.stack);
        let then_clause = pop_unwrap!(vm.stack);
        let condition = pop_unwrap!(vm.stack);

        match condition {
            Value::Boolean(true) => call_lambda(vm, &then_clause),
            Value::Boolean(false) => call_lambda(vm, &else_clause),
            _ => Err(Error::InvalidValueType(condition.clone())),
        }
    });
}

fn stack_builtins(builtins: &mut Vec<TardiFn>, index: &mut HashMap<String, usize>) {
    builtin!(builtins, index, "dup", |vm: &mut VM, _chunk: &mut Chunk| {
        let top = vm.stack.last().ok_or(Error::StackUnderflow)?;
        vm.stack.push(top.clone());
        Ok(())
    });

    builtin!(builtins, index, "nip", |vm: &mut VM, _chunk: &mut Chunk| {
        let top = pop_unwrap!(vm.stack);
        pop_unwrap!(vm.stack);
        vm.stack.push(shared(top));
        Ok(())
    });

    builtin!(builtins, index, "pop", |vm: &mut VM, _chunk: &mut Chunk| {
        pop_unwrap!(vm.stack);
        Ok(())
    });

    builtin!(
        builtins,
        index,
        "over",
        |vm: &mut VM, _chunk: &mut Chunk| {
            let index = vm.stack.len();
            if index >= 2 {
                let item = &vm.stack[index - 2];
                vm.stack.push(item.clone());
            } else {
                return Err(Error::StackUnderflow);
            }
            Ok(())
        }
    );

    builtin!(builtins, index, "rot", |vm: &mut VM, _chunk: &mut Chunk| {
        let index = vm.stack.len();
        if index >= 3 {
            let item = vm.stack.remove(index - 3);
            vm.stack.push(item);
        } else {
            return Err(Error::StackUnderflow);
        }
        Ok(())
    });
}

fn op_code_builtins(builtins: &mut Vec<TardiFn>, index: &mut HashMap<String, usize>) {
    builtin!(
        builtins,
        index,
        "-get-constant-",
        |vm: &mut VM, chunk: &mut Chunk| {
            // TODO: can I DRY this up with what's in `VM::execute`?
            let index = pop_unwrap!(vm.stack);
            if let Value::Integer(index) = index {
                // TODO: error case for if index is too large
                let constant = &chunk.constants[index as usize];
                vm.stack.push(shared(constant.clone()));
                Ok(())
            } else {
                Err(Error::InvalidValueType(index.clone()))
            }
        }
    );
    builtin!(
        builtins,
        index,
        "-jump-",
        |vm: &mut VM, _chunk: &mut Chunk| {
            let address = pop_unwrap!(vm.stack);
            let address = if let Value::Integer(v) = address {
                v as usize
            } else if let Value::Address(a) = address {
                a
            } else {
                return Err(Error::InvalidValueType(address.clone()));
            };

            // Offset the address because it's going to get incremented
            // outside of this function's control.
            vm.ip = address - 1;

            Ok(())
        }
    );
    builtin!(
        builtins,
        index,
        "-mark-jump-",
        |vm: &mut VM, _chunk: &mut Chunk| {
            let ip = vm.ip;
            let address = pop_unwrap!(vm.stack);
            let address = if let Value::Integer(v) = address {
                v as usize
            } else if let Value::Address(a) = address {
                a
            } else {
                return Err(Error::InvalidValueType(address.clone()));
            };

            vm.call_stack.push(shared(Value::from(ip + 1)));
            vm.ip = address - 1;

            Ok(())
        }
    );
    builtin!(
        builtins,
        index,
        "-call-tardi-fn-",
        |vm: &mut VM, chunk: &mut Chunk| {
            let top = pop_unwrap!(vm.stack);
            // TODO: do I really want to allow addresses here?
            let index = match top {
                Value::Integer(v) => v as usize,
                Value::Address(v) => v,
                _ => {
                    return Err(Error::InvalidValueType(top));
                }
            };
            let mut tardi_fn = chunk.builtins[index].clone();
            tardi_fn.call(vm, chunk)?;
            Ok(())
        }
    );
}

fn insert_builtin(
    builtins: &mut Vec<TardiFn>,
    index: &mut HashMap<String, usize>,
    name: &str,
    tardi_fn: Rc<RefCell<dyn FnMut(&mut VM, &mut Chunk) -> Result<()>>>,
) {
    let name = name.to_string();
    let tardi_fn = TardiFn {
        name: name.clone(),
        function: tardi_fn,
    };
    index.insert(name.clone(), builtins.len());
    builtins.push(tardi_fn);
}
