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
use combine::value;
use combine::ParseError;
use combine::Parser;
use combine::RangeStream;
use hir::expr::Element;
use hir::expr::ElementKind;
use hir::expr::Expr;
use hir::expr::Fun;
use hir::expr::Jump;
use hir::expr::Literal;
use hir::expr::PlaceExpr;
use hir::expr::Tag;
use hir::expr::Unary;
use hir::expr::UnaryType;

mod array;
pub mod control_flow;
mod float;
mod infix;
pub mod integer;
mod record;
mod string;

parser! {
    fn literal['a, I]()(I) -> Literal
    where [
        I: RangeStream<Token = char, Range = &'a str>,
        I::Error: ParseError<I::Token, I::Range, I::Position>,
    ] {
        choice((
            char_literal().map(Literal::UInt),
            float::float().map(Literal::Float),
            integer_u64().map(Literal::UInt),
            attempt(keyword("false")).with(value(Literal::False)),
            attempt(keyword("true") ).with(value(Literal::True)),
            attempt(keyword("void") ).with(value(Literal::Void)),
        ))
    }
}
fn jump<'a, I, T>() -> impl Parser<I, Output = Jump<'a, T>>
where
    I: RangeStream<Token = char, Range = &'a str>,
    I::Error: ParseError<I::Token, I::Range, I::Position>,
    T: Default,
{
    choice((
        lex(keyword("break"))
            .with(optional(expr(0)))
            .map(|expr| Jump::Break(expr.map(Box::new))),
        lex(keyword("continue")).map(|_| Jump::Continue),
        lex(keyword("return"))
            .with(optional(expr(0)))
            .map(|expr| Jump::Return(expr.map(Box::new))),
    ))
}
fn unary<'a, I, T>() -> impl Parser<I, Output = Unary<'a, T>>
where
    I: RangeStream<Token = char, Range = &'a str>,
    I::Error: ParseError<I::Token, I::Range, I::Position>,
    T: Default,
{
    let kind = || {
        choice((
            char('!').with(value(UnaryType::Not)),
            char('&').with(value(UnaryType::Ref)),
            char('-').with(value(UnaryType::Minus)),
            char('>').with(value(UnaryType::Move)),
            attempt(keyword("clone")).with(value(UnaryType::Clone)),
        ))
    };
    (lex(kind()), expr(6)).map(|(kind, expr)| Unary {
        kind,
        expr: Box::new(expr),
    })
}
fn tag<'a, I, T>() -> impl Parser<I, Output = Tag<'a, T>>
where
    I: RangeStream<Token = char, Range = &'a str>,
    I::Error: ParseError<I::Token, I::Range, I::Position>,
    T: Default,
{
    lex(char('@'))
        .with((lex(ident()), optional(expr(6))))
        .map(|(tag, expr)| Tag {
            tag,
            expr: expr.map(Box::new),
        })
}
fn fun<'a, I, T>() -> impl Parser<I, Output = Fun<'a, T>>
where
    I: RangeStream<Token = char, Range = &'a str>,
    I::Error: ParseError<I::Token, I::Range, I::Position>,
    T: Default,
{
    (attempt(parameter().skip(lex(string("=>")))), expr(0)).map(|(param, body)| Fun {
        param,
        body: Box::new(body),
    })
}
fn prefix_expr_<'a, I, T>() -> impl Parser<I, Output = Expr<'a, T>>
where
    I: RangeStream<Token = char, Range = &'a str>,
    I::Error: ParseError<I::Token, I::Range, I::Position>,
    T: Default,
{
    choice((
        fun().map(Expr::Fun),
        attempt(range()).map(Expr::ArrayRange),
        array().map(Expr::Array),
        attempt(between(lex(char('(')), lex(char(')')), expr(0))),
        record().map(Expr::Record),
        lex(string_literal()).map(|vec| {
            let vec = vec
                .into_iter()
                .map(|byte| Element {
                    expr: Expr::Literal(Literal::UInt(byte as u64)),
                    kind: ElementKind::Element,
                })
                .collect();
            Expr::Array(vec)
        }),
        unary().map(Expr::Unary),
        tag().map(Expr::Tag),
        attempt(lex(ident())).map(|ident| Expr::Place(PlaceExpr::Var(ident))),
        control_flow::control_flow().map(Expr::ControlFlow),
        lex(literal()).map(Expr::Literal),
        jump().map(Expr::Jump),
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
    use hir::expr::Assign;
    use hir::expr::Binary;
    use hir::expr::BinaryType;
    use hir::expr::PlaceExpr;

    #[test]
    fn group() {
        let src = "(foo)";
        let expected: Expr<()> = Expr::Place(PlaceExpr::Var("foo"));
        assert_eq!(expr(0).easy_parse(src), Ok((expected, "")));
    }
    #[test]
    fn precedence() {
        let src = "foo + bar * baz";
        let expected: Expr<()> = Expr::Binary(Binary {
            kind: BinaryType::Add,
            left: Box::new(Expr::Place(PlaceExpr::Var("foo"))),
            right: Box::new(Expr::Binary(Binary {
                kind: BinaryType::Multiply,
                left: Box::new(Expr::Place(PlaceExpr::Var("bar"))),
                right: Box::new(Expr::Place(PlaceExpr::Var("baz"))),
            })),
        });
        assert_eq!(expr(0).easy_parse(src), Ok((expected, "")));
        let src = "foo * bar + baz";
        let expected: Expr<()> = Expr::Binary(Binary {
            kind: BinaryType::Add,
            left: Box::new(Expr::Binary(Binary {
                kind: BinaryType::Multiply,
                left: Box::new(Expr::Place(PlaceExpr::Var("foo"))),
                right: Box::new(Expr::Place(PlaceExpr::Var("bar"))),
            })),
            right: Box::new(Expr::Place(PlaceExpr::Var("baz"))),
        });
        assert_eq!(expr(0).easy_parse(src), Ok((expected, "")));
    }
    #[test]
    fn right_associative() {
        let src = "foo <- bar <- baz";
        let expected: Expr<()> = Expr::Assign(Assign {
            place: Box::new(PlaceExpr::Var("foo")),
            expr: Box::new(Expr::Assign(Assign {
                place: Box::new(PlaceExpr::Var("bar")),
                expr: Box::new(Expr::Place(PlaceExpr::Var("baz"))),
            })),
        });
        assert_eq!(expr(0).easy_parse(src), Ok((expected, "")));
    }
    #[test]
    fn ignore_higher_precedence() {
        let src = "foo + bar";
        let expected: Expr<()> = Expr::Place(PlaceExpr::Var("foo"));
        let left = "+ bar";
        assert_eq!(expr(6).easy_parse(src), Ok((expected, left)));
    }
    #[test]
    fn ignore_range() {
        let src = "foo..";
        let expected: Expr<()> = Expr::Place(PlaceExpr::Var("foo"));
        let left = "..";
        assert_eq!(expr(0).easy_parse(src), Ok((expected, left)));
    }
}
