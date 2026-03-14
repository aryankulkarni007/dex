/// Source location — line and column of a token in the source file.
/// Used to attach position info to AST nodes for error reporting.
#[derive(Debug, Clone)]
pub struct Span {
    /// 1-indexed line number
    pub line: usize,
    /// 1-indexed column number
    pub column: usize,
}

/// A node of type T with an attached source location.
/// Every significant AST node is wrapped in this.
#[derive(Debug, Clone)]
pub struct Spanned<T> {
    /// The actual AST node
    pub node: T,
    /// Where in the source this node came from
    pub span: Span,
}

/// A Spanned expression — the primary unit of evaluation
pub type SpannedExpr = Spanned<Expr>;
/// A Spanned statement — used inside function bodies and blocks
pub type SpannedStmt = Spanned<Stmt>;
/// A Spanned declaration — top-level items like functions, structs, bindings
pub type SpannedDecl = Spanned<Decl>;

/// A type annotation in the Dex type system.
/// Used in variable bindings, function params, and return types.
#[derive(Debug, Clone)]
pub enum Type {
    /// `int` — 64-bit signed integer
    Int,
    /// `flt` — 64-bit float
    Flt,
    /// `str` — UTF-8 string
    Str,
    /// `bool` — true or false
    Bool,
    /// `abyss` — the void/unit type, used for functions that return nothing
    Abyss,
    /// No annotation given — type will be inferred by the type checker
    Inferred,
    /// `[T]` — a list of elements of type T
    List(Box<Type>),
    /// `{K: V}` — a map from keys of type K to values of type V
    Map(Box<Type>, Box<Type>),
    /// `T | error` — a result type that can be either T or an error
    Result(Box<Type>),
    /// A named type like `Point` — resolved during type checking
    Named(String),
}

/// A function parameter or struct field.
/// `name: type_ann` e.g. `x: int`
#[derive(Debug, Clone)]
pub struct Param {
    /// The parameter name e.g. `x`
    pub name: String,
    /// The type annotation e.g. `Type::Int`. May be `Type::Inferred` if omitted.
    pub type_ann: Type,
}

/// A function declaration.
/// Syntax: `@(params) name -> return_type { body }`
#[derive(Debug, Clone)]
pub struct FuncDecl {
    /// The function name e.g. `add`
    pub name: String,
    /// The parameter list e.g. `[Param { name: "x", type_ann: Int }]`
    pub params: Vec<Param>,
    /// The declared return type e.g. `Type::Int`
    pub return_type: Type,
    /// The function body — a list of statements
    pub body: Vec<SpannedStmt>,
}

/// A struct declaration.
/// Syntax: `struct Name { fields... methods... }`
#[derive(Debug, Clone)]
pub struct StructDecl {
    /// The struct name e.g. `Point`
    pub name: String,
    /// The struct fields e.g. `x: flt, y: flt`
    pub variables: Vec<Param>,
    /// The struct methods — full function declarations
    pub methods: Vec<FuncDecl>,
}

/// A variable binding.
/// Syntax: `name: type = value` or `mut name = value`
#[derive(Debug, Clone)]
pub struct BindingDecl {
    /// Whether the binding is mutable (`mut`)
    pub mutable: bool,
    /// The variable name e.g. `x`
    pub name: String,
    /// Optional type annotation e.g. `Some(Type::Int)` for `x: int = 5`
    pub type_ann: Option<Type>,
    /// The value expression e.g. `Expr::Int(5)`
    pub value: SpannedExpr,
}

/// A top-level declaration in a Dex program.
/// These are the items that can appear at the outermost scope.
#[derive(Debug, Clone)]
pub enum Decl {
    /// A function declaration e.g. `@(x: int) add -> int { ... }`
    Func(FuncDecl),
    /// A struct declaration e.g. `struct Point { x: flt, y: flt }`
    Struct(StructDecl),
    /// A top-level variable binding e.g. `x = 10`
    Binding(BindingDecl),
}

