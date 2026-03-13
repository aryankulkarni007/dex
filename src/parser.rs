use crate::{
    ast::{BinaryOp, BindingDecl, Decl, Expr, FuncDecl, Param, Stmt, StructDecl, Type, UnaryOp},
    token::TokenKind,
    Token,
};

#[derive(Debug)]
pub struct ParserError {
    message: String,
    line: usize,
    column: usize,
}

pub struct Parser {
    tokens: Vec<Token>,
    position: usize,
    errors: Vec<ParserError>,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Parser {
            tokens,
            position: 0,
            errors: Vec::new(),
        }
    }

    fn synchronize(&mut self) {
        while !matches!(
            self.peek().kind,
            TokenKind::At | TokenKind::Struct | TokenKind::EOF
        ) {
            self.advance();
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
                message: format!(
                    "expected {:?} but got {:?} ('{}') at line {}, column {}",
                    kind, token.kind, token.value, token.line, token.column
                ),
                line: token.line,
                column: token.column,
            })
        }
    }

    fn check(&self, kind: TokenKind) -> bool {
        self.peek().kind == kind
    }

    fn peek_offset(&self, offset: usize) -> &TokenKind {
        let index = self.position + offset;
        if index < self.tokens.len() {
            &self.tokens[index].kind
        } else {
            &TokenKind::EOF
        }
    }

    fn parse_binding(&mut self) -> Result<BindingDecl, ParserError> {
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
        let value = self.parse_expr()?;

        Ok(BindingDecl {
            mutable,
            name,
            type_ann,
            value,
        })
    }

    fn parse_stmt(&mut self) -> Result<Stmt, ParserError> {
        if self.check(TokenKind::Mut) {
            return Ok(Stmt::Binding(self.parse_binding()?));
        }

        if self.check(TokenKind::Identifier) {
            let next = self.peek_offset(1);
            if matches!(next, TokenKind::Equals)
                || (matches!(next, TokenKind::Colon))
                    && !(*self.peek_offset(2) == TokenKind::Identifier
                        && *self.peek_offset(3) == TokenKind::LeftParen)
            {
                return Ok(Stmt::Binding(self.parse_binding()?));
            }
        }
        let expr = self.parse_expr()?;
        if self.check(TokenKind::Equals) {
            self.advance();
            let value = self.parse_expr()?;
            Ok(Stmt::Expr(Expr::Assign(Box::new(expr), Box::new(value))))
        } else {
            Ok(Stmt::Expr(expr))
        }
    }

    fn parse_params(&mut self) -> Result<Vec<Param>, ParserError> {
        let mut params = Vec::new();
        while self.peek().kind != TokenKind::RightParen {
            let name = self.expect(TokenKind::Identifier)?.value.clone();
            let type_ann;
            if self.check(TokenKind::Colon) {
                self.advance();
                type_ann = self.parse_type()?;
            } else {
                type_ann = Type::Inferred;
            }
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

        let body = self.parse_block()?;

        Ok(Decl::Func(FuncDecl {
            name,
            params,
            return_type: type_ann,
            body,
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
    fn parse_pipeline(&mut self) -> Result<Expr, ParserError> {
        let mut left = self.parse_or()?;

        while self.check(TokenKind::PipeLine) {
            self.advance();
            let right = self.parse_or()?;
            left = Expr::Pipeline(Box::new(left), Box::new(right));
        }
        Ok(left)
    }

    fn parse_or(&mut self) -> Result<Expr, ParserError> {
        let mut left = self.parse_and()?;

        while self.check(TokenKind::Or) {
            let op = match self.advance().kind {
                TokenKind::Or => BinaryOp::Or,
                _ => unreachable!(),
            };
            let right = self.parse_and()?;
            left = Expr::Binary(Box::new(left), op, Box::new(right));
        }
        Ok(left)
    }

    fn parse_and(&mut self) -> Result<Expr, ParserError> {
        let mut left = self.parse_comparison()?;

        while self.check(TokenKind::And) {
            let op = match self.advance().kind {
                TokenKind::And => BinaryOp::And,
                _ => unreachable!(),
            };
            let right = self.parse_comparison()?;
            left = Expr::Binary(Box::new(left), op, Box::new(right));
        }
        Ok(left)
    }

    fn parse_comparison(&mut self) -> Result<Expr, ParserError> {
        let mut left = self.parse_addition()?;
        while self.check(TokenKind::DoubleEquals)
            || self.check(TokenKind::LeftAngle)
            || self.check(TokenKind::RightAngle)
            || self.check(TokenKind::GreaterEquals)
            || self.check(TokenKind::LessEqual)
            || self.check(TokenKind::NotEquals)
        {
            let op = match self.advance().kind {
                TokenKind::DoubleEquals => BinaryOp::Equality,
                TokenKind::LeftAngle => BinaryOp::Lesser,
                TokenKind::RightAngle => BinaryOp::Greater,
                TokenKind::GreaterEquals => BinaryOp::Geq,
                TokenKind::LessEqual => BinaryOp::Leq,
                TokenKind::NotEquals => BinaryOp::Nequality,
                _ => unreachable!(),
            };
            let right = self.parse_addition()?;
            left = Expr::Binary(Box::new(left), op, Box::new(right));
        }
        Ok(left)
    }

    fn parse_addition(&mut self) -> Result<Expr, ParserError> {
        let mut left = self.parse_multiplication()?;

        while self.check(TokenKind::Plus) || self.check(TokenKind::Minus) {
            let op = match self.advance().kind {
                TokenKind::Plus => BinaryOp::Add,
                TokenKind::Minus => BinaryOp::Minus,
                _ => unreachable!(),
            };
            let right = self.parse_multiplication()?;
            left = Expr::Binary(Box::new(left), op, Box::new(right));
        }
        Ok(left)
    }

    fn parse_multiplication(&mut self) -> Result<Expr, ParserError> {
        let mut left = self.parse_power()?;

        while self.check(TokenKind::Star)
            || self.check(TokenKind::Slash)
            || self.check(TokenKind::Percent)
        {
            let op = match self.advance().kind {
                TokenKind::Star => BinaryOp::Multiply,
                TokenKind::Slash => BinaryOp::Divide,
                TokenKind::Percent => BinaryOp::Modulo,
                _ => unreachable!(),
            };
            let right = self.parse_power()?;
            left = Expr::Binary(Box::new(left), op, Box::new(right));
        }
        Ok(left)
    }

    fn parse_power(&mut self) -> Result<Expr, ParserError> {
        let left = self.parse_unary()?;

        if self.check(TokenKind::Caret) {
            self.advance();
            let right = self.parse_power()?; // right side recurses
            return Ok(Expr::Binary(
                Box::new(left),
                BinaryOp::Exponent,
                Box::new(right),
            ));
        }

        Ok(left)
    }

    fn parse_unary(&mut self) -> Result<Expr, ParserError> {
        if self.check(TokenKind::Bang) {
            self.advance();
            let expr = self.parse_unary()?; // was parse_primary
            Ok(Expr::Unary(UnaryOp::Not, Box::new(expr)))
        } else if self.check(TokenKind::Minus) {
            self.advance();
            let expr = self.parse_unary()?; // was parse_primary
            Ok(Expr::Unary(UnaryOp::Neg, Box::new(expr)))
        } else {
            let expr = self.parse_primary()?;
            self.parse_postfix(expr)
        }
    }

    fn parse_block(&mut self) -> Result<Vec<Stmt>, ParserError> {
        self.expect(TokenKind::LeftBrace)?;
        let mut stmts = Vec::new();
        while !self.check(TokenKind::RightBrace) && !self.check(TokenKind::EOF) {
            stmts.push(self.parse_stmt()?);
        }
        self.expect(TokenKind::RightBrace)?;
        Ok(stmts)
    }

    fn parse_postfix(&mut self, expr: Expr) -> Result<Expr, ParserError> {
        let mut expr = expr;
        loop {
            if self.check(TokenKind::Dot) {
                self.advance();
                let field_name = self.expect(TokenKind::Identifier)?.value.clone();
                expr = Expr::FieldAccess(Box::new(expr), field_name)
            } else if self.check(TokenKind::Colon)
                && self.peek_offset(1) == &TokenKind::Identifier
                && self.peek_offset(2) == &TokenKind::LeftParen
            {
                self.advance();
                let name = self.expect(TokenKind::Identifier)?.value.clone();
                let args = self.parse_args()?;
                expr = Expr::MethodCall(Box::new(expr), name, args)
            } else if self.check(TokenKind::LeftParen) {
                let args = self.parse_args()?;
                expr = Expr::Call(Box::new(expr), args)
            } else if self.check(TokenKind::Question) {
                self.advance();
                expr = Expr::Try(Box::new(expr))
            } else {
                break;
            }
        }
        Ok(expr)
    }

    fn parse_primary(&mut self) -> Result<Expr, ParserError> {
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
            TokenKind::Identifier => {
                let token = self.advance().clone();
                let name = token.value;
                if self.check(TokenKind::Arrow) {
                    self.advance(); // consume ->
                    let body = self.parse_expr()?;
                    Ok(Expr::Lambda(name, Box::new(body)))
                } else {
                    Ok(Expr::Identifier(name))
                }
            }
            TokenKind::LeftParen => {
                self.advance(); // consume '('
                let expr = self.parse_expr()?;
                self.expect(TokenKind::RightParen)?;
                Ok(expr)
            }
            TokenKind::True => {
                self.advance();
                Ok(Expr::Bool(true))
            }
            TokenKind::False => {
                self.advance();
                Ok(Expr::Bool(false))
            }
            TokenKind::If => {
                self.advance();
                let condition = self.parse_expr()?;
                let body = self.parse_block()?;
                let else_block = if self.check(TokenKind::Else) {
                    self.advance();
                    Some(self.parse_block()?)
                } else {
                    None
                };
                Ok(Expr::If(Box::new(condition), body, else_block))
            }
            TokenKind::I => {
                self.advance();
                self.expect(TokenKind::LeftParen)?;
                let mut vars = Vec::new();
                while !self.check(TokenKind::In) {
                    let var = self.expect(TokenKind::Identifier)?.value.clone();
                    vars.push(var);
                    if self.check(TokenKind::Comma) {
                        self.advance();
                    } else {
                        break;
                    }
                }
                self.expect(TokenKind::In)?;
                let iter = self.parse_expr()?;
                self.expect(TokenKind::RightParen)?;
                self.expect(TokenKind::Arrow)?;
                let body = self.parse_block()?;
                Ok(Expr::Loop(vars, Box::new(iter), body))
            }
            TokenKind::LeftBracket => {
                self.advance();
                let mut lmts = Vec::new();
                while !self.check(TokenKind::RightBracket) {
                    let lmt = self.parse_expr()?;
                    lmts.push(lmt);
                    if self.check(TokenKind::Comma) {
                        self.advance();
                    } else {
                        break;
                    }
                }
                self.expect(TokenKind::RightBracket)?;
                Ok(Expr::ListLiteral(lmts))
            }
            TokenKind::LeftBrace => {
                self.advance();
                let mut pairs = Vec::new();
                while !self.check(TokenKind::RightBrace) {
                    let key = self.parse_expr()?;
                    self.expect(TokenKind::Colon)?;
                    let value = self.parse_expr()?;
                    pairs.push((key, value));
                    if self.check(TokenKind::Comma) {
                        self.advance();
                    } else {
                        break;
                    };
                }
                self.expect(TokenKind::RightBrace)?;
                Ok(Expr::MapLiteral(pairs))
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

    fn parse_expr(&mut self) -> Result<Expr, ParserError> {
        self.parse_pipeline()
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

    fn parse_args(&mut self) -> Result<Vec<Expr>, ParserError> {
        let mut args = Vec::new();
        self.expect(TokenKind::LeftParen)?;

        if !self.check(TokenKind::RightParen) {
            loop {
                args.push(self.parse_expr()?);
                if self.check(TokenKind::Comma) {
                    self.advance();
                } else {
                    break;
                }
            }
        }

        self.expect(TokenKind::RightParen)?;
        Ok(args)
    }

    fn parse_decl(&mut self) -> Result<Decl, ParserError> {
        match self.peek().kind {
            TokenKind::At => self.parse_func_decl(),
            TokenKind::Struct => self.parse_struct_decl(),
            _ => self.parse_binding_decl(),
        }
    }

    pub fn parse(&mut self) -> (Vec<Decl>, Vec<ParserError>) {
        let mut decls = Vec::new();
        let mut errors = Vec::new();

        while !self.check(TokenKind::EOF) {
            match self.parse_decl() {
                Ok(decl) => decls.push(decl),
                Err(e) => {
                    errors.push(e);
                    self.synchronize();
                }
            }
        }
        (decls, errors)
    }
}
