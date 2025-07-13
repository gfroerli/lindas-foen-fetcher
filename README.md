# LINDAS Hydrodata Fetcher

This Rust application fetches open data from the [LINDAS
service](https://lindas.admin.ch/) and relays it to the [Gfrörli
API](https://github.com/gfroerli/api). Specifically, it fetches water
temperature data from the FOEN (BAFU) over a SPARQL endpoint.

## Configuration

Copy `config.example.toml` to `config.toml` and modify the station IDs as
needed.

### Station IDs

The station IDs correspond to Swiss hydrological monitoring stations. You can
find all available stations at:
<https://www.hydrodaten.admin.ch/en/seen-und-fluesse/stations#temperature>

## Logging

The application uses structured logging with configurable levels. Logging is configured through the `[logging]` section in your config file.

### Log Levels

The `level` field accepts standard env_logger syntax:
- `error` - Only error messages
- `warn` - Warning and error messages  
- `info` - Informational, warning, and error messages
- `debug` - Debug, informational, warning, and error messages
- `trace` - All log messages

You can also specify per-module log levels:
```
level = "info,lindas_hydrodata_fetcher=debug"
```

This sets the default level to `info` but enables `debug` logging for the application modules.

### Examples

```toml
[logging]
# Production: only show important information
level = "info"

# Development: show detailed application logs
level = "info,lindas_hydrodata_fetcher=debug"

# Troubleshooting: show all logs including dependencies
level = "debug"
```

## Build & Commands

- **Run binary**: `cargo run`
- **Format code**: `cargo fmt`
- **Run linter**: `cargo clippy`
- **Run tests**: `cargo test`

## Usage

1. Ensure you have a `config.toml` file in the project root
2. Run the application with `cargo run`
3. The application will fetch the latest water temperature data for all
   configured stations

## Development

Before committing, always run:

    cargo fmt && cargo test && cargo clippy

## License

Copyright © 2025 Coredump Hackerspace.

Licensed under the AGPLv3 or later, see `LICENSE.md`.