/// A statement inside a function body or block.
#[derive(Debug, Clone)]
pub enum Stmt {
    /// A variable binding e.g. `x = 10` or `y: int = 5`
    Binding(BindingDecl),
    /// A standalone expression e.g. `print(x)` or `point1:move()`
    Expr(SpannedExpr),
}

/// A binary operator between two expressions.
#[derive(Debug, Clone)]
pub enum BinaryOp {
    /// `+`
    Add,
    /// `-`
    Minus,
    /// `*`
    Multiply,
    /// `/`
    Divide,
    /// `%`
    Modulo,
    /// `^`
    Exponent,
    /// `==`
    Equality,
    /// `!=`
    Nequality,
    /// `<`
    Lesser,
    /// `>`
    Greater,
    /// `<=`
    Leq,
    /// `>=`
    Geq,
    /// `&&`
    And,
    /// `||`
    Or,
}

/// A unary (single-operand) operator.
#[derive(Debug, Clone)]
pub enum UnaryOp {
    /// `!` — logical negation, only valid on `bool`
    Not,
    /// `-` — numeric negation, valid on `int` and `flt`
    Neg,
}

/// An expression — something that can be evaluated to produce a value.
#[derive(Debug, Clone)]
pub enum Expr {
    /// Integer literal e.g. `42` → `i64`
    Int(i64),
    /// Float literal e.g. `3.14` → `f64`
    Flt(f64),
    /// String literal e.g. `"hello"` → `String`
    Str(String),
    /// Boolean literal `true` or `false`
    Bool(bool),
    /// A variable reference e.g. `x` — looked up in the current scope
    Identifier(String),
    /// A binary operation e.g. `x + y`
    /// Fields: (left_operand, operator, right_operand)
    Binary(Box<SpannedExpr>, BinaryOp, Box<SpannedExpr>),
    /// A unary operation e.g. `!x` or `-x`
    /// Fields: (operator, operand)
    Unary(UnaryOp, Box<SpannedExpr>),
    /// A function call e.g. `foo(1, 2)`
    /// Fields: (callee_expr, arg_list)
    Call(Box<SpannedExpr>, Vec<SpannedExpr>),
    /// A method call e.g. `point:distance(other)`
    /// Fields: (receiver_expr, method_name, arg_list)
    MethodCall(Box<SpannedExpr>, String, Vec<SpannedExpr>),
    /// A pipeline operation e.g. `list >> filter(f)`
    /// Fields: (left_expr, right_expr)
    Pipeline(Box<SpannedExpr>, Box<SpannedExpr>),
    /// An assignment to an existing variable or field e.g. `self.x = 5`
    /// Fields: (target_expr, value_expr)
    Assign(Box<SpannedExpr>, Box<SpannedExpr>),
    /// A lambda expression e.g. `n -> n * 2`
    /// Fields: (param_name, body_expr)
    Lambda(String, Box<SpannedExpr>),
    /// The `?` postfix error propagation operator e.g. `foo()?`
    /// Fields: (inner_expr)
    Try(Box<SpannedExpr>),
    /// A list literal e.g. `[1, 2, 3]`
    /// Fields: (element_list)
    ListLiteral(Vec<SpannedExpr>),
    /// A map literal e.g. `{"a": 1, "b": 2}`
    /// Fields: (key_value_pairs)
    MapLiteral(Vec<(SpannedExpr, SpannedExpr)>),
    /// An if/else expression e.g. `if x > 0 { ... } else { ... }`
    /// Fields: (condition, then_body, optional_else_body)
    If(Box<SpannedExpr>, Vec<SpannedStmt>, Option<Vec<SpannedStmt>>),
    /// A for loop e.g. `I (n in list) -> { ... }`
    /// Fields: (loop_variables, iterable_expr, body)
    Loop(Vec<String>, Box<SpannedExpr>, Vec<SpannedStmt>),
    /// Field access on a struct e.g. `point.x`
    /// Fields: (object_expr, field_name)
    FieldAccess(Box<SpannedExpr>, String),
    /// list expression, index expression
    Index(Box<SpannedExpr>, Box<SpannedExpr>),
}
