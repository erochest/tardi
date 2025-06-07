use std::collections::{HashMap, HashSet};
use std::fs;
use std::io::{Read, Write};
use std::path::PathBuf;
use std::str::FromStr;

use crate::compiler::Compiler;
use crate::error::{Error, Result};
use crate::error::VMError;
use crate::module::Module;
use crate::shared::shared;
use crate::value::{TardiIoError, TardiReader, TardiWriter, ValueData};
use crate::vm::VM;

use super::{push_op, InternalBuilder};

pub const IO: &str = "std/io";

pub struct IoModule;
impl InternalBuilder for IoModule {
    fn define_module(
        &self,
        _module_manager: &crate::module::ModuleManager,
        op_table: &mut Vec<crate::shared::Shared<crate::value::lambda::Lambda>>,
    ) -> crate::module::Module {
        let mut index = HashMap::new();

        push_op(op_table, &mut index, "write-file", write_file);
        push_op(op_table, &mut index, "read-file", read_file);
        push_op(op_table, &mut index, "<writer>", writer);
        push_op(op_table, &mut index, "<reader>", reader);
        push_op(op_table, &mut index, "file-path>>", get_file_path);
        push_op(op_table, &mut index, "close", close);
        push_op(op_table, &mut index, "write", write);
        push_op(op_table, &mut index, "write-line", write_line);
        push_op(op_table, &mut index, "write-lines", write_lines);
        push_op(op_table, &mut index, "flush", flush);
        push_op(op_table, &mut index, "read", read);
        push_op(op_table, &mut index, "read-line", read_line);
        push_op(op_table, &mut index, "read-lines", read_lines);
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

fn push_false(vm: &mut VM) -> Result<()> {
    vm.push(shared(false.into()))
}

/// contents path -- result-flag
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

/// path -- contents result-flag
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

/// path -- writer
fn writer(vm: &mut VM, _compiler: &mut Compiler) -> Result<()> {
    let path = vm.pop()?;
    let path = path.borrow();
    let path = path
        .as_string()
        .ok_or_else(|| VMError::TypeMismatch("<writer> path must be string".to_string()))?;
    let path = PathBuf::from_str(path)?;

    let writer = TardiWriter::from_path(&path)?;

    let value_data = ValueData::Writer(writer);
    vm.push(shared(value_data.into()))?;

    Ok(())
}

/// path -- reader
fn reader(vm: &mut VM, _compiler: &mut Compiler) -> Result<()> {
    let path = vm.pop()?;
    let path = path.borrow();
    let path = path
        .as_string()
        .ok_or_else(|| VMError::TypeMismatch("<reader> path must be string".to_string()))?;
    let path = PathBuf::from_str(path)?;

    let reader = TardiReader::from_path(&path)?;

    let value_data = ValueData::Reader(reader);
    vm.push(shared(value_data.into()))?;

    Ok(())
}

// TODO: consume it here
/// writer -- result-flag
fn close(vm: &mut VM, _compiler: &mut Compiler) -> Result<()> {
    let file_value = vm.pop()?;
    let file_repr = file_value.borrow().to_string();
    let mut file_value = file_value.borrow_mut();
    // TODO: propagate errors
    if let Some(writer) = file_value.data.as_writer_mut() {
        writer.flush()?;
    } else if let Some(_) = file_value.data.as_reader_mut() {
        // No need to close reader. We need to drop it, but that's hard
        // to do between Shared<_> and the stack.
    } else {
        return Err(VMError::TypeMismatch(format!(
            "close must be a writer or reader: {}",
            file_repr
        ))
        .into());
    }

    push_true(vm)
}

/// writer|reader -- path
fn get_file_path(vm: &mut VM, _compiler: &mut Compiler) -> Result<()> {
    let file_value = vm.pop()?;
    let file_value = file_value.borrow();
    let path = file_value
        .data
        .as_writer()
        .and_then(|w| w.get_path())
        .or_else(|| file_value.data.as_reader().and_then(|r| r.get_path()))
        .ok_or_else(|| {
            VMError::TypeMismatch(format!(
                "file-path>> must be a writer or a reader: {}",
                file_value
            ))
        })?;

    vm.push(shared(path.into()))?;

    Ok(())
}

// TODO: if it's None, return an error `#f`
/// contents writer -- result-flag
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

// TODO: if it's None, return an error `#f`
/// line writer -- result-flag
fn write_line(vm: &mut VM, _compiler: &mut Compiler) -> Result<()> {
    let writer = vm.pop()?;
    let mut writer = writer.borrow_mut();
    let writer = writer
        .data
        .as_writer_mut()
        .ok_or_else(|| VMError::TypeMismatch("write-line must be a writer".to_string()))?;
    let line = vm.pop()?;
    let line = line.borrow();
    let line = line
        .as_string()
        .ok_or_else(|| VMError::TypeMismatch("write-line contents must be string".to_string()))?;

    writeln!(writer, "{}", line)?;
    push_true(vm)
}

// TODO: if it's None, return an error `#f`
/// line-vector writer -- result-flag
fn write_lines(vm: &mut VM, _compiler: &mut Compiler) -> Result<()> {
    let writer = vm.pop()?;
    let mut writer = writer.borrow_mut();
    let writer = writer
        .data
        .as_writer_mut()
        .ok_or_else(|| VMError::TypeMismatch("write-line must be a writer".to_string()))?;
    let line_seq = vm.pop()?;
    let line_seq = line_seq.borrow();
    let line_seq = line_seq.as_list().ok_or_else(|| {
        VMError::TypeMismatch("write-lines contents must be a vector".to_string())
    })?;

    for line in line_seq {
        writeln!(writer, "{}", line.borrow())?;
    }

    push_true(vm)
}

// TODO: if it's None, return an error `#f`
/// writer -- result-flag
fn flush(vm: &mut VM, _compiler: &mut Compiler) -> Result<()> {
    let writer = vm.pop()?;
    let mut writer = writer.borrow_mut();
    let writer = writer
        .data
        .as_writer_mut()
        .ok_or_else(|| VMError::TypeMismatch("write-line must be a writer".to_string()))?;

    writer.flush()?;

    push_true(vm)
}

// TODO: if it's None, return an error `#f`
/// reader -- content result-flag
fn read(vm: &mut VM, _compiler: &mut Compiler) -> Result<()> {
    let reader = vm.pop()?;
    let reader_repr = reader.borrow().to_repr();
    let mut reader = reader.borrow_mut();
    let reader = reader
        .data
        .as_reader_mut()
        .ok_or_else(|| VMError::TypeMismatch(format!("read must be a reader: {}", reader_repr)))?;

    let mut content = String::new();
    reader.read_to_string(&mut content)?;

    vm.push(shared(content.into()))?;
    push_true(vm)
}

// TODO: if it's None, return an error `#f`
/// reader -- line result-flag
fn read_line(vm: &mut VM, _compiler: &mut Compiler) -> Result<()> {
    let reader = vm.pop()?;
    let reader_repr = reader.borrow().to_repr();
    let mut reader = reader.borrow_mut();
    let reader = reader
        .data
        .as_reader_mut()
        .ok_or_else(|| VMError::TypeMismatch(format!("read-line must be a reader: {}", reader_repr)))?;

    let content = reader.read_line()?;

    vm.push(shared(content.into()))?;
    push_true(vm)
}

/// reader -- line-vector result-flag
fn read_lines(vm: &mut VM, _compiler: &mut Compiler) -> Result<()> {
    let reader = vm.pop()?;
    let reader_repr = reader.borrow().to_repr();
    let mut reader = reader.borrow_mut();
    let reader = reader
        .data
        .as_reader_mut()
        .ok_or_else(|| VMError::TypeMismatch(format!("read-lines must be a reader: {}", reader_repr)))?;

    if reader.is_consumed() {
        push_false(vm)?;
        push_false(vm)
    } else {
        let lines = reader.read_lines()?;
        vm.push(shared(lines.into()))?;
        push_true(vm)
    }
}
