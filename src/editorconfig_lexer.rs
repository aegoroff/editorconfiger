use nom::branch::alt;
use nom::bytes::complete::is_not;
use nom::error::{FromExternalError, ParseError, VerboseError};
use nom::sequence;
use nom::{character::complete, combinator, IResult};

/// Represents .editorconfig lexical token abstraction that contain necessary data
#[derive(Debug, PartialEq, Copy, Clone)]
pub enum Token<'a> {
    Head(&'a str),
    Pair(&'a str, &'a str),
    Comment(&'a str),
}

pub struct TokenIterator<'a> {
    input: &'a str,
    inline_comment: &'a str,
}

pub fn tokenize(input: &str) -> Vec<Token> {
    TokenIterator::new(input).collect()
}

impl<'a> TokenIterator<'a> {
    pub fn new(input: &'a str) -> Self {
        Self {
            input,
            inline_comment: "",
        }
    }

    fn parse_line(&mut self, trail: &'a str, val: &'a str) -> Option<Token<'a>> {
        let parsed_line = line::<'a, VerboseError<&'a str>>(val);
        self.input = trail;
        if let Ok((comment, token)) = parsed_line {
            self.inline_comment = comment;
            return Some(token);
        }

        None
    }
}

impl<'a> Iterator for TokenIterator<'a> {
    type Item = Token<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if !self.inline_comment.is_empty() {
            if let Ok((_, inline)) = comment::<'a, VerboseError<&'a str>>(self.inline_comment) {
                self.inline_comment = "";
                return Some(inline);
            }
            self.inline_comment = "";
        }

        loop {
            if self.input.is_empty() {
                break;
            }
            let mut parser = sequence::terminated(complete::not_line_ending, complete::line_ending);
            let parsed: IResult<&'a str, &'a str, VerboseError<&'a str>> = parser(self.input);
            return match parsed {
                Ok((trail, val)) => match self.parse_line(trail, val) {
                    None => continue,
                    Some(token) => return Some(token),
                },
                Err(_) => self.parse_line("", self.input), // EOF
            };
        }
        None
    }
}

fn line<'a, E>(input: &'a str) -> IResult<&'a str, Token<'a>, E>
where
    E: ParseError<&'a str> + std::fmt::Debug + FromExternalError<&'a str, nom::Err<char>>,
{
    alt((head::<E>, key_value::<E>, comment::<E>))(input)
}

fn head<'a, E>(input: &'a str) -> IResult<&'a str, Token<'a>, E>
where
    E: ParseError<&'a str> + std::fmt::Debug + FromExternalError<&'a str, nom::Err<char>>,
{
    let parser = sequence::preceded(complete::char('['), is_not("\n\r;#"));

    //  capture data until last ] to support brackets inside section head
    combinator::map_res(parser, |val: &str| match val.rfind(']') {
        Some(ix) => Ok(Token::Head(&val[..ix])),
        None => Err(nom::Err::Failure(']')),
    })(input)
}

fn key_value<'a, E>(input: &'a str) -> IResult<&'a str, Token<'a>, E>
where
    E: ParseError<&'a str> + std::fmt::Debug,
{
    let parser = sequence::separated_pair(is_not("=;#"), complete::char('='), is_not("=;#"));

    combinator::map(parser, |(k, v): (&str, &str)| {
        Token::Pair(k.trim(), v.trim())
    })(input)
}

fn comment<'a, E>(input: &'a str) -> IResult<&'a str, Token<'a>, E>
where
    E: ParseError<&'a str> + std::fmt::Debug,
{
    combinator::map(
        combinator::recognize(sequence::preceded(
            alt((complete::char('#'), complete::char(';'))),
            is_not("\n\r"),
        )),
        |val: &str| Token::Comment(val),
    )(input)
}

#[cfg(test)]
mod tests {
    use super::*;
    use spectral::prelude::*;

