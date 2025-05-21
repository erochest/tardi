use std::collections::HashMap;

use crate::compiler::Compiler;
use crate::error::{Result, VMError};
use crate::module::{Module, ModuleManager, STRINGS};
use crate::shared::{shared, Shared};
use crate::value::lambda::Lambda;
use crate::value::{Value, ValueData};
use crate::vm::VM;

use super::{push_op, InternalBuilder};

pub struct StringsBuilder;
impl InternalBuilder for StringsBuilder {
    fn define_module(
        &self,
        _module_manager: &ModuleManager,
        op_table: &mut Vec<Shared<Lambda>>,
    ) -> Module {
        let mut index = HashMap::new();

        push_op(op_table, &mut index, "<string>", create_string);
        push_op(op_table, &mut index, ">string", to_string);
        push_op(op_table, &mut index, "utf8>string", utf8_to_string);
        push_op(op_table, &mut index, "concat", string_concat);
        push_op(op_table, &mut index, "nth", nth);
        push_op(op_table, &mut index, ">utf8", to_utf8);
        push_op(op_table, &mut index, "empty?", is_empty);
        push_op(op_table, &mut index, "in?", is_in);
        push_op(op_table, &mut index, "starts-with?", starts_with);
        push_op(op_table, &mut index, "ends-with?", ends_with);
        push_op(op_table, &mut index, "index-of?", index_of);
        push_op(op_table, &mut index, "length", length);
        push_op(op_table, &mut index, "replace-all", replace_all);
        push_op(op_table, &mut index, "split", split);
        push_op(op_table, &mut index, "split-all", split_all);
        push_op(op_table, &mut index, "split-at", split_at);
        push_op(op_table, &mut index, "split-whitespace", split_whitespace);
        push_op(op_table, &mut index, "lines", lines);
        push_op(op_table, &mut index, "strip-start", strip_start);
        push_op(op_table, &mut index, "strip-end", strip_end);
        push_op(op_table, &mut index, "substring", substring);
        push_op(op_table, &mut index, ">lowercase", to_lowercase);
        push_op(op_table, &mut index, ">uppercase", to_uppercase);

        Module {
            imported: HashMap::new(),
            path: None,
            name: STRINGS.to_string(),
            defined: index,
        }
    }
}

// TODO: move this operations from the VM into here

// String operations
/// <string> ( -- string )
fn create_string(vm: &mut VM, _compiler: &mut Compiler) -> Result<()> {
    vm.create_string()
}

/// >string ( obj -- string )
fn to_string(vm: &mut VM, _compiler: &mut Compiler) -> Result<()> {
    vm.to_string()
}

/// utf8>string ( vec -- string )
fn utf8_to_string(vm: &mut VM, _compiler: &mut Compiler) -> Result<()> {
    vm.utf8_to_string()
}

/// concat ( str1 str2 -- str1-2 )
fn string_concat(vm: &mut VM, _compiler: &mut Compiler) -> Result<()> {
    vm.string_concat()
}

// fn pop_str<'a>(vm: &'a mut VM, err_message: &'static str) -> Result<(&'a str, Shared<Value>)> {
//     let value = vm.pop()?;
//     // let borrowed = value.borrow();
//     let s = value
//         .borrow()
//         .get_string()
//         .ok_or_else(|| VMError::TypeMismatch("nth string".to_string()))?;
//     Ok((s, value))
// }

/// nth ( s i -- c )
fn nth(vm: &mut VM, _compiler: &mut Compiler) -> Result<()> {
    let i = vm
        .pop()?
        .borrow()
        .get_integer()
        .ok_or_else(|| VMError::TypeMismatch("nth index should be integer".to_string()))?;
    let s = vm.pop()?;
    let s = s.borrow();
    let s = s
        .get_string()
        .ok_or_else(|| VMError::TypeMismatch("nth string".to_string()))?;
    // TODO: should strings always be represented as `Vec<char>`?
    let c = s
        .chars()
        .nth(i as usize)
        .map(ValueData::Char)
        .map(Value::new)
        .unwrap_or_else(|| Value::new(ValueData::Boolean(false)));
    vm.push(shared(c))?;
    Ok(())
}

/// >utf8 ( str -- bytes-vec )
fn to_utf8(vm: &mut VM, _compiler: &mut Compiler) -> Result<()> {
    let s = vm.pop()?;
    let s = s.borrow();
    let s = s
        .get_string()
        .ok_or_else(|| VMError::TypeMismatch(">utf8".to_string()))?;

    let bytes = s.as_bytes().to_vec();
    let bytes = Value::from(bytes);
    vm.push(shared(bytes))?;

    Ok(())
}

// empty? ( s -- ? )
fn is_empty(vm: &mut VM, _compiler: &mut Compiler) -> Result<()> {
    let s = vm.pop()?;
    let s = s.borrow();
    let s = s
        .get_string()
        .ok_or_else(|| VMError::TypeMismatch("empty?".to_string()))?;

    vm.push(shared(Value::from(s.is_empty())))?;

    Ok(())
}

