# Syntax & Structure

MICAL source files are UTF-8 encoded text files. The language is line-oriented: line breaks serve as the primary delimiters between entries.

## Line Endings

Both LF (`\n`) and CRLF (`\r\n`) are recognized. The lexer normalizes CRLF to a single newline token.

## Whitespace

MICAL distinguishes three whitespace characters:

- **Space** (`U+0020`): used for indentation and as a separator between keys and values.
- **Tab** (`U+0009`): forbidden for indentation. A line that begins with a tab (after any leading spaces) produces a parse error and the line is skipped.
- **Newline** (`U+000A`): terminates lines and entries.

All other characters are non-whitespace and form part of keys or values.

### Indentation

Indentation must consist of spaces only. It is cosmetic in most contexts (entries, prefix blocks), but semantic inside [Block Strings](./block_strings.md) where it determines content boundaries.

## Structure of a Source File

A MICAL file consists of an optional shebang line followed by a sequence of **items**. Each item is one of:

1. **Entry** — a [key](./keys.md)-[value](./values.md) pair on a single line.
2. **Prefix Block** — a [key](./keys.md) followed by `{`, a body of items, and a closing `}`. See [Prefix Blocks](./prefix_blocks.md).
3. **Comment** — begins with `#` followed by a space, newline, or EOF.
4. **Directive** — begins with `#` immediately followed by a word (no space).

Blank lines (empty or whitespace-only) are ignored between items.

## Comments

A `#` at the beginning of a line (after optional indentation) starts a comment if it is followed by a space (`# ...`), a newline, or EOF.

```mical
# This is a comment
#
```

There are no inline comments. Within a value, the `#` character is literal:

```mical
key value # this is part of the value
```

```json
{ "key": "value # this is part of the value" }
```

## Directives

A `#` at the beginning of a line (before any indentation) immediately followed by a word (no space between `#` and the word) is parsed as a directive.

```mical
#include path/to/file
#version 1.0
```

A directive consists of a name (the word after `#`) and arguments (the rest of the line, parsed as a Line String). It is syntactically distinct from a comment because there is no space between `#` and the first character.

Directives are semantically equivalent to comments from the language's perspective — they do not affect the key-value output. They exist to allow tooling (formatters, LSPs) and applications to embed meta-information:

```mical
#format disable
#scheme https://example.com/schema.json
```

An application that wants to support features like file inclusion can do so via directives (e.g. `#include`), but this is not part of the core language semantics.

Note: a `#` that appears after indentation (spaces) at the start of a line is always treated as a comment, never as a directive, even if it is immediately followed by a word.

```mical
  #indented_word value
```

This line is a comment, not a directive.
