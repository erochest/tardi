use std::collections::{HashMap, HashSet};
use std::io::{Read, Write};
use std::path::PathBuf;
use std::str::FromStr;
use std::{fs, io};

use crate::compiler::Compiler;
use crate::error::Result;
use crate::error::VMError;
use crate::module::Module;
use crate::shared::shared;
use crate::value::{TardiReader, TardiWriter, ValueData};
use crate::vm::VM;

use super::{push_false, push_op, push_true, InternalBuilder};

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

        push_op(op_table, &mut index, "<stdin>", stdin);
        push_op(op_table, &mut index, "<stdout>", stdout);
        push_op(op_table, &mut index, "<stderr>", stderr);
        // TODO: push_op(op_table, &mut index, "<string-reader>", string_reader);
        // TODO: push_op(op_table, &mut index, "<string-writer>", string_writer);

        push_op(op_table, &mut index, "print", print);
        push_op(op_table, &mut index, "println", println);
        push_op(op_table, &mut index, "nl", nl);

        push_op(op_table, &mut index, "eprint", eprint);
        push_op(op_table, &mut index, "eprintln", eprintln);
        push_op(op_table, &mut index, "enl", enl);

        push_op(op_table, &mut index, ".", dot);
        push_op(op_table, &mut index, ".s", dot_stack);

        Module {
            imported: HashMap::new(),
            path: None,
            name: IO.to_string(),
            defined: index,
            exported: HashSet::new(),
        }
    }
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
    } else if file_value.data.as_reader_mut().is_some() {
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
    let reader = reader.data.as_reader_mut().ok_or_else(|| {
        VMError::TypeMismatch(format!("read-line must be a reader: {}", reader_repr))
    })?;

    let content = reader.read_line()?;

    vm.push(shared(content.into()))?;
    push_true(vm)
}

/// reader -- line-vector result-flag
fn read_lines(vm: &mut VM, _compiler: &mut Compiler) -> Result<()> {
    let reader = vm.pop()?;
    let reader_repr = reader.borrow().to_repr();
    let mut reader = reader.borrow_mut();
    let reader = reader.data.as_reader_mut().ok_or_else(|| {
        VMError::TypeMismatch(format!("read-lines must be a reader: {}", reader_repr))
    })?;

    if reader.is_consumed() {
        push_false(vm)?;
        push_false(vm)
    } else {
        let lines = reader.read_lines()?;
        vm.push(shared(lines.into()))?;
        push_true(vm)
    }
}

/// -- <stdin>
fn stdin(vm: &mut VM, _compiler: &mut Compiler) -> Result<()> {
    let reader = TardiReader::Stdin;
    vm.push(shared(ValueData::Reader(reader).into()))
}

/// -- <stdout>
fn stdout(vm: &mut VM, _compiler: &mut Compiler) -> Result<()> {
    let writer = TardiWriter::Stdout;
    vm.push(shared(ValueData::Writer(writer).into()))
}

/// -- <stderr>
fn stderr(vm: &mut VM, _compiler: &mut Compiler) -> Result<()> {
    let writer = TardiWriter::Stderr;
    vm.push(shared(ValueData::Writer(writer).into()))
}

fn flush_stdout() -> Result<()> {
    let stdout = io::stdout();
    let mut stdout = stdout.lock();
    stdout.flush()?;
    Ok(())
}

fn flush_stderr() -> Result<()> {
    let stderr = io::stderr();
    let mut stderr = stderr.lock();
    stderr.flush()?;
    Ok(())
}

/// object --
fn print(vm: &mut VM, _compiler: &mut Compiler) -> Result<()> {
    let object = vm.pop()?;

    print!("{}", object.borrow());
    flush_stdout()
}

/// object --
fn println(vm: &mut VM, _compiler: &mut Compiler) -> Result<()> {
    let object = vm.pop()?;

    println!("{}", object.borrow());
    flush_stdout()
}

/// --
fn nl(_vm: &mut VM, _compiler: &mut Compiler) -> Result<()> {
    println!();
    flush_stdout()
}

/// object --
fn eprint(vm: &mut VM, _compiler: &mut Compiler) -> Result<()> {
    let object = vm.pop()?;
    eprint!("{}", object.borrow());
    flush_stderr()
}

/// object --
fn eprintln(vm: &mut VM, _compiler: &mut Compiler) -> Result<()> {
    let object = vm.pop()?;
    eprintln!("{}", object.borrow());
    flush_stderr()
}

/// --
fn enl(_vm: &mut VM, _compiler: &mut Compiler) -> Result<()> {
    eprintln!();
    flush_stderr()
}

/// object --
fn dot(vm: &mut VM, _compiler: &mut Compiler) -> Result<()> {
    let object = vm.pop()?;
    println!("{}", object.borrow().to_repr());
    flush_stdout()
}

/// ...s -- ...s
fn dot_stack(vm: &mut VM, _compiler: &mut Compiler) -> Result<()> {
    for value in vm.stack.iter() {
        println!("{}", value.borrow().to_repr());
    }
    flush_stdout()
}
