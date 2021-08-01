use crate::ast::{BinOp, Expr, Expr::*, Literal};

pub trait Interpretable {
    fn eval(&self) -> Result<LoxResult, String>;
}

#[derive(PartialEq, PartialOrd, Debug)]
pub enum LoxResult {
    Number(f64),
    Str(String),
    Bool(bool),
    Nil,
}

impl LoxResult {
    fn get_type(&self) -> LoxType {
        match self {
            Self::Number(_) => LoxType::Number,
            Self::Str(_) => LoxType::Str,
            Self::Bool(_) => LoxType::Bool,
            Self::Nil => LoxType::Nil,
        }
    }

    fn unwrap_number(self) -> f64 {
        match self {
            Self::Number(n) => n,
            _ => panic!("LoxResult is not a number"),
        }
    }

    fn unwrap_string(self) -> String {
        match self {
            Self::Str(n) => n.clone(),
            _ => panic!("LoxResult is not a number"),
        }
    }
}

#[derive(PartialEq, Debug)]
enum LoxType {
    Number,
    Str,
    Bool,
    Nil,
}

impl Interpretable for Expr {
    fn eval(&self) -> std::result::Result<LoxResult, String> {
        let res = match self {
            Literal(l) => match l {
                Literal::Number(n) => LoxResult::Number(*n),
                Literal::Str(n) => LoxResult::Str(n.clone()),
                Literal::True => LoxResult::Bool(true),
                Literal::False => LoxResult::Bool(false),
                Literal::Nil => LoxResult::Nil,
            },
            Self::Unary { operator, right } => {
                let right = right.eval()?;

                match operator {
                    crate::ast::UnaryOp::LogicNegate => match right {
                        LoxResult::Bool(b) => LoxResult::Bool(!b),
                        _ => Err(format!("Cant negate type {:?}", right.get_type()))?,
                    },
                    crate::ast::UnaryOp::Negate => match right {
                        LoxResult::Number(n) => LoxResult::Number(-n),
                        _ => Err(format!("Cant negate type {:?}", right.get_type()))?,
                    },
                }
            }
            Self::Grouping(e) => e.eval()?,
            Self::Ternary {
                condition,
                left,
                right,
            } => {
                let condition = condition.eval()?;
                let condition = match condition {
                    LoxResult::Bool(b) => b,
                    r => Err(format!(
                        "The condition of a ternary operator must resolve to a boolean but was {:?}",
                        r.get_type()
                    ))?,
                };
                if condition {
                    left.eval()?
                } else {
                    right.eval()?
                }
            }
            Self::Binary {
                left,
                right,
                operator,
            } => {
                let l = left.eval()?;
                let r = right.eval()?;
                if l.get_type() != r.get_type() {
                    Err(format!(
                        "Cant operate on {:?} and {:?}",
                        l.get_type(),
                        r.get_type(),
                    ))?;
                }
                let res = match operator {
                    // TODO: This should handle concat of strings too
                    BinOp::Sum => match l.get_type() {
                        LoxType::Number => LoxResult::Number(l.unwrap_number() + r.unwrap_number()),
                        LoxType::Str => LoxResult::Str(l.unwrap_string() + &r.unwrap_string()),
                        n => Err(format!("Can't perform Sum on {:?}", n))?,
                    },
                    BinOp::Substraction => LoxResult::Number(l.unwrap_number() - r.unwrap_number()),
                    BinOp::Product => LoxResult::Number(l.unwrap_number() * r.unwrap_number()),
                    BinOp::Division => LoxResult::Number(l.unwrap_number() / r.unwrap_number()),
                    BinOp::Equals => LoxResult::Bool(l == r),
                    BinOp::GreaterThan => LoxResult::Bool(l > r),
                    BinOp::GreaterThanEquals => LoxResult::Bool(l >= r),
                    BinOp::LessThan => LoxResult::Bool(l < r),
                    BinOp::LessThanEquals => LoxResult::Bool(l <= r),
                    BinOp::NotEquals => LoxResult::Bool(l != r),
                    BinOp::Comma => r,
                };

                res
            }
        };
        Ok(res)
    }
}
