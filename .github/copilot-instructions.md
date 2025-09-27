# GitHub Copilot Development Instructions for Rust ECS Framework

## Testing Requirements

### Mandatory Test Execution
- **ALWAYS** run `cargo test` at the end of task development
- All tests must compile and pass without failures
- Integration tests use the game and demo modules as specified
- **ALWAYS** run `cargo clippy` at the end of task development
- Address all clippy warnings or justify any ignored warnings in PR comments
- **ALWAYS** run `cargo fmt --check` at the end of task development
- Code must be properly formatted according to rustfmt standards

### Test Coverage Standards
- **Unit tests required** for all new functionalities
- Use existing game modules (`src/game/`) and demos for integration testing:
  - `cargo run game` - Main game integration test
  - `cargo run demo woodcutter` - Woodcutter system demo
  - `cargo run demo navigation` - Navigation system demo
  - `cargo run replay-demo` - Replay analysis demo

## Code Quality Standards

### Compilation Warnings
- **NO compilation warnings** allowed at the end of task development
- If a warning is unavoidable, it MUST be:
  1. Marked as explicitly ignored using `#[allow(warning_type)]`
  2. **Clearly documented in PR comments** explaining why the warning was ignored
  3. Include rationale for why the warning cannot be resolved

### Implementation Standards
- **Complete solutions only** - no temporary, stub, mock, or empty implementations unless explicitly requested
- All functions must have full, working implementations
- If functionality cannot be implemented due to technical limitations, **inform about it in PR comments**

## Technology Limitations Communication
When Copilot encounters impossible or technically infeasible requirements:
- Add clear comments in PR explaining the limitation
- Provide alternative approaches when possible
- Document why the specific approach is not feasible
- This is standard procedure - developers expect AI agents to communicate limitations

## Debugging and Development Tools

### Frame Diff Debugging (CRITICAL - NEVER REMOVE)
- **MANDATORY**: All game loops and demos MUST call `world.print_last_frame_diff();` after each `world.update()` call
- This prints detailed debugging information about what changed in each frame
- Include this in:
  - Main game loop (`cargo run game`)
  - All demo loops (`cargo run demo woodcutter`, `cargo run demo navigation`, `cargo run demo carpenter`, etc.)
  - Any custom simulation loops
- This functionality is crucial for debugging and MUST NEVER be removed or omitted

### Replay Logging System
- Use the existing replay logging system for debugging complex issues
- Recording log files are automatically generated for:
  - Game sessions (`cargo run game`)
  - System interactions during tests
  - Component state changes
- Check replay logs in generated directories when debugging:
  - `replay_logs/` - Standard game replays
  - `test_logs/` - Test session replays
  - `demo_replay_logs/` - Demo session replays

### Development Workflow
1. Run existing tests to understand current state: `cargo test`
2. Implement changes with full functionality
3. Remove deprecated functionality if needed - we're in heavy development mode
  - Don't allow two functionalities doing the same or similar things
4. Add comprehensive unit tests for new features
5. Use game/demo integration tests for end-to-end validation
6. Ensure no compilation, clippy warnings, or formatting issues remain
7. Document any unavoidable compilation or clippy warning suppressions in PR comments
8. Final verification: `cargo test`, `cargo clippy`, and `cargo fmt --check` must pass completely

## Architecture Integration
- This is a debuggable ECS framework emphasizing change tracking and replay functionality
- All new systems should integrate with the existing replay logging infrastructure
- Components should implement the `Diff` trait when appropriate for change tracking
- Systems must declare input/output components using the framework's type system

## Example Integration Test Usage
```bash
# Test core ECS functionality
cargo test

# Test game integration (uses ECS with multiple systems)
cargo run game

# Test specific system demos
cargo run demo woodcutter
cargo run demo navigation

# Test replay functionality
cargo run replay-demo
```

## Quality Assurance Checklist
- [ ] `cargo test` passes without failures
- [ ] No compilation warnings (or properly documented suppressions)
- [ ] `cargo clippy` passes without warnings (or properly documented suppressions)
- [ ] `cargo fmt --check` passes (code is properly formatted)
- [ ] Unit tests added for new functionality  
- [ ] Integration tests use game/demo modules
- [ ] Replay logging checked for debugging capabilities
- [ ] All implementations are complete and functional
- [ ] Technical limitations clearly communicated in PR if any