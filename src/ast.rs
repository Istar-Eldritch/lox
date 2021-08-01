#[derive(Debug)]
pub enum Expr {
    Binary {
        left: Box<Expr>,
        operator: BinOp,
        right: Box<Expr>,
    },
    Grouping(Box<Expr>),
    Literal(Literal),
    Unary {
        operator: UnaryOp,
        right: Box<Expr>,
    },
}

#[derive(Debug)]
pub enum BinOp {
    Sum,
    Substraction,
    Product,
    Division,
    Equals,
    NotEquals,
    GreaterThan,
    GreaterThanEquals,
    LessThan,
    LessThanEquals,
}

#[derive(Debug)]
pub enum Literal {
    Str(String),
    Number(f64),
    True,
    False,
    Nil,
}

#[derive(Debug)]
pub enum UnaryOp {
    Negate,
    LogicNegate,
}