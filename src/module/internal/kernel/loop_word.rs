use std::collections::VecDeque;
use std::convert::{TryFrom, TryInto};

use crate::compiler::Compiler;
use crate::env::Environment;
use crate::shared::{shared, Shared};
use crate::value::{SharedValue, Value, ValueData};
use crate::vm::{OpCode, VM};

use crate::error::{Result, VMError};

use super::KERNEL;

// TODO: `loop` and `inline` can be handled similarly
// TODO: would i need to compile lambdas directly, though?
pub fn loop_word(vm: &mut VM, _compiler: &mut Compiler) -> Result<()> {
    log::trace!("loop_word");
    let env = vm.environment.as_ref().unwrap().clone();

    let accum = vm.peek().ok_or(VMError::StackUnderflow)?;

    {
        let (start_ip, return_ip) = get_lambda_span(accum)?;

        env.borrow_mut()
            .set_instruction(return_ip - 2, OpCode::Jump as usize)?;
        env.borrow_mut().set_instruction(return_ip - 1, start_ip)?;

        let mut lambda_stack = VecDeque::new();
        lambda_stack.push_front(start_ip);

        while let Some(mut cursor) = lambda_stack.pop_back() {
            loop {
                let op = env.borrow().get_instruction(cursor)?;
                match OpCode::try_from(op) {
                    Ok(OpCode::Break) => {
                        loop_break(&env, return_ip, &mut cursor)?;
                        break;
                    }
                    Ok(OpCode::Continue) => {
                        loop_continue(&env, start_ip, &mut cursor)?;
                        break;
                    }
                    Ok(OpCode::Return) => break,
                    Ok(OpCode::Lit) => {
                        loop_lit(&env, &mut lambda_stack, &mut cursor)?;
                    }
                    Ok(OpCode::Jump) => cursor += 1,
                    _ => {}
                }
                cursor += 1;
            }
        }
    }

    loop_apply(accum);
    Ok(())
}

fn loop_apply(accum: &SharedValue) {
    let mut accum = accum.borrow_mut();
    accum
        .as_list_mut()
        .unwrap()
        .push(shared(Value::new(ValueData::Symbol {
            module: KERNEL.to_string(),
            word: "apply".to_string(),
        })));
}

fn loop_lit(
    env: &Shared<Environment>,
    lambda_stack: &mut VecDeque<usize>,
    cursor: &mut usize,
) -> Result<()> {
    *cursor += 1;

    let const_index = env.borrow().get_instruction(*cursor)?;
    let env = env.borrow();
    let lit = env
        .get_constant(const_index)
        .ok_or(VMError::InvalidConstantIndex(const_index))?;

    if let Some(child_ip) = lit.as_function().and_then(|f| f.get_ip()) {
        lambda_stack.push_front(child_ip);
    }

    Ok(())
}

fn loop_continue(env: &Shared<Environment>, start_ip: usize, cursor: &mut usize) -> Result<()> {
    *cursor += 1;
    let next: OpCode = env.borrow().get_instruction(*cursor)?.try_into()?;
    if matches!(next, OpCode::Nop) {
        env.borrow_mut().set_instruction(*cursor, start_ip)?;
    }
    Ok(())
}

fn loop_break(env: &Shared<Environment>, return_ip: usize, cursor: &mut usize) -> Result<()> {
    *cursor += 1;
    let next: OpCode = env.borrow().get_instruction(*cursor)?.try_into()?;
    if matches!(next, OpCode::Nop) {
        env.borrow_mut().set_instruction(*cursor, return_ip)?;
    }
    Ok(())
}

fn get_lambda_span(accum: &SharedValue) -> Result<(usize, usize)> {
    let mut accum = accum.borrow_mut();

    let list = accum
        .as_list_mut()
        .ok_or_else(|| VMError::TypeMismatch("macro accumulator list".to_string()))?;
    let last = list.last_mut().ok_or_else(|| {
        VMError::TypeMismatch("loop expects non-empty accumalator list".to_string())
    })?;

    loop_set_lambda_loop(last)?;

    let last = last.borrow();
    let lambda = last.as_function();
    let lambda = lambda.ok_or_else(|| VMError::TypeMismatch("loop expects lambda".to_string()))?;

    let start_ip = lambda
        .get_ip()
        .ok_or_else(|| VMError::TypeMismatch("loop expects compiled lambda".to_string()))?;
    let ip_length = lambda
        .get_length()
        .ok_or_else(|| VMError::TypeMismatch("loop expects compiled lambda".to_string()))?;
    let return_ip = start_ip + ip_length - 1;

    Ok((start_ip, return_ip))
}

fn loop_set_lambda_loop(last: &mut SharedValue) -> Result<()> {
    let mut last = last.borrow_mut();
    let lambda = last.as_function_mut();
    let lambda = lambda.ok_or_else(|| VMError::TypeMismatch("loop expects lambda".to_string()))?;
    lambda.set_loop(true);
    Ok(())
}
