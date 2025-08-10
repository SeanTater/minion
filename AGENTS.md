# AGENTS.md

Agent guidelines for working with the Minion codebase.

## Build/Test Commands
```bash
cargo run                    # Run the game (don't run in terminal - graphics won't work)
cargo build --release       # Release build
cargo check                  # Fast compile check
cargo fmt                    # Format code
cargo clippy                 # Lint code
cargo test                   # Run all tests
cargo test test_name         # Run specific test
cargo test --lib damage      # Run tests in damage module
```

## Code Style
- **Imports**: Use `use crate::` for internal modules, group std/external/internal imports
- **Types**: Use newtype wrappers with `derive_more` (Speed(f32), Damage(f32), Distance(f32))
- **Error handling**: Use `thiserror::Error` with `MinionResult<T>` alias, fail fast with `?`
- **Formatting**: Use inline format strings `println!("foo {bar}")` not `println!("foo {}", bar)`
- **Generics**: Prefer phantom types for type safety (ResourcePool<Health>)
- **Tests**: Use `#[cfg(test)]` modules, test complex logic only
- **Components**: Derive Debug, Clone, Copy, PartialEq for Bevy components
- **Modules**: Re-export everything with `pub use` in mod.rs files
