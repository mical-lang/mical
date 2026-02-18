# Keys

A key identifies a configuration entry. In an entry, the key is followed by whitespace and then a [value](./values.md). In a [prefix block](./prefix_blocks.md), the key is followed by whitespace and then `{`.

There are two kinds of keys: **word keys** and **quoted keys**.

The separator between a key and a value must be **spaces**. Tab characters in the separator position are not allowed and produce a parse error (see [Key-Value Separator](#key-value-separator)).

## Word Key

A word key is a contiguous sequence of non-whitespace characters. It is terminated by the first space, tab, newline, or EOF.

The set of characters that can start a word key is broad: letters, digits, punctuation marks such as `.`, `-`, `+`, `|`, `>`, `{`, `}`, and essentially any character that is not whitespace or a quote. Notably, tokens like `true`, `false`, and numeric literals (`42`, `-5`) are valid word keys when they appear in key position.

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

A word key consumes all characters until whitespace or EOF, regardless of what those characters are. This means braces embedded within other characters are part of the key:

```mical
foo{    value
a{b     value
```

These produce the keys `foo{` and `a{b`.

### Word key errors

A key alone on a line with no value is a parse error:

```mical
lonely
```

This produces the error: "missing value for the key".

## Quoted Key

A quoted key is enclosed in matching double quotes (`"..."`) or single quotes (`'...'`). Quoted keys may contain any character that is valid in a string literal, including spaces. Empty quoted keys are allowed.

```mical
"double"            value
'single'            value
"key with spaces"   value
""                  value
''                  value
```

Escape sequences inside quoted keys follow the same rules as [quoted strings](./values.md#quoted-string): `\\` and the matching quote character can be escaped with a backslash.

### Quoted key errors

After the closing quote, the next character must be whitespace (space, tab, newline) or EOF. If non-whitespace characters appear immediately after the closing quote, a parse error is produced:

```mical
"quoted"ppp value
```

This produces the error: "unexpected token after quoted key". The characters after the closing quote are discarded, and if a valid value follows a space, it is still parsed.

An unclosed quoted key (where the closing quote is missing before the end of the line) produces the error: "missing closing quote". Since the entire line is consumed into the key, no value is present and an additional error "missing value for the key" is produced.

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