// in? ( haystack needle -- ? )
fn is_in(vm: &mut VM, _compiler: &mut Compiler) -> Result<()> {
    let needle = vm.pop()?;
    let needle = needle.borrow();

    let s = vm.pop()?;
    let s = s.borrow();
    let haystack = s
        .get_string()
        .ok_or_else(|| VMError::TypeMismatch("in? haystack".to_string()))?;

    let result = if let Some(n) = needle.get_string() {
        haystack.contains(n)
    } else if let Some(n) = needle.get_char() {
        haystack.contains(n)
    } else {
        return Err(VMError::TypeMismatch("in? needle".to_string()).into());
    };

    vm.push(shared(Value::from(result)))?;

    Ok(())
}

// starts-with? ( str sub -- ? )
fn starts_with(vm: &mut VM, _compiler: &mut Compiler) -> Result<()> {
    let sub = vm.pop()?;
    let sub = sub.borrow();
    let sub = sub
        .get_string()
        .ok_or_else(|| VMError::TypeMismatch("starts-with? sub".to_string()))?;

    let s = vm.pop()?;
    let s = s.borrow();
    let s = s
        .get_string()
        .ok_or_else(|| VMError::TypeMismatch("starts-with? str".to_string()))?;

    vm.push(shared(Value::from(s.starts_with(sub))))?;

    Ok(())
}

// ends-with? ( str sub -- ? )
fn ends_with(vm: &mut VM, _compiler: &mut Compiler) -> Result<()> {
    let sub = vm.pop()?;
    let sub = sub.borrow();
    let sub = sub
        .get_string()
        .ok_or_else(|| VMError::TypeMismatch("ends-with? sub".to_string()))?;

    let s = vm.pop()?;
    let s = s.borrow();
    let s = s
        .get_string()
        .ok_or_else(|| VMError::TypeMismatch("ends-with? str".to_string()))?;

    vm.push(shared(Value::from(s.ends_with(sub))))?;

    Ok(())
}

// index-of? ( str sub -- ? )
fn index_of(vm: &mut VM, _compiler: &mut Compiler) -> Result<()> {
    let sub = vm.pop()?;
    let sub = sub.borrow();
    let sub = sub
        .get_string()
        .ok_or_else(|| VMError::TypeMismatch("index-of? sub".to_string()))?;

    let s = vm.pop()?;
    let s = s.borrow();
    let s = s
        .get_string()
        .ok_or_else(|| VMError::TypeMismatch("index-of? str".to_string()))?;

    let result = if let Some(i) = s.find(sub) {
        Value::from(i as i64)
    } else {
        Value::from(false)
    };

    vm.push(shared(result))?;

    Ok(())
}

// length ( s -- l )
fn length(vm: &mut VM, _compiler: &mut Compiler) -> Result<()> {
    let s = vm.pop()?;
    let s = s.borrow();
    let s = s
        .get_string()
        .ok_or_else(|| VMError::TypeMismatch("length".to_string()))?;

    vm.push(shared(Value::from(s.len() as i64)))?;

    Ok(())
}

// replace-all ( str find repl -- out )
fn replace_all(vm: &mut VM, _compiler: &mut Compiler) -> Result<()> {
    let replace = vm.pop()?;
    let replace = replace.borrow();
    let replace = replace
        .get_string()
        .ok_or_else(|| VMError::TypeMismatch("replace-all replace".to_string()))?;

    let target = vm.pop()?;
    let target = target.borrow();
    let target = target
        .get_string()
        .ok_or_else(|| VMError::TypeMismatch("replace-all target".to_string()))?;

    let s = vm.pop()?;
    let s = s.borrow();
    let s = s
        .get_string()
        .ok_or_else(|| VMError::TypeMismatch("replace-all str".to_string()))?;

    let result = s.replace(target, replace);
    vm.push(shared(Value::from(result)))?;

    Ok(())
}

// split ( str sub -- prefix suffix/#f )
fn split(vm: &mut VM, _compiler: &mut Compiler) -> Result<()> {
    let sub = vm.pop()?;
    let sub = sub.borrow();
    let sub = sub
        .get_string()
        .ok_or_else(|| VMError::TypeMismatch("split sub".to_string()))?;

    let s_value = vm.pop()?;
    let s = s_value.borrow();
    let s = s
        .get_string()
        .ok_or_else(|| VMError::TypeMismatch("split str".to_string()))?;

    if let Some((prefix, suffix)) = s.split_once(sub) {
        vm.push(shared(Value::from(prefix)))?;
        vm.push(shared(Value::from(suffix)))
    } else {
        vm.push(s_value.clone())?;
        vm.push(shared(Value::from(false)))
    }
}

// split-all ( str sub -- vec )
fn split_all(vm: &mut VM, _compiler: &mut Compiler) -> Result<()> {
    let sub = vm.pop()?;
    let sub = sub.borrow();
    let sub = sub
        .get_string()
        .ok_or_else(|| VMError::TypeMismatch("split-all sub".to_string()))?;

    let s = vm.pop()?;
    let s = s.borrow();
    let s = s
        .get_string()
        .ok_or_else(|| VMError::TypeMismatch("split-all str".to_string()))?;

    let splits = Value::from(s.split(sub).collect::<Vec<_>>());
    vm.push(shared(splits))
}

