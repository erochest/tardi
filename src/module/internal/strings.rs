use std::collections::{HashMap, HashSet};

use crate::compiler::Compiler;
use crate::error::{Result, VMError};
use crate::module::{Module, ModuleManager};
use crate::shared::{shared, Shared};
use crate::value::lambda::Lambda;
use crate::value::{Value, ValueData};
use crate::vm::VM;

use super::{push_op, InternalBuilder};

pub const STRINGS: &str = "std/strings";

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
            exported: HashSet::new(),
        }
    }
}

// TODO: move this operations from the VM into here

// String operations
/// <string> ( -- string )
fn create_string(vm: &mut VM, _compiler: &mut Compiler) -> Result<()> {
    vm.push(shared(ValueData::String(String::new()).into()))
}

/// >string ( obj -- string )
fn to_string(vm: &mut VM, _compiler: &mut Compiler) -> Result<()> {
    let value = vm.pop()?.borrow().clone();
    vm.push(shared(ValueData::String(value.to_string()).into()))
}

/// utf8>string ( vec -- string )
fn utf8_to_string(vm: &mut VM, _compiler: &mut Compiler) -> Result<()> {
    let list = vm.pop()?;
    let list = list.borrow();
    let list = list.as_list();

    if let Some(items) = list {
        let mut bytes = Vec::new();
        for item in items {
            if let Some(n) = item.borrow().as_integer() {
                if (0..=255).contains(&n) {
                    bytes.push(n as u8);
                    continue;
                }
            }
            return Err(VMError::TypeMismatch("UTF-8 byte value".to_string()).into());
        }

        match String::from_utf8(bytes) {
            Ok(s) => vm.push(shared(ValueData::String(s).into())),
            Err(_) => Err(VMError::TypeMismatch("invalid UTF-8 sequence".to_string()).into()),
        }
    } else {
        Err(VMError::TypeMismatch("list of bytes".to_string()).into())
    }
}

/// concat ( str1 str2 -- str1-2 )
fn string_concat(vm: &mut VM, _compiler: &mut Compiler) -> Result<()> {
    let b = vm.pop()?;
    let a = vm.pop()?;

    let result = {
        let a = a.borrow();
        let a = a.as_string();
        let b = b.borrow();
        let b = b.as_string();
        match (a, b) {
            (Some(s1), Some(s2)) => {
                let mut new_string = s1.to_string();
                new_string.push_str(s2);
                Ok(new_string)
            }
            _ => Err(VMError::TypeMismatch("string concatenation".to_string())),
        }
    }?;

    vm.push(shared(ValueData::String(result).into()))
}

