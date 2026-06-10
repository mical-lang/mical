# Values

A value is the right-hand side of a [key](./keys.md)-value entry. MICAL determines the type of a value by inspecting the tokens that follow the separator space on the same line.

## Type Determination Algorithm

After parsing the key and consuming the separator space(s), the parser first strips any [inline comment](./syntax.md#comments) and trailing spaces from the rest of the line, then examines the remaining tokens to decide which value type to produce. The algorithm is:

1. If the first token is a double or single quote (`"` or `'`), parse a **Quoted String**.
2. If the first token is `|` or `>`:
   - If, after an optional indentation indicator (a digit `1`–`9`) and an optional chomping indicator (`+` or `-`), the rest of the line is blank, parse a **[Block String](./block_strings.md)**.
   - Otherwise, fall through to Line String.
3. If the first token is `true` or `false` and the rest of the line is blank, parse a **Boolean**.
4. If the first token is a numeral and the rest of the line is blank, parse an **Integer**.
5. If the first token is `+` or `-`, the second token is a numeral, and the rest of the line is blank, parse an **Integer** (with sign).
6. Otherwise, parse a **Line String** (fallback).

"The rest of the line is blank" means that, after comment stripping, the next token is a newline, EOF, or trailing space(s) followed by newline/EOF.

Because comments are stripped first, an inline comment does not demote a typed value to a Line String:

```mical
a 42 # count
b true # enabled
```

```json
{
  "a": 42,
  "b": true
}
```

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

A `#` inside a quoted string is literal; comments are not recognized within quotes:

```mical
msg "before # after"
```

```json
{ "msg": "before # after" }
```

Newlines cannot appear inside a quoted string; reaching a newline before the closing quote produces the error: "missing closing quote".

A quoted string must be the entire value on the line (an inline comment after it is allowed). If any non-whitespace, non-comment content appears after the closing quote, the error "unexpected token after value" is produced:

```mical
a "value" extra
b "value" # comment
```

The first line is a parse error. The second line is a valid quoted string `"value"`.

## Boolean

The tokens `true` and `false` are parsed as boolean values only when they constitute the entire value on the line (with at most trailing whitespace and an inline comment).

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

Underscores may be used as visual separators within digits. An integer is recognized only when it constitutes the entire value on the line (with at most trailing whitespace and an inline comment).

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

The Line String is the fallback value type. When the value does not match any of the above types, the parser consumes all remaining characters on the line (after comment stripping, up to the newline or EOF) as a single string token.

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

Line Strings preserve quotes and other punctuation literally — there is no array or object syntax in value position:

```mical
a value "quoted" text
b [1, 2, 3]
```

```json
{
  "a": "value \"quoted\" text",
  "b": "[1, 2, 3]"
}
```

Note that quotes appearing mid-line do not protect a `#` from comment recognition; comment stripping is positional (see [Comments](./syntax.md#comments)):

```mical
a value "x # y" tail
```

```json
{ "a": "value \"x" }
```

A `#` not preceded by whitespace is part of the Line String; a `#` preceded by a space starts a comment:

```mical
a hello#world
b hello # comment
```

```json
{
  "a": "hello#world",
  "b": "hello"
}
```

A Line String therefore cannot contain ` # ` — use a Quoted String or a [Block String](./block_strings.md) for such values.

### Trailing spaces

For all value types, trailing spaces before the newline (or before an inline comment) are stripped and not included in the value. This applies uniformly to Booleans, Integers, and Line Strings.

```mical
a hello·
b true·
c 42·
```

(where `·` represents a trailing space)

All three entries have the trailing space stripped: `"hello"`, `true`, `42`.

## Block String

Block Strings are multi-line values. Their syntax and algorithm are described in detail in [Block Strings](./block_strings.md).
