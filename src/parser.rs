use std::f32::consts::E;

use crate::{
    ast::{BindingDecl, Decl, Expr, FuncDecl, Param, StructDecl, Type},
    token::TokenKind,
    Token,
};

pub struct ParserError {
    message: String,
    line: usize,
    column: usize,
}

pub struct Parser {
    tokens: Vec<Token>,
    position: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Parser {
            tokens,
            position: 0,
        }
    }

    fn peek(&self) -> &Token {
        &self.tokens[self.position]
    }

    fn advance(&mut self) -> &Token {
        let pos = self.position;
        self.position += 1;
        &self.tokens[pos]
    }

    fn expect(&mut self, kind: TokenKind) -> Result<Token, ParserError> {
        let token = self.peek();
        if token.kind == kind {
            Ok(self.advance().clone())
        } else {
            Err(ParserError {
                message: format!("expected {:?} but got {:?}", kind, token.kind),
                line: token.line,
                column: token.column,
            })
        }
    }

    fn check(&self, kind: TokenKind) -> bool {
        self.peek().kind == kind
    }

    fn parse_params(&mut self) -> Result<Vec<Param>, ParserError> {
        let mut params = Vec::new();
        while self.peek().kind != TokenKind::RightParen {
            let name = self.expect(TokenKind::Identifier)?.value.clone();
            self.expect(TokenKind::Colon)?;
            let type_ann = self.parse_type()?;
            params.push(Param { name, type_ann });
            if self.check(TokenKind::Comma) {
                self.advance();
            } else {
                break;
            }
        }
        Ok(params)
    }

    fn parse_func_decl(&mut self) -> Result<Decl, ParserError> {
        self.expect(TokenKind::At)?;
        self.expect(TokenKind::LeftParen)?;

        let params: Vec<Param> = self.parse_params()?;
        self.expect(TokenKind::RightParen)?;

        let name = self.expect(TokenKind::Identifier)?.value.clone();
        self.expect(TokenKind::Arrow)?;

        let type_ann = self.parse_type()?;

        self.expect(TokenKind::LeftBrace)?;
        while self.peek().kind != TokenKind::RightBrace {
            self.advance();
        }
        self.expect(TokenKind::RightBrace)?;

        Ok(Decl::Func(FuncDecl {
            name,
            params,
            return_type: type_ann,
            body: Vec::new(),
        }))
    }

    fn parse_struct_decl(&mut self) -> Result<Decl, ParserError> {
        self.expect(TokenKind::Struct)?;
        let name = self.expect(TokenKind::Identifier)?.value.clone();
        self.expect(TokenKind::LeftBrace)?;

        let mut methods = Vec::new();
        let mut variables = Vec::new();

        while self.peek().kind != TokenKind::RightBrace {
            if self.peek().kind == TokenKind::At {
                if let Decl::Func(func) = self.parse_func_decl()? {
                    methods.push(func);
                }
            } else {
                let field_name = self.expect(TokenKind::Identifier)?.value.clone();
                self.expect(TokenKind::Colon)?;
                let field_type = self.parse_type()?;
                variables.push(Param {
                    name: field_name,
                    type_ann: field_type,
                });
                if self.check(TokenKind::Comma) {
                    self.advance();
                }
            }
        }
        self.expect(TokenKind::RightBrace)?;
        Ok(Decl::Struct(StructDecl {
            name,
            variables,
            methods,
        }))
    }

    fn parse_binding_decl(&mut self) -> Result<Decl, ParserError> {
        let mutable = self.check(TokenKind::Mut);
        if mutable {
            self.advance();
        }
        let name = self.expect(TokenKind::Identifier)?.value.clone();
        let type_ann = if self.check(TokenKind::Colon) {
            self.advance();
            Some(self.parse_type()?)
        } else {
            None
        };
        self.expect(TokenKind::Equals)?;
        Ok(Decl::Binding(BindingDecl {
            mutable,
            name,
            type_ann,
            value: self.parse_expr()?,
        }))
    }

    fn parse_expr(&mut self) -> Result<Expr, ParserError> {
        match self.peek().kind {
            TokenKind::IntLiteral => {
                let token = self.advance().clone();
                let value = token.value.parse::<i64>().map_err(|_| ParserError {
                    message: format!("invalid integer '{}'", token.value),
                    line: token.line,
                    column: token.column,
                })?;
                Ok(Expr::Int(value))
            }
            TokenKind::FltLiteral => {
                let token = self.advance().clone();
                let value = token.value.parse::<f64>().map_err(|_| ParserError {
                    message: format!("invalid float '{}'", token.value),
                    line: token.line,
                    column: token.column,
                })?;
                Ok(Expr::Flt(value))
            }
            TokenKind::StrLiteral => {
                let token = self.advance().clone();
                Ok(Expr::Str(token.value))
            }
            _ => {
                let tok = self.peek();
                Err(ParserError {
                    message: format!(
                        "unexpected token '{}' at line {}, column {}",
                        tok.value, tok.line, tok.column
                    ),
                    line: tok.line,
                    column: tok.column,
                })
            }
        }
    }

    fn parse_type(&mut self) -> Result<Type, ParserError> {
        match self.peek().kind {
            TokenKind::IntType => {
                self.advance();
                Ok(Type::Int)
            }
            TokenKind::FltType => {
                self.advance();
                Ok(Type::Flt)
            }
            TokenKind::StrType => {
                self.advance();
                Ok(Type::Str)
            }
            TokenKind::BoolType => {
                self.advance();
                Ok(Type::Bool)
            }
            TokenKind::AbyssType => {
                self.advance();
                Ok(Type::Abyss)
            }
            TokenKind::LeftBracket => {
                self.advance();
                let inner = self.parse_type()?;
                self.expect(TokenKind::RightBracket)?;
                Ok(Type::List(Box::new(inner)))
            }
            TokenKind::LeftBrace => {
                self.advance();
                let key = self.parse_type()?;
                self.expect(TokenKind::Colon)?;
                let val = self.parse_type()?;
                self.expect(TokenKind::RightBrace)?;
                Ok(Type::Map(Box::new(key), Box::new(val)))
            }
            TokenKind::Identifier => {
                let name = self.peek().value.clone();
                self.advance();
                Ok(Type::Named(name))
            }
            _ => {
                let token = self.peek();
                Err(ParserError {
                    message: format!("expected type, got {:?}", token.kind),
                    line: token.line,
                    column: token.column,
                })
            }
        }
    }

    pub fn parse_decl(&mut self) -> Result<Decl, ParserError> {
        match self.peek().kind {
            TokenKind::At => self.parse_func_decl(),
            TokenKind::Struct => self.parse_struct_decl(),
            _ => self.parse_binding_decl(),
        }
    }

    pub fn parse(&mut self) -> Vec<Decl> {
        let mut decls = Vec::new();
        while !self.check(TokenKind::EOF) {
            match self.parse_decl() {
                Ok(decl) => decls.push(decl),
                Err(e) => {
                    eprintln!("{}", e.message);
                    break;
                }
            }
        }
        decls
    }
}
