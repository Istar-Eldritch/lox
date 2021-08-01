#[derive(Debug, PartialEq)]
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
    Ternary {
        condition: Box<Expr>,
        left: Box<Expr>,
        right: Box<Expr>,
    },
}

#[derive(Debug, PartialEq)]
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
    Comma,
}

#[derive(Debug, PartialEq)]
pub enum Literal {
    Str(String),
    Number(f64),
    True,
    False,
    Nil,
}

#[derive(Debug, PartialEq)]
pub enum UnaryOp {
    Negate,
    LogicNegate,
}
