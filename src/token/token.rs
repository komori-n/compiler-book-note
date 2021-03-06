pub type Num = i64;

#[derive(Debug)]
pub struct Program {
    pub stmts: Vec<Expr>
}

#[derive(Debug, PartialEq)]
pub enum Expr {
    Num(Num),
    Ident(String),
    BinaryOperation(BinaryOperation),
    Return(Box<Expr>),
    If(Box<Expr>, Box<Expr>, Option<Box<Expr>>),
    While(Box<Expr>, Box<Expr>),
    For(Option<Box<Expr>>, Option<Box<Expr>>, Option<Box<Expr>>, Box<Expr>),
    Block(Vec<Expr>)
}

#[derive(Debug, PartialEq)]
pub struct BinaryOperation {
    pub op: OperatorKind,
    pub left: Box<Expr>,
    pub right: Box<Expr>
}

#[derive(Debug, PartialEq)]
pub enum OperatorKind {
    Add,
    Substruct,
    Multiply,
    Divide,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,
    Equal,
    NotEqual,
    Assign,
}
