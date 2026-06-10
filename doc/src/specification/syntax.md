# Syntax & Structure

MICAL source files are UTF-8 encoded text files. The language is line-oriented: line breaks serve as the primary delimiters between entries.

## Line Endings

Both LF (`\n`) and CRLF (`\r\n`) are recognized. The lexer normalizes CRLF to a single newline token.

## Whitespace

MICAL distinguishes three whitespace characters:

- **Space** (`U+0020`): used for indentation and as a separator between keys and values.
- **Tab** (`U+0009`): forbidden for indentation. A line that begins with a tab (after any leading spaces) produces a parse error. This rule does not apply inside the body of a [block string](./block_strings.md#line-classification), where lines are classified by their leading spaces alone and tabs beyond the base indent are literal content.
- **Newline** (`U+000A`): terminates lines and entries.

All other characters are non-whitespace and form part of keys or values.

Throughout this specification, a **blank line** (also called a whitespace-only line) is a line that is empty or consists solely of spaces. A line whose leading whitespace includes a tab is not blank — the tab rule above applies to it.

### Indentation

Indentation must consist of spaces only. Indentation is **semantic**: it defines the structure of [Prefix Blocks](./prefix_blocks.md) and the content boundaries of [Block Strings](./block_strings.md).

Top-level items must start at column 0 (no indentation). A line indented deeper than the current context, where no prefix block or block string introduces a new level, is a parse error (see [Prefix Blocks](./prefix_blocks.md#body-indentation)).

Blank lines and comment-only lines are exempt from indentation rules: they may appear at any indentation and never open or close a block.

## Structure of a Source File

A MICAL file consists of a sequence of **items**. Each item is one of:

1. **Entry** — a [key](./keys.md)-[value](./values.md) pair on a single line.
2. **Prefix Block** — a [key](./keys.md) alone on its line, followed by a body of more-indented items. See [Prefix Blocks](./prefix_blocks.md).
3. **Comment** — see [Comments](#comments).
4. **Directive** — begins at column 0 with `#` immediately followed by a word (no space).

Blank lines and comment-only lines are ignored between items.

## Comments

A comment begins with a `#` that is either:

- at the start of a line (after optional indentation), or
- preceded by at least one space,

and extends to the end of the line. The comment, together with any spaces immediately preceding it, is removed before the line is interpreted.

```mical
# This is a full-line comment
key value # this is an inline comment
```

```json
{ "key": "value" }
```

A `#` that is **not** preceded by whitespace is a literal character:

```mical
path /tmp/file#1
key  val#ue
```

```json
{
  "path": "/tmp/file#1",
  "key": "val#ue"
}
```

Comments are **not** recognized in the following contexts, where `#` is always literal:

- inside [quoted strings](./values.md#quoted-string) and [quoted keys](./keys.md#quoted-key),
- inside the body of a [block string](./block_strings.md).

Because comment stripping happens before the value is interpreted, it affects [type determination](./values.md#type-determination-algorithm): `a 42 # count` produces the integer `42`, and `desc | # note` opens a block string.

A value that needs to contain ` # ` (space, hash, space) must be written as a quoted string or a block string:

```mical
msg "before # after"
```

```json
{ "msg": "before # after" }
```

## Directives

A `#` at column 0 (before any indentation) immediately followed by a word (no space between `#` and the word) is parsed as a directive.

```mical
#include path/to/file
#schema https://example.com/schema.json
```

A directive consists of a name (the word after `#`) and arguments (the rest of the line, parsed as a Line String; inline comments are recognized in the argument part). It is syntactically distinct from a comment because there is no space between `#` and the first character.

Directives are a kind of comment: they never contribute key-value entries to the output. Unlike plain comments, they are addressed to the processor and may influence how it evaluates the file — for example, an implementation may support file inclusion via `#include`. An implementation is free to ignore any directive it does not support; no directive is part of the core language semantics.

Directives are recognized only at column 0. A `#` that appears after indentation, or anywhere else on a line (preceded by a space), is always a comment, even if it is immediately followed by a word:

```mical
  #indented_word value
key value #word
```

The first line is a comment. The second line is an entry whose value is `value` — `#word` is an inline comment.
