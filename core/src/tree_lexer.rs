use crate::lexer::Bracket;
use crate::lexer::LexerError;
use crate::lexer::Opening;
use crate::lexer::Token;
use crate::lexer::TokenSpans;
pub enum TreeSpansResult<'a> {
    Token(&'a str, Token<'a>),
    TokenError(&'a str, LexerError<'a>),
    TreeError(BracketError<'a>),
    In(&'a str, Bracket),
    Out(&'a str, Bracket),
}
#[derive(Debug, PartialEq, Eq)]
pub enum BracketError<'a> {
    Mismatch((&'a str, Bracket), (&'a str, Bracket)),
    Unexpected(&'a str, Bracket),
    Unmatched(&'a str, Bracket),
}
pub struct TreeSpans<'a> {
    tokens: TokenSpans<'a>,
    closes: Vec<(&'a str, Bracket)>,
    done: bool,
    err: bool,
}
impl<'a> TreeSpans<'a> {
    pub fn new<T: Into<Self>>(src: T) -> Self {
        src.into()
    }
}
impl<'a, T> From<T> for TreeSpans<'a>
where
    T: Into<TokenSpans<'a>>,
{
    fn from(val: T) -> Self {
        TreeSpans {
            tokens: val.into(),
            closes: vec![],
            done: false,
            err: false,
        }
    }
}
impl<'a> Iterator for TreeSpans<'a> {
    type Item = TreeSpansResult<'a>;
    fn next(&mut self) -> Option<Self::Item> {
        if self.done {
            None
        } else if self.err {
            if let Some((src, bracket)) = self.closes.pop() {
                Some(TreeSpansResult::TreeError(BracketError::Unmatched(
                    src, bracket,
                )))
            } else {
                self.done = true;
                None
            }
        } else if let Some((src, token)) = self.tokens.next() {
            let res = match token {
                Ok(Token::Bracket(Opening::Open, bracket)) => {
                    self.closes.push((src, bracket));
                    TreeSpansResult::In(src, bracket)
                }
                Ok(Token::Bracket(Opening::Close, bracket)) => match self.closes.pop() {
                    Some((_, open)) if open == bracket => TreeSpansResult::Out(src, bracket),
                    Some((first_src, open)) => {
                        self.done = true;
                        TreeSpansResult::TreeError(BracketError::Mismatch(
                            (first_src, open),
                            (src, bracket),
                        ))
                    }
                    None => {
                        self.done = true;
                        TreeSpansResult::TreeError(BracketError::Unexpected(src, bracket))
                    }
                },
                Ok(token) => TreeSpansResult::Token(src, token),
                Err(err) => TreeSpansResult::TokenError(src, err),
            };
            Some(res)
        } else if let Some((src, bracket)) = self.closes.pop() {
            self.err = true;
            Some(TreeSpansResult::TreeError(BracketError::Unmatched(
                src, bracket,
            )))
        } else {
            self.done = true;
            None
        }
    }
}
#[derive(Debug, PartialEq, Eq)]
pub enum TreeError<'a> {
    Token(&'a str, LexerError<'a>),
    Tree(BracketError<'a>),
}
pub type Trees<'a> = Vec<Tree<'a>>;
#[derive(Debug, PartialEq)]
pub enum Tree<'a> {
    Token(Token<'a>),
    Tree(Bracket, Trees<'a>),
}
impl<'a> Tree<'a> {
    pub fn lex(src: &'a str) -> Result<Vec<Self>, Vec<TreeError<'a>>> {
        let mut errors = vec![];
        let mut stack = vec![];
        let mut current = vec![];
        for token in TreeSpans::new(src) {
            match token {
                TreeSpansResult::Token(_, token) => {
                    if errors.is_empty() {
                        current.push(Self::Token(token));
                    }
                }
                TreeSpansResult::In(_, _) => {
                    if errors.is_empty() {
                        stack.push(current.drain(..).collect());
                    }
                }
                TreeSpansResult::Out(_, bracket) => {
                    if errors.is_empty() {
                        let previous = current;
                        current = stack.pop().unwrap();
                        current.push(Self::Tree(bracket, previous));
                    }
                }
                TreeSpansResult::TokenError(src, err) => {
                    errors.push(TreeError::Token(src, err));
                }
                TreeSpansResult::TreeError(err) => {
                    errors.push(TreeError::Tree(err));
                }
            }
        }
        if errors.is_empty() {
            debug_assert!(stack.is_empty());
            Ok(current)
        } else {
            Err(errors)
        }
    }
}
#[cfg(test)]
mod test {
    use super::Tree;
    use crate::lexer::Bracket;
    #[test]
    fn tree_lex() {
        assert_eq!(
            Tree::lex("()[{}]"),
            Ok(vec![
                Tree::Tree(Bracket::Paren, vec![]),
                Tree::Tree(Bracket::Bracket, vec![Tree::Tree(Bracket::Brace, vec![])]),
            ]),
        );
    }
}