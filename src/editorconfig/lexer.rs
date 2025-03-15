use miette::{miette, LabeledSpan, Result, SourceSpan};
use nom::branch::alt;
use nom::bytes::complete::is_not;
use nom::error::{Error, FromExternalError, ParseError};
use nom::{character::complete, combinator, IResult};
use nom::{sequence, Parser};

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
pub fn tokenize(input: &str) -> impl Iterator<Item = Result<Token<'_>>> {
    TokenIterator::new(input)
}

struct TokenIterator<'a> {
    input: &'a str,
    not_parsed_trail: &'a str,
    offset: usize,
    original_input_len: usize,
}

impl<'a> TokenIterator<'a> {
    /// Creates a new `TokenIterator` to parse the given input string.
    fn new(input: &'a str) -> Self {
        Self {
            input,
            not_parsed_trail: "",
            offset: 0,
            original_input_len: input.len(),
        }
    }

    /// Parses a line of text and returns the appropriate token if successful.
    ///
    /// This method takes the remaining trail after parsing and updates the iterator's state accordingly.
    /// If no data to parse (val argument), it returns `None`.
    fn parse_line(&mut self, trail: &'a str, val: &'a str) -> Option<Result<Token<'a>>> {
        self.offset = self.original_input_len - trail.len() - val.len();
        if self.offset > 0 {
            self.offset -= 1;
        }
        self.input = trail;
        if val.is_empty() {
            return None;
        }
        let r = line::<'a, Error<&'a str>>(val);
        match r {
            Ok((remain, token)) => {
                self.not_parsed_trail = remain;
                Some(Ok(token))
            }
            Err(e) => {
                let msg = match e {
                    nom::Err::Incomplete(needed) => match needed {
                        nom::Needed::Unknown => "not enough data in input".to_owned(),
                        nom::Needed::Size(non_zero) => {
                            format!("not enough {non_zero} bytes in input")
                        }
                    },
                    nom::Err::Error(ref e) => {
                        format!("error occured with '{}' code", e.code.description())
                    }
                    nom::Err::Failure(ref f) => f.to_string(),
                };
                let span = SourceSpan::new(self.offset.into(), val.len());
                let report = miette!(
                    labels = vec![LabeledSpan::at(
                        span,
                        format!("The problem is here. Details: {msg}")
                    ),],
                    help = "Incorrect .editorconfig file syntax",
                    "Lexer error"
                );
                Some(Err(report))
            }
        }
    }
}

impl<'a> Iterator for TokenIterator<'a> {
    type Item = Result<Token<'a>>;

    fn next(&mut self) -> Option<Self::Item> {
        if !self.not_parsed_trail.is_empty() {
            let parsed_comment = comment::<'a, Error<&'a str>>(self.not_parsed_trail);

            self.not_parsed_trail = "";
            // if there were an error while parsing inline comment (for example it's not started from # or ;)
            // just throw it and continue parsing
            // It may be sensible to warn user about it. Should think over it.
            if let Ok((_, inline_comment)) = parsed_comment {
                return Some(Ok(inline_comment));
            }
        }

        // `self.input` will point to trail after each self.parse_line call
        // so we advance over input until EOF
        loop {
            if self.input.is_empty() {
                break;
            }
            let mut parser = sequence::terminated(complete::not_line_ending, complete::line_ending);
            let parsed: IResult<&'a str, &'a str, Error<&'a str>> = parser.parse(self.input);
            return if let Ok((trail, val)) = parsed {
                if let Some(token) = self.parse_line(trail, val) {
                    return Some(token);
                }
                continue;
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
    alt((head::<E>, key_value::<E>, comment::<E>)).parse(input)
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
    })
    .parse(input)
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
    })
    .parse(input)
}

fn comment<'a, E>(input: &'a str) -> IResult<&'a str, Token<'a>, E>
where
    E: ParseError<&'a str> + std::fmt::Debug,
{
    combinator::map(
        combinator::recognize(sequence::preceded(
            alt((complete::char('#'), complete::char(';'))),
            alt((is_not("\n\r"), combinator::eof)),
        )),
        Token::Comment,
    )
    .parse(input)
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
                "[a]\nk=v#",
                vec![Token::Head("a"), Token::Pair("k", "v"), Token::Comment("#")],
            ),
            (
                "[a]\nk=v#\n[b]\nk1=v1#",
                vec![
                    Token::Head("a"),
                    Token::Pair("k", "v"),
                    Token::Comment("#"),
                    Token::Head("b"),
                    Token::Pair("k1", "v1"),
                    Token::Comment("#"),
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
            ("#", vec![Token::Comment("#")]),
            ("# ", vec![Token::Comment("# ")]),
            ("# a", vec![Token::Comment("# a")]),
        ];

        // Act & Assert
        for (validator, input, expected) in table_test!(cases) {
            let actual: Vec<Token> = tokenize(input).filter_map(|t| t.ok()).collect();

            validator
                .given(input)
                .when("tokenize")
                .then(&format!("it should be {expected:?}"))
                .assert_eq(expected, actual);
        }
    }

    #[test]
    fn tokenize_real_file() {
        // Arrange
        let s = r#"# Editor configuration, see http://editorconfig.org
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
"#;

        // Act
        let result: Vec<Token> = tokenize(s).filter_map(|t| t.ok()).collect();

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
