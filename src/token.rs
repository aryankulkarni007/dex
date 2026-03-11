#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
pub enum TokenKind {
    IntLiteral,
    FltLiteral,
    StrLiteral,

    Mut,
    If,
    Else,
    // Match,
    In,
    Then,
    Struct,
    Error,
    I,
    AbyssType,
    IntType,
    FltType,
    StrType,
    BoolType,
    True,
    False,

    Equals,
    Colon,
    Semicolon,
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    LeftBracket,
    RightBracket,
    Arrow,
    PipeArrow,
    PipeLine,
    Pipe,
    Question,
    Bang,
    DoubleBang,
    At,
    Dot,
    DotDot,
    Plus,
    Minus,
    Star,
    Slash,
    Caret,
    Percent,
    LeftAngle,
    RightAngle,
    Underscore,
    Comma,

    DoubleEquals,
    NotEquals,
    LessEqual,
    GreaterEquals,

    And, // &&
    Or,  // ||

    Band,
    Bor,
    Bxor,
    Bnot,

    Identifier,
    EOF,
}

#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
pub struct Token {
    pub kind: TokenKind,
    pub value: String,
    pub line: usize,
    pub column: usize,
}
