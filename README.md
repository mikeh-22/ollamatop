# Ollama Top

A terminal UI application to display Ollama model stats, including context window usage.

## Features

- Display Ollama model statistics
- Visual gauge for context window usage
- Real-time sparkline charts for token usage
- Responsive table layout
- Keyboard shortcuts for navigation

## Prerequisites

- Ollama must be installed and running
- Rust toolchain

## Installation

```bash
git clone https://github.com/mikeh/ollamatop
cd ollamatop
cargo install --path .
```

## Usage

```bash
cargo run --release
```

The application will connect to the Ollama API at `http://localhost:11434` and display the current model statistics.

## Keyboard Shortcuts

- `q` - Quit the application
- `r` - Refresh stats manually

## Development

```bash
cargo build
cargo test
cargo clippy
```

## CI/CD

The project includes GitHub Actions workflows for:
- Testing and linting
- Multi-platform builds (Linux, macOS, Windows)
- Dependency caching