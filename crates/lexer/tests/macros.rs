#[macro_export]
macro_rules! assert_token {
    // Entry point
    ($src:literal, [$($tokens:tt)*]) => {
        assert_token!(@normalize $src [] $($tokens)*)
    };

    // Normalize: () -> {}
    (@normalize $src:literal [$($normalized:tt)*] $kind:ident ( $len:literal ) $(, $($rest:tt)*)?) => {
        assert_token!(@normalize $src [$($normalized)* $kind { $len },] $($($rest)*)?)
    };
    // Normalize: {} (pass through)
    (@normalize $src:literal [$($normalized:tt)*] $kind:ident { $($fields:tt)* } $(, $($rest:tt)*)?) => {
        assert_token!(@normalize $src [$($normalized)* $kind { $($fields)* },] $($($rest)*)?)
    };
    // Normalize done -> check
    (@normalize $src:literal [$($normalized:tt)*]) => {
        assert_token!(@check $src [$($normalized)*])
    };

    // Check step: actual assertion
    (@check $src:literal [$( $kind:ident { $len:literal $(, $($field_name:ident: $field_expr:expr),* $(,)? )?} ),* $(,)?]) => {
        let tokens = ::mical_lexer::tokenize($src).collect::<Vec<_>>();
        let mut i = 0;
        #[allow(unused_assignments)]
        {$(
            let token = &tokens[i];
            ::pretty_assertions::assert_eq!(token, &::mical_syntax::token::Token {
                len: $len,
                kind: ::mical_syntax::token::TokenKind::$kind $({ $($field_name: $field_expr),* })?
            });
            i += 1;
        )*}
        ::pretty_assertions::assert_eq!(None, tokens.get(i));
    };
}
