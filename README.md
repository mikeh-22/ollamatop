# Ollama Top

A terminal UI for monitoring Ollama model statistics in real time.

## Features

- Live stats that refresh every 2 seconds
- Context window usage gauge
- Token history sparkline (last 20 refreshes)
- Prompt / completion token breakdown
- Navigate between all locally-installed models

## Prerequisites

- [Ollama](https://ollama.com) installed and running
- Rust toolchain (stable)

## Installation

```bash
git clone https://github.com/mikeh-22/ollamatop
cd ollamatop
cargo install --path .
```

## Usage

```bash
ollamatop
```

Or run directly without installing:

```bash
cargo run --release
```

By default the app connects to `http://localhost:11434`. To use a different
host set the `OLLAMA_HOST` environment variable:

```bash
OLLAMA_HOST=http://192.168.1.10:11434 ollamatop
```

### UI layout

```
┌─────────────────────────────────────────────┐
│           Ollama Top  (Ready)               │  header
└─────────────────────────────────────────────┘
  llama3.2:latest | 3B | q4_0 | Modified: …   model info
  Context Usage ████████░░░░░░░  42.3%         gauge
 ┌─────────────────────────────────────────────┐
 │ Statistics                                  │
 │ Response Time:  183.40 ms                   │
 │ Completions:    7                           │
 │ Total Tokens:   298                         │
 │ Current Tokens: 298                         │
 └─────────────────────────────────────────────┘
  Token History  ▁▂▃▄▅▆▇█                     sparkline
  Token Breakdown
  Prompt:      26
  Completion:  298
```

## Keyboard shortcuts

| Key | Action |
|-----|--------|
| `↑` / `k` | Previous model |
| `↓` / `j` | Next model |
| `q` | Quit |

## Development

```bash
cargo build
cargo test --bin ollamatop
cargo clippy -- -D warnings
```

## CI/CD

GitHub Actions runs on every push and pull request:

- Unit tests and Clippy lint on Linux, macOS, and Windows
- Release builds uploaded as artifacts
- Integration tests (requires a running Ollama instance)
- `cargo audit` security scan
