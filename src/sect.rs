
/*
Variants
    *.e1
    **.std
    *.{e1}
    **.[e1]
    *.{e1,e2}
    *.{e1,e2,f1.e1}
    {*.e1,*.e2}
    {f1,f2}.e1
    {f1,f2}
    {f1,.f2}
    {f1.e1,*.e1}
    f.e1
    f1
    *.*
    *
*/

use lalrpop_util::lexer::Token;

lalrpop_mod!(
    #[allow(clippy::all)]
    #[allow(unused)]
    pub section
);

pub fn parse(string: &str) -> Vec<String> {
    let parser = section::DefinesParser::new();
    let result : Result<Vec<String>, lalrpop_util::ParseError<usize, Token<'_>, &'static str>> = parser.parse(string);
    if result.is_ok() {
        result.unwrap()
    } else {
        println!("string:{} error: {}", string, result.unwrap_err());
        vec![]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_success() {
        // Arrange
        let parser = section::DefinesParser::new();

        // Assert
        assert!(parser.parse("22").is_ok());
        assert!(parser.parse("*.e1").is_ok());
        assert!(parser.parse("**.e1").is_ok());
        assert!(parser.parse("*.{e1}").is_ok());
        assert!(parser.parse("*.[ch]").is_ok());
        assert!(parser.parse("f.e1").is_ok());
        assert!(parser.parse("f1").is_ok());
        assert!(parser.parse("*.*").is_ok());
        assert!(parser.parse("*").is_ok());
        assert!(parser.parse("**").is_ok());
        assert!(parser.parse("*.{e1,e2}").is_ok());
        assert!(parser.parse("*.{e1,e2,f1.e1}").is_ok());
        assert!(parser.parse("{*.e1,*.e2}").is_ok());
        assert!(parser.parse("{f1,f2}.e1").is_ok());
        assert!(parser.parse("{f1,f2}").is_ok());
        assert!(parser.parse("{f1,.f2}").is_ok());
        assert!(parser.parse("{f1.e1,*.e1}").is_ok());
        assert!(parser.parse("{f1.e1,*.f1.e1}").is_ok());
    }

    #[test]
    fn parse_get_data_single() {
        // Arrange
        let parser = section::DefinesParser::new();

        // Act
        let result : Result<Vec<String>, lalrpop_util::ParseError<usize, Token<'_>, &'static str>> = parser.parse("*.{e1}");

        // Assert
        let data = result.unwrap();
        assert_eq!(1, data.len());
        assert_eq!("*.e1", data[0]);
    }

    #[test]
    fn parse_get_data_many() {
        // Arrange

        // Act
        let result = parse("*.{e1, e2}");

        // Assert
        assert_eq!(2, result.len());
        assert_eq!("*.e1", result[0]);
        assert_eq!("*.e2", result[1]);
    }

    #[test]
    fn parse_get_data_many_ext() {
        // Arrange

        // Act
        let result = parse("*.[ch]");

        // Assert
        assert_eq!(2, result.len());
        assert_eq!("*.c", result[0]);
        assert_eq!("*.h", result[1]);
    }
}
