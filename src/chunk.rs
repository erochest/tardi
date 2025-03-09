use std::cell::RefCell;
use std::convert::TryFrom;
use std::rc::Rc;
use std::{fmt, result};

use ahash::{HashMap, HashMapExt};

use crate::builtins::define_builtins;
use crate::error::Result;
use crate::op_code::OpCode;
use crate::value::{Function, Value};
use crate::vm::VM;

#[derive(Clone)]
pub struct TardiFn {
    pub name: String,
    pub function: Rc<RefCell<dyn FnMut(&mut VM, &mut Chunk) -> Result<()>>>,
}

impl TardiFn {
    pub fn new(
        name: &str,
        function: Rc<RefCell<dyn FnMut(&mut VM, &mut Chunk) -> Result<()>>>,
    ) -> Self {
        TardiFn {
            name: name.to_string(),
            function,
        }
    }

    pub fn call(&mut self, vm: &mut VM, chunk: &mut Chunk) -> Result<()> {
        let mut function = self.function.borrow_mut();
        function(vm, chunk)
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
            | OpCode::Modulo
            | OpCode::Not
            | OpCode::Equal
            | OpCode::Less
            | OpCode::Greater
            | OpCode::ToCallStack
            | OpCode::FromCallStack
            | OpCode::CopyCallStack
            | OpCode::Drop
            | OpCode::Swap => self.debug_op_code(w, op_code, i)?,
            OpCode::Jump | OpCode::MarkJump => self.debug_op_jump(w, op_code, i)?,
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

#[cfg(test)]
mod tests;
