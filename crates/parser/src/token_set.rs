use mical_cli_syntax::SyntaxKind;

#[derive(Clone, Copy)]
pub(crate) struct TokenSet(u64);

const _: () = const {
    assert!(SyntaxKind::COUNT <= 64);
};

impl TokenSet {
    pub(crate) const fn new<const N: usize>(kinds: [SyntaxKind; N]) -> TokenSet {
        let mut bits = 0;
        let mut i = 0;
        while i < N {
            bits |= mask(kinds[i]);
            i += 1;
        }
        TokenSet(bits)
    }

    pub(crate) const fn contains(&self, kind: SyntaxKind) -> bool {
        self.0 & mask(kind) != 0
    }
}

const fn mask(kind: SyntaxKind) -> u64 {
    debug_assert!(kind as usize <= 64);
    1u64 << (kind as usize)
}
