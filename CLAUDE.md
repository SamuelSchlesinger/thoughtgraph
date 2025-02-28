# ThoughtGraph Project Guide

## Build Commands
- Build: `cargo build`
- Run tests: `cargo test`
- Run single test: `cargo test test_name`
- Check: `cargo check`
- Format code: `cargo fmt`
- Run clippy lints: `cargo clippy`
- Run with doc tests: `cargo test --doc`

## Code Style Guidelines
- **Formatting**: Use `cargo fmt` for consistent formatting
- **Naming**: Use snake_case for functions/variables, CamelCase for types/structs
- **Documentation**: Every public item must have doc comments (`///`)
- **Error Handling**: Use `Result` with descriptive error messages
- **Imports**: Group imports by std → external → local crates
- **Types**: Prefer strong typing with descriptive names
- **Testing**: Write unit tests for all public functions

## Documentation Guidelines
- All public functions should have doc comments with description, arguments, return values, and examples
- Use markdown formatting in doc comments for readability
- Include examples in doc comments for complex functions
- Add doc tests to validate examples in documentation
- Use consistent voice and tone in documentation

## Project Structure
- Core data structures: `ThoughtID`, `Thought`, `TagID`, `Tag`, `Reference`
- Core operations: `Query` enum, `Command` enum
- Main implementation in `ThoughtGraph` struct
  - `command()` - Apply modifications to the graph
  - `query()` - Search the graph for thoughts
  - `process_auto_references()` - Handle automatic references from content
  - `save_to_file()` / `load_from_file()` - Serialization/deserialization

## Auto-Reference Feature
The auto-reference feature allows users to create connections between thoughts by mentioning thought IDs in square brackets:
- Format: `[thought-id]` in thought content
- The system detects these references and creates bidirectional links
- Creates backreferences automatically for easy navigation
- Auto-references are processed when creating or updating thoughts