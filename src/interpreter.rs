use std::{collections::HashMap, iter};

use crate::ast::{
    BinaryOp, Decl, Expr, FuncDecl, Span, Spanned, SpannedExpr, SpannedStmt, Stmt, StructDecl,
    UnaryOp,
};

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
    Builtin(String),
    Struct(String, usize),
    StructDef(StructDecl),
}

impl Value {
    pub fn display(&self) -> String {
        match self {
            Value::Int(n) => n.to_string(),
            Value::Flt(f) => {
                if f.fract() == 0.0 {
                    format!("{:.1}", f)
                } else {
                    f.to_string()
                }
            }
            Value::Str(s) => format!("\"{}\"", s),
            Value::Bool(b) => b.to_string(),
            Value::Abyss => "abyss".to_string(),
            Value::List(items) => {
                let inner = items
                    .iter()
                    .map(|item| item.display())
                    .collect::<Vec<String>>()
                    .join(", ");
                format!("[{}]", inner)
            }
            Value::Map(pairs) => {
                let inner = pairs
                    .iter()
                    .map(|(k, v)| format!("{}: {}", k.display(), v.display()))
                    .collect::<Vec<String>>()
                    .join(", ");
                format!("{{ {} }}", inner)
            }
            Value::Error(e) => format!("error: {}", e),
            Value::Function(_, _) => "<function>".to_string(),
            Value::Builtin(name) => format!("<builtin: {}>", name),
            Value::Struct(name, idx) => format!("<{} at {}>", name, idx),
            Value::StructDef(struct_decl) => {
                let fields = struct_decl
                    .variables
                    .iter()
                    .map(|v| v.name.clone())
                    .collect::<Vec<String>>()
                    .join(", ");
                format!("<struct def: {}({})>", struct_decl.name, fields)
            }
        }
    }
}

pub struct InterpreterError {
    message: String,
    line: usize,
    column: usize,
}

pub struct Interpreter {
    scopes: Vec<HashMap<String, Value>>,
    heap: Vec<HashMap<String, Value>>,
}

impl Interpreter {
    pub fn new() -> Self {
        let mut interpreter = Interpreter {
            scopes: vec![HashMap::new()],
            heap: Vec::new(),
        };
        interpreter.register_builtins();
        interpreter
    }

    fn allocate_struct(&mut self, fields: HashMap<String, Value>) -> usize {
        let index = self.heap.len();
        self.heap.push(fields);
        index
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
                    Expr::FieldAccess(object, field_name) => {
                        let obj = self.eval_expr(object)?;
                        if let Value::Struct(_, idx) = obj {
                            self.heap[idx].insert(field_name.clone(), val);
                            Ok(Value::Abyss)
                        } else {
                            Err(self.error("field assignment on non-struct value", span))
                        }
                    }
                    _ => Err(self.error("invalid assignment target", span)),
                }
            }
            Expr::Call(callee, call_args) => {
                let value = self.eval_expr(callee)?;
                match value {
                    Value::Function(param_names, body) => {
                        let args = call_args
                            .iter()
                            .map(|a| self.eval_expr(a))
                            .collect::<Result<Vec<_>, _>>()?;
                        self.scopes.push(HashMap::new());
                        param_names
                            .iter()
                            .zip(args)
                            .for_each(|(name, val)| self.define(name.clone(), val));
                        let result = self.eval_block(&body);
                        self.scopes.pop();
                        result
                    }
                    Value::Builtin(name) => match name.as_str() {
                        "print" => {
                            call_args.iter().try_for_each(
                                |arg| -> Result<(), InterpreterError> {
                                    let val = self.eval_expr(arg)?;
                                    println!("{}", val.display());
                                    Ok(())
                                },
                            )?;

                            Ok(Value::Abyss)
                        }
                        _ => Err(self.error(&format!("unknown builtin '{}'", name), span)),
                    },
                    Value::StructDef(struct_decl) => {
                        let args: Vec<Value> = call_args
                            .iter()
                            .map(|a| self.eval_expr(a))
                            .collect::<Result<Vec<_>, _>>()?;
                        let fields: HashMap<String, Value> = struct_decl
                            .variables
                            .iter()
                            .zip(args)
                            .map(|(p, v)| (p.name.clone(), v))
                            .collect();

                        let index = self.allocate_struct(fields);
                        Ok(Value::Struct(struct_decl.name.clone(), index))
                    }
                    _ => Err(self.error("cannot call a non-function value", span)),
                }
            }
            Expr::FieldAccess(expr, field_name) => {
                let obj = self.eval_expr(expr)?;
                match obj {
                    Value::Struct(_, idx) => self.heap[idx]
                        .get(field_name)
                        .cloned()
                        .ok_or_else(|| self.error(&format!("no field '{}'", field_name), span)),
                    _ => Err(self.error("field access on non-struct value", span)),
                }
            }
            Expr::MethodCall(receiver, method_name, call_args) => {
                let obj = self.eval_expr(receiver)?;
                match obj.clone() {
                    Value::Struct(type_name, index) => {
                        let struct_def = match self.lookup(&type_name) {
                            Some(Value::StructDef(def)) => def,
                            _ => {
                                return Err(
                                    self.error(&format!("unknown struct '{}'", type_name), span)
                                )
                            }
                        };
                        let method = struct_def
                            .methods
                            .iter()
                            .find(|m| &m.name == method_name)
                            .ok_or_else(|| {
                                self.error(
                                    &format!("no method '{}' on '{}'", method_name, type_name),
                                    span,
                                )
                            })?
                            .clone();
                        self.scopes.push(HashMap::new());
                        self.define("self".to_string(), obj);

                        let args: Vec<Value> = call_args
                            .iter()
                            .map(|a| self.eval_expr(a))
                            .collect::<Result<Vec<_>, _>>()?;

                        for (param, val) in method.params.iter().skip(1).zip(args) {
                            self.define(param.name.clone(), val);
                        }
                        let result = self.eval_block(&method.body);
                        self.scopes.pop();
                        result
                    }
                    _ => Err(self.error("method call on non-struct value", span)),
                }
            }
            _ => Err(self.error("expression type not yet implemented", span)),
        }
    }

    fn register_builtins(&mut self) {
        self.define("print".to_string(), Value::Builtin("print".to_string()));
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

    pub fn interpret(&mut self, decls: Vec<Spanned<Decl>>) -> Result<Value, InterpreterError> {
        for decl in decls {
            match decl.node {
                Decl::Func(func) => {
                    let value = Value::Function(
                        func.params.iter().map(|p| p.name.clone()).collect(),
                        func.body,
                    );
                    self.define(func.name, value);
                }
                Decl::Binding(binding) => {
                    let value = self.eval_expr(&binding.value)?;
                    self.define(binding.name, value);
                }
                Decl::Struct(struct_decl) => {
                    self.define(struct_decl.name.clone(), Value::StructDef(struct_decl));
                }
            }
        }
        let main = self.lookup("main").ok_or_else(|| InterpreterError {
            message: "no main function found".to_string(),
            line: 0,
            column: 0,
        })?;
        match main {
            Value::Function(_, body) => self.eval_block(&body),
            _ => Err(InterpreterError {
                message: "main is not a function".to_string(),
                line: 0,
                column: 0,
            }),
        }
    }
}
