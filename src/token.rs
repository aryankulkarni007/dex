#[derive(Debug)]
#[allow(dead_code)]
pub enum TokenKind {
    IntLiteral,
    FloatLiteral,
    StringLiteral,

    Flex,
    If,
    Else,
    When,
    In,
    Then,
    Structure,
    Error,
    I,
    AbyssType,
    IntegerType,
    FloatType,
    StringType,
    BooleanType,
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

#[derive(Debug)]
#[allow(dead_code)]
pub struct Token {
    pub kind: TokenKind,
    pub value: String,
    pub line: usize,
    pub column: usize,
}
