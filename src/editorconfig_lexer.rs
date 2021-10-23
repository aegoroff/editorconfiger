use nom::branch::alt;
use nom::bytes::complete::is_not;
use nom::combinator::ParserIterator;
use nom::error::{FromExternalError, ParseError, VerboseError};
use nom::sequence;
#[allow(unused)]
use nom::Parser;
use nom::{character::complete, combinator, IResult};

#[derive(Debug, PartialEq)]
pub enum EditorConfigLine<'a> {
    Head(&'a str),
    Pair(&'a str, &'a str),
    Comment(&'a str),
}

pub fn tokenize<'a>(input: &'a str) -> Vec<EditorConfigLine<'a>> {
    let mut it = lines::<VerboseError<&'a str>>(input);
    let mut result: Vec<EditorConfigLine<'a>> = it
        .map(|x| line::<'a, VerboseError<&'a str>>(x))
        .filter(Result::is_ok)
        .map(Result::unwrap)
        .map(|(_trail, val)| val)
        .collect();

    if let Ok((last, _)) = it.finish() {
        if !last.is_empty() {
            if let Ok((_, val)) = line::<'a, VerboseError<&'a str>>(last) {
                result.push(val);
            }
        }
    };

    result
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

fn line<'a, E>(input: &'a str) -> IResult<&'a str, EditorConfigLine<'a>, E>
where
    E: ParseError<&'a str> + std::fmt::Debug + FromExternalError<&'a str, nom::Err<char>>,
{
    alt((head::<E>, key_value::<E>, comment::<E>))(input)
}

fn head<'a, E>(input: &'a str) -> IResult<&'a str, EditorConfigLine<'a>, E>
where
    E: ParseError<&'a str> + std::fmt::Debug + FromExternalError<&'a str, nom::Err<char>>,
{
    let parser = sequence::preceded(complete::char('['), is_not("\n\r"));

    //  capture data until last ] to support brackets inside section head
    combinator::map_res(parser, |val: &str| match val.rfind(']') {
        Some(ix) => Ok(EditorConfigLine::Head(&val[..ix])),
        None => Err(nom::Err::Failure(']')),
    })(input)
}

fn key_value<'a, E>(input: &'a str) -> IResult<&'a str, EditorConfigLine<'a>, E>
where
    E: ParseError<&'a str> + std::fmt::Debug,
{
    let parser = sequence::separated_pair(is_not("=;#"), complete::char('='), is_not("=;#"));

    combinator::map(parser, |(k, v): (&str, &str)| {
        EditorConfigLine::Pair(k.trim(), v.trim())
    })(input)
}

fn comment<'a, E>(input: &'a str) -> IResult<&'a str, EditorConfigLine<'a>, E>
where
    E: ParseError<&'a str> + std::fmt::Debug,
{
    combinator::map(
        combinator::recognize(sequence::preceded(
            alt((complete::char('#'), complete::char(';'))),
            is_not("\n\r"),
        )),
        |val: &str| EditorConfigLine::Comment(val),
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
            ("[*.md]", vec![EditorConfigLine::Head("*.md")]),
            ("[*.[md]]", vec![EditorConfigLine::Head("*.[md]")]),
            ("[*.[md]", vec![EditorConfigLine::Head("*.[md")]),
            ("[ *.[md] ]", vec![EditorConfigLine::Head(" *.[md] ")]),
            (
                "[a]\n[b]",
                vec![EditorConfigLine::Head("a"), EditorConfigLine::Head("b")],
            ),
            (
                "[a]\r\n[b]",
                vec![EditorConfigLine::Head("a"), EditorConfigLine::Head("b")],
            ),
            (
                "[a]\n\n[b]",
                vec![EditorConfigLine::Head("a"), EditorConfigLine::Head("b")],
            ),
            ("[a]", vec![EditorConfigLine::Head("a")]),
            ("[a]\r\n", vec![EditorConfigLine::Head("a")]),
            (
                "[a]\nk=v",
                vec![
                    EditorConfigLine::Head("a"),
                    EditorConfigLine::Pair("k", "v"),
                ],
            ),
            (
                "[a]\nk=v ; test",
                vec![
                    EditorConfigLine::Head("a"),
                    EditorConfigLine::Pair("k", "v"),
                ],
            ),
            (
                "[a]\nk=v; test",
                vec![
                    EditorConfigLine::Head("a"),
                    EditorConfigLine::Pair("k", "v"),
                ],
            ),
            (
                "[a]\nk=v\n[b]",
                vec![
                    EditorConfigLine::Head("a"),
                    EditorConfigLine::Pair("k", "v"),
                    EditorConfigLine::Head("b"),
                ],
            ),
            (
                "[a]\n# test\nk=v\n[b]",
                vec![
                    EditorConfigLine::Head("a"),
                    EditorConfigLine::Comment("# test"),
                    EditorConfigLine::Pair("k", "v"),
                    EditorConfigLine::Head("b"),
                ],
            ),
            (
                "[a]\n; test\nk=v\n[b]",
                vec![
                    EditorConfigLine::Head("a"),
                    EditorConfigLine::Comment("; test"),
                    EditorConfigLine::Pair("k", "v"),
                    EditorConfigLine::Head("b"),
                ],
            ),
            (
                "[a]\n# test\nk = v \n[b]",
                vec![
                    EditorConfigLine::Head("a"),
                    EditorConfigLine::Comment("# test"),
                    EditorConfigLine::Pair("k", "v"),
                    EditorConfigLine::Head("b"),
                ],
            ),
            (
                "[a\n# test\nk = v \n[b]",
                vec![
                    EditorConfigLine::Comment("# test"),
                    EditorConfigLine::Pair("k", "v"),
                    EditorConfigLine::Head("b"),
                ],
            ),
            (
                "[a\n# test\nk =  \n[b]",
                vec![
                    EditorConfigLine::Comment("# test"),
                    EditorConfigLine::Pair("k", ""),
                    EditorConfigLine::Head("b"),
                ],
            ),
            (
                "[a\n# test\nk = v \n[b]test",
                vec![
                    EditorConfigLine::Comment("# test"),
                    EditorConfigLine::Pair("k", "v"),
                    EditorConfigLine::Head("b"),
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
            EditorConfigLine::Comment("# Editor configuration, see http://editorconfig.org"),
            EditorConfigLine::Pair("root", "true"),
            EditorConfigLine::Head("*"),
            EditorConfigLine::Pair("charset", "utf-8"),
            EditorConfigLine::Pair("indent_style", "space"),
            EditorConfigLine::Pair("indent_size", "2"),
            EditorConfigLine::Pair("insert_final_newline", "true"),
            EditorConfigLine::Pair("trim_trailing_whitespace", "true : error"),
            EditorConfigLine::Head("*.md"),
            EditorConfigLine::Pair("max_line_length", "off"),
            EditorConfigLine::Pair("trim_trailing_whitespace", "false"),
        ];
        assert_that!(result).is_equal_to(expected);
    }
}
