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
