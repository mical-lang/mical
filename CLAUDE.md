# CLAUDE.md

MICAL is a configuration language (like JSON, TOML, YAML). This repository is the Rust CLI implementation (`mical-cli`) that parses `.mical` files and outputs JSON. The language itself is designed to have implementations in multiple programming languages.

The language specification lives in `doc/src/specification/` (mdBook, this repo is the source of truth). Refer to these when implementing language features or fixing parser/config behavior.

## Commands

```bash
cargo xtest --workspace              # Run all tests (requires cargo-nextest)
cargo xtest -p mical-cli-lexer       # Run tests for a single crate
cargo xtest -E 'test(name)'          # Run a single test by name
cargo fmt --all --check              # Check formatting
cargo clippy --workspace             # Lint
cargo codegen                        # Regenerate all codegen
cargo codegen --check                # Verify codegen is up-to-date
cargo insta review                   # Update snapshots interactively
```

CI sets `RUSTFLAGS="-D warnings"` — all warnings are errors.

## Crate Structure

Processing pipeline: source text → lexer → parser (CST) → AST → config evaluation → JSON

All crates depend on `mical-cli-syntax` (prod). Crates do **not** depend on each other in prod — the CLI binary (`main.rs` at root) wires the pipeline. Test code uses dev-dependencies to construct the full pipeline.

```
mical-cli-syntax    SyntaxKind, TokenKind, AST node types (rowan-based)
mical-cli-lexer     source text → token stream
mical-cli-parser    tokens → rowan GreenNode CST + SyntaxErrors
mical-cli-config    AST → flat key-value Config, JSON output
mical-cli-formatter .mical file formatter via CST visitor
```

## Code Generation

`cargo codegen` generates files from grammar definitions and `test-suite/`. Do not edit generated files by hand.

- `crates/syntax/src/` — `SyntaxKind` enum, AST node types
- `crates/parser/tests/snapshots.rs` — parser snapshot test entries
- `crates/config/tests/snapshots.rs` — config snapshot test entries
- `crates/formatter/src/visitor.rs` — `FormatVisitor` trait

After modifying syntax definitions or adding test cases to `test-suite/`, run `cargo codegen` and commit the output.

## Testing

- **cargo-nextest** required (`cargo xtest` wraps it)
- Test inputs live in `test-suite/{name}/` — each has `input.mical`, `output.json`, optionally `error.txt`
- Snapshot tests via **insta** — snapshots in `crates/*/tests/snapshots/`
- To add a test: create a directory in `test-suite/` with `input.mical` + `output.json`, then `cargo codegen`
