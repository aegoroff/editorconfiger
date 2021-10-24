use nom::branch::alt;
use nom::bytes::complete::is_not;
use nom::combinator::ParserIterator;
use nom::error::{FromExternalError, ParseError, VerboseError};
use nom::sequence;
#[allow(unused)]
use nom::Parser;
use nom::{character::complete, combinator, IResult};

/// Represents .editorconfig lexical token abstraction that contain necessary data
#[derive(Debug, PartialEq, Copy, Clone)]
pub enum Token<'a> {
    Head(&'a str),
    Pair(&'a str, &'a str),
    Comment(&'a str),
}

pub fn tokenize<'a>(input: &'a str) -> Vec<Token<'a>> {
    let mut it = lines::<VerboseError<&'a str>>(input);
    let mut result: Vec<Token<'a>> = it
        .map(|x| line::<'a, VerboseError<&'a str>>(x))
        .filter(Result::is_ok)
        .map(Result::unwrap)
        .flat_map(|(trail, val)| parse_inline_comment::<'a, VerboseError<&'a str>>(trail, val))
        .collect();

    if let Ok((last, _)) = it.finish() {
        if !last.is_empty() {
            if let Ok((trail, val)) = line::<'a, VerboseError<&'a str>>(last) {
                let it = parse_inline_comment::<'a, VerboseError<&'a str>>(trail, val);
                result.extend(it);
            }
        }
    };

    result
}

struct TokenWithCommentIterator<'a> {
    token: Token<'a>,
    comment: Option<Token<'a>>,
    count: i32,
}

impl<'a> TokenWithCommentIterator<'a> {
    fn new(token: Token<'a>, comment: Option<Token<'a>>) -> Self {
        Self {
            token,
            comment,
            count: 0,
        }
    }
}

impl<'a> Iterator for TokenWithCommentIterator<'a> {
    type Item = Token<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        self.count += 1;
        match self.count {
            1 => Some(self.token),
            2 => self.comment,
            _ => None,
        }
    }
}

fn parse_inline_comment<'a, E>(trail: &'a str, token: Token<'a>) -> impl Iterator<Item = Token<'a>>
where
    E: ParseError<&'a str> + std::fmt::Debug + FromExternalError<&'a str, nom::Err<char>>,
{
    if !trail.is_empty() {
        if let Ok((_, inline)) = comment::<'a, VerboseError<&'a str>>(trail) {
            return TokenWithCommentIterator::new(token, Some(inline));
        }
    }
    TokenWithCommentIterator::new(token, None)
}

fn lines<'a, E>(
    input: &'a str,
) -> ParserIterator<&'a str, E, impl FnMut(&'a str) -> Result<(&str, &str), nom::Err<E>>>
where
    E: ParseError<&'a str> + std::fmt::Debug,
{
    combinator::iterator(
        input,
        sequence::terminated(complete::not_line_ending, complete::line_ending),
    )
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
