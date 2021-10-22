use nom::branch::alt;
use nom::bytes::complete::is_not;
use nom::character::complete::multispace0;
use nom::combinator::ParserIterator;
use nom::error::{ParseError, VerboseError};
use nom::sequence;
use nom::{character::complete, combinator, IResult};

#[derive(Debug, PartialEq)]
pub enum EditorConfigLine<'a> {
    Head(&'a str),
    Pair(&'a str, &'a str),
    Comment(&'a str),
}

pub fn parse<'a>(input: &'a str) -> Vec<EditorConfigLine<'a>> {
    parse_str(input, line::<VerboseError<&'a str>>)
}

fn parse_str<'a>(
    input: &'a str,
    mut line_parser: impl FnMut(&'a str) -> Option<EditorConfigLine<'a>>,
) -> Vec<EditorConfigLine<'a>> {
    let mut it = lines::<VerboseError<&str>>(input);
    let lit = it.filter_map(|x| line_parser(x));
    let mut result: Vec<EditorConfigLine<'a>> = lit.collect();

    if let Ok((last, _)) = it.finish() {
        if !last.is_empty() {
            if let Some(line) = line_parser(last) {
                result.push(line);
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

fn line<'a, E>(input: &'a str) -> Option<EditorConfigLine<'a>>
where
    E: ParseError<&'a str> + std::fmt::Debug,
{
    // Section head
    if let Some(matched) = head::<E>(input) {
        return Some(EditorConfigLine::Head(matched));
    }

    // Key/value line
    let result: IResult<&'a str, (&'a str, &'a str), E> = key_value(input);
    let kv = match result {
        Ok((_trail, (k, v))) => {
            let kt = trim_spaces::<E>(k);
            let vt = trim_spaces::<E>(v);
            if let Some(kt) = kt {
                if let Some(vt) = vt {
                    return Some(EditorConfigLine::Pair(kt, vt));
                }
            }
            None
        }
        Err(_e) => None,
    };

    if let Some(EditorConfigLine::Pair(_k, _v)) = kv {
        return kv;
    }

    // Comment
    let result: IResult<&'a str, &'a str, E> = comment(input);
    let c = match result {
        Ok((_trail, c)) => Some(EditorConfigLine::Comment(c)),
        Err(_e) => None,
    };

    if let Some(EditorConfigLine::Comment(_c)) = c {
        return c;
    }

    None
}

fn head<'a, E>(input: &'a str) -> Option<&'a str>
where
    E: ParseError<&'a str> + std::fmt::Debug,
{
    let mut action = sequence::preceded(complete::char('['), is_not("\n\r"));
    let parsed: IResult<&str, &str, E> = action(input);
    if let Ok((_trail, head)) = parsed {
        let rix = head.rfind(']');
        if let Some(rix) = rix {
            return Some(&head[..rix]);
        }
    }
    None
}

fn key_value<'a, E>(input: &'a str) -> IResult<&'a str, (&'a str, &'a str), E>
where
    E: ParseError<&'a str> + std::fmt::Debug,
{
    let mut action = sequence::separated_pair(is_not("=;#"), complete::char('='), is_not("=;#"));
    action(input)
}

fn comment<'a, E>(input: &'a str) -> IResult<&'a str, &'a str, E>
where
    E: ParseError<&'a str> + std::fmt::Debug,
{
    let mut action = sequence::preceded(
        alt((complete::char('#'), complete::char(';'))),
        is_not("\n\r"),
    );
    action(input)
}

fn trim_spaces<'a, E>(input: &'a str) -> Option<&'a str>
where
    E: ParseError<&'a str> + std::fmt::Debug,
{
    let mut action = sequence::preceded(multispace0, is_not("\n\r"));
    let parsed: IResult<&str, &str, E> = action(input);

    if let Ok((_trail, val)) = parsed {
        return Some(val.trim_end());
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use spectral::prelude::*;

    #[test]
    fn parse_test() {
        // Arrange
        let cases = vec![
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
                    EditorConfigLine::Comment(" test"),
                    EditorConfigLine::Pair("k", "v"),
                    EditorConfigLine::Head("b"),
                ],
            ),
            (
                "[a]\n; test\nk=v\n[b]",
                vec![
                    EditorConfigLine::Head("a"),
                    EditorConfigLine::Comment(" test"),
                    EditorConfigLine::Pair("k", "v"),
                    EditorConfigLine::Head("b"),
                ],
            ),
            (
                "[a]\n# test\nk = v \n[b]",
                vec![
                    EditorConfigLine::Head("a"),
                    EditorConfigLine::Comment(" test"),
                    EditorConfigLine::Pair("k", "v"),
                    EditorConfigLine::Head("b"),
                ],
            ),
            (
                "[a\n# test\nk = v \n[b]",
                vec![
                    EditorConfigLine::Comment(" test"),
                    EditorConfigLine::Pair("k", "v"),
                    EditorConfigLine::Head("b"),
                ],
            ),
            (
                "[a\n# test\nk =  \n[b]",
                vec![
                    EditorConfigLine::Comment(" test"),
                    EditorConfigLine::Head("b"),
                ],
            ),
        ];

        // Act & Assert
        cases.into_iter().for_each(|(case, expected)| {
            let result = parse(case);
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
        let result = parse(s);

        // Assert
        let expected = vec![
            EditorConfigLine::Comment(" Editor configuration, see http://editorconfig.org"),
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

    #[test]
    fn trim() {
        // Arrange
        let s = "  test  ";

        // Act
        let trimmed = trim_spaces::<VerboseError<&str>>(s).unwrap();

        // Assert
        assert_that!(trimmed).is_equal_to("test");
    }

    #[test]
    fn trim_spaces_inside() {
        // Arrange
        let s = "  test test  ";

        // Act
        let trimmed = trim_spaces::<VerboseError<&str>>(s).unwrap();

        // Assert
        assert_that!(trimmed).is_equal_to("test test");
    }
}
