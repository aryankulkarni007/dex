use std::collections::HashMap;

use crate::ast::{BinaryOp, Decl, Expr, Span, Spanned, SpannedExpr, SpannedStmt, Stmt, UnaryOp};

#[derive(Debug, Clone)]
pub enum Value {
    Int(i64),
    Flt(f64),
    Str(String),
    Bool(bool),
    List(Vec<Value>),
    Map(Vec<(Value, Value)>),
    Abyss,
    Error(String),
    Function(Vec<String>, Vec<SpannedStmt>),
}

#[derive(Debug)]
pub struct InterpreterError {
    message: String,
    line: usize,
    column: usize,
}

pub struct Interpreter {
    scopes: Vec<HashMap<String, Value>>,
}

impl Interpreter {
    pub fn new() -> Self {
        Interpreter {
            scopes: vec![HashMap::new()],
        }
    }

    fn lookup(&self, name: &str) -> Option<Value> {
        self.scopes
            .iter()
            .rev()
            .find_map(|scope| scope.get(name).cloned())
    }

    fn error(&self, message: &str, span: &Span) -> InterpreterError {
        InterpreterError {
            message: message.to_string(),
            line: span.line,
            column: span.column,
        }
    }

    fn eval_expr(&mut self, expr: &SpannedExpr) -> Result<Value, InterpreterError> {
        let span = &expr.span;
        match &expr.node {
            Expr::Int(n) => Ok(Value::Int(*n)),
            Expr::Flt(f) => Ok(Value::Flt(*f)),
            Expr::Str(s) => Ok(Value::Str(s.clone())),
            Expr::Bool(b) => Ok(Value::Bool(*b)),
            Expr::Identifier(id) => self
                .lookup(id)
                .ok_or_else(|| self.error(&format!("undefined variable '{}'", id), span)),
            Expr::Binary(left, op, right) => {
                let left = self.eval_expr(left)?;
                let right = self.eval_expr(right)?;
                match op {
                    BinaryOp::Add => match (left, right) {
                        (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a + b)),
                        (Value::Flt(a), Value::Flt(b)) => Ok(Value::Flt(a + b)),
                        _ => Err(self.error("type mismatch: +", span)),
                    },
                    BinaryOp::Minus => match (left, right) {
                        (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a - b)),
                        (Value::Flt(a), Value::Flt(b)) => Ok(Value::Flt(a - b)),
                        _ => Err(self.error("type mismatch: -", span)),
                    },
                    BinaryOp::Multiply => match (left, right) {
                        (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a * b)),
                        (Value::Flt(a), Value::Flt(b)) => Ok(Value::Flt(a * b)),
                        _ => Err(self.error("type mismatch: *", span)),
                    },
                    BinaryOp::Divide => match (left, right) {
                        (Value::Int(a), Value::Int(b)) => {
                            if b == 0 {
                                Err(self.error("division by zero", span))
                            } else {
                                Ok(Value::Int(a / b))
                            }
                        }
                        (Value::Flt(a), Value::Flt(b)) => {
                            if b == 0.0 {
                                Err(self.error("division by zero", span))
                            } else {
                                Ok(Value::Flt(a / b))
                            }
                        }
                        _ => Err(self.error("type mismatch: /", span)),
                    },
                    BinaryOp::Modulo => match (left, right) {
                        (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a % b)),
                        (Value::Flt(a), Value::Flt(b)) => Ok(Value::Flt(a % b)),
                        _ => Err(self.error("type mismatch: %", span)),
                    },
                    BinaryOp::Exponent => match (left, right) {
                        (Value::Int(a), Value::Int(b)) => Ok(Value::Int(i64::pow(a, b as u32))),
                        (Value::Flt(a), Value::Flt(b)) => Ok(Value::Flt(f64::powf(a, b))),
                        _ => Err(self.error("type mismatch: ^", span)),
                    },
                    BinaryOp::Equality => match (left, right) {
                        (Value::Int(a), Value::Int(b)) => Ok(Value::Bool(a == b)),
                        (Value::Flt(a), Value::Flt(b)) => Ok(Value::Bool(a == b)),
                        (Value::Bool(a), Value::Bool(b)) => Ok(Value::Bool(a == b)),
                        (Value::Str(a), Value::Str(b)) => Ok(Value::Bool(a == b)),
                        _ => Err(self.error("type mismatch: ==", span)),
                    },
                    BinaryOp::Nequality => match (left, right) {
                        (Value::Int(a), Value::Int(b)) => Ok(Value::Bool(a != b)),
                        (Value::Flt(a), Value::Flt(b)) => Ok(Value::Bool(a != b)),
                        _ => Err(self.error("type mismatch: !=", span)),
                    },
                    BinaryOp::Lesser => match (left, right) {
                        (Value::Int(a), Value::Int(b)) => Ok(Value::Bool(a < b)),
                        (Value::Flt(a), Value::Flt(b)) => Ok(Value::Bool(a < b)),
                        _ => Err(self.error("type mismatch: <", span)),
                    },

                    BinaryOp::Greater => match (left, right) {
                        (Value::Int(a), Value::Int(b)) => Ok(Value::Bool(a > b)),
                        (Value::Flt(a), Value::Flt(b)) => Ok(Value::Bool(a > b)),
                        _ => Err(self.error("type mismatch: >", span)),
                    },
                    BinaryOp::Geq => match (left, right) {
                        (Value::Int(a), Value::Int(b)) => Ok(Value::Bool(a >= b)),
                        (Value::Flt(a), Value::Flt(b)) => Ok(Value::Bool(a >= b)),
                        _ => Err(self.error("type mismatch: >=", span)),
                    },
                    BinaryOp::Leq => match (left, right) {
                        (Value::Int(a), Value::Int(b)) => Ok(Value::Bool(a <= b)),
                        (Value::Flt(a), Value::Flt(b)) => Ok(Value::Bool(a <= b)),
                        _ => Err(self.error("type mismatch: <=", span)),
                    },

