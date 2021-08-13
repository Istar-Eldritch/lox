#[derive(Debug, PartialEq)]
pub enum Stmt {
    Expression(Expr),
    Print(Expr),
}

#[derive(Debug, PartialEq)]
pub enum Expr {
    Binary {
        left: Box<Expr>,
        operator: BinOp,
        right: Box<Expr>,
        index: usize,
        len: usize,
    },
    Grouping {
        expr: Box<Expr>,
        index: usize,
        len: usize,
    },
    Literal {
        value: Literal,
        index: usize,
        len: usize,
    },
    Unary {
        operator: UnaryOp,
        right: Box<Expr>,
        index: usize,
        len: usize,
    },
    Ternary {
        condition: Box<Expr>,
        left: Box<Expr>,
        right: Box<Expr>,
        index: usize,
        len: usize,
    },
}

impl Expr {
    pub fn index(&self) -> usize {
        match self {
            Self::Literal {
                value: _,
                len: _,
                index,
            } => *index,
            Self::Grouping {
                expr: _,
                len: _,
                index,
            } => *index,
            Self::Unary {
                operator: _,
                right: _,
                len: _,
                index,
            } => *index,
            Self::Binary {
                operator: _,
                left: _,
                right: _,
                len: _,
                index,
            } => *index,
            Self::Ternary {
                condition: _,
                left: _,
                right: _,
                len: _,
                index,
            } => *index,
        }
    }

    pub fn len(&self) -> usize {
        match self {
            Self::Literal {
                value: _,
                len,
                index: _,
            } => *len,
            Self::Grouping {
                expr: _,
                len,
                index: _,
            } => *len,
            Self::Unary {
                operator: _,
                right: _,
                len,
                index: _,
            } => *len,
            Self::Binary {
                operator: _,
                left: _,
                right: _,
                len,
                index: _,
            } => *len,
            Self::Ternary {
                condition: _,
                left: _,
                right: _,
                len,
                index: _,
            } => *len,
        }
    }
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
