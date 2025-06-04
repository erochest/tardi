use std::collections::{HashMap, HashSet};
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::str::FromStr;

use crate::compiler::Compiler;
use crate::error::Result;
use crate::error::VMError;
use crate::module::Module;
use crate::shared::shared;
use crate::value::{TardiWriter, ValueData};
use crate::vm::VM;

use super::{push_op, InternalBuilder};

pub const IO: &str = "std/io";

pub struct IoModule;
impl InternalBuilder for IoModule {
    fn define_module(
        &self,
        module_manager: &crate::module::ModuleManager,
        op_table: &mut Vec<crate::shared::Shared<crate::value::lambda::Lambda>>,
    ) -> crate::module::Module {
        let mut index = HashMap::new();

        push_op(op_table, &mut index, "write-file", write_file);
        push_op(op_table, &mut index, "read-file", read_file);
        push_op(op_table, &mut index, "<writer>", writer);
        // TODO: push_op(op_table, &mut index, "<reader>", reader);
        push_op(op_table, &mut index, "file-path>>", get_file_path);
        push_op(op_table, &mut index, "close", close);
        push_op(op_table, &mut index, "write", write);
        // TODO: push_op(op_table, &mut index, "write-line", write-line);
        // TODO: push_op(op_table, &mut index, "write-lines", write-lines);
        // TODO: push_op(op_table, &mut index, "flush", flush);
        // TODO: push_op(op_table, &mut index, "read", read);
        // TODO: push_op(op_table, &mut index, "read-line", read-line);
        // TODO: push_op(op_table, &mut index, "read-lines", read-lines);
        // TODO: push_op(op_table, &mut index, "stdin", stdin);
        // TODO: push_op(op_table, &mut index, "stdout", stdout);
        // TODO: push_op(op_table, &mut index, "stderr", stderr);
        // TODO: push_op(op_table, &mut index, "print", print);
        // TODO: push_op(op_table, &mut index, "println", println);
        // TODO: push_op(op_table, &mut index, "nl", nl);
        // TODO: push_op(op_table, &mut index, "eprint", eprint);
        // TODO: push_op(op_table, &mut index, "eprintln", eprintln);
        // TODO: push_op(op_table, &mut index, "enl", enl);
        // TODO: push_op(op_table, &mut index, ".", .);
        // TODO: push_op(op_table, &mut index, ".s", .s);

        Module {
            imported: HashMap::new(),
            path: None,
            name: IO.to_string(),
            defined: index,
            exported: HashSet::new(),
        }
    }
}

fn push_true(vm: &mut VM) -> Result<()> {
    vm.push(shared(true.into()))
}

fn write_file(vm: &mut VM, _compiler: &mut Compiler) -> Result<()> {
    let path = vm.pop()?;
    let path = path.borrow();
    let path = path
        .as_string()
        .ok_or_else(|| VMError::TypeMismatch("write-file path must be string".to_string()))?;
    let contents = vm.pop()?;
    let contents = contents.borrow();
    let contents = contents
        .as_string()
        .ok_or_else(|| VMError::TypeMismatch("write-file contents must be string".to_string()))?;

    // TODO: needs to propagate errors
    fs::write(path, contents)?;

    push_true(vm)
}

fn read_file(vm: &mut VM, _compiler: &mut Compiler) -> Result<()> {
    let path = vm.pop()?;
    let path = path.borrow();
    let path = path
        .as_string()
        .ok_or_else(|| VMError::TypeMismatch("read-file path must be string".to_string()))?;

    // TODO: needs to propagate errors
    let contents = fs::read_to_string(path)?;

    vm.push(shared(contents.into()))?;
    push_true(vm)
}

fn writer(vm: &mut VM, _compiler: &mut Compiler) -> Result<()> {
    let path = vm.pop()?;
    let path = path.borrow();
    let path = path
        .as_string()
        .ok_or_else(|| VMError::TypeMismatch("open path must be string".to_string()))?;
    let path = PathBuf::from_str(path)?;

    let writer = TardiWriter::from_path(&path)?;

    let value_data = ValueData::Writer(writer);
    vm.push(shared(value_data.into()))?;

    Ok(())
}

fn close(vm: &mut VM, _compiler: &mut Compiler) -> Result<()> {
    let file_value = vm.pop()?;
    let mut file_value = file_value.borrow_mut();
    let writer = file_value
        .data
        .as_writer_mut()
        .ok_or_else(|| VMError::TypeMismatch("close must be a writer".to_string()))?;

    // TODO: propagate errors
    writer.flush()?;

    push_true(vm)
}

fn get_file_path(vm: &mut VM, _compiler: &mut Compiler) -> Result<()> {
    let file_value = vm.pop()?;
    let file_value = file_value.borrow();
    let path = file_value
        .data
        .as_writer()
        .ok_or_else(|| VMError::TypeMismatch("file-path>> must be a writer".to_string()))?
        .get_path();

    let value_data = path
        .map(ValueData::from)
        .unwrap_or_else(|| ValueData::Boolean(false));
    vm.push(shared(value_data.into()))?;

    Ok(())
}

// TODO: add stack effect comments
fn write(vm: &mut VM, _compiler: &mut Compiler) -> Result<()> {
    let file_value = vm.pop()?;
    let mut file_value = file_value.borrow_mut();
    let writer = file_value
        .data
        .as_writer_mut()
        .ok_or_else(|| VMError::TypeMismatch("write must be a writer".to_string()))?;

    let contents = vm.pop()?;
    let contents = contents.borrow();
    let contents = contents
        .as_string()
        .ok_or_else(|| VMError::TypeMismatch("write contents must be string".to_string()))?;

    writer.write_all(contents.as_bytes())?;
    push_true(vm)
}
