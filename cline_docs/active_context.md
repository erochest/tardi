# Active Context

## Current Work Focus
- Implementing the core VM operations for retrieving literal values using Indirect Threading
- Setting up the basic project structure and development environment

## Recent Changes
- Project initialization
- Creation of the project brief and initial documentation
- Decision to implement VM using Indirect Threading (ITC) for a balance of performance and safety

## Next Steps
1. Implement VM with core operations for retrieving literal values
   - Set up the function pointer table for ITC implementation
   - Implement the basic interpreter loop
   - Add initial literal value operations
2. Add operations for stack manipulation
3. Implement return stack, jumps, and operations for moving data between the data stack and return stack

## Active Decisions and Considerations
- Decided on Indirect Threading (ITC) for VM implementation to avoid unsafe code while maintaining reasonable performance
- Deciding on the syntax for literal values and stack operations
- Considering the best approach for implementing the return stack and jump operations
- May revisit Direct Threading in the future if performance becomes a critical concern

## Open Questions
- What specific literal types should be supported in the initial implementation?
- How should error handling be implemented in the VM?
- What testing strategy should be used for the VM and language features?