                    BinaryOp::And => match (left, right) {
                        (Value::Bool(a), Value::Bool(b)) => Ok(Value::Bool(a && b)),
                        _ => Err(self.error("type mismatch: &&", span)),
                    },
                    BinaryOp::Or => match (left, right) {
                        (Value::Bool(a), Value::Bool(b)) => Ok(Value::Bool(a || b)),
                        _ => Err(self.error("type mismatch: ||", span)),
                    },
                }
            }
            Expr::Unary(op, right) => {
                let right = self.eval_expr(right)?;
                match op {
                    UnaryOp::Not => match right {
                        Value::Bool(a) => Ok(Value::Bool(!a)),
                        _ => Err(self.error("type mismatch: !", span)),
                    },
                    UnaryOp::Neg => match right {
                        Value::Int(a) => Ok(Value::Int(-a)),
                        Value::Flt(a) => Ok(Value::Flt(-a)),
                        _ => Err(self.error("type mismatch: -", span)),
                    },
                }
            }
            Expr::ListLiteral(lmts) => {
                // we iterate through the list and eval_expr on each lmt
                let list: Result<Vec<Value>, InterpreterError> =
                    lmts.iter().map(|expr| self.eval_expr(expr)).collect();
                Ok(Value::List(list?))
            }
            Expr::MapLiteral(pairs) => {
                let map: Result<Vec<(Value, Value)>, InterpreterError> = pairs
                    .iter()
                    .map(|(key_expr, val_expr)| {
                        let k = self.eval_expr(key_expr)?;
                        let v = self.eval_expr(val_expr)?;
                        Ok((k, v))
                    })
                    .collect();
                Ok(Value::Map(map?))
            }
            Expr::Lambda(param, body) => {
                let stmt = Spanned {
                    node: Stmt::Expr(*body.clone()),
                    span: body.span.clone(),
                };
                Ok(Value::Function(vec![param.clone()], vec![stmt]))
            }
            Expr::If(cond, body, else_block) => {
                let cond_val = self.eval_expr(cond)?;
                match cond_val {
                    Value::Bool(true) => self.eval_block(body),
                    Value::Bool(false) => match else_block {
                        Some(block) => self.eval_block(block),
                        None => Ok(Value::Abyss),
                    },
                    _ => Err(self.error("condition must be a boolean", span)),
                }
            }
            Expr::Loop(vars, iterable, body) => {
                let iter_val = self.eval_expr(iterable)?;
                match iter_val {
                    Value::List(elements) => {
                        for element in elements {
                            self.scopes.push(HashMap::new());
                            self.define(vars[0].clone(), element);
                            let result = self.eval_block(body);
                            self.scopes.pop();
                            result?;
                        }
                        Ok(Value::Abyss)
                    }
                    _ => Err(self.error("can only loop over a list", span)),
                }
            }
            Expr::Assign(target, value) => {
                let val = self.eval_expr(value)?;
                match &target.node {
                    Expr::Identifier(name) => {
                        if self.assign(name, val) {
                            Ok(Value::Abyss)
                        } else {
                            Err(self.error(&format!("undefined variable '{}'", name), span))
                        }
                    }
                    Expr::FieldAccess(_, _) => todo!(),
                    _ => Err(self.error("invalid assignment target", span)),
                }
            }
            Expr::Call(callee, call_args) => {
                let value = self.eval_expr(callee)?;
                match value {
                    Value::Function(param_names, body) => {
                        let args: Result<Vec<Value>, _> =
                            call_args.iter().map(|a| self.eval_expr(a)).collect();
                        let args = args?;
                        self.scopes.push(HashMap::new());
                        for (name, val) in param_names.iter().zip(args) {
                            self.define(name.clone(), val);
                        }
                        let result = self.eval_block(&body);
                        self.scopes.pop();
                        result
                    }
                    _ => todo!(),
                }
            }
            _ => todo!(),
        }
    }

    fn assign(&mut self, name: &str, value: Value) -> bool {
        for scope in self.scopes.iter_mut().rev() {
            if scope.contains_key(name) {
                scope.insert(name.to_string(), value);
                return true;
            }
        }
        false
    }

    fn eval_block(&mut self, stmts: &[SpannedStmt]) -> Result<Value, InterpreterError> {
        let mut last = Value::Abyss;
        for stmt in stmts {
            if let Some(val) = self.eval_stmt(&stmt.node)? {
                last = val;
            }
        }
        Ok(last)
    }

    fn define(&mut self, name: String, value: Value) {
        self.scopes.last_mut().unwrap().insert(name, value);
    }

    fn eval_stmt(&mut self, stmt: &Stmt) -> Result<Option<Value>, InterpreterError> {
        match stmt {
            Stmt::Binding(binding) => {
                let value = self.eval_expr(&binding.value)?;
                self.define(binding.name.clone(), value);
                Ok(None)
            }
            Stmt::Expr(expr) => {
                let value = self.eval_expr(expr)?;
                Ok(Some(value))
            }
        }
    }

    pub fn interpret(&mut self, decls: Vec<Decl>) -> Result<Value, InterpreterError> {
        todo!()
    }
}
