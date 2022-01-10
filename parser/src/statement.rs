use crate::{
    expr::{control_flow::control_flow, expr},
    ident_keyword::ident,
    lex,
    pattern::{parameter, pattern},
};
use combine::{
    attempt, choice,
    error::StreamError,
    look_ahead, optional,
    parser::char::{char, string},
    sep_by1,
    stream::StreamErrorFor,
    ParseError, Parser, RangeStream,
};
use hir::{
    expr::{Assign, Expr, Fun},
    statement::{Declare, FunDeclare, Statement},
};

pub(crate) enum StatementReturn<'a, T> {
    Statement(Statement<'a, T>),
    Return(Expr<'a, T>),
}
fn statement_return_<'a, I, P, T>(
    end_look_ahead: P,
) -> impl Parser<I, Output = StatementReturn<'a, T>>
where
    I: RangeStream<Token = char, Range = &'a str>,
    I::Error: ParseError<I::Token, I::Range, I::Position>,
    P: Parser<I>,
    T: Default,
{
    let control_flow_statement = || {
        (control_flow(), optional(lex(char(';')))).map(|(control_flow, semicolon)| {
            let expr = Expr::ControlFlow(control_flow);
            match semicolon {
                Some(_) => StatementReturn::Statement(Statement::Expr(expr)),
                None => StatementReturn::Return(expr),
            }
        })
    };
    let fun_body = || {
        choice((
            control_flow()
                .skip(optional(lex(char(';'))))
                .map(|control_flow| Expr::ControlFlow(control_flow)),
            expr(0).skip(lex(char(';'))),
        ))
    };
    let fun_declare = || {
        (
            attempt((ident(), parameter().skip(lex(string("=>"))))),
            fun_body(),
        )
            .map(|((ident, param), body)| {
                Statement::FunDeclare(FunDeclare {
                    ident,
                    fun: Fun {
                        param,
                        body: Box::new(body),
                    },
                    ty: T::default(),
                })
            })
    };
    let place = || {
        expr(1).and_then(|expr| {
            if let Expr::Place(place) = expr {
                Ok(place)
            } else {
                Err(<StreamErrorFor<I>>::expected_static_message(
                    "place expression",
                ))
            }
        })
    };
    let declare = || {
        (attempt(pattern().skip(lex(char('=')))), expr(0))
            .skip(lex(char(';')))
            .map(|(pattern, expr)| {
                StatementReturn::Statement(Statement::Declare(Declare { pattern, expr }))
            })
    };
    let parallel_assign = || {
        (
            attempt(sep_by1(place(), lex(char(','))).skip(lex(string("<-")))),
            sep_by1(expr(0), lex(char(','))),
        )
            .and_then(|(place, expr)| {
                let place: Vec<_> = place;
                let expr: Vec<_> = expr;
                if place.len() != expr.len() {
                    return Err(<StreamErrorFor<I>>::message_static_message(
                        "mismatching count of place and value expressions",
                    ));
                }
                let assign = place
                    .into_iter()
                    .zip(expr.into_iter())
                    .map(|(place, expr)| Assign {
                        place: Box::new(place),
                        expr: Box::new(expr),
                    })
                    .collect();
                Ok(Expr::Assign(assign))
            })
    };
    let expr = || {
        (
            choice((parallel_assign(), expr(0))),
            choice((
                lex(char(';')).map(|_| true),
                look_ahead(end_look_ahead).map(|_| false),
            )),
        )
            .map(|(expr, implicit_return)| {
                if implicit_return {
                    StatementReturn::Return(expr)
                } else {
                    StatementReturn::Statement(Statement::Expr(expr))
                }
            })
    };
    choice((
        control_flow_statement(),
        declare(),
        fun_declare().map(StatementReturn::Statement),
        expr(),
    ))
}
combine::parser! {
    pub(crate) fn statement_return['a, I, P, T](end_look_ahead: P)(I) -> StatementReturn<'a, T>
    where [
        I: RangeStream<Token = char, Range = &'a str>,
        I::Error: ParseError<I::Token, I::Range, I::Position>,
        P: Parser<I>,
        T: Default,
    ] {
        statement_return_(end_look_ahead)
    }
}
pub(crate) fn statement<'a, I, T>() -> impl Parser<I, Output = Statement<'a, T>>
where
    I: RangeStream<Token = char, Range = &'a str>,
    I::Error: ParseError<I::Token, I::Range, I::Position>,
    T: Default,
{
    statement_return(char(';')).map(|statement_return| match statement_return {
        StatementReturn::Statement(statement) => statement,
        StatementReturn::Return(expr) => Statement::Expr(expr),
    })
}
#[cfg(test)]
mod test {
    use crate::{
        statement::{statement, Assign, Expr},
        Statement,
    };
    use combine::EasyParser;
    use hir::{
        expr::{Literal, PlaceExpr},
        pattern::{Pattern, Var},
        statement::Declare,
    };

    #[test]
    fn parallel_assign() {
        let src = "foo, bar <- bar, foo;";
        let expected: Statement<()> = Statement::Expr(Expr::Assign(
            vec![
                Assign {
                    place: Box::new(PlaceExpr::Var("foo")),
                    expr: Box::new(Expr::Place(PlaceExpr::Var("bar"))),
                },
                Assign {
                    place: Box::new(PlaceExpr::Var("bar")),
                    expr: Box::new(Expr::Place(PlaceExpr::Var("foo"))),
                },
            ]
            .into(),
        ));
        assert_eq!(statement().easy_parse(src), Ok((expected, "")));
    }
    #[test]
    fn chain_assign() {
        let src = "foo <- bar <- baz;";
        let expected: Statement<()> = Statement::Expr(Expr::Assign(
            vec![Assign {
                place: Box::new(PlaceExpr::Var("foo")),
                expr: Box::new(Expr::Assign(
                    vec![Assign {
                        place: Box::new(PlaceExpr::Var("bar")),
                        expr: Box::new(Expr::Place(PlaceExpr::Var("baz"))),
                    }]
                    .into(),
                )),
            }]
            .into(),
        ));
        assert_eq!(statement().easy_parse(src), Ok((expected, "")));
    }
    #[test]
    fn var() {
        let src = "foo = 10;";
        let expected: Statement<()> = Statement::Declare(Declare {
            pattern: Pattern::Var(Var {
                ident: "foo",
                mutable: false,
                bind_to_ref: false,
                ty: (),
            }),
            expr: Expr::Literal(Literal::UInt(10)),
        });
        assert_eq!(statement().easy_parse(src), Ok((expected, "")));
    }
}
