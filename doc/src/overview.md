# Language Overview

MICAL is a line-oriented configuration language designed for flat structures and readability.
A MICAL file is a sequence of key-value entries. Scope nesting is expressed by indentation, but the output remains logically flat.

## Key-Value Entries

Each line represents a single entry: a key followed by a space and a value.

```mical
host    localhost
port    8080
enabled true
```

```json
{
  "host": "localhost",
  "port": 8080,
  "enabled": true
}
```

## Keys

Keys come in two forms:

- **Word key**: any sequence of non-whitespace characters. `name`, `server.port`, `-flag`, `42` are all valid keys.
- **Quoted key**: a string enclosed in `"` or `'`, allowing spaces and empty keys.

```mical
name        hello
server.port 8080
"user name" Alice
""          empty-key
```

## Values

The type of a value is determined by its content on the line.

- **`true` / `false`** → Boolean (only when the entire value is the keyword alone)
- **Integer literal** (e.g. `42`, `+1`, `-10`) → Integer (only when the entire value is the literal alone)
- **`"..."` / `'...'`** → Quoted String (must be the sole value; trailing content is an error)
- **`|` / `>`** followed by newline → [Block String](#block-strings)
- **Everything else** → Line String (the rest of the line, as-is)

The fallback to Line String is intentional: anything that does not exactly match the typed forms is treated as a plain string.

```mical
flag  true
count 42
name  "Alice"
path  /usr/local/bin
text  10 items
note  true story
```

```json
{
  "flag": true,
  "count": 42,
  "name": "Alice",
  "path": "/usr/local/bin",
  "text": "10 items",
  "note": "true story"
}
```

Note that `10 items` is a Line String (not integer) and `true story` is a Line String (not boolean) because they contain trailing content.

## Comments

A `#` at the start of a line, or preceded by a space, starts a comment that runs to the end of the line. A `#` without a preceding space is part of the value.

```mical
# This is a comment
key value # this is an inline comment
url https://example.com/page#section
```

```json
{
  "key": "value",
  "url": "https://example.com/page#section"
}
```

## Prefix Blocks

Blocks group entries under a common key prefix. A key alone on its line, followed by more-indented entries, opens a block. **Blocks do not create nested objects and do not insert any separator (such as `.`).**

```mical
server.
  host localhost
  port 8080
```

```json
{
  "server.host": "localhost",
  "server.port": 8080
}
```

Since no separator is inserted, omitting the `.` changes the result:

```mical
http_
  port 80
```

This produces the key `http_port`, equivalent to writing `http_port 80`.

## Block Strings

Multi-line string values use the `|` (literal) or `>` (folded) header. Indentation of the line immediately after the header defines the base indent, which is stripped from each line.

```mical
description |
    MICAL is simple.
    It keeps your config clean.
```

```json
{
  "description": "MICAL is simple.\nIt keeps your config clean.\n"
}
```

An explicit indentation indicator (`|2`) fixes the base indent when the first line should itself start with spaces. Chomping indicators (`+` keep, `-` strip, default clip) control trailing newlines. See [Block Strings](./specification/block_strings.md) for the full algorithm.
