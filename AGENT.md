# LINDAS Hydrodata Fetcher

This Rust application fetches open data from the [LINDAS
service](https://lindas.admin.ch/). Specifically, it fetches water temperature
data from the FOEN (BAFU) over a SPARQL endpoint.

## Build & Commands

- Run binary: `cargo run`
- Format code: `cargo fmt`
- Run linter: `cargo clippy`
- Run tests: `cargo test`

## Code Style

- Follow Rust conventions
- Always apply rustfmt
- When using imports, use the convention of grouping them by category, separated
  by an empty line: Std imports, external imports, internal imports. Within each
  category, sort alphabetically.
- Use merged imports: One `use` statement per crate
- Sort all dependencies in `Cargo.toml` alphabetically

## Testing

- Run tests with `cargo test`
- Add unit tests to the same module as the code being tested
- Add integration tests on top level

## Architecture

- Single binary

## Security

- Never commit secrets or API keys to repository
- Use environment variables or config files for sensitive data

## Git Workflow

- ALWAYS run `cargo fmt`, `cargo test` and `cargo clippy` before committing

## Configuration

When adding new configuration options, update all relevant places:

1. Example configuration in `config.example.toml`
2. Configuration schemas in `src/config.rs`
3. Documentation in README.md

All configuration keys use consistent naming and MUST be documented.

## Decisions

Whenever there is a situation where you need to choose between two approaches,
don't just pick one. Instead, ask.

This includes:

- Choosing between two possible architectural approaches
- Choosing between two libraries to use
...and similar situations.
