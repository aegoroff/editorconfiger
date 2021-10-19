use nom::branch::alt;
use nom::bytes::complete::is_not;
use nom::character::complete::multispace0;
use nom::combinator::ParserIterator;
use nom::error::{ParseError, VerboseError};
use nom::sequence;
use nom::{character::complete, combinator, IResult, Parser};

#[derive(Debug, PartialEq)]
enum EditorConfigLine<'a> {
    Head(&'a str),
    Pair(&'a str, &'a str),
    Comment(&'a str),
}

fn parse_editorconfig<'a>(input: &'a str) -> Vec<EditorConfigLine<'a>> {
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
            result.push(line_parser(last).unwrap());
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
    let result: IResult<&'a str, &'a str, E> = s_expr(is_not("]"))(input);
    let head = match result {
        Ok((_trail, matched)) => Some(EditorConfigLine::Head(matched)),
        Err(_e) => None,
    };

    if let Some(EditorConfigLine::Head(_h)) = head {
        return head;
    }

    // Key/value line
    let result: IResult<&'a str, (&'a str, &'a str), E> = key_value(input);
    let kv = match result {
        Ok((_trail, (k, v))) => {
            let kt = trim_spaces::<E>(k);
            let vt = trim_spaces::<E>(v);
            if let Ok((_trail, kt)) = kt {
                if let Ok((_trail, vt)) = vt {
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

fn s_expr<'a, F, E>(inner: F) -> impl FnMut(&'a str) -> IResult<&'a str, &'a str, E>
where
    F: Parser<&'a str, &'a str, E>,
    E: ParseError<&'a str> + std::fmt::Debug,
{
    sequence::delimited(complete::char('['), inner, complete::char(']'))
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

fn trim_spaces<'a, E>(input: &'a str) -> IResult<&'a str, &'a str, E>
where
    E: ParseError<&'a str> + std::fmt::Debug,
{
    ws(is_not(" \t"))(input)
}

fn ws<'a, F, E>(inner: F) -> impl FnMut(&'a str) -> IResult<&'a str, &'a str, E>
where
    F: Parser<&'a str, &'a str, E>,
    E: ParseError<&'a str> + std::fmt::Debug,
{
    sequence::delimited(multispace0, inner, multispace0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use spectral::prelude::*;

    #[test]
    fn parse() {
        // Arrange
        let cases = vec![
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
            let result = parse_editorconfig(case);
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
trim_trailing_whitespace = true

[*.md]
max_line_length = off
trim_trailing_whitespace = false
"##;

        // Act
        let result = parse_editorconfig(s);

        // Assert
        let expected = vec![
            EditorConfigLine::Comment(" Editor configuration, see http://editorconfig.org"),
            EditorConfigLine::Pair("root", "true"),
            EditorConfigLine::Head("*"),
            EditorConfigLine::Pair("charset", "utf-8"),
            EditorConfigLine::Pair("indent_style", "space"),
            EditorConfigLine::Pair("indent_size", "2"),
            EditorConfigLine::Pair("insert_final_newline", "true"),
            EditorConfigLine::Pair("trim_trailing_whitespace", "true"),
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
        let (trail, trimmed) = trim_spaces::<VerboseError<&str>>(s).unwrap();

        // Assert
        assert_that!(trimmed).is_equal_to("test");
        assert_that!(trail).is_equal_to("");
    }

    #[test]
    #[ignore]
    fn trim_spaces_inside() {
        // Arrange
        let s = "  test test  ";

        // Act
        let (trail, trimmed) = trim_spaces::<VerboseError<&str>>(s).unwrap();

        // Assert
        assert_that!(trimmed).is_equal_to("test test");
        assert_that!(trail).is_equal_to("");
    }
}
