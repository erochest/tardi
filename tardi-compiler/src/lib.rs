use tardi_core::error::Result;
use tardi_core::module::Module;
use tardi_core::op_code::OpCode;

pub fn initialize() -> Result<Module> {
    let mut module = Module::default();
    module.instructions.push(OpCode::CreateEnvironment);
    Ok(module)
}

#[cfg(test)]
mod tests {
    use tardi_core::op_code::OpCode;

    use super::*;

    #[test]
    fn initialize_returns_empty_tokens_creates_initial_module_and_function() {
        let module = initialize();
        assert!(module.is_ok());
        let module = module.unwrap();
        assert!(
            module
                .instructions
                .iter()
                .any(|op| matches!(op, OpCode::CreateEnvironment))
        );
    }
}
