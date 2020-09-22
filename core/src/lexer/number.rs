use util::lexer::Lex;

pub struct Num;
impl<'a> Lex<'a> for Num {
    fn lex_first(src: &'a str) -> Option<(usize, Self)> {
        let mut chars = src.chars();
        let first = chars.next();
        let num = if let Some('.') = first {
            chars.next()
        } else {
            first
        };
        if let Some('0'..='9') = num {
            let mut prev_e = false;
            let mut chars = src[1..].char_indices().peekable();
            while let Some((i, ch)) = chars.next() {
                let e = matches!(ch, 'e' | 'E');
                let resume = match ch {
                    '-' | '+' => prev_e,
                    '_' => true,
                    '.' => matches!(chars.peek(), Some((_, '0'..='9'))),
                    ch => ch.is_alphanumeric(),
                };
                if resume {
                    prev_e = e;
                } else {
                    return Some((i + 1, Self));
                }
            }
            Some((src.len(), Self))
        } else {
            None
        }
    }
}
