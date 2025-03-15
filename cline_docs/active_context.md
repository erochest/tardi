# Active Context

## Current Work Focus
- Implementing compiler and scanner components
- Designing the Program structure for compiled code
- Implementing literal value handling through constants table

## Recent Changes
- Implemented basic VM structure with function pointer table and stack operations
- Created initial error handling system with VMError types
- Set up test infrastructure using cargo-nextest
- Implemented basic stack operations (push/pop)
- Decided on iterator-based token stream for scanner
- Chose 'Program' as the name for compiled code representation
- Planned separate error types for compilation phases

## Next Steps
1. Implement scanner components
   - Create Token and TokenType structures
   - Implement Scanner with iterator interface
   - Add position tracking
2. Implement Program structure
   - Constants table management
   - Operation table construction
   - Instruction generation
3. Implement compiler
   - Token stream consumption
   - Constant pooling
   - Literal handling
4. Update VM to work with Program structure
   - Add lit operation
   - Modify VM to use Program object

## Active Decisions and Considerations
- Using iterator pattern for scanner output
- Program structure will contain constants, instructions, and op_table
- Separate error types for scanner, compiler, and VM phases
- lit operation will use constant table indices
- Decided on Indirect Threading (ITC) for VM implementation to avoid unsafe code while maintaining reasonable performance
- May revisit Direct Threading in the future if performance becomes a critical concern
