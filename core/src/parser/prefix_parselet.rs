use crate::lexer::Keyword;
use crate::lexer::Operator;
use crate::lexer::Token;
use crate::parser::node_type::NodeType;
use crate::parser::node_type::UnaryOp;
use crate::parser::Node;
use crate::parser::ParseResult;
use crate::parser::SpanToken;
use util::parser::Parser;
use util::tree_vec::Tree;

pub(super) fn keyword_literal<'a>(
    prefix: SpanToken<'a>,
    _: &mut Parser<impl Iterator<Item = SpanToken<'a>>>,
) -> Option<ParseResult<'a>> {
    let SpanToken {
        span,
        token: prefix,
    } = prefix;
    if let Token::Keyword(keyword) = prefix {
        let node = match keyword {
            Keyword::True => NodeType::True,
            Keyword::False => NodeType::False,
            Keyword::Null => NodeType::Null,
            _ => return None,
        };
        Some(Ok(Tree::new(Node {
            span,
            node,
            unpack: false,
        })))
    } else {
        None
    }
}
pub(super) fn clone<'a>(
    prefix: SpanToken<'a>,
    tokens: &mut Parser<impl Iterator<Item = SpanToken<'a>>>,
) -> Option<ParseResult<'a>> {
    let SpanToken {
        span,
        token: prefix,
    } = prefix;
    if let Token::Keyword(Keyword::Clone) = prefix {
        let operand = match tokens.partial_parse(90) {
            Ok(node) => node,
            Err(err) => return Some(Err(err)),
        };
        Some(Ok(Tree {
            content: Node {
                span: span.up_to(operand.content.span),
                node: NodeType::Unary(UnaryOp::Clone),
                unpack: false,
            },
            children: operand.into_tree_vec(),
        }))
    } else {
        None
    }
}
pub(super) fn operator<'a>(
    prefix: SpanToken<'a>,
    tokens: &mut Parser<impl Iterator<Item = SpanToken<'a>>>,
) -> Option<ParseResult<'a>> {
    let SpanToken {
        span,
        token: prefix,
    } = prefix;
    if let Token::Operator(operator) = prefix {
        let operator = match operator {
            Operator::Plus => UnaryOp::Plus,
            Operator::Minus => UnaryOp::Minus,
            Operator::Bang => UnaryOp::Not,
            Operator::Amp => UnaryOp::Ref,
            _ => return None,
        };
        let operand = match tokens.partial_parse(90) {
            Ok(node) => node,
            Err(err) => return Some(Err(err)),
        };
        Some(Ok(Tree {
            content: Node {
                span: span.up_to(operand.content.span),
                node: NodeType::Unary(operator),
                unpack: false,
            },
            children: operand.into_tree_vec(),
        }))
    } else {
        None
    }
}
