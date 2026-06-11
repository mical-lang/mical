# MICAL Test Suite

Language-independent test cases for MICAL implementations, written against
the specification in `doc/src/specification/`.

## Directory Structure

Each test case is a directory containing:

| File          | Required | Description                                 |
| ------------- | -------- | ------------------------------------------- |
| `input.mical` | yes      | Source input to parse and evaluate          |
| `output.json` | yes      | Expected JSON output of evaluation          |
| `error.txt`   | no       | Expected diagnostic messages (one per line) |

`cargo codegen` scans for directories containing `input.mical` and generates
snapshot test entry points.

The parser asserts these cases as CST + syntax error snapshots; the config
crate asserts `output.json`. For error cases, `output.json` records the
best-effort output after recovery. `error.txt` is documentation for
cross-implementation use — it lists the primary spec-mandated diagnostics and
is not read by any Rust code; implementations may emit additional recovery
diagnostics.

## Test Categories

Hand-written cases are grouped by spec chapter via their name prefixes:

- comments, directives, line endings, indentation errors (Syntax & Structure)
- `key-*` / `error-key-*` (Keys)
- `value-*` / `error-value-*` (Values)
- `block-string-*` / `error-block-string-*` (Block Strings)
- `prefix-block-*` / `error-prefix-block-*`, `error-unexpected-indentation`
  (Prefix Blocks)

### block-string-gen-\*

Systematic block-string cases over style (`|`, `>`), explicit indentation
indicator, chomping (clip/strip/keep), body patterns, and termination
(dedent/EOF/EOF without newline):

```
block-string-gen-{style}-[i{n}-]{chomp}-{body}-{termination}
```

They are generated (and regenerated from scratch) by
[generate-block-string-tests.rb](generate-block-string-tests.rb), which
computes the expected output in pure Ruby. Do not edit them by hand.

## Adding a Test

1. Create a directory: `test-suite/{name}/`
2. Add `input.mical` and `output.json` (and `error.txt` if errors are expected)
3. Run `cargo codegen` to regenerate test entry points
4. Run `cargo xtest --workspace` to verify
5. Run `cargo insta review` to accept new snapshots

## Using from Other Implementations

This test suite is designed to be reusable across MICAL implementations in
any language. Each implementation should:

1. Read `input.mical`
2. Parse and evaluate it
3. Compare JSON output against `output.json`
4. If `error.txt` exists, verify that diagnostics match
