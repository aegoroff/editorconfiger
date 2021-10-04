use nom::{combinator::iterator, IResult, character::complete, sequence::terminated};

fn parse_str(input: &str) -> Vec<&str> {
    let mut it = iterator(input, terminated(complete::not_line_ending, complete::line_ending));
    let mut parsed : Vec<&str> = it.collect();
    let res: IResult<_,_> = it.finish();
    parsed.push(res.unwrap_or_default().0);
    parsed
}

#[cfg(test)]
mod tests {
    use super::*;
    use spectral::prelude::*;

    #[test]
    fn parse() {
        // Arrange
        let config = r###"
[*]
a = b
c = d"###;

        // Act
        let result = parse_str(config);

        // Assert
        assert_that!(result).has_length(4);
    }

}
