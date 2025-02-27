# ThoughtGraph Project Guide

## Build Commands
- Build: `cargo build`
- Run tests: `cargo test`
- Run single test: `cargo test test_name`
- Check: `cargo check`
- Format code: `cargo fmt`
- Run clippy lints: `cargo clippy`

## Code Style Guidelines
- **Formatting**: Use `cargo fmt` for consistent formatting
- **Naming**: Use snake_case for functions/variables, CamelCase for types/structs
- **Documentation**: Every public item must have doc comments (`///`)
- **Error Handling**: Use `Result` with descriptive error messages
- **Imports**: Group imports by std → external → local crates
- **Types**: Prefer strong typing with descriptive names
- **Testing**: Write unit tests for all public functions

## Project Structure
- Core data structures: `ThoughtID`, `Thought`, `TagID`, `Tag`, `Reference`
- Core operations: `Query` enum, `Command` enum
- Main implementation in `ThoughtGraph` struct