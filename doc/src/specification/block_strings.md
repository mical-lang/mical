# Block Strings

Block strings are multi-line string values. They provide control over indentation stripping, newline handling, and text folding.

## Header Syntax

A block string begins with a header on the same line as the key:

```
key <style>[indent][chomp]
    content line 1
    content line 2
```

The header consists of, in this fixed order:

- **Style indicator** (required): `|` for literal style, `>` for folded style.
- **Indentation indicator** (optional): a single digit `1`–`9`. See [Explicit Indentation](#explicit-indentation).
- **Chomping indicator** (optional): `+` (keep), `-` (strip), or omitted (clip).

After the header, only whitespace and an optional [inline comment](./syntax.md#comments) may appear before the newline (or EOF). If any other content follows on the same line, **the value is not a block string** and falls back to a [Line String](./values.md#line-string):

```mical
a |not block
b >not fold
c |+not block
d |abc
e |-2
f | # comment
```

Values `a` through `e` are Line Strings: `"|not block"`, `">not fold"`, `"|+not block"`, `"|abc"`, `"|-2"` (the indicators are out of order: indentation must come before chomping). Value `f` **is** a block string, because the comment is stripped before the header is interpreted.

## Base Indent

The body of a block string consists of the lines after the header. Content lines are identified by their indentation relative to a **base indent**, denoted \\( I_{base} \\), which is stripped from each content line.

Let \\( I_{parent} \\) denote the indentation level of the entry's key (the number of leading spaces on the key's line). \\( I_{base} \\) is determined in one of two ways:

### Automatic Detection

When no indentation indicator is given, \\( I_{base} \\) is inferred from **the line immediately after the header**. There is no scanning or skipping:

- If that line is completely empty (no characters before the newline), inference is impossible and the parse error "cannot infer base indent from an empty line" is produced.
- Otherwise, \\( I_{base} \\) is the number of leading spaces on that line. A **whitespace-only line is treated as content** for this purpose: all of its spaces count as leading spaces, and the line itself becomes an empty line within the block.

\\( I_{base} \\) must satisfy \\( I_{base} > I_{parent} \\). If the line has \\( I_{base} \le I_{parent} \\), the block string has no content lines (the body is empty) and that line belongs to the outer scope. If EOF immediately follows the header, the body is empty.

```mical
key |
    content starts here
```

`key` is at indentation 0, so \\( I_{parent} = 0 \\). The next line has 4 leading spaces, so \\( I_{base} = 4 \\).

An empty line immediately after the header is an error:

```mical
key |

  content
```

This produces the error "cannot infer base indent from an empty line". To start a block string with empty lines, use the [explicit indentation indicator](#explicit-indentation) — no inference takes place:

```mical
key |2

  content
```

```json
{ "key": "\ncontent\n" }
```

A whitespace-only line in inference position sets \\( I_{base} \\) by its space count, which may then make the following lines under-indented:

```mical
key |
···
··a
```

(where `·` represents a space) The whitespace-only line has 3 spaces, so \\( I_{base} = 3 \\). The line `··a` then has content at \\( I_L = 2 < I_{base} \\), producing the error "block string line has insufficient indentation".

### Explicit Indentation

When the indentation indicator \\( n \\) is given, the base indent is fixed relative to the key's indentation:

\\[ I_{base} = I_{parent} + n \\]

This is required when the first content line should itself start with spaces, which automatic detection cannot distinguish from indentation:

```mical
key |2
      indented first line
  second line
```

\\( I_{parent} = 0 \\), so \\( I_{base} = 2 \\). The first line has 6 leading spaces; stripping 2 leaves `"    indented first line"`. The second line strips to `"second line"`.

```json
{ "key": "    indented first line\nsecond line\n" }
```

## Line Classification

After determining \\( I_{base} \\), the parser processes each subsequent line. Let \\( I_L \\) be the number of leading spaces on line \\( L \\). The cases below are checked in order; the first match applies.

1. **Blank line** (empty or spaces only; see [Whitespace](./syntax.md#whitespace)): treated as an empty line within the block, regardless of how many spaces it contains.

2. **Content line** (\\( I_L \ge I_{base} \\)): The first \\( I_{base} \\) spaces are stripped. The remaining characters (including any extra spaces beyond \\( I_{base} \\)) become the line's content.

3. **Block termination** (\\( I_L \le I_{parent} \\)): The block ends. This line belongs to the outer scope and is not part of the block string.

4. **Insufficient indentation error** (\\( I_{parent} < I_L < I_{base} \\)): produces the error "block string line has insufficient indentation".

Tabs have no special role in this classification: \\( I_L \\) counts leading **spaces** only, and the general [tab rule](./syntax.md#whitespace) does not apply within the body. A tab on a content line — even immediately after the base indent — is literal content. A line whose leading spaces stop short of \\( I_{base} \\) because of a tab falls under case 3 or 4 like any other line.

The block also ends at EOF. Blank lines never terminate the block: trailing empty lines before the terminating line (or EOF) belong to the block, and the [chomping indicator](#chomping-indicators) decides how they appear in the output.

Comments are never recognized within the block — `#` is a literal character on every line of the body:

```mical
key |
  line 1
  # not a comment
  line 2 # also not a comment
```

```json
{ "key": "line 1\n# not a comment\nline 2 # also not a comment\n" }
```

## Indentation Stripping Example

```mical
foo |
  a
   b
```

\\( I_{parent} = 0 \\), \\( I_{base} = 2 \\) (the line after the header, `  a`, has 2 leading spaces).

- Line `  a`: \\( I_L = 2 \ge 2 \\). Strip 2 spaces → content `"a"`.
- Line `   b`: \\( I_L = 3 \ge 2 \\). Strip 2 spaces → content `" b"` (the extra space is preserved).

Result (literal style, default chomp): `"a\n b\n"`.

## Nested Block Strings

When a block string appears inside a [prefix block](./prefix_blocks.md), \\( I_{parent} \\) is the indentation of the entry's key within the prefix block.

```mical
section.
  desc |
    block line
  other value
```

Here `desc` is at indentation 2, so \\( I_{parent} = 2 \\). The line after the header, `    block line`, has 4 leading spaces, so \\( I_{base} = 4 \\). The line `  other value` has \\( I_L = 2 = I_{parent} \\), so the block ends and `other value` is a separate entry within `section.`.

## Empty Lines Within a Block

After \\( I_{base} \\) has been determined, empty lines (containing only a newline) and whitespace-only lines (containing only spaces followed by a newline) within the block are preserved as empty lines in the output. The number of spaces on a whitespace-only line is irrelevant — it is an empty line whether it has fewer or more spaces than \\( I_{base} \\).

```mical
foo |
  a

  b
```

The completely empty line between `a` and `b` is an empty line in the output. Result: `"a\n\nb\n"`.

```mical
foo |
··a
·····
··b
```

(where `·` represents a space) The line with five spaces is whitespace-only and treated as an empty line. Result: `"a\n\nb\n"`.

Note that the line **immediately after the header** is special when automatic detection is used: it participates in [base indent inference](#automatic-detection), so it must not be completely empty.

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

#### Fold Suppression for More-Indented Lines

Lines whose stripped content starts with spaces (i.e. lines with indentation beyond \\( I_{base} \\)) suppress folding. The newline **before** and **after** a more-indented line is preserved as a literal `\n`, not replaced by a space.

```mical
key >
  a
  b
    c
  d
  e
```

After stripping \\( I_{base} = 2 \\) spaces, the lines are: `"a"`, `"b"`, `"  c"`, `"d"`, `"e"`. Line `"  c"` starts with spaces (more-indented), so:

- The newline between `"b"` and `"  c"` is preserved (not folded).
- The newline between `"  c"` and `"d"` is preserved (not folded).
- Adjacent non-indented lines (`"a"`/`"b"` and `"d"`/`"e"`) are folded as normal.

```json
{ "key": "a b\n  c\nd e\n" }
```

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
