use crate::lexer::Bracket;
use crate::lexer::Opening;
use crate::lexer::Operator;
use crate::lexer::Token;
use crate::parser::bracket::BracketFragment;
use crate::parser::bracket::BracketSyntax;
use crate::parser::error::ErrorType;
use crate::parser::error::TokenKind;
use crate::parser::error_start;
use crate::parser::node_type::Binary;
use crate::parser::node_type::NodeType;
use crate::parser::Node;
use crate::parser::ParseResult;
use crate::parser::Parser;
use std::iter::once;
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
        Some(_) | None => Err(error_start(
            &span[span.len()..],
            ErrorType::NoExpectation(&[TokenKind::Ident]),
        )),
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
        Some(Token::Bracket(Opening::Open, Bracket::Bracket)) => {
            index_or_slice(parser, left, span, true)
        }
        Some(_) | None => Err(error_start(
            &span[span.len()..],
            ErrorType::NoExpectation(&[
                TokenKind::Operator(Operator::Dot),
                TokenKind::Bracket(Opening::Open, Bracket::Bracket),
            ]),
        )),
    }
}
pub(super) fn index_or_slice<'a>(
    parser: &mut Parser<'a>,
    left: ParseResult<'a>,
    left_bracket_span: &'a str,
    optional: bool,
) -> ParseResult<'a> {
    let right = BracketFragment::parse_rest(parser).and_then(|bracket_fragment| {
        let (node, right_first, right_second) = match (bracket_fragment.syntax, optional) {
            (BracketSyntax::Single(expr), false) => (NodeType::Index, Some(expr), None),
            (BracketSyntax::Single(expr), true) => (NodeType::OptionalIndex, Some(expr), None),
            (BracketSyntax::Range(first, range_type, second), false) => {
                (NodeType::Slice(range_type), first, second)
            }
            (BracketSyntax::Range(first, range_type, second), true) => {
                (NodeType::OptionalSlice(range_type), first, second)
            }
            _ => {
                let bracket_span = span_from_spans(
                    parser.src,
                    left_bracket_span,
                    bracket_fragment.right_bracket_span,
                );
                return Err(error_start(bracket_span, ErrorType::NonIndexNorSlice));
            }
        };
        Ok((
            bracket_fragment.right_bracket_span,
            node,
            right_first,
            right_second,
        ))
    });
    let (left, (right_bracket_span, node, right_first, right_second)) =
        aggregate_error(left, right)?;
    let span = span_from_spans(parser.src, left.content.span, right_bracket_span);
    let children = once(left)
        .chain(right_first.into_iter())
        .chain(right_second.into_iter())
        .collect();
    Ok(Tree {
        content: Node { node, span },
        children,
    })
}
