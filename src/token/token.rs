use anyhow::{anyhow, Context, Result};
use nom::{Offset, branch::alt, character::complete::char, character::complete::{digit1, multispace0}, combinator::{eof, map, map_res, opt, verify}, delimited, error::{ErrorKind, VerboseError}, sequence::{delimited, tuple}};
use nom::{error::Error};
use std::{hint::unreachable_unchecked, iter::Peekable};
use std::rc::Rc;
use std::cell::RefCell;

type IResult<I, O> = nom::IResult<I, O, VerboseError<I>>;


// <expr>       = <mul> [('+' | '-') <expr>]
// <mul>        = <primary> [('*' | '/') <mul>]
// <primary>    = <num> | <paren_expr>
// <paren_expr> = '(' <expr> ')'
// <num>        = [0-9]+

pub type Num = i64;

#[derive(Debug, PartialEq)]
pub enum Expr {
    Num(Num),
    BinaryOperation(BinaryOperation),
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
    Sub,
    Mul,
    Div
}

impl Expr {
    pub fn parse(s: &str) -> Result<Expr, nom::Err<VerboseError<&str>>> {
        tuple((
            expr_parser,
            eof
        ))(s)
            .map(|(_, (expr, _))| expr)
    }

    pub fn compile(&self) {
        println!(".intel_syntax noprefix");
        println!(".global main");
        println!("main:");

        self.generate();

        println!("    pop rax");
        println!("    ret");
    }

    fn generate(&self) {
        match self {
            Expr::Num(num) => {
                println!("    push {}", num);
            },
            Expr::BinaryOperation(bin_op) => {
                bin_op.left.generate();
                bin_op.right.generate();

                println!("    pop rdi");
                println!("    pop rax");

                match bin_op.op {
                    OperatorKind::Add => println!("    add rax, rdi"),
                    OperatorKind::Sub => println!("    sub rax, rdi"),
                    OperatorKind::Mul => println!("    imul rax, rdi"),
                    OperatorKind::Div => {
                        println!("    cqo");
                        println!("    idiv rax, rdi");
                    },
                }

                println!("    push rax");
            }
        }
    }
}

fn offset_op_parser(s: &str) -> IResult<&str, OperatorKind> {
    alt((
        map(
            char('+'),
            |_| OperatorKind::Add,
        ),
        map(
            char('-'),
            |_| OperatorKind::Sub,
        )
    ))(s)
}

fn scale_op_parser(s: &str) -> IResult<&str, OperatorKind> {
    alt((
        map(
            char('*'),
            |_| OperatorKind::Mul,
        ),
        map(
            char('/'),
            |_| OperatorKind::Div,
        )
    ))(s)
}

pub fn expr_parser(s: &str) -> IResult<&str, Expr> {
    map(
        tuple((
            ws(mul_parser),
            opt(tuple((
                ws(offset_op_parser),
                ws(expr_parser)
            )))
        )),
        |(left, opt)| {
            if let Some((op, right)) = opt {
                Expr::BinaryOperation(BinaryOperation {
                    op,
                    left: Box::new(left),
                    right: Box::new(right)
                })
            } else {
                left
            }
        }
    )(s)
}

fn mul_parser(s: &str) -> IResult<&str, Expr> {
    map(
        tuple((
            ws(primary_parser),
            opt(tuple((
                ws(scale_op_parser),
                ws(primary_parser)
            )))
        )),
        |(left, opt)| {
            if let Some((op, right)) = opt {
                Expr::BinaryOperation(BinaryOperation {
                    op,
                    left: Box::new(left),
                    right: Box::new(right)
                })
            } else {
                left
            }
        }
    )(s)
}

fn primary_parser(s: &str) -> IResult<&str, Expr> {

    alt((
        map(
            ws(num_parser),
            |num| Expr::Num(num)
        ),
        ws(paren_expr_parser)
    ))(s)
}

fn paren_expr_parser(s: &str) -> IResult<&str, Expr> {
    let (no_used, _) = char('(')(s)?;
    let (no_used, expr) = expr_parser(no_used)?;
    let (no_used, _) = char(')')(no_used)?;

    Ok((no_used, expr))
}

fn num_parser(s: &str) -> IResult<&str, Num> {
    map_res(
        digit1,
        |s: &str| s.parse::<Num>()
    )(s)
}

fn ws<'a, F: 'a, O>(inner: F) -> impl FnMut(&'a str) -> IResult<&'a str, O>
    where
    F: Fn(&'a str) -> IResult<&'a str, O>,
{
    delimited(
        multispace0,
        inner,
        multispace0
    )
}

#[cfg(test)]
mod tests {
    use std::fmt::Binary;

    use super::*;

    #[test]
    fn offset_op_parser_test() {
        assert_eq!(offset_op_parser("+abc").unwrap(), ("abc", OperatorKind::Add));
        assert_eq!(offset_op_parser("-abc").unwrap(), ("abc", OperatorKind::Sub));
        assert!(offset_op_parser("*abc").is_err());
        assert!(offset_op_parser("/abc").is_err());
    }

    #[test]
    fn scale_op_parser_test() {
        assert!(scale_op_parser("+abc").is_err());
        assert!(scale_op_parser("-abc").is_err());
        assert_eq!(scale_op_parser("*abc").unwrap(), ("abc", OperatorKind::Mul));
        assert_eq!(scale_op_parser("/abc").unwrap(), ("abc", OperatorKind::Div));
    }

    #[test]
    fn expr_parser_test() {
        assert_eq!(expr_parser("334abc").unwrap(), ("abc", Expr::Num(334)));
        assert_eq!(expr_parser("334+264*227").unwrap(), ("",
            Expr::BinaryOperation(BinaryOperation {
                op: OperatorKind::Add,
                left: Box::new(Expr::Num(334)),
                right: Box::new(Expr::BinaryOperation(BinaryOperation {
                    op: OperatorKind::Mul,
                    left: Box::new(Expr::Num(264)),
                    right: Box::new(Expr::Num(227))
                }))
            })));
    }

    #[test]
    fn primary_parser_test() {
        assert_eq!(primary_parser("334abc").unwrap(), ("abc", Expr::Num(334)));
        assert_eq!(primary_parser("(264)abc").unwrap(), ("abc", Expr::Num(264)));
        assert!(primary_parser("(227").is_err());
        assert!(primary_parser("(def)").is_err());
    }

    #[test]
    fn mul_parser_test() {
        assert_eq!(mul_parser("334abc").unwrap(), ("abc", Expr::Num(334)));
        assert_eq!(mul_parser("(334)abc").unwrap(), ("abc", Expr::Num(334)));
        assert_eq!(mul_parser("334*264").unwrap(), ("", Expr::BinaryOperation(BinaryOperation {
            op: OperatorKind::Mul,
            left: Box::new(Expr::Num(334)),
            right: Box::new(Expr::Num(264))
        })));
    }

    #[test]
    fn paren_expr_parser_test() {
        assert_eq!(paren_expr_parser("(334)abc").unwrap(), ("abc", Expr::Num(334)));
        assert!(paren_expr_parser("(abc)").is_err());
    }

    #[test]
    fn num_parser_test() {
        assert_eq!(num_parser("334abc").unwrap(), ("abc", 334));
        assert!(num_parser("abc").is_err());
    }
}