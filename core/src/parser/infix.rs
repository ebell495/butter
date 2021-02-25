use crate::lexer::Bracket;
use crate::lexer::Opening;
use crate::lexer::Operator;
use crate::lexer::Token;
use crate::parser::error::ErrorType;
use crate::parser::error_start;
use crate::parser::node_type::Binary;
use crate::parser::node_type::NodeType;
use crate::parser::Node;
use crate::parser::ParseResult;
use crate::parser::Parser;
use util::aggregate_error;
use util::join_trees;
use util::parser::ParserIter;
use util::span::span_from_spans;
use util::tree_vec::Tree;

pub(super) fn operator<'a>(
    parser: &mut Parser<'a>,
    left: ParseResult<'a>,
    span: &'a str,
    operator: Operator,
) -> ParseResult<'a> {
    match operator {
        Operator::Dot => property_access(parser, left, span, NodeType::Property),
        Operator::LeftArrow => assign(parser, left),
        Operator::Question => question(parser, left, span),
        operator => expr_operator(parser, left, operator),
    }
}
fn expr_operator<'a>(
    parser: &mut Parser<'a>,
    left: ParseResult<'a>,
    operator: Operator,
) -> ParseResult<'a> {
    let (operator, precedence) = match operator {
        Operator::Star => (Binary::Multiply, 80),
        Operator::Slash => (Binary::Div, 80),
        Operator::DoubleSlash => (Binary::FloorDiv, 80),
        Operator::Percent => (Binary::Mod, 80),
        Operator::Plus => (Binary::Add, 70),
        Operator::Minus => (Binary::Sub, 70),
        Operator::DoublePlus => (Binary::Concatenate, 70),
        Operator::DoubleEqual => (Binary::Equal, 60),
        Operator::NotEqual => (Binary::NotEqual, 60),
        Operator::Less => (Binary::Less, 60),
        Operator::LessEqual => (Binary::LessEqual, 60),
        Operator::Greater => (Binary::Greater, 60),
        Operator::GreaterEqual => (Binary::GreaterEqual, 60),
        Operator::Amp => (Binary::And, 50),
        Operator::DoubleAmp => (Binary::LazyAnd, 50),
        Operator::Pipe => (Binary::Or, 40),
        Operator::DoublePipe => (Binary::LazyOr, 40),
        Operator::DoubleQuestion => (Binary::NullOr, 30),
        operator => panic!("expected expression operator, found {:?}", operator),
    };
    let (left, right) = aggregate_error(left, parser.partial_parse(precedence))?;
    Ok(Tree {
        content: Node {
            span: span_from_spans(parser.src, left.content.span, right.content.span),
            node: NodeType::Binary(operator),
        },
        children: join_trees![left, right],
    })
}
fn assign<'a>(parser: &mut Parser<'a>, left: ParseResult<'a>) -> ParseResult<'a> {
    let left = left.and_then(|node| {
        if node.content.node.place() {
            Ok(node)
        } else {
            Err(error_start(node.content.span, ErrorType::NonPlace))
        }
    });
    let (left, right) = aggregate_error(left, parser.partial_parse(19))?;
    Ok(Tree {
        content: Node {
            span: span_from_spans(parser.src, left.content.span, right.content.span),
            node: NodeType::Assign,
        },
        children: join_trees![left, right],
    })
}
fn property_access<'a>(
    parser: &mut Parser<'a>,
    left: ParseResult<'a>,
    span: &'a str,
    node: NodeType,
) -> ParseResult<'a> {
    let right = match parser.peek_token() {
        Some(Token::Ident) => {
            let span = parser.next().unwrap().span;
            Ok(Tree::new(Node {
                span,
                node: NodeType::Ident,
            }))
        }
        Some(_) | None => Err(error_start(&span[span.len()..], ErrorType::NoIdent)),
    };
    let (left, right) = aggregate_error(left, right)?;
    Ok(Tree {
        content: Node {
            span: span_from_spans(parser.src, left.content.span, right.content.span),
            node,
        },
        children: join_trees![left, right],
    })
}
fn question<'a>(parser: &mut Parser<'a>, left: ParseResult<'a>, span: &'a str) -> ParseResult<'a> {
    match parser.peek_token() {
        Some(Token::Operator(Operator::Dot)) => {
            property_access(parser, left, span, NodeType::OptionalProperty)
        }
        Some(Token::Bracket(Opening::Open, Bracket::Bracket)) => todo!(),
        Some(_) | None => Err(error_start(&span[span.len()..], ErrorType::NoOptionalChain)),
    }
}
