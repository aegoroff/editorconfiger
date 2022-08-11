use nom::branch::alt;
use nom::bytes::complete::is_not;
use nom::error::{FromExternalError, ParseError, VerboseError};
use nom::sequence;
use nom::{character::complete, combinator, IResult};

/// Represents .editorconfig lexical token abstraction that contain necessary data
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum Token<'a> {
    /// Section head
    Head(&'a str),
    /// Key/value pair
    Pair(&'a str, &'a str),
    /// Comment including inline comments in head or key/value lines
    Comment(&'a str),
}

/// Splits input into tokens
pub fn tokenize(input: &str) -> impl Iterator<Item = Token> {
    TokenIterator::new(input)
}

struct TokenIterator<'a> {
    input: &'a str,
    not_parsed_trail: &'a str,
}

impl<'a> TokenIterator<'a> {
    fn new(input: &'a str) -> Self {
        Self {
            input,
            not_parsed_trail: "",
        }
    }

    fn parse_line(&mut self, trail: &'a str, val: &'a str) -> Option<Token<'a>> {
        let parsed_line = line::<'a, VerboseError<&'a str>>(val);
        self.input = trail;
        if let Ok((remain, token)) = parsed_line {
            // not parsed trail considered as inline comment
            // and will be parsed later
            self.not_parsed_trail = remain;
            Some(token)
        } else {
            None
        }
    }
}

impl<'a> Iterator for TokenIterator<'a> {
    type Item = Token<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if !self.not_parsed_trail.is_empty() {
            let parsed_comment = comment::<'a, VerboseError<&'a str>>(self.not_parsed_trail);
            self.not_parsed_trail = "";
            // if there were an error while parsing inline comment (for example it's not started from # or ;)
            // just throw it and continue parsing
            // It may be sensible to warn user about it. Should think over it.
            if let Ok((_, inline_comment)) = parsed_comment {
                return Some(inline_comment);
            }
        }

        // self.input will point to trail after each self.parse_line call
        // so we advance over input until EOF
        loop {
            if self.input.is_empty() {
                break;
            }
            let mut parser = sequence::terminated(complete::not_line_ending, complete::line_ending);
            let parsed: IResult<&'a str, &'a str, VerboseError<&'a str>> = parser(self.input);
            return if let Ok((trail, val)) = parsed {
                if let Some(token) = self.parse_line(trail, val) {
                    return Some(token);
                } else {
                    continue;
                }
            } else {
                self.parse_line("", self.input)
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
    const COMMENT_START_AND_SEPARATOR_CHARS: &str = "=;#";
    let parser = sequence::separated_pair(
        is_not(COMMENT_START_AND_SEPARATOR_CHARS),
        complete::char('='),
        is_not(COMMENT_START_AND_SEPARATOR_CHARS),
    );

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
        Token::Comment,
    )(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tokenize_test() {
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
        for (validator, input, expected) in table_test!(cases) {
            let actual: Vec<Token> = tokenize(input).collect();

            validator
                .given(&format!("{}", input))
                .when("tokenize")
                .then(&format!("it should be {:#?}", expected))
                .assert_eq(expected, actual);
        }
    }

    #[test]
    fn tokenize_real_file() {
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
        let result: Vec<Token> = tokenize(s).collect();

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
        assert_eq!(result, expected);
    }
}
