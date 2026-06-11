#[macro_export]
macro_rules! scanner_of {
    ($src:expr) => {
        ::mical_cli_lexer::scan_lines($src)
            .next()
            .map_or_else(::mical_cli_lexer::LineScanner::empty, |line| line.scan())
    };
}

#[macro_export]
macro_rules! assert_lines {
    ($src:expr, [$( ($text:literal, term: $term:literal, indent: $indent:literal, $head:ident) ),* $(,)?]) => {{
        let actual = ::mical_cli_lexer::scan_lines($src)
            .map(|line| (line.text(), line.terminator_len(), line.indent(), line.head()))
            .collect::<Vec<_>>();
        let expected = vec![$(
            ($text, $term, $indent, ::mical_cli_lexer::LineHead::$head)
        ),*];
        ::pretty_assertions::assert_eq!(actual, expected);
    }};
}

#[macro_export]
macro_rules! assert_word {
    ($src:literal, $len:literal) => {
        ::pretty_assertions::assert_eq!($crate::scanner_of!($src).scan_word(), $len)
    };
}

#[macro_export]
macro_rules! assert_quoted {
    ($src:literal, None) => {
        ::pretty_assertions::assert_eq!($crate::scanner_of!($src).scan_quoted(), None)
    };
    ($src:literal, { $($fields:tt)* }) => {
        ::pretty_assertions::assert_eq!(
            $crate::scanner_of!($src).scan_quoted(),
            Some(::mical_cli_lexer::QuotedToken { $($fields)* })
        )
    };
}

#[macro_export]
macro_rules! assert_separator {
    ($src:literal, space: $space:literal, tab_run: $tab_run:literal) => {
        ::pretty_assertions::assert_eq!(
            $crate::scanner_of!($src).scan_separator(),
            ::mical_cli_lexer::Separator { space_len: $space, tab_run_len: $tab_run }
        )
    };
}

#[macro_export]
macro_rules! assert_split_comment {
    ($src:literal, value: $value:literal, space: $space:literal, comment: $comment:literal) => {
        ::pretty_assertions::assert_eq!(
            $crate::scanner_of!($src).split_comment(),
            ::mical_cli_lexer::SplitComment {
                value_len: $value,
                trailing_space_len: $space,
                comment_len: $comment,
            }
        )
    };
}

#[macro_export]
macro_rules! assert_directive {
    (not $src:literal) => {
        assert!(!::mical_cli_lexer::scan_lines($src).next().is_some_and(|line| line.is_directive()))
    };
    ($src:literal) => {
        assert!(::mical_cli_lexer::scan_lines($src).next().is_some_and(|line| line.is_directive()))
    };
}

#[macro_export]
macro_rules! assert_value {
    ($src:literal, $variant:ident $({ $($fields:tt)* })?) => {{
        let scanner = $crate::scanner_of!($src);
        let value_len = scanner.split_comment().value_len;
        ::pretty_assertions::assert_eq!(
            scanner.classify_value(value_len),
            ::mical_cli_lexer::ValueKind::$variant $({ $($fields)* })?
        )
    }};
}
