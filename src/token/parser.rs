use crate::token::{Expr, OperatorKind, Program, BinaryOperation, Num};
use nom::{branch::alt, bytes::complete::tag, character::complete::char, character::complete::{digit1, multispace0, one_of}, combinator::{map, map_res, opt}, error::{VerboseError}, multi::many0, sequence::{delimited, tuple}};

type IResult<I, O> = nom::IResult<I, O, VerboseError<I>>;


// <program>    = <stmt>*
// <stmt>       = <expr> ';'
// <expr>       = <assign>
// <assign>     = <equality> ("=" assign)?
// <equality>   = <relational> ("==" | "!=" relational)*
// <relational> = <add> (("<" | "<=" | ">" | ">=") <add>)*
// <add>        = <mul> [('+' | '-') <mul>]*
// <mul>        = <unary> [('*' | '/') <unary>]*
// <unary>      = ('+' | '-')? primary
// <primary>    = <num> | <paren_expr>
// <paren_expr> = '(' <expr> ')'
// <num>        = [0-9]+

impl Program {
    pub fn parse(s: &str) -> Result<Program, nom::Err<VerboseError<&str>>> {
        many0(stmt_parser)(s)
            .map(|(_, stmts)| Program { stmts })
    }
}

macro_rules! operator_parser {
    ($x:expr => $y:expr) => {
        map(
            tag($x),
            |_| $y
        )
    };
    ($($x:expr => $y:expr),*) => {
        alt((
            $(map(
                tag($x),
                |_| $y
            )),*
        ))
    }
}

pub fn binary_parser<'a, F: 'a, G: 'a, H: 'a>(left_child_parser: F, op_parser: G, right_child_parser: H)
    -> impl FnMut(&'a str) -> IResult<&'a str, Expr>
where
    F: FnMut(&'a str) -> IResult<&'a str, Expr>,
    G: FnMut(&'a str) -> IResult<&'a str, OperatorKind>,
    H: FnMut(&'a str) -> IResult<&'a str, Expr>,
{
    map(
        tuple((
            ws(left_child_parser),
            opt(tuple((
                ws(op_parser),
                ws(right_child_parser)
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
    )
}

fn stmt_parser(s: &str) -> IResult<&str, Expr> {
    map(
        tuple((
            ws(expr_parser),
            ws(char(';'))
        )),
        |(stmt, _)| stmt
    )(s)
}

fn expr_parser(s: &str) -> IResult<&str, Expr> {
    assign_parser(s)
}

fn assign_parser(s: &str) -> IResult<&str, Expr> {
    binary_parser(
        equality_parser,
        operator_parser!(
            "=" => OperatorKind::Assign
        ),
        assign_parser
    )(s)
}

fn equality_parser(s: &str) -> IResult<&str, Expr> {
    binary_parser(
        relational_parser,
        operator_parser!(
            "==" => OperatorKind::Equal,
            "!=" => OperatorKind::NotEqual
        ),
        equality_parser
    )(s)
}

fn relational_parser(s: &str) -> IResult<&str, Expr> {
    binary_parser(
        add_parser,
        operator_parser!(
            "<=" => OperatorKind::LessEqual,
            "<" => OperatorKind::Less,
            ">=" => OperatorKind::GreaterEqual,
            ">" => OperatorKind::Greater
        ),
        relational_parser
    )(s)
}

fn add_parser(s: &str) -> IResult<&str, Expr> {
    binary_parser(
        mul_parser,
        operator_parser!(
            "+" => OperatorKind::Add,
            "-" => OperatorKind::Substruct
        ),
        add_parser
    )(s)
}

fn mul_parser(s: &str) -> IResult<&str, Expr> {
    binary_parser(
        unary_parser,
        operator_parser!(
            "*" => OperatorKind::Multiply,
            "/" => OperatorKind::Divide
        ),
        mul_parser
    )(s)
}

fn unary_parser(s: &str) -> IResult<&str, Expr> {
    map(
        tuple((
            opt(ws(operator_parser!(
                "+" => OperatorKind::Add,
                "-" => OperatorKind::Substruct
            ))),
            primary_parser
        )),
        |(opt, primary)| {
            if opt == Some(OperatorKind::Substruct) {
                Expr::BinaryOperation(BinaryOperation {
                    op: OperatorKind::Substruct,
                    left: Box::new(Expr::Num(0)),
                    right: Box::new(primary)
                })
            } else {
                primary
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
        map(
            ws(ident_parser),
            |ident| Expr::Ident(ident)
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

fn ident_parser(s: &str) -> IResult<&str, i64> {
    one_of("qwertyuiopasdfghjklzxcvbnm")(s)
        .map(|(no_used, ident)| {
            (no_used, 8 * (ident.to_string().as_bytes()[0] - b'a') as i64)
        })
}

fn ws<'a, F: 'a, O>(inner: F) -> impl FnMut(&'a str) -> IResult<&'a str, O>
    where
    F: FnMut(&'a str) -> IResult<&'a str, O>,
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
    fn op_parser_test() {
        let mut offset_parser = operator_parser!(
            "+" => OperatorKind::Add,
            "-" => OperatorKind::Sub
        );

        let _: IResult<&str, OperatorKind> = offset_parser("+abc");

        assert_eq!(offset_parser("+abc").unwrap(), ("abc", OperatorKind::Add));
        assert_eq!(offset_parser("-abc").unwrap(), ("abc", OperatorKind::Sub));
        assert!(offset_parser("/abc").is_err());
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
    fn unary_parser_test() {
        assert_eq!(unary_parser("+334").unwrap(), ("", Expr::Num(334)));
        assert!(unary_parser("--334").is_err());
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
            op: OperatorKind::Multiply,
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
