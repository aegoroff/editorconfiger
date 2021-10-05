use nom::bytes::complete::{is_not};
use nom::error::VerboseError;
use nom::sequence::{delimited};
use nom::{character::complete, IResult, Parser};

fn parse_str(input: &str) -> IResult<&str, &str, VerboseError<&str>> {
    s_expr(is_not("[]"))(input)
}

fn s_expr<'a, O1, F>(inner: F) -> impl FnMut(&'a str) -> IResult<&'a str, O1, VerboseError<&'a str>>
where
    F: Parser<&'a str, O1, VerboseError<&'a str>>,
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
            ("[*]", "*"),
            ("[ * ]", " * "),
            ("[123]", "123"),
            ("[]", ""),
        ];

        // Act & Assert
        cases.into_iter().for_each(|case| {
            let result = parse_str(case.0);
            assert_that!(result.unwrap_or_default().1).is_equal_to(case.1);
        });
    }
}
