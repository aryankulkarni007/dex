#[derive(Debug)]
pub enum Type {
    Int,
    Flt,
    Str,
    Bool,
    Abyss,
    List(Box<Type>),
    Map(Box<Type>, Box<Type>),
    Result(Box<Type>),
    Named(String),
}

#[derive(Debug)]
pub struct Param {
    pub name: String,
    pub type_ann: Type,
}

#[derive(Debug)]
pub struct FuncDecl {
    pub name: String,
    pub params: Vec<Param>,
    pub return_type: Type,
    pub body: Vec<Expr>,
}

#[derive(Debug)]
pub struct StructDecl {
    pub name: String,
    pub variables: Vec<Param>,
    pub methods: Vec<FuncDecl>,
}

#[derive(Debug)]
pub struct BindingDecl {
    pub mutable: bool,
    pub name: String,
    pub type_ann: Option<Type>,
    pub value: Expr,
}

#[derive(Debug)]
pub enum Decl {
    Func(FuncDecl),
    Struct(StructDecl),
    Binding(BindingDecl),
}

#[derive(Debug)]
pub enum BinaryOp {
    Add,
    Minus,
    Multiply,
    Divide,
    Modulo,
    Exponent,
    Equality,
    Nequality,
    Lesser,
    Greater,
    Leq,
    Geq,
    And,
    Or,
}

#[derive(Debug)]
pub enum UnaryOp {
    Not,
    Neg, // unary minus
         // Print,      // !
         // DebugPrint, // !!
}

#[derive(Debug)]
pub enum Expr {
    Int(i64),
    Flt(f64),
    Str(String),
    Bool(bool),
    Abyss,
    Identifier(String),
    Binary(Box<Expr>, BinaryOp, Box<Expr>),
    Unary(UnaryOp, Box<Expr>),
    Call(Box<Expr>, Vec<Expr>),
    MethodCall(Box<Expr>, String, Vec<Expr>),
    Pipeline(Box<Expr>, Box<Expr>),
    Assign(String, Box<Expr>),
    Lambda(String, Box<Expr>),
    Try(Box<Expr>),
    ListLiteral(Vec<Expr>),
    MapLiteral(Vec<(Expr, Expr)>),
    If(Box<Expr>, Vec<Expr>, Option<Vec<Expr>>),
    Loop(Vec<String>, Box<Expr>, Vec<Expr>),
}
