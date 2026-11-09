# rust-utils

A collection of versatile Rust utility crates designed to be used in various Rust projects.

## Project Overview

`rust-utils` is a monorepo containing multiple specialized crates for common tasks such as I/O operations, networking, security, and more. Each crate is designed to be independent and highly reusable.

## Repository Structure

```
.
├── .agents/           # AI Agent instructions and skills
├── crates/            # Individual utility crates
│   ├── io/            # I/O related utilities
│   ├── network/       # Networking related utilities
│   └── security/      # Security related utilities
├── .github/           # GitHub Actions workflows
├── Cargo.toml         # Workspace configuration
└── README.md          # Project documentation
```

## Getting Started

### Prerequisites

- [Rust toolchain](https://rustup.rs/) (latest stable)
- `cargo`

### Development

To work on the project, clone the repository and navigate to the root directory.

#### Build all crates

```bash
cargo build --workspace
```

#### Run tests

```bash
cargo test --workspace
```

#### Check formatting and linting

```bash
cargo fmt --all -- --check
cargo clippy --workspace -- -D warnings
```

## Development Workflow

This project follows a GitHub Flow-based strategy with a `dev` branch.

1. Create a new feature branch from `dev`: `git checkout -b feature/<name> dev`
2. Work on your changes.
3. Push your branch and create a Pull Request to `dev`.
4. Once merged into `dev`, changes can be merged into `main` via manual review.

## License

[Specify License Here]
