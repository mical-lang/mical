mod utils;

#[test]
fn basic_entries() {
    let source = include_str!("./cases/basic_entries.mical");
    let snapshot = utils::make_snapshot("basic_entries", source);
    utils::assert_snapshot!("", snapshot);
}

#[test]
fn typed_values() {
    let source = include_str!("./cases/typed_values.mical");
    let snapshot = utils::make_snapshot("typed_values", source);
    utils::assert_snapshot!("", snapshot);
}

#[test]
fn quoted_strings() {
    let source = include_str!("./cases/quoted_strings.mical");
    let snapshot = utils::make_snapshot("quoted_strings", source);
    utils::assert_snapshot!("", snapshot);
}

#[test]
fn quoted_keys() {
    let source = include_str!("./cases/quoted_keys.mical");
    let snapshot = utils::make_snapshot("quoted_keys", source);
    utils::assert_snapshot!("", snapshot);
}

#[test]
fn prefix_block() {
    let source = include_str!("./cases/prefix_block.mical");
    let snapshot = utils::make_snapshot("prefix_block", source);
    utils::assert_snapshot!("", snapshot);
}

#[test]
fn nested_prefix_block() {
    let source = include_str!("./cases/nested_prefix_block.mical");
    let snapshot = utils::make_snapshot("nested_prefix_block", source);
    utils::assert_snapshot!("", snapshot);
}

#[test]
fn prefix_block_values() {
    let source = include_str!("./cases/prefix_block_values.mical");
    let snapshot = utils::make_snapshot("prefix_block_values", source);
    utils::assert_snapshot!("", snapshot);
}

#[test]
fn block_string_literal() {
    let source = include_str!("./cases/block_string_literal.mical");
    let snapshot = utils::make_snapshot("block_string_literal", source);
    utils::assert_snapshot!("", snapshot);
}

#[test]
fn block_string_strip() {
    let source = include_str!("./cases/block_string_strip.mical");
    let snapshot = utils::make_snapshot("block_string_strip", source);
    utils::assert_snapshot!("", snapshot);
}

#[test]
fn block_string_keep() {
    let source = include_str!("./cases/block_string_keep.mical");
    let snapshot = utils::make_snapshot("block_string_keep", source);
    utils::assert_snapshot!("", snapshot);
}

#[test]
fn block_string_keep_multi_trailing() {
    let source = include_str!("./cases/block_string_keep_multi_trailing.mical");
    let snapshot = utils::make_snapshot("block_string_keep_multi_trailing", source);
    utils::assert_snapshot!("", snapshot);
}

#[test]
fn block_string_clip_trailing() {
    let source = include_str!("./cases/block_string_clip_trailing.mical");
    let snapshot = utils::make_snapshot("block_string_clip_trailing", source);
    utils::assert_snapshot!("", snapshot);
}

#[test]
fn block_string_empty_body() {
    let source = include_str!("./cases/block_string_empty_body.mical");
    let snapshot = utils::make_snapshot("block_string_empty_body", source);
    utils::assert_snapshot!("", snapshot);
}

#[test]
fn block_string_empty_lines_inside() {
    let source = include_str!("./cases/block_string_empty_lines_inside.mical");
    let snapshot = utils::make_snapshot("block_string_empty_lines_inside", source);
    utils::assert_snapshot!("", snapshot);
}

#[test]
fn block_string_extra_indent() {
    let source = include_str!("./cases/block_string_extra_indent.mical");
    let snapshot = utils::make_snapshot("block_string_extra_indent", source);
    utils::assert_snapshot!("", snapshot);
}

#[test]
fn block_string_folded() {
    let source = include_str!("./cases/block_string_folded.mical");
    let snapshot = utils::make_snapshot("block_string_folded", source);
    utils::assert_snapshot!("", snapshot);
}

#[test]
fn block_string_folded_strip() {
    let source = include_str!("./cases/block_string_folded_strip.mical");
    let snapshot = utils::make_snapshot("block_string_folded_strip", source);
    utils::assert_snapshot!("", snapshot);
}

#[test]
fn block_string_folded_keep() {
    let source = include_str!("./cases/block_string_folded_keep.mical");
    let snapshot = utils::make_snapshot("block_string_folded_keep", source);
    utils::assert_snapshot!("", snapshot);
}

#[test]
fn block_string_in_prefix() {
    let source = include_str!("./cases/block_string_in_prefix.mical");
    let snapshot = utils::make_snapshot("block_string_in_prefix", source);
    utils::assert_snapshot!("", snapshot);
}

#[test]
fn duplicate_keys() {
    let source = include_str!("./cases/duplicate_keys.mical");
    let snapshot = utils::make_snapshot("duplicate_keys", source);
    utils::assert_snapshot!("", snapshot);
}

#[test]
fn comments_directives() {
    let source = include_str!("./cases/comments_directives.mical");
    let snapshot = utils::make_snapshot("comments_directives", source);
    utils::assert_snapshot!("", snapshot);
}

#[test]
fn shebang() {
    let source = include_str!("./cases/shebang.mical");
    let snapshot = utils::make_snapshot("shebang", source);
    utils::assert_snapshot!("", snapshot);
}

#[test]
fn invalid_escape() {
    let source = include_str!("./cases/invalid_escape.mical");
    let snapshot = utils::make_snapshot("invalid_escape", source);
    utils::assert_snapshot!("", snapshot);
}