/// nth ( s i -- c )
fn nth(vm: &mut VM, _compiler: &mut Compiler) -> Result<()> {
    let i = vm
        .pop()?
        .borrow()
        .as_integer()
        .ok_or_else(|| VMError::TypeMismatch("nth index should be integer".to_string()))?;
    let s = vm.pop()?;
    let s = s.borrow();
    let s = s
        .as_string()
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
        .as_string()
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
        .as_string()
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
        .as_string()
        .ok_or_else(|| VMError::TypeMismatch("in? haystack".to_string()))?;

    let result = if let Some(n) = needle.as_string() {
        haystack.contains(n)
    } else if let Some(n) = needle.as_char() {
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
        .as_string()
        .ok_or_else(|| VMError::TypeMismatch("starts-with? sub".to_string()))?;

    let s = vm.pop()?;
    let s = s.borrow();
    let s = s
        .as_string()
        .ok_or_else(|| VMError::TypeMismatch("starts-with? str".to_string()))?;

    vm.push(shared(Value::from(s.starts_with(sub))))?;

    Ok(())
}

// ends-with? ( str sub -- ? )
fn ends_with(vm: &mut VM, _compiler: &mut Compiler) -> Result<()> {
    let sub = vm.pop()?;
    let sub = sub.borrow();
    let sub = sub
        .as_string()
        .ok_or_else(|| VMError::TypeMismatch("ends-with? sub".to_string()))?;

    let s = vm.pop()?;
    let s = s.borrow();
    let s = s
        .as_string()
        .ok_or_else(|| VMError::TypeMismatch("ends-with? str".to_string()))?;

    vm.push(shared(Value::from(s.ends_with(sub))))?;

    Ok(())
}

// index-of? ( str sub -- ? )
fn index_of(vm: &mut VM, _compiler: &mut Compiler) -> Result<()> {
    let sub = vm.pop()?;
    let sub = sub.borrow();
    let sub = sub
        .as_string()
        .ok_or_else(|| VMError::TypeMismatch("index-of? sub".to_string()))?;

    let s = vm.pop()?;
    let s = s.borrow();
    let s = s
        .as_string()
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
        .as_string()
        .ok_or_else(|| VMError::TypeMismatch("length".to_string()))?;

    vm.push(shared(Value::from(s.len() as i64)))?;

    Ok(())
}

// replace-all ( str find repl -- out )
fn replace_all(vm: &mut VM, _compiler: &mut Compiler) -> Result<()> {
    let replace = vm.pop()?;
    let replace = replace.borrow();
    let replace = replace
        .as_string()
        .ok_or_else(|| VMError::TypeMismatch("replace-all replace".to_string()))?;

    let target = vm.pop()?;
    let target = target.borrow();
    let target = target
        .as_string()
        .ok_or_else(|| VMError::TypeMismatch("replace-all target".to_string()))?;

    let s = vm.pop()?;
    let s = s.borrow();
    let s = s
        .as_string()
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
        .as_string()
        .ok_or_else(|| VMError::TypeMismatch("split sub".to_string()))?;

    let s_value = vm.pop()?;
    let s = s_value.borrow();
    let s = s
        .as_string()
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
        .as_string()
        .ok_or_else(|| VMError::TypeMismatch("split-all sub".to_string()))?;

    let s = vm.pop()?;
    let s = s.borrow();
    let s = s
        .as_string()
        .ok_or_else(|| VMError::TypeMismatch("split-all str".to_string()))?;

    let splits = Value::from(s.split(sub).collect::<Vec<_>>());
    vm.push(shared(splits))
}

// split_at ( str i -- prefix suffix/#f )
fn split_at(vm: &mut VM, _compiler: &mut Compiler) -> Result<()> {
    let i = vm.pop()?;
    let i = i.borrow();
    let i = i
        .as_integer()
        .ok_or_else(|| VMError::TypeMismatch("split-at index".to_string()))?;
    let i = i as usize;

    let s_value = vm.pop()?;
    let s = s_value.borrow();
    let s = s
        .as_string()
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
        .as_string()
        .ok_or_else(|| VMError::TypeMismatch("split-whitespace".to_string()))?;

    let parts = Value::from(s.split_whitespace().collect::<Vec<_>>());
    vm.push(shared(parts))
}

// lines ( str -- vec )
fn lines(vm: &mut VM, _compiler: &mut Compiler) -> Result<()> {
    let s = vm.pop()?;
    let s = s.borrow();
    let s = s
        .as_string()
        .ok_or_else(|| VMError::TypeMismatch("lines".to_string()))?;

    let lines = Value::from(s.lines().collect::<Vec<_>>());
    vm.push(shared(lines))
}

// strip-start ( str pref -- out )
fn strip_start(vm: &mut VM, _compiler: &mut Compiler) -> Result<()> {
    let prefix = vm.pop()?;
    let prefix = prefix.borrow();
    let prefix = prefix
        .as_string()
        .ok_or_else(|| VMError::TypeMismatch("strip-start prefix".to_string()))?;

    let s_value = vm.pop()?;
    let s = s_value.borrow();
    let s = s
        .as_string()
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
        .as_string()
        .ok_or_else(|| VMError::TypeMismatch("strip-end suffix".to_string()))?;

    let s_value = vm.pop()?;
    let s = s_value.borrow();
    let s = s
        .as_string()
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
        .as_integer()
        .ok_or_else(|| VMError::TypeMismatch("substring end".to_string()))? as usize;

    let start = vm.pop()?;
    let start = start.borrow();
    let start = start
        .as_integer()
        .ok_or_else(|| VMError::TypeMismatch("substring start".to_string()))?
        as usize;

    let s = vm.pop()?;
    let s = s.borrow();
    let s = s
        .as_string()
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
        .as_string()
        .ok_or_else(|| VMError::TypeMismatch(">lowercase".to_string()))?;

    vm.push(shared(Value::from(s.to_lowercase())))
}

// uppercase ( s -- s' )
fn to_uppercase(vm: &mut VM, _compiler: &mut Compiler) -> Result<()> {
    let s = vm.pop()?;
    let s = s.borrow();
    let s = s
        .as_string()
        .ok_or_else(|| VMError::TypeMismatch(">uppercase".to_string()))?;

    vm.push(shared(Value::from(s.to_uppercase())))
}
