# Prefix Blocks

Prefix blocks group entries under a common key prefix. They are a syntactic convenience that does not introduce nested objects in the evaluated output.

## Syntax

A prefix block consists of:

1. A [key](./keys.md) (word key or quoted key).
2. A space separator.
3. An opening brace `{`.
4. The rest of the line after `{` must be blank (only optional whitespace followed by a newline or EOF). If `{` is followed by non-whitespace content on the same line, the entire thing is parsed as a regular entry with a [Line String](./values.md#line-string) value — not as a block.
5. A body of items (entries, nested prefix blocks, comments, directives).
6. A closing brace `}` on its own line, optionally preceded and followed by whitespace.

```mical
section {
  key value
}
```

### Opening brace recognition

The `{` is recognized as a block opener only when it is the sole non-whitespace character remaining on the line after the key separator. If anything else follows on the same line, the `{` is part of the value.

```mical
a { port 80 }
```

This is an entry with key `a` and Line String value `"{ port 80 }"`.

```mical
a {not a block
```

This is an entry with key `a` and Line String value `"{not a block"`.

```mical
a {
  key value
}
```

This is a prefix block because `{` is followed only by a newline.

A `{` with trailing spaces before the newline is also recognized:

```mical
section {··
  key value
}
```

(where `··` represents spaces) — this is still a prefix block.

### Closing brace recognition

The closing `}` is recognized when it appears on a line by itself (optionally surrounded by whitespace). Specifically, the parser checks whether the line consists of optional leading spaces, a `}`, and then only whitespace until the newline or EOF.

If `}` appears on a line with other content, it is treated as a regular key, not as a closing brace:

```mical
section {
  } value
}
```

Here `} value` is an entry within the block (key `}`, value `"value"`), and the standalone `}` on the last line closes the block.

### Missing closing brace

If EOF is reached before a closing `}` is found, the error "missing closing '}' for prefix block" is produced. The block is still included in the AST with a missing close brace.

## Prefix Concatenation

During evaluation, the key of the prefix block is prepended to each key inside the block. No separator character (such as `.`) is automatically inserted. The concatenation is a simple string join.

```mical
server {
  .host localhost
  .port 8080
}
```

The inner keys are `.host` and `.port`. Prepending `server` yields `server.host` and `server.port`.

```json
{
  "server.host": "localhost",
  "server.port": 8080
}
```

If the inner keys do not start with `.`, the prefix is joined directly:

```mical
http_ {
  port 80
}
```

```json
{ "http_port": 80 }
```

This is equivalent to writing `http_port 80` at the top level.

## Nesting

Prefix blocks can be nested. The prefixes accumulate from the outermost block inward.

```mical
outer {
  inner {
    key value
  }
}
```

The key `key` is inside `inner`, which is inside `outer`. The accumulated key is `outerinnerkey`.

```json
{ "outerinnerkey": "value" }
```

To get dotted keys, include the dots explicitly:

```mical
a. {
  b. {
    c value
  }
}
```

```json
{ "a.b.c": "value" }
```

## Blocks With Various Value Types

Entries inside prefix blocks support all value types — Line Strings, Integers, Booleans, Quoted Strings, and [Block Strings](./block_strings.md).

```mical
block {
  str hello world
  num 42
  flag true
  neg -1
  quoted "value"
}
```

```json
{
  "blockstr": "hello world",
  "blocknum": 42,
  "blockflag": true,
  "blockneg": -1,
  "blockquoted": "value"
}
```

## Empty Blocks

A prefix block with no entries is valid:

```mical
empty {
}
```

This produces no key-value entries in the output.
