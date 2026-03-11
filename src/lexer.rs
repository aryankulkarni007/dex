use crate::token::{Token, TokenKind};

pub struct Lexer {
    source: String,
    position: usize,
    line: usize,
    column: usize,
}

impl Lexer {
    pub fn new(source: String) -> Lexer {
        Lexer {
            source,
            position: 0,
            line: 1,
            column: 1,
        }
    }

    fn peek(&self) -> Option<char> {
        self.source[self.position..].chars().next()
    }

    fn peek_next(&self) -> Option<char> {
        // Supposedly this is more Rust way of writing this
        if let Some(current) = self.peek() {
            let next = self.position + current.len_utf8();
            return self.source.get(next..)?.chars().next();
        }
        None
    }

    fn advance(&mut self) -> Option<char> {
        if let Some(current) = self.peek() {
            self.position += current.len_utf8();
            if current == '\n' {
                self.line += 1;
                self.column = 1;
            } else {
                self.column += 1;
            }
            return Some(current);
        }
        None
    }

    pub fn tokenize(&mut self) -> Vec<Token> {
        let mut tokens = Vec::new();
        while let Some(c) = self.peek() {
            match c {
                ' ' | '\t' | '\r' | '\n' => {
                    self.advance();
                }
                '"' => {
                    let line = self.line;
                    let column = self.column;

                    // so that the first apostrophe is not included in the string
                    self.advance();
                    let start = self.position;

                    let mut terminated = false;
                    while let Some(c) = self.peek() {
                        if c == '"' {
                            let end = self.position; // before consuming the quote

                            self.advance();
                            terminated = true;

                            let text = self.source[start..end].to_string();
                            tokens.push(Token {
                                kind: TokenKind::StrLiteral,
                                value: text,
                                line,
                                column,
                            });
                            break;
                        } else {
                            self.advance();
                        }
                    }
                    if !terminated {
                        eprintln!(
                            "unterminated string literal at line {}, col {}",
                            line, column
                        );
                    }
                }
                '#' => {
                    if self.peek_next() == Some('-') {
                        // multiline — consume until -#
                        self.advance(); // #
                        self.advance(); // -
                        while let Some(c) = self.peek() {
                            if c == '-' && self.peek_next() == Some('#') {
                                self.advance(); // -
                                self.advance(); // #
                                break;
                            }
                            self.advance();
                        }
                    } else {
                        // single line — consume until newline
                        while let Some(c) = self.peek() {
                            if c == '\n' {
                                break;
                            }
                            self.advance();
                        }
                    }
                }
                '0'..='9' => {
                    let start = self.position;
                    let line = self.line;
                    let column = self.column;

                    while let Some(c) = self.peek() {
                        if c.is_ascii_digit() {
                            self.advance();
                        } else {
                            break;
                        }
                    }

                    let is_float = self.peek() == Some('.')
                        && self
                            .peek_next()
                            .map(|c| c.is_ascii_digit())
                            .unwrap_or(false);
                    if is_float {
                        self.advance();
                        while let Some(c) = self.peek() {
                            if c.is_ascii_digit() {
                                self.advance();
                            } else {
                                break;
                            }
                        }
                    }

                    // So obvious - we save the start of the number and slice the string until the
                    // last number bro cmon
                    let value = self.source[start..self.position].to_string();
                    let kind = if is_float {
                        TokenKind::FltLiteral
                    } else {
                        TokenKind::IntLiteral
                    };
                    tokens.push(Token {
                        kind,
                        value,
                        line,
                        column,
                    })
                }
                'a'..='z' | 'A'..='Z' | '_' => {
                    let start = self.position;
                    let line = self.line;
                    let column = self.column;

                    while let Some(c) = self.peek() {
                        if c.is_alphanumeric() || c == '_' {
                            self.advance();
                        } else {
                            break;
                        }
                    }

                    let word = self.source[start..self.position].to_string();
                    let kind = match word.as_str() {
                        "mut" => TokenKind::Mut,
                        "if" => TokenKind::If,
                        "else" => TokenKind::Else,
                        // "match" => TokenKind::Match,
                        "in" => TokenKind::In,
                        "then" => TokenKind::Then,
                        "struct" => TokenKind::Struct,
                        "error" => TokenKind::Error,
                        "int" => TokenKind::IntType,
                        "flt" => TokenKind::FltType,
                        "str" => TokenKind::StrType,
                        "bool" => TokenKind::BoolType,
                        "abyss" => TokenKind::AbyssType,
                        "true" => TokenKind::True,
                        "false" => TokenKind::False,
                        "I" => TokenKind::I,
                        "_" => TokenKind::Underscore,
                        "band" => TokenKind::Band,
                        "bor" => TokenKind::Bor,
                        "bxor" => TokenKind::Bxor,
                        "bnot" => TokenKind::Bnot,
                        _ => TokenKind::Identifier,
                    };

                    tokens.push(Token {
                        kind,
                        value: word,
                        line,
                        column,
                    })
                }
                '=' => {
                    let line = self.line;
                    let column = self.column;

                    if self.peek_next() == Some('=') {
                        self.advance();
                        self.advance();
                        tokens.push(Token {
                            kind: TokenKind::DoubleEquals,
                            value: "==".to_string(),
                            line,
                            column,
                        });
                    } else {
                        tokens.push(Token {
                            kind: TokenKind::Equals,
                            value: "=".to_string(),
                            line,
                            column,
                        });
                        self.advance();
                    }
                }
                ':' => {
                    tokens.push(Token {
                        kind: TokenKind::Colon,
                        value: ":".to_string(),
                        line: self.line,
                        column: self.column,
                    });
                    self.advance();
                }
                ';' => {
                    tokens.push(Token {
                        kind: TokenKind::Semicolon,
                        value: ";".to_string(),
                        line: self.line,
                        column: self.column,
                    });
                    self.advance();
                }
                '(' => {
                    tokens.push(Token {
                        kind: TokenKind::LeftParen,
                        value: "(".to_string(),
                        line: self.line,
                        column: self.column,
                    });
                    self.advance();
                }
                ')' => {
                    tokens.push(Token {
                        kind: TokenKind::RightParen,
                        value: ")".to_string(),
                        line: self.line,
                        column: self.column,
                    });
                    self.advance();
                }
                '{' => {
                    tokens.push(Token {
                        kind: TokenKind::LeftBrace,
                        value: "{".to_string(),
                        line: self.line,
                        column: self.column,
                    });
                    self.advance();
                }
                '}' => {
                    tokens.push(Token {
                        kind: TokenKind::RightBrace,
                        value: "}".to_string(),
                        line: self.line,
                        column: self.column,
                    });
                    self.advance();
                }
                '[' => {
                    tokens.push(Token {
                        kind: TokenKind::LeftBracket,
                        value: "[".to_string(),
                        line: self.line,
                        column: self.column,
                    });
                    self.advance();
                }
                ']' => {
                    tokens.push(Token {
                        kind: TokenKind::RightBracket,
                        value: "]".to_string(),
                        line: self.line,
                        column: self.column,
                    });
                    self.advance();
                }
                '?' => {
                    tokens.push(Token {
                        kind: TokenKind::Question,
                        value: "?".to_string(),
                        line: self.line,
                        column: self.column,
                    });
                    self.advance();
                }
                '!' => {
                    let line = self.line;
                    let column = self.column;

                    if self.peek_next() == Some('!') {
                        self.advance();
                        self.advance();
                        tokens.push(Token {
                            kind: TokenKind::DoubleBang,
                            value: "!!".to_string(),
                            line,
                            column,
                        });
                    } else if self.peek_next() == Some('=') {
                        self.advance();
                        self.advance();
                        tokens.push(Token {
                            kind: TokenKind::NotEquals,
                            value: "!=".to_string(),
                            line,
                            column,
                        });
                    } else {
                        tokens.push(Token {
                            kind: TokenKind::Bang,
                            value: "!".to_string(),
                            line,
                            column,
                        });
                        self.advance();
                    }
                }
                '@' => {
                    tokens.push(Token {
                        kind: TokenKind::At,
                        value: "@".to_string(),
                        line: self.line,
                        column: self.column,
                    });
                    self.advance();
                }
                '.' => {
                    let line = self.line;
                    let column = self.column;

                    if self.peek_next() == Some('.') {
                        self.advance();
                        self.advance();
                        tokens.push(Token {
                            kind: TokenKind::DotDot,
                            value: "..".to_string(),
                            line,
                            column,
                        });
                    } else {
                        tokens.push(Token {
                            kind: TokenKind::Dot,
                            value: ".".to_string(),
                            line: self.line,
                            column: self.column,
                        });
                        self.advance();
                    }
                }
                '+' => {
                    tokens.push(Token {
                        kind: TokenKind::Plus,
                        value: "+".to_string(),
                        line: self.line,
                        column: self.column,
                    });
                    self.advance();
                }
                '-' => {
                    let line = self.line;
                    let column = self.column;

                    if self.peek_next() == Some('>') {
                        self.advance();
                        self.advance();
                        tokens.push(Token {
                            kind: TokenKind::Arrow,
                            value: "->".to_string(),
                            line,
                            column,
                        })
                    } else {
                        self.advance();
                        tokens.push(Token {
                            kind: TokenKind::Minus,
                            value: "-".to_string(),
                            line,
                            column,
                        })
                    }
                }
                '~' => {
                    let line = self.line;
                    let column = self.column;

                    if self.peek_next() == Some('>') {
                        self.advance();
                        self.advance();
                        tokens.push(Token {
                            kind: TokenKind::PipeArrow,
                            value: "~>".to_string(),
                            line,
                            column,
                        })
                    } else {
                        eprintln!(
                            "unexpected character '{}' at line {}, col {}",
                            c, self.line, self.column
                        );
                        self.advance();
                    }
                }
                '*' => {
                    tokens.push(Token {
                        kind: TokenKind::Star,
                        value: "*".to_string(),
                        line: self.line,
                        column: self.column,
                    });
                    self.advance();
                }
                '/' => {
                    tokens.push(Token {
                        kind: TokenKind::Slash,
                        value: "/".to_string(),
                        line: self.line,
                        column: self.column,
                    });
                    self.advance();
                }
                '^' => {
                    tokens.push(Token {
                        kind: TokenKind::Caret,
                        value: "^".to_string(),
                        line: self.line,
                        column: self.column,
                    });
                    self.advance();
                }
                '%' => {
                    tokens.push(Token {
                        kind: TokenKind::Percent,
                        value: "%".to_string(),
                        line: self.line,
                        column: self.column,
                    });
                    self.advance();
                }
                '<' => {
                    let line = self.line;
                    let column = self.column;
                    if self.peek_next() == Some('=') {
                        self.advance();
                        self.advance();
                        tokens.push(Token {
                            kind: TokenKind::LessEqual,
                            value: "<=".to_string(),
                            line,
                            column,
                        });
                    } else {
                        tokens.push(Token {
                            kind: TokenKind::LeftAngle,
                            value: "<".to_string(),
                            line: self.line,
                            column: self.column,
                        });
                        self.advance();
                    }
                }
                '>' => {
                    let line = self.line;
                    let column = self.column;

                    if self.peek_next() == Some('>') {
                        self.advance();
                        self.advance();
                        tokens.push(Token {
                            kind: TokenKind::PipeLine,
                            value: ">>".to_string(),
                            line,
                            column,
                        })
                    } else if self.peek_next() == Some('=') {
                        self.advance();
                        self.advance();
                        tokens.push(Token {
                            kind: TokenKind::GreaterEquals,
                            value: ">=".to_string(),
                            line,
                            column,
                        });
                    } else {
                        tokens.push(Token {
                            kind: TokenKind::RightAngle,
                            value: ">".to_string(),
                            line: self.line,
                            column: self.column,
                        });
                        self.advance();
                    }
                }
                ',' => {
                    tokens.push(Token {
                        kind: TokenKind::Comma,
                        value: ",".to_string(),
                        line: self.line,
                        column: self.column,
                    });
                    self.advance();
                }
                '&' => {
                    let line = self.line;
                    let column = self.column;
                    if self.peek_next() == Some('&') {
                        self.advance();
                        self.advance();
                        tokens.push(Token {
                            kind: TokenKind::And,
                            value: "&&".to_string(),
                            line,
                            column,
                        });
                    } else {
                        eprintln!(
                            "unexpected character '&' at line {}, col {}",
                            self.line, self.column
                        );
                        self.advance();
                    }
                }
                '|' => {
                    let line = self.line;
                    let column = self.column;
                    if self.peek_next() == Some('|') {
                        self.advance();
                        self.advance();
                        tokens.push(Token {
                            kind: TokenKind::Or,
                            value: "||".to_string(),
                            line,
                            column,
                        });
                    } else {
                        tokens.push(Token {
                            kind: TokenKind::Pipe,
                            value: "|".to_string(),
                            line,
                            column,
                        });
                        self.advance();
                    }
                }
                _ => {
                    eprintln!(
                        "unexpected character '{}' at line {}, col {}",
                        c, self.line, self.column
                    );
                    self.advance();
                }
            }
        }
        tokens.push(Token {
            kind: TokenKind::EOF,
            value: "".to_string(),
            line: self.line,
            column: self.column,
        });
        tokens
    }
}
