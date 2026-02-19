# Values

A value is the right-hand side of a [key](./keys.md)-value entry. MICAL determines the type of a value by inspecting the tokens that follow the separator space on the same line.

## Type Determination Algorithm

After parsing the key and consuming the separator space(s), the parser examines the remaining tokens on the line to decide which value type to produce. The algorithm is:

1. If the first token is a double or single quote (`"` or `'`), parse a **Quoted String**.
2. If the first token is `|` or `>`:
   - If, after an optional chomping indicator (`+` or `-`), the rest of the line is blank (only whitespace or newline/EOF), parse a **[Block String](./block_strings.md)**.
   - Otherwise, fall through to Line String.
3. If the first token is `true` or `false` and the rest of the line is blank, parse a **Boolean**.
4. If the first token is a numeral and the rest of the line is blank, parse an **Integer**.
5. If the first token is `+` or `-`, the second token is a numeral, and the rest of the line is blank, parse an **Integer** (with sign).
6. Otherwise, parse a **Line String** (fallback).

"The rest of the line is blank" means the next token is a newline, EOF, or a single trailing space followed by newline/EOF.

## Quoted String

A quoted string begins and ends with matching quote characters (`"..."` or `'...'`).

```mical
a "hello"
b 'world'
c ""
d ''
```

```json
{
  "a": "hello",
  "b": "world",
  "c": "",
  "d": ""
}
```

Within a quoted string, the backslash `\` serves as an escape character. The following escape sequences are recognized:

| Sequence | Result              |
|----------|---------------------|
| `\\`     | Literal backslash   |
| `\"`     | Double quote        |
| `\'`     | Single quote        |
| `\n`     | Newline (LF)        |
| `\r`     | Carriage return (CR)|
| `\t`     | Tab                 |

All six sequences are recognized regardless of the quoting style (single or double). Any other character following a backslash is an error.

Newlines cannot appear inside a quoted string; reaching a newline before the closing quote produces the error: "missing closing quote".

A quoted string must be the entire value on the line. If any non-whitespace content appears after the closing quote, the error "unexpected token after value" is produced:

```mical
key "value" extra
```

This is a parse error.

## Boolean

The tokens `true` and `false` are parsed as boolean values only when they constitute the entire value on the line (with at most trailing whitespace).

```mical
a true
b false
```

```json
{
  "a": true,
  "b": false
}
```

If any other content follows on the same line, the value falls back to a Line String:

```mical
a trueish
b falsehood
c true value
d false value
```

```json
{
  "a": "trueish",
  "b": "falsehood",
  "c": "true value",
  "d": "false value"
}
```

## Integer

An integer is an optional sign (`+` or `-`) followed by a numeral. Supported numeral formats:

- Decimal: `0`, `42`, `1_000`
- Binary: `0b1010`
- Octal: `0o777`
- Hexadecimal: `0xFF`, `0xDEAD_BEEF`

Underscores may be used as visual separators within digits. An integer is recognized only when it constitutes the entire value on the line (with at most trailing whitespace).

```mical
a 0
b 42
c +1
d -1
e 0xFF
```

```json
{
  "a": 0,
  "b": 42,
  "c": 1,
  "d": -1,
  "e": 255
}
```

If any other content follows on the same line, the value falls back to a Line String:

```mical
a 42 items
b -10 trailing
c + 1
d +
```

```json
{
  "a": "42 items",
  "b": "-10 trailing",
  "c": "+ 1",
  "d": "+"
}
```

Note that `+ 1` (with a space between the sign and the numeral) is a Line String, not an integer. The sign must be immediately adjacent to the numeral.

## Line String

The Line String is the fallback value type. When the value does not match any of the above types, the parser consumes all remaining characters on the line (up to the newline or EOF) as a single string token.

```mical
key value
name hello world
path /usr/local/bin
```

```json
{
  "key": "value",
  "name": "hello world",
  "path": "/usr/local/bin"
}
```

Line Strings preserve all characters literally, including `#`, quotes, braces, and any other punctuation:

```mical
a hello # not a comment
b { port 80 }
c value "quoted" text
```

```json
{
  "a": "hello # not a comment",
  "b": "{ port 80 }",
  "c": "value \"quoted\" text"
}
```

### Trailing spaces

For all value types, a trailing space before the newline is stripped and not included in the value. This applies uniformly to Booleans, Integers, and Line Strings.

```mical
a hello·
b true·
c 42·
```

(where `·` represents a trailing space)

All three entries have the trailing space stripped: `"hello"`, `true`, `42`.

## Block String

Block Strings are multi-line values. Their syntax and algorithm are described in detail in [Block Strings](./block_strings.md).
