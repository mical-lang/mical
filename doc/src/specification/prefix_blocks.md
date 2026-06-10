# Prefix Blocks

Prefix blocks group entries under a common key prefix. They are a syntactic convenience that does not introduce nested objects in the evaluated output.

## Syntax

A prefix block consists of:

1. A [key](./keys.md) (word key or quoted key) alone on its line — the **opener**. After [comment stripping](./syntax.md#comments), nothing but the key may remain on the line.
2. A body of items (entries, nested prefix blocks, block strings), each indented **deeper** than the opener's key.

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

### Opener recognition

A key alone on a line is recognized as a block opener when the next item line — skipping blank lines and comment-only lines — is indented deeper than the key:

```mical
server.
  # comment lines do not affect recognition
  host localhost
```

If the next item line is **not** deeper (same indentation, shallower, or EOF), the key is not an opener and the parse error "missing value for the key" is produced (see [Keys](./keys.md#key-without-a-value)):

```mical
lonely
next 1
```

There is no syntax for an empty block: a prefix block always contains at least one item. (A prefix block contributes no output of its own, so an empty block would produce nothing anyway.)

An inline comment after the key does not prevent opener recognition:

```mical
server. # the web server
  host localhost
```

## Body Indentation

The first item of the body determines the **body indent** \\( I_{body} \\), which must be strictly greater than the opener key's indentation \\( I_{key} \\). Every subsequent item of the body must be indented at exactly \\( I_{body} \\).

The amount of indentation is free (any number of spaces ≥ 1 deeper than the opener) and is chosen independently for each block.

An item line ends the block when its indentation is \\( \le I_{key} \\). The indentation of such a line must match the body indent of one of the enclosing blocks (or column 0 for the top level); otherwise the parse error "indentation does not match any enclosing block" is produced.

```mical
a.
  b.
    c 1
x 2
```

The line `x 2` at column 0 closes both `b.` and `a.`.

```json
{
  "a.b.c": 1,
  "x": 2
}
```

```mical
a.
    x 1
  y 2
```

The line `y 2` has indentation 2, which matches neither the body indent of `a.` (4) nor the top level (0) — this is a parse error.

A line indented deeper than \\( I_{body} \\), where the preceding item is not a block opener (and not a block string header), is also a parse error: "unexpected indentation".

```mical
a 1
  b 2
```

`a 1` is an entry, not an opener, so the indented `b 2` is a parse error.

Blank lines and comment-only lines may appear at any indentation and never open or close a block:

```mical
server.
  host localhost
# a comment at column 0 does not close the block
  port 8080
```

```json
{
  "server.host": "localhost",
  "server.port": 8080
}
```

## Prefix Concatenation

During evaluation, the key of the prefix block is prepended to each key inside the block. No separator character (such as `.`) is automatically inserted. The concatenation is a simple string join.

```mical
server
  .host localhost
  .port 8080
```

The inner keys are `.host` and `.port`. Prepending `server` yields `server.host` and `server.port`.

```json
{
  "server.host": "localhost",
  "server.port": 8080
}
```

Equivalently, the dot can be written on the opener instead:

```mical
server.
  host localhost
  port 8080
```

If neither side provides a separator, the prefix is joined directly:

```mical
http_
  port 80
```

```json
{ "http_port": 80 }
```

This is equivalent to writing `http_port 80` at the top level.

## Nesting

Prefix blocks can be nested: an opener inside a block body introduces a deeper body. The prefixes accumulate from the outermost block inward.

```mical
outer
  inner
    key value
```

The key `key` is inside `inner`, which is inside `outer`. The accumulated key is `outerinnerkey`.

```json
{ "outerinnerkey": "value" }
```

To get dotted keys, include the dots explicitly:

```mical
a.
  b.
    c value
```

```json
{ "a.b.c": "value" }
```

Entries and nested blocks can be mixed freely within a body:

```mical
server.
  host localhost
  tls.
    cert /etc/ssl/cert.pem
  port 8080
```

```json
{
  "server.host": "localhost",
  "server.tls.cert": "/etc/ssl/cert.pem",
  "server.port": 8080
}
```

## Blocks With Various Value Types

Entries inside prefix blocks support all value types — Line Strings, Integers, Booleans, Quoted Strings, and [Block Strings](./block_strings.md).

```mical
block.
  str hello world
  num 42
  flag true
  neg -1
  quoted "value"
  text |
    multi
    line
```

```json
{
  "block.str": "hello world",
  "block.num": 42,
  "block.flag": true,
  "block.neg": -1,
  "block.quoted": "value",
  "block.text": "multi\nline\n"
}
```

For block strings inside a prefix block, the parent indentation \\( I_{parent} \\) is the indentation of the entry's key within the body — see [Nested Block Strings](./block_strings.md#nested-block-strings).
