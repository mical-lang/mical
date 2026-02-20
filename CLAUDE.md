# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Build & Development Commands

```bash
cargo build                          # Build debug
cargo build --release                # Build release

cargo xtest --workspace              # Run all tests (requires cargo-nextest)
cargo xtest --workspace --release    # Run tests in release mode
cargo xtest -p mical-cli-lexer       # Run tests for a single crate
cargo xtest -E 'test(name)'          # Run a single test by name

cargo fmt --all --check              # Check formatting
cargo clippy --workspace             # Lint

cargo codegen                        # Regenerate all codegen (syntax + parser + config)
cargo codegen --check                # Verify codegen is up-to-date
```

CI sets `RUSTFLAGS="-D warnings"` — all warnings are errors.

## Architecture

MICAL is a flat, line-oriented configuration language. The CLI parses `.mical` files and outputs JSON.

**Processing pipeline:** source text → lexer → parser (CST) → AST → config evaluation → JSON

### Crate dependency chain

```
mical-cli-syntax   (SyntaxKind, TokenKind, AST node definitions; depends on rowan)
       ↓
mical-cli-lexer    (tokenize: &str → token stream)
       ↓
mical-cli-parser   (parse: tokens → rowan GreenNode CST + SyntaxErrors)
       ↓
mical-cli-config   (Config::from_source_file: AST → flat key-value store, JSON output)
       ↓
mical-cli (main.rs) — CLI binary using clap: `mical eval`, `mical dev`
```

### Code generation (xtask)

`cargo codegen` generates three things:
- **syntax**: `SyntaxKind` enum and AST node types in `crates/syntax/src/`
- **parser**: snapshot test entry points in `crates/parser/tests/snapshots.rs` from `test-suite/` directories
- **config**: snapshot test entry points in `crates/config/tests/snapshots.rs` from `test-suite/` directories

After modifying syntax definitions or adding test cases to `test-suite/`, run `cargo codegen` and commit the generated output.

## Testing

- Uses **cargo-nextest** (required; `cargo xtest` wraps it)
- **Test inputs** live in `test-suite/{name}/` — each directory has `input.mical`, `output.json`, and optionally `error.txt`
- **Snapshot tests** via **insta** — snapshots live in `crates/*/tests/snapshots/`
- **Config tests** also compare JSON output against `test-suite/*/output.json` (direct comparison)
- **Property tests** via **proptest** in `crates/config/tests/query_prefix.rs`
- To add a test: create a directory in `test-suite/` with `input.mical` + `output.json`, then run `cargo codegen`
- To update snapshots: `cargo insta review`
