# Specification

This section provides the detailed specification for the MICAL language.

- **[Syntax & Structure](./syntax.md)**: File encoding, line processing, comments, directives, indentation, and the overall structure of a MICAL file.
- **[Keys](./keys.md)**: Word keys and quoted keys, their syntax rules, and error cases.
- **[Values](./values.md)**: Type determination algorithm, each value type, and the fallback behavior.
- **[Block Strings](./block_strings.md)**: Multi-line string syntax, base indent detection, the explicit indentation indicator, line classification, styles, and chomping indicators.
- **[Prefix Blocks](./prefix_blocks.md)**: Indentation-based block syntax, opener recognition, body indentation rules, prefix concatenation, and nesting.
