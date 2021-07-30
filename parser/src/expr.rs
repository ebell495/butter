use crate::expr::array::array;
use crate::expr::array::range;
use crate::expr::infix::expr_0;
use crate::expr::infix::expr_6;
use crate::expr::infix::infix_expr_op;
use crate::expr::integer::integer_u64;
use crate::expr::record::record;
use crate::expr::string::char_literal;
use crate::expr::string::string_literal;
use crate::ident_keyword::ident;
use crate::ident_keyword::keyword;
use crate::lex;
use crate::pattern::parameter;
use combine::attempt;
use combine::between;
use combine::chainl1;
use combine::choice;
use combine::optional;
use combine::parser;
use combine::parser::char::char;
use combine::parser::char::string;
use combine::ParseError;
use combine::Parser;
use combine::RangeStream;
use hir::expr::operator::Tag;
use hir::expr::Expr;
use hir::expr::Fun;
use hir::expr::Literal;

mod array;
pub mod control_flow;
mod float;
mod infix;
pub mod integer;
mod record;
mod string;

// TODO: separate literal parser
fn prefix_expr_<'a, I, T>() -> impl Parser<I, Output = Expr<'a, T>>
where
    I: RangeStream<Token = char, Range = &'a str>,
    I::Error: ParseError<I::Token, I::Range, I::Position>,
    T: Default,
{
    choice((
        (attempt(parameter().skip(lex(string("=>")))), expr(0)).map(|(param, body)| {
            Expr::Fun(Fun {
                param,
                body: Box::new(body),
            })
        }),
        attempt(range()).map(Expr::ArrayRange),
        array().map(Expr::Array),
        attempt(between(lex(char('(')), lex(char(')')), expr(0))),
        record().map(Expr::Record),
        lex(char_literal()).map(|uint| Expr::Literal(Literal::UInt(uint))),
        lex(string_literal()).map(Expr::Array),
        lex(char('!'))
            .with(expr(6))
            .map(|expr| Expr::Not(Box::new(expr))),
        lex(char('&'))
            .with((optional(attempt(lex(keyword("mut")))), expr(6)))
            .map(|(mutability, expr)| {
                let boxed = Box::new(expr);
                match mutability {
                    Some(_) => Expr::MutRef(boxed),
                    None => Expr::Ref(boxed),
                }
            }),
        lex(char('-'))
            .with(expr(6))
            .map(|expr| Expr::Minus(Box::new(expr))),
        lex(char('@'))
            .with((lex(ident()), optional(expr(6))))
            .map(|(tag, expr)| {
                Expr::Tag(Tag {
                    tag,
                    expr: expr.map(Box::new),
                })
            }),
        attempt(lex(ident())).map(Expr::Var),
        control_flow::control_flow().map(Expr::ControlFlow),
        lex(keyword("false")).map(|_| Expr::Literal(Literal::False)),
        lex(keyword("true")).map(|_| Expr::Literal(Literal::True)),
        lex(keyword("void")).map(|_| Expr::Literal(Literal::Void)),
        lex(keyword("break"))
            .with(optional(expr(0)))
            .map(|expr| Expr::Break(expr.map(Box::new))),
        lex(keyword("continue")).map(|_| Expr::Continue),
        lex(keyword("return"))
            .with(optional(expr(0)))
            .map(|expr| Expr::Return(expr.map(Box::new))),
        lex(float::float()).map(|float| Expr::Literal(Literal::Float(float))),
        lex(integer_u64()).map(|uint| Expr::Literal(Literal::UInt(uint))),
    ))
}
parser! {
    fn prefix_expr['a, I, T]()(I) -> Expr<'a, T>
    where [
        I: RangeStream<Token = char, Range = &'a str>,
        I::Error: ParseError<I::Token, I::Range, I::Position>,
        T: Default,
    ] {
        prefix_expr_()
    }
}
fn expr_<'a, I, T>(precedence: u8) -> impl Parser<I, Output = Expr<'a, T>>
where
    I: RangeStream<Token = char, Range = &'a str>,
    I::Error: ParseError<I::Token, I::Range, I::Position>,
    T: Default,
{
    match precedence {
        0 => expr_0().left().left(),
        1..=5 => chainl1(expr(precedence + 1), attempt(infix_expr_op(precedence)))
            .right()
            .left(),
        6 => expr_6().left().right(),
        _ => prefix_expr().right().right(),
    }
}
parser! {
    pub fn expr['a, I, T](precedence: u8)(I) -> Expr<'a, T>
    where [
        I: RangeStream<Token = char, Range = &'a str>,
        I::Error: ParseError<I::Token, I::Range, I::Position>,
        T: Default,
    ] {
        expr_(*precedence)
    }
}
#[cfg(test)]
mod test {
    use crate::expr::expr;
    use crate::expr::Expr;
    use combine::EasyParser;
    use hir::expr::operator::Assign;
    use hir::expr::operator::Binary;
    use hir::expr::PlaceExpr;

    #[test]
    fn group() {
        let src = "(foo)";
        let expected = <Expr<()>>::Var("foo");
        assert_eq!(expr(0).easy_parse(src), Ok((expected, "")));
    }
    #[test]
    fn precedence() {
        let src = "foo + bar * baz";
        let expected = <Expr<()>>::Add(Binary {
            left: Box::new(Expr::Var("foo")),
            right: Box::new(Expr::Multiply(Binary {
                left: Box::new(Expr::Var("bar")),
                right: Box::new(Expr::Var("baz")),
            })),
        });
        assert_eq!(expr(0).easy_parse(src), Ok((expected, "")));
        let src = "foo * bar + baz";
        let expected = <Expr<()>>::Add(Binary {
            left: Box::new(Expr::Multiply(Binary {
                left: Box::new(Expr::Var("foo")),
                right: Box::new(Expr::Var("bar")),
            })),
            right: Box::new(Expr::Var("baz")),
        });
        assert_eq!(expr(0).easy_parse(src), Ok((expected, "")));
    }
    #[test]
    fn right_associative() {
        let src = "foo <- bar <- baz";
        let expected = <Expr<()>>::Assign(Assign {
            place: Box::new(PlaceExpr::Var("foo")),
            expr: Box::new(Expr::Assign(Assign {
                place: Box::new(PlaceExpr::Var("bar")),
                expr: Box::new(Expr::Var("baz")),
            })),
        });
        assert_eq!(expr(0).easy_parse(src), Ok((expected, "")));
    }
    #[test]
    fn ignore_higher_precedence() {
        let src = "foo + bar";
        let expected = <Expr<()>>::Var("foo");
        let left = "+ bar";
        assert_eq!(expr(6).easy_parse(src), Ok((expected, left)));
    }
    #[test]
    fn ignore_range() {
        let src = "foo..";
        let expected = <Expr<()>>::Var("foo");
        let left = "..";
        assert_eq!(expr(0).easy_parse(src), Ok((expected, left)));
    }
}
