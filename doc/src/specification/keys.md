# Keys

A key identifies a configuration entry. In an entry, the key is followed by whitespace and then a [value](./values.md). A key alone on its line (with no value) opens a [prefix block](./prefix_blocks.md) when it is followed by a more-indented item.

There are two kinds of keys: **word keys** and **quoted keys**.

The separator between a key and a value must be **spaces**. Tab characters in the separator position are not allowed and produce a parse error (see [Key-Value Separator](#key-value-separator)).

## Word Key

A word key is a contiguous sequence of non-whitespace characters. It is terminated by the first space, tab, newline, or EOF.

A word key may start with any character that is not whitespace, a quote (`"`, `'`), or `#` (a line starting with `#` is a comment or directive): letters, digits, and punctuation such as `.`, `-`, `+` are all allowed. Notably, tokens like `true`, `false`, and numeric literals (`42`, `-5`) are valid word keys when they appear in key position.

```mical
hello     world
42        value
-57       value
+13       value
true      value
false     value
server.port 8080
```

In all cases above, the first token on the line is the key and the text after the separating space is the value. Because `42`, `-57`, `true`, etc. are followed by content, they are parsed as word keys, not as typed values.

A word key consumes all characters until whitespace or EOF, regardless of what those characters are. After the first character, even quotes and `#` are part of the key:

```mical
a#b     value
can't   value
```

These produce the keys `a#b` and `can't`.

### Key without a value

A key alone on a line is interpreted by looking at the following lines:

- If the next item line (skipping blank lines and comment-only lines) is indented deeper than the key, the key opens a [prefix block](./prefix_blocks.md).
- Otherwise, it is a parse error: "missing value for the key".

```mical
lonely
next 1
```

`lonely` is followed by `next 1` at the same indentation, so it is not a block opener — this produces the error "missing value for the key".

Note that because inline comments are stripped before interpretation, a line like `key # comment` is treated the same as a key alone on its line.

## Quoted Key

A quoted key is enclosed in matching double quotes (`"..."`) or single quotes (`'...'`). Quoted keys may contain any character that is valid in a string literal, including spaces and `#`. Empty quoted keys are allowed.

```mical
"double"            value
'single'            value
"key with spaces"   value
""                  value
''                  value
```

Escape sequences inside quoted keys follow the same rules as [quoted strings](./values.md#quoted-string): `\\` and the matching quote character can be escaped with a backslash.

A quoted key alone on its line can open a prefix block, following the same rule as word keys.

### Quoted key errors

After the closing quote, the next character must be whitespace (space, tab, newline) or EOF. If non-whitespace characters appear immediately after the closing quote, a parse error is produced:

```mical
"quoted"ppp value
```

This produces the error: "unexpected token after quoted key". A `#` immediately after the closing quote (without a separating space) is also this error, not a comment.

An unclosed quoted key (where the closing quote is missing before the end of the line) produces the error: "missing closing quote".

```mical
"unterminated value
```

## Key-Value Separator

The key and the value are separated by one or more spaces. Multiple spaces between the key and the value are allowed and are purely cosmetic:

```mical
a  value
b   42
c    true
```

Tab characters between the key and the value produce the error: "tab separating is not allowed".

## Duplicate Keys

Multiple entries may share the same key. Each occurrence is a distinct entry and all are preserved in the evaluated output.

```mical
tag  web
tag  server
tag  production
```

```json
{
  "tag": ["web", "server", "production"]
}
```

This produces three entries, all with the key `tag`. No entry is overwritten or merged.

The same rule applies to keys formed by [prefix concatenation](./prefix_blocks.md#prefix-concatenation):

```mical
item.
  tag important
item.
  tag urgent
```

```json
{
  "item.tag": ["important", "urgent"]
}
```

Both entries with key `item.tag` are preserved.
