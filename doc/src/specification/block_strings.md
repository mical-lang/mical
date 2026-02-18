# Block Strings

Block strings are multi-line string values. They provide control over indentation stripping, newline handling, and text folding.

## Header Syntax

A block string begins with a header on the same line as the key:

```
key <style>[chomp]
    content line 1
    content line 2
```

The header consists of:

- **Style indicator** (required): `|` for literal style, `>` for folded style.
- **Chomping indicator** (optional): `+` (keep), `-` (strip), or omitted (clip).

After the style and optional chomping indicator, only whitespace (trailing spaces) and a newline (or EOF) may appear. If any other content follows on the same line, **the value is not a block string** and falls back to a [Line String](./values.md#line-string):

```mical
a |not block
b >not fold
c |+not block
d |abc
e > text after
```

All five values above are Line Strings: `"|not block"`, `">not fold"`, `"|+not block"`, `"|abc"`, `"> text after"`.

## Base Indent Detection

The body of a block string starts on the line after the header. The parser scans forward to find the **first line with content** (skipping empty lines and whitespace-only lines). The indentation of that first content line (number of leading spaces) is the **base indent**, denoted \\( I_{base} \\).

Let \\( I_{parent} \\) denote the indentation level of the entry's key (the number of leading spaces on the key's line).

\\( I_{base} \\) must satisfy \\( I_{base} > I_{parent} \\). If the first content line has \\( I_{base} \le I_{parent} \\), the block string has no content lines (the body is empty) and parsing stops.

If no content line exists (only empty lines or EOF follow the header), the block string has an empty body.

### Example

```mical
key |
    content starts here
```

`key` is at indentation 0, so \\( I_{parent} = 0 \\). The first content line has 4 leading spaces, so \\( I_{base} = 4 \\).

## Line Classification

After determining \\( I_{base} \\), the parser processes each subsequent line. Let \\( I_L \\) be the number of leading spaces on line \\( L \\).

1. **Content line** (\\( I_L \ge I_{base} \\)): The first \\( I_{base} \\) spaces are stripped. The remaining characters (including any extra spaces beyond \\( I_{base} \\)) become the line's content.

2. **Block termination** (\\( I_L \le I_{parent} \\)): The block ends. This line belongs to the outer scope and is not part of the block string.

3. **Whitespace-only line** (\\( I_{parent} < I_L < I_{base} \\) and no non-space content after the spaces): treated as an empty line within the block.

4. **Insufficient indentation error** (\\( I_{parent} < I_L < I_{base} \\) and the line has non-space content): produces the error "block string line has insufficient indentation".

5. **Completely empty line** (no characters before the newline): treated as an empty line within the block.

6. **Non-space at column 0**: The block ends (equivalent to case 2 with \\( I_L = 0 \\)).

7. **Tab encountered**: Because tab indentation is globally forbidden, a tab at the start of a line terminates the block and produces an error.

### Continuation condition

After processing a content line or an empty line, the parser checks whether the block continues by peeking at the next line:

- If the next line is empty (newline immediately), the block continues.
- If the next line starts with spaces and has \\( I_L > I_{parent} \\), the block continues (this covers both content lines and error lines).
- If the next line starts with non-space content at column 0, or has \\( I_L \le I_{parent} \\), the block ends.
- If EOF follows, the block ends.

## Indentation Stripping Example

```mical
foo |
  a
   b
```

\\( I_{parent} = 0 \\), \\( I_{base} = 2 \\) (first content line `  a` has 2 spaces).

- Line `  a`: \\( I_L = 2 \ge 2 \\). Strip 2 spaces → content `"a"`.
- Line `   b`: \\( I_L = 3 \ge 2 \\). Strip 2 spaces → content `" b"` (the extra space is preserved).

Result (literal style, default chomp): `"a\n b\n"`.

## Nested Block Strings

When a block string appears inside a [prefix block](./prefix_blocks.md), \\( I_{parent} \\) is the indentation of the entry's key within the prefix block.

```mical
section {
  desc |
    block line
  other value
}
```

Here `desc` is at indentation 2, so \\( I_{parent} = 2 \\). The first content line `    block line` has \\( I_{base} = 4 \\). The line `  other value` has \\( I_L = 2 = I_{parent} \\), so the block ends and `other value` is a separate entry.

## Empty Lines Within a Block

Empty lines (containing only a newline, or only spaces followed by a newline) within the block are preserved as empty lines in the output. Even whitespace-only lines with fewer spaces than \\( I_{base} \\) (but more than \\( I_{parent} \\)) are treated as empty lines, not errors.

```mical
foo |

  a
```

The completely empty line before `··a` is an empty line in the output (where `·` represents a space). Result: `"\na\n"`.

```mical
foo |
·
··a
```

The line with a single space (`·`, \\( I_L = 1 \\), which satisfies \\( 0 < 1 < 2 \\) and is whitespace-only) is also treated as an empty line. Result: `"\na\n"`.

```mical
foo |
···
··a
```

The line with three spaces (`···`, \\( I_L = 3 \\)) satisfies \\( I_L \ge I_{base} = 2 \\). After stripping \\( I_{base} \\) spaces, one space remains, but the line is still whitespace-only (the remaining space is followed by a newline). This is treated as an empty line. Result: `"\na\n"`.

## Styles

### Literal Style (`|`)

In literal style, newlines between content lines are preserved as `\n` in the output.

```mical
key |
  line 1
  line 2
```

```json
{ "key": "line 1\nline 2\n" }
```

### Folded Style (`>`)

In folded style, single newlines between content lines are replaced by spaces. A sequence of two or more newlines (i.e. content separated by empty lines) preserves one newline per empty line.

```mical
text >
  This is a long
  sentence split
  over lines.

  New paragraph.
```

```json
{ "text": "This is a long sentence split over lines.\nNew paragraph.\n" }
```

(Similar to YAML's folded block scalar.)

## Chomping Indicators

Chomping indicators control how trailing newlines at the end of the block string are handled during evaluation:

### Clip (default, no indicator)

All trailing empty lines are removed, then exactly one newline is appended.

```mical
key |
  hello
  world

```

```json
{ "key": "hello\nworld\n" }
```

The trailing empty line in the source is removed during clip, and a single `\n` is appended.

### Strip (`-`)

All trailing newlines are removed. No final newline is appended.

```mical
key |-
  hello
  world

```

```json
{ "key": "hello\nworld" }
```

### Keep (`+`)

All trailing empty lines are preserved.

```mical
key |+
  line


foo bar
```

The two empty lines after `line` (before the block ends at `foo bar`) are all preserved:

```json
{ "key": "line\n\n\n" }
```

The block ends when `foo bar` appears at \\( I_L = 0 = I_{parent} \\).
