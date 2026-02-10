pub mod syntax_kind;
pub use syntax_kind::*;

pub mod ast;
pub mod token;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum MicalLanguage {}

impl rowan::Language for MicalLanguage {
    type Kind = SyntaxKind;
    fn kind_from_raw(raw: rowan::SyntaxKind) -> Self::Kind {
        SyntaxKind::from(raw)
    }
    fn kind_to_raw(kind: Self::Kind) -> rowan::SyntaxKind {
        kind.into()
    }
}

pub type SyntaxNode = rowan::SyntaxNode<MicalLanguage>;
pub type SyntaxToken = rowan::SyntaxToken<MicalLanguage>;
pub type SyntaxElement = rowan::SyntaxElement<MicalLanguage>;
pub type SyntaxNodeChildren = rowan::SyntaxNodeChildren<MicalLanguage>;
pub type SyntaxElementChildren = rowan::SyntaxElementChildren<MicalLanguage>;
pub type Preorder = rowan::api::Preorder<MicalLanguage>;
pub type PreorderWithTokens = rowan::api::PreorderWithTokens<MicalLanguage>;
pub type SyntaxNodePtr = rowan::ast::SyntaxNodePtr<MicalLanguage>;
pub use rowan::{GreenNode, TextLen, TextRange, TextSize};
