use mical_config::Config;
use mical_syntax::{
    SyntaxNode,
    ast::{AstNode, SourceFile},
};
use std::fmt::Write;

pub fn make_snapshot(name: &str, source: &str) -> String {
    let (green, parser_errors) = mical_parser::parse(mical_lexer::tokenize(source));
    assert!(parser_errors.is_empty(), "unexpected parser errors: {:?}", parser_errors);
    let syntax = SyntaxNode::new_root(green);
    let source_file = SourceFile::cast(syntax).unwrap();
    let (config, errors) = Config::from_source_file(source_file);

    let mut f = String::new();
    fn h(f: &mut String, level: u8, text: &str) {
        for _ in 0..level {
            write!(f, "#").unwrap();
        }
        writeln!(f, " {}\n", text).unwrap();
    }
    fn code(f: &mut String, lang: &str, text: &str) {
        writeln!(f, "```{lang}").unwrap();
        writeln!(f, "{}", text).unwrap();
        writeln!(f, "```\n").unwrap();
    }

    h(&mut f, 1, name);

    h(&mut f, 2, "Input");
    code(&mut f, "mical", source);

    if !errors.is_empty() {
        h(&mut f, 2, "Error");
        let error_text: String =
            errors.iter().map(|e| format!("{e}")).collect::<Vec<_>>().join("\n");
        code(&mut f, "", &error_text);
    }

    h(&mut f, 2, "Json");
    let json_str = serde_json::to_string_pretty(&config.to_json()).unwrap();
    code(&mut f, "json", &json_str);

    format!("{}vim:ft=markdown", f)
}

#[macro_export]
#[doc(hidden)]
macro_rules! __insta_assert_snapshot_wrapper {
    ($group:expr, $snapshot:expr) => {
        insta::with_settings!({
            prepend_module_to_snapshot => false,
            snapshot_path => format!("snapshots/{}", $group),
            omit_expression => true,
        }, {
            insta::assert_snapshot!($snapshot);
        });
    };
}
pub use __insta_assert_snapshot_wrapper as assert_snapshot;
