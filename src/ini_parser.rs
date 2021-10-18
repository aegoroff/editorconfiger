use nom::bytes::complete::is_not;
use nom::combinator::ParserIterator;
use nom::error::VerboseError;
use nom::sequence::{delimited, terminated};
use nom::{character::complete, combinator, IResult, Parser};

enum IniLine<'a> {
    Head(&'a str),
    Line(&'a str, &'a str),
}

fn parse_ini<'a>(input: &'a str) -> Vec<IniLine<'a>> {
    parse_str(input, line::<VerboseError<&'a str>>)
}

fn parse_str<'a>(
    input: &'a str,
    mut line_parser: impl FnMut(&'a str) -> Option<IniLine<'a>>,
) -> Vec<IniLine<'a>> {
    let mut it = lines::<VerboseError<&str>>(input);
    let mut result: Vec<IniLine<'a>> = it.filter_map(|x| line_parser(x)).collect();
    let r: IResult<_, _, _> = it.finish();
    let last = r.unwrap().0;
    if last.len() > 0 {
        result.push(line_parser(last).unwrap());
    }
    result
}

fn lines<'a, E>(
    input: &'a str,
) -> ParserIterator<&'a str, E, impl FnMut(&'a str) -> Result<(&str, &str), nom::Err<E>>>
where
    E: nom::error::ParseError<&'a str> + std::fmt::Debug,
{
    combinator::iterator(
        input,
        terminated(complete::not_line_ending, complete::line_ending),
    )
}

fn line<'a, E>(input: &'a str) -> Option<IniLine<'a>>
where
    E: nom::error::ParseError<&'a str> + std::fmt::Debug,
{
    let result: IResult<&'a str, &'a str, E> = s_expr(is_not("]"))(input);
    match result {
        Ok((_trail, matched)) => Some(IniLine::Head(matched)),
        Err(e) => None,
    }
}

fn s_expr<'a, F, E>(inner: F) -> impl FnMut(&'a str) -> IResult<&'a str, &'a str, E>
where
    F: Parser<&'a str, &'a str, E>,
    E: nom::error::ParseError<&'a str> + std::fmt::Debug,
{
    delimited(complete::char('['), inner, complete::char(']'))
}

#[cfg(test)]
mod tests {
    use super::*;
    use spectral::prelude::*;

    #[test]
    fn parse() {
        // Arrange
        let cases = vec![
            ("[a]\n[b]", vec![IniLine::Head("a"), IniLine::Head("b")]),
            ("[a]\r\n[b]", vec![IniLine::Head("a"), IniLine::Head("b")]),
            ("[a]\n\n[b]", vec![IniLine::Head("a"), IniLine::Head("b")]),
            ("[a]", vec![IniLine::Head("a")]),
            ("[a]\r\n", vec![IniLine::Head("a")]),
        ];

        // Act & Assert
        cases.into_iter().for_each(|case| {
            let result = parse_ini(case.0);
            assert_that!(result).has_length(case.1.len());
        });
    }
}
