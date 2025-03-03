use std::convert::TryFrom;
use std::{fmt, result};

use ahash::{HashMap, HashMapExt};

use crate::error::{Error, Result};
use crate::op_code::OpCode;
use crate::value::{Function, Value};
use crate::vm::{Return, VM};

pub struct TardiFn {
    pub name: String,
    pub function: Box<dyn FnMut(&mut VM) -> Result<()>>,
}

impl TardiFn {
    pub fn new(name: &str, function: Box<dyn FnMut(&mut VM) -> Result<()>>) -> Self {
        TardiFn {
            name: name.to_string(),
            function,
        }
    }

    pub fn call(&mut self, vm: &mut VM) -> Result<()> {
        (*self.function)(vm)
    }
}

impl fmt::Debug for TardiFn {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "<TardiFn>")
    }
}

#[derive(Default)]
pub struct Chunk {
    pub constants: Vec<Value>,
    pub code: Vec<u8>,
    pub builtins: Vec<TardiFn>,
    pub builtin_index: HashMap<String, usize>,
    pub dictionary: HashMap<String, Function>,
}

impl fmt::Debug for Chunk {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "CONSTANTS")?;
        for (i, value) in self.constants.iter().enumerate() {
            write!(f, " [{} => {:?}]", i, value)?;
        }
        writeln!(f)?;

        writeln!(f, "DICTIONARY")?;
        for (name, v) in self.dictionary.iter() {
            write!(f, " [{} => {:?}]", name, v)?;
        }
        writeln!(f)?;

        writeln!(f, "CODE")?;
        let mut i = 0;
        while i < self.code.len() {
            let op = OpCode::try_from(self.code[i]);
            match op {
                Ok(op) => i = self.debug_op(f, &op, i)?,
                Err(err) => writeln!(f, "ERROR: {:?}", err)?,
            }
        }

        Ok(())
    }
}

impl Chunk {
    pub fn debug_op<W: fmt::Write>(
        &self,
        w: &mut W,
        op_code: &OpCode,
        i: usize,
    ) -> result::Result<usize, fmt::Error> {
        let mut i = i;

        i = match op_code {
            OpCode::GetConstant => self.debug_op_code_constant(w, op_code, i)?,
            OpCode::Add
            | OpCode::Sub
            | OpCode::Mult
            | OpCode::Div
            | OpCode::Not
            | OpCode::Equal
            | OpCode::Less
            | OpCode::Greater => self.debug_op_code(w, op_code, i)?,
            OpCode::Jump | OpCode::MarkJump | OpCode::MarkCall => {
                self.debug_op_jump(w, op_code, i)?
            }
            OpCode::CallTardiFn => self.debug_op_builtin(w, op_code, i)?,
            OpCode::Return => self.debug_op_code(w, op_code, i)?,
        };

        Ok(i + 1)
    }

    fn debug_op_code<W: fmt::Write>(
        &self,
        w: &mut W,
        op_code: &OpCode,
        i: usize,
    ) -> result::Result<usize, fmt::Error> {
        self.write_ip_number(w, i)?;
        self.write_op_code(w, op_code)?;
        writeln!(w)?;
        Ok(i)
    }

    fn debug_op_code_constant<W: fmt::Write>(
        &self,
        w: &mut W,
        op_code: &OpCode,
        i: usize,
    ) -> result::Result<usize, fmt::Error> {
        let mut i = i;

        self.write_ip_number(w, i)?;
        self.write_op_code(w, op_code)?;

        i += 1;

        let index = self.code[i];
        let value = &self.constants[index as usize];
        writeln!(w, " {:0>4}. {: <16}", index, value)?;

        Ok(i)
    }

    fn debug_op_jump<W: fmt::Write>(
        &self,
        w: &mut W,
        op_code: &OpCode,
        i: usize,
    ) -> result::Result<usize, fmt::Error> {
        let mut i = i;

        self.write_ip_number(w, i)?;
        self.write_op_code(w, op_code)?;

        i += 1;

        let index = self.code[i];
        writeln!(w, " {:0>4}", index)?;

        Ok(i)
    }

    fn debug_op_builtin<W: fmt::Write>(
        &self,
        w: &mut W,
        op_code: &OpCode,
        i: usize,
    ) -> result::Result<usize, fmt::Error> {
        let mut i = i;

        self.write_ip_number(w, i)?;
        self.write_op_code(w, op_code)?;

        i += 1;

        let index = self.code[i];
        let builtin = &self.builtins[index as usize];
        writeln!(w, " {:0>4}. {}", index, builtin.name)?;

        Ok(i)
    }

    fn write_ip_number<W: fmt::Write>(&self, w: &mut W, i: usize) -> fmt::Result {
        write!(w, "{:0>4}. ", i)
    }

    fn write_op_code<W: fmt::Write>(&self, w: &mut W, op_code: &OpCode) -> fmt::Result {
        let debugged = format!("{:?}", op_code);
        write!(w, "{: <16} | ", debugged)
    }
}

// TODO: debugging output of a chunk
impl Chunk {
    pub fn new() -> Self {
        let (builtins, builtin_index) = define_builtins();
        Self {
            code: Vec::new(),
            constants: Vec::new(),
            builtins,
            builtin_index,
            dictionary: HashMap::new(),
        }
    }

    pub fn add_constant(&mut self, value: Value) -> usize {
        self.constants.push(value);
        self.constants.len() - 1
    }

    pub fn push_op_code(&mut self, op_code: OpCode, param: u8) {
        self.code.push(op_code as u8);
        self.code.push(param);
    }
}

fn define_builtins() -> (Vec<TardiFn>, HashMap<String, usize>) {
    let mut builtins = Vec::new();
    let mut index = HashMap::new();

    // TODO: can I create a macro to DRY this up even more?
    insert_builtin(
        &mut builtins,
        &mut index,
        "call",
        Box::new(|vm: &mut VM| {
            let top = vm.stack.pop().ok_or(Error::StackUnderflow)?;
            if let Value::Lambda(_, ip) = top {
                vm.call_stack.push(Return::new(vm.ip + 1));
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
            vm.stack.pop().ok_or(Error::StackUnderflow)?;
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
            let top = vm.stack.pop().ok_or(Error::StackUnderflow)?;
            vm.stack.pop().ok_or(Error::StackUnderflow)?;
            vm.stack.push(top.clone());
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
            let b = vm.stack.pop().ok_or(Error::StackUnderflow)?;
            let a = vm.stack.pop().ok_or(Error::StackUnderflow)?;
            vm.stack.push(b);
            vm.stack.push(a);
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

#[cfg(test)]
mod tests;
