//! Tardi programming language implementation

/// Re-export error types
pub mod error;

/// Virtual Machine implementation
pub mod vm;

/// Scanner implementation
pub mod scanner;

/// Compiler implementation
pub mod compiler;

// Re-exports
pub use compiler::Compiler;
pub use error::Result;
pub use scanner::Scanner;
pub use vm::VM;
pub use vm::value::Value;
