use ahash::{HashMap, HashMapExt};

use crate::chunk::TardiFn;
use crate::error::{Error, Result};
use crate::pop_unwrap;
use crate::value::Value;
use crate::vm::{shared, VM};

pub fn define_builtins() -> (Vec<TardiFn>, HashMap<String, usize>) {
    let mut builtins = Vec::new();
    let mut index = HashMap::new();

    // TODO: can I create a macro to DRY this up even more?
    insert_builtin(
        &mut builtins,
        &mut index,
        "call",
        Box::new(|vm: &mut VM| {
            let top = pop_unwrap!(vm.stack);
            if let Value::Lambda(_, ip) = top {
                vm.call_stack.push(shared(Value::from(vm.ip + 1)));
                vm.ip = ip - 1;
            } else {
                return Err(Error::UncallableObject(top));
            }
            Ok(())
        }),
    );

    insert_builtin(
        &mut builtins,
        &mut index,
        "drop",
        Box::new(|vm: &mut VM| {
            pop_unwrap!(vm.stack);
            Ok(())
        }),
    );

    insert_builtin(
        &mut builtins,
        &mut index,
        "dup",
        Box::new(|vm: &mut VM| {
            // TODO: need to wrap values on the stack in Rc<RefCell<_>>
            let top = vm.stack.last().ok_or(Error::StackUnderflow)?;
            vm.stack.push(top.clone());
            Ok(())
        }),
    );

    insert_builtin(
        &mut builtins,
        &mut index,
        "nip",
        Box::new(|vm: &mut VM| {
            let top = pop_unwrap!(vm.stack);
            pop_unwrap!(vm.stack);
            vm.stack.push(shared(top));
            Ok(())
        }),
    );

    insert_builtin(
        &mut builtins,
        &mut index,
        "pop",
        Box::new(|vm: &mut VM| {
            pop_unwrap!(vm.stack);
            Ok(())
        }),
    );

    insert_builtin(
        &mut builtins,
        &mut index,
        "over",
        Box::new(|vm: &mut VM| {
            let index = vm.stack.len();
            if index >= 2 {
                let item = &vm.stack[index - 2];
                vm.stack.push(item.clone());
            } else {
                return Err(Error::StackUnderflow);
            }
            Ok(())
        }),
    );

    insert_builtin(
        &mut builtins,
        &mut index,
        "rot",
        Box::new(|vm: &mut VM| {
            let index = vm.stack.len();
            if index >= 3 {
                let item = vm.stack.remove(index - 3);
                vm.stack.push(item);
            } else {
                return Err(Error::StackUnderflow);
            }
            Ok(())
        }),
    );

    insert_builtin(
        &mut builtins,
        &mut index,
        "swap",
        Box::new(|vm: &mut VM| {
            let b = pop_unwrap!(vm.stack);
            let a = pop_unwrap!(vm.stack);
            vm.stack.push(shared(b));
            vm.stack.push(shared(a));
            Ok(())
        }),
    );

    (builtins, index)
}

fn insert_builtin(
    builtins: &mut Vec<TardiFn>,
    index: &mut HashMap<String, usize>,
    name: &str,
    tardi_fn: Box<dyn FnMut(&mut VM) -> Result<()>>,
) {
    let name = name.to_string();
    let tardi_fn = TardiFn {
        name: name.clone(),
        function: tardi_fn,
    };
    index.insert(name.clone(), builtins.len());
    builtins.push(tardi_fn);
}
