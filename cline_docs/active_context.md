# Active Context

## Current Work Focus
- Implementing literal value handling in the VM
- Designing the instruction format to support both operations and literal values
- Setting up the basic project structure and development environment

## Recent Changes
- Implemented basic VM structure with function pointer table and stack operations
- Created initial error handling system with VMError types
- Set up test infrastructure using cargo-nextest
- Implemented basic stack operations (push/pop)

## Next Steps
1. Extend VM to handle literal values in instruction stream
   - Design instruction format that can represent both operations and literals
   - Implement mechanism to distinguish between operation indices and literal values
   - Add operations for pushing different types of literals (integers, floats, booleans)
2. Add operations for stack manipulation
3. Implement return stack, jumps, and operations for moving data between the data stack and return stack

## Active Decisions and Considerations
- Decided on Indirect Threading (ITC) for VM implementation to avoid unsafe code while maintaining reasonable performance
- Need to design instruction format that can handle both operation indices and literal values
- Considering approaches for encoding literals in the instruction stream:
  - Could use a special opcode for "push literal" followed by the value
  - Or could use a tagged union type for instructions to distinguish ops from literals
- May revisit Direct Threading in the future if performance becomes a critical concern

## Open Questions
- What's the most efficient way to represent literals in the instruction stream?
- How should we distinguish between operation indices and literal values?
- Should we use different opcodes for different literal types (push_int, push_float, etc.)?
