use combine::attempt;
use combine::choice;
use combine::parser::char::space;
use combine::parser::char::string;
use combine::parser::range::take_while;
use combine::sep_by;
use combine::sep_end_by;
use combine::skip_many;
use combine::skip_many1;
use combine::ParseError;
use combine::Parser;
use combine::RangeStream;

mod expr;
mod ident_keyword;
mod pattern;

// TODO: this is bad
fn sep_optional_end_by<'a, I, EP, SP, C>(
    element: fn() -> EP,
    separator: fn() -> SP,
) -> impl Parser<I, Output = C>
where
    I: RangeStream<Token = char, Range = &'a str>,
    I::Error: ParseError<I::Token, I::Range, I::Position>,
    EP: Parser<I>,
    SP: Parser<I>,
    C: Extend<EP::Output> + Default,
{
    choice((
        attempt(sep_end_by(element(), separator())),
        sep_by(element(), separator()),
    ))
}
fn comments<'a, I>() -> impl Parser<I, Output = ()>
where
    I: RangeStream<Token = char, Range = &'a str>,
    I::Error: ParseError<I::Token, I::Range, I::Position>,
{
    skip_many1((attempt(string("--")), take_while(|ch: char| ch != '\n')).expected("comment"))
}
fn insignificants<'a, I>() -> impl Parser<I, Output = ()>
where
    I: RangeStream<Token = char, Range = &'a str>,
    I::Error: ParseError<I::Token, I::Range, I::Position>,
{
    skip_many(skip_many1(space()).or(comments()))
}
fn lex<'a, I, P>(parser: P) -> impl Parser<I, Output = P::Output>
where
    I: RangeStream<Token = char, Range = &'a str>,
    I::Error: ParseError<I::Token, I::Range, I::Position>,
    P: Parser<I>,
{
    parser.skip(insignificants())
}
#[cfg(test)]
mod test {
    use crate::parser::insignificants;
    use combine::Parser;

    #[test]
    fn insignificant() {
        assert_eq!(
            insignificants()
                .parse("  -- comment\n  -- more comment")
                .unwrap(),
            ((), "")
        )
    }
}