    #[test]
    fn parse_test() {
        // Arrange
        let cases = vec![
            ("", vec![]),
            ("[*.md]", vec![Token::Head("*.md")]),
            (
                "[*.md] ; test",
                vec![Token::Head("*.md"), Token::Comment("; test")],
            ),
            ("[*.[md]]", vec![Token::Head("*.[md]")]),
            ("[*.[md]", vec![Token::Head("*.[md")]),
            ("[ *.[md] ]", vec![Token::Head(" *.[md] ")]),
            ("[a]\n[b]", vec![Token::Head("a"), Token::Head("b")]),
            ("[a]\r\n[b]", vec![Token::Head("a"), Token::Head("b")]),
            ("[a]\n\n[b]", vec![Token::Head("a"), Token::Head("b")]),
            ("[a]", vec![Token::Head("a")]),
            ("[a]\r\n", vec![Token::Head("a")]),
            ("[a]\nk=v", vec![Token::Head("a"), Token::Pair("k", "v")]),
            (
                "[a]\nk=v ; test",
                vec![
                    Token::Head("a"),
                    Token::Pair("k", "v"),
                    Token::Comment("; test"),
                ],
            ),
            (
                "[a]\nk=v ; test\n[b]",
                vec![
                    Token::Head("a"),
                    Token::Pair("k", "v"),
                    Token::Comment("; test"),
                    Token::Head("b"),
                ],
            ),
            (
                "[a]\nk=v; test",
                vec![
                    Token::Head("a"),
                    Token::Pair("k", "v"),
                    Token::Comment("; test"),
                ],
            ),
            (
                "[a]\nk=v\n[b]",
                vec![Token::Head("a"), Token::Pair("k", "v"), Token::Head("b")],
            ),
            (
                "[a]\n# test\nk=v\n[b]",
                vec![
                    Token::Head("a"),
                    Token::Comment("# test"),
                    Token::Pair("k", "v"),
                    Token::Head("b"),
                ],
            ),
            (
                "[a]\n; test\nk=v\n[b]",
                vec![
                    Token::Head("a"),
                    Token::Comment("; test"),
                    Token::Pair("k", "v"),
                    Token::Head("b"),
                ],
            ),
            (
                "[a]\n# test\nk = v \n[b]",
                vec![
                    Token::Head("a"),
                    Token::Comment("# test"),
                    Token::Pair("k", "v"),
                    Token::Head("b"),
                ],
            ),
            (
                "[a\n# test\nk = v \n[b]",
                vec![
                    Token::Comment("# test"),
                    Token::Pair("k", "v"),
                    Token::Head("b"),
                ],
            ),
            (
                "[a\n# test\nk =  \n[b]",
                vec![
                    Token::Comment("# test"),
                    Token::Pair("k", ""),
                    Token::Head("b"),
                ],
            ),
            (
                "[a\n# test\nk = v \n[b]test",
                vec![
                    Token::Comment("# test"),
                    Token::Pair("k", "v"),
                    Token::Head("b"),
                ],
            ),
        ];

        // Act & Assert
        cases.into_iter().for_each(|(case, expected)| {
            println!("parse_test case: {}", case);
            let result = tokenize(case);
            assert_that!(result).is_equal_to(expected);
        });
    }

    #[test]
    fn parse_real_file() {
        // Arrange
        let s = r##"# Editor configuration, see http://editorconfig.org
root = true

[*]
charset = utf-8
indent_style = space
indent_size = 2
insert_final_newline = true
trim_trailing_whitespace = true : error

[*.md]
max_line_length = off
trim_trailing_whitespace = false
"##;

        // Act
        let result = tokenize(s);

        // Assert
        let expected = vec![
            Token::Comment("# Editor configuration, see http://editorconfig.org"),
            Token::Pair("root", "true"),
            Token::Head("*"),
            Token::Pair("charset", "utf-8"),
            Token::Pair("indent_style", "space"),
            Token::Pair("indent_size", "2"),
            Token::Pair("insert_final_newline", "true"),
            Token::Pair("trim_trailing_whitespace", "true : error"),
            Token::Head("*.md"),
            Token::Pair("max_line_length", "off"),
            Token::Pair("trim_trailing_whitespace", "false"),
        ];
        assert_that!(result).is_equal_to(expected);
    }
}