// split_at ( str i -- prefix suffix/#f )
fn split_at(vm: &mut VM, _compiler: &mut Compiler) -> Result<()> {
    let i = vm.pop()?;
    let i = i.borrow();
    let i = i
        .get_integer()
        .ok_or_else(|| VMError::TypeMismatch("split-at index".to_string()))?;
    let i = i as usize;

    let s_value = vm.pop()?;
    let s = s_value.borrow();
    let s = s
        .get_string()
        .ok_or_else(|| VMError::TypeMismatch("split-at str".to_string()))?;

    if i <= s.len() {
        let (prefix, suffix) = s.split_at(i);
        vm.push(shared(prefix.into()))?;
        vm.push(shared(suffix.into()))
    } else {
        vm.push(s_value.clone())?;
        vm.push(shared(Value::from(false)))
    }
}

// split-whitespace ( str -- vec )
fn split_whitespace(vm: &mut VM, _compiler: &mut Compiler) -> Result<()> {
    let s = vm.pop()?;
    let s = s.borrow();
    let s = s
        .get_string()
        .ok_or_else(|| VMError::TypeMismatch("split-whitespace".to_string()))?;

    let parts = Value::from(s.split_whitespace().collect::<Vec<_>>());
    vm.push(shared(parts))
}

// lines ( str -- vec )
fn lines(vm: &mut VM, _compiler: &mut Compiler) -> Result<()> {
    let s = vm.pop()?;
    let s = s.borrow();
    let s = s
        .get_string()
        .ok_or_else(|| VMError::TypeMismatch("lines".to_string()))?;

    let lines = Value::from(s.lines().collect::<Vec<_>>());
    vm.push(shared(lines))
}

// strip-start ( str pref -- out )
fn strip_start(vm: &mut VM, _compiler: &mut Compiler) -> Result<()> {
    let prefix = vm.pop()?;
    let prefix = prefix.borrow();
    let prefix = prefix
        .get_string()
        .ok_or_else(|| VMError::TypeMismatch("strip-start prefix".to_string()))?;

    let s_value = vm.pop()?;
    let s = s_value.borrow();
    let s = s
        .get_string()
        .ok_or_else(|| VMError::TypeMismatch("strip-start str".to_string()))?;

    let result = s
        .strip_prefix(prefix)
        .map(|s| shared(Value::from(s)))
        .unwrap_or_else(|| s_value.clone());
    vm.push(result)
}

// strip-end ( str suff -- out )
fn strip_end(vm: &mut VM, _compiler: &mut Compiler) -> Result<()> {
    let suffix = vm.pop()?;
    let suffix = suffix.borrow();
    let suffix = suffix
        .get_string()
        .ok_or_else(|| VMError::TypeMismatch("strip-end suffix".to_string()))?;

    let s_value = vm.pop()?;
    let s = s_value.borrow();
    let s = s
        .get_string()
        .ok_or_else(|| VMError::TypeMismatch("strip-end str".to_string()))?;

    let result = s
        .strip_suffix(suffix)
        .map(|s| shared(Value::from(s)))
        .unwrap_or_else(|| s_value.clone());
    vm.push(result)
}

// substring ( str start end -- sub )
fn substring(vm: &mut VM, _compiler: &mut Compiler) -> Result<()> {
    let end = vm.pop()?;
    let end = end.borrow();
    let end = end
        .get_integer()
        .ok_or_else(|| VMError::TypeMismatch("substring end".to_string()))? as usize;

    let start = vm.pop()?;
    let start = start.borrow();
    let start = start
        .get_integer()
        .ok_or_else(|| VMError::TypeMismatch("substring start".to_string()))?
        as usize;

    let s = vm.pop()?;
    let s = s.borrow();
    let s = s
        .get_string()
        .ok_or_else(|| VMError::TypeMismatch("substring str".to_string()))?;

    // TODO: this is missing some edges. one or two
    let result = Value::from(&s[start..end]);
    vm.push(shared(result))
}

// lowercase ( s -- s' )
fn to_lowercase(vm: &mut VM, _compiler: &mut Compiler) -> Result<()> {
    let s = vm.pop()?;
    let s = s.borrow();
    let s = s
        .get_string()
        .ok_or_else(|| VMError::TypeMismatch(">lowercase".to_string()))?;

    vm.push(shared(Value::from(s.to_lowercase())))
}

// uppercase ( s -- s' )
fn to_uppercase(vm: &mut VM, _compiler: &mut Compiler) -> Result<()> {
    let s = vm.pop()?;
    let s = s.borrow();
    let s = s
        .get_string()
        .ok_or_else(|| VMError::TypeMismatch(">uppercase".to_string()))?;

    vm.push(shared(Value::from(s.to_uppercase())))
}
