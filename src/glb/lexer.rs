use std::{fmt::Display, str::CharIndices};

pub struct Lexer<'a> {
    chars: std::iter::Peekable<CharIndices<'a>>,
    whole: &'a str,
}

#[derive(PartialEq, Debug)]
pub enum Token<'a> {
    LeftBraket,
    RightBraket,
    LeftBrace,
    RightBrace,
    Comma,
    String(&'a str),
}

impl<'a> Lexer<'a> {
    #[must_use]
    pub fn new(content: &'a str) -> Self {
        Self {
            chars: content.char_indices().peekable(),
            whole: content,
        }
    }

    fn string(&mut self, start: usize) -> Token<'a> {
        let mut finish = start;

        while let Some((i, next)) = self.chars.peek() {
            match *next {
                '{' | '}' | ',' | '[' | ']' => break,
                _ => {
                    finish = *i;
                    self.chars.next(); // consume
                    continue;
                }
            }
        }

        Token::String(&self.whole[start..=finish])
    }
}

impl<'a> Iterator for Lexer<'a> {
    type Item = Token<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let (i, current) = self.chars.next()?;
            return match current {
                '{' => Some(Token::LeftBrace),
                '}' => Some(Token::RightBrace),
                '[' => Some(Token::LeftBraket),
                ']' => Some(Token::RightBraket),
                ',' => Some(Token::Comma),
                ' ' | '\t' | '\r' | '\n' => continue, // skip whitespaces
                _ => Some(self.string(i)),
            };
        }
    }
}

impl Display for Token<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Token::String(s) => write!(f, "STRING \"{s}\""),
            Token::LeftBrace => write!(f, "LBRACE {{"),
            Token::RightBrace => write!(f, "RBRACE }}"),
            Token::LeftBraket => write!(f, "LRBRAKET ["),
            Token::RightBraket => write!(f, "RBRAKET ]"),
            Token::Comma => write!(f, "COMMA ,"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case("22", vec![Token::String("22")])]
    #[case("*.e1", vec![Token::String("*.e1")])]
    #[case("**.e1", vec![Token::String("**.e1")])]
    #[case("*.{e1}", vec![Token::String("*."), Token::LeftBrace, Token::String("e1"), Token::RightBrace])]
    #[case("*.[ch]", vec![Token::String("*."), Token::LeftBraket, Token::String("ch"), Token::RightBraket])]
    #[case("*.{e1,e2}", vec![Token::String("*."), Token::LeftBrace, Token::String("e1"), Token::Comma, Token::String("e2"), Token::RightBrace])]
    #[case("*.{e1,e2,f1.e1}", vec![Token::String("*."), Token::LeftBrace, Token::String("e1"), Token::Comma, Token::String("e2"), Token::Comma, Token::String("f1.e1"), Token::RightBrace])]
    #[case("{f1.e1,f1.[ch]}", vec![Token::LeftBrace, Token::String("f1.e1"), Token::Comma, Token::String("f1."), Token::LeftBraket, Token::String("ch"), Token::RightBraket, Token::RightBrace])]
    #[case("test/{p1,p2}/*", vec![Token::String("test/"), Token::LeftBrace, Token::String("p1"), Token::Comma, Token::String("p2"), Token::RightBrace, Token::String("/*")])]
    #[case("{*.e1,*.e2}", vec![Token::LeftBrace, Token::String("*.e1"), Token::Comma, Token::String("*.e2"), Token::RightBrace])]
    #[trace]
    fn parse_cases(#[case] input_str: &str, #[case] expected: Vec<Token>) {
        // Arrange
        let lexer = Lexer::new(input_str);

        // Act
        let actual: Vec<Token> = lexer.into_iter().collect();

        // Assert
        assert_eq!(expected, actual);
    }
}
