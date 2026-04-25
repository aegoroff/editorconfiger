use miette::{LabeledSpan, Result, SourceSpan, miette};
use winnow::Parser;
use winnow::ascii::{line_ending, till_line_ending};
use winnow::combinator::{alt, eof, preceded, separated_pair};
use winnow::token::{one_of, take_till};

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
        let mut line_input = val;
        let r = line.parse_next(&mut line_input);
        match r {
            Ok(token) => {
                self.not_parsed_trail = line_input;
                Some(Ok(token))
            }
            Err(e) => {
                let msg = e.to_string();
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
            let mut parsed_comment_input = self.not_parsed_trail;
            let parsed_comment = comment.parse_next(&mut parsed_comment_input);

            self.not_parsed_trail = "";
            // if there were an error while parsing inline comment (for example it's not started from # or ;)
            // just throw it and continue parsing
            // It may be sensible to warn user about it. Should think over it.
            if let Ok(inline_comment) = parsed_comment {
                return Some(Ok(inline_comment));
            }
        }

        // `self.input` will point to trail after each self.parse_line call
        // so we advance over input until EOF
        loop {
            if self.input.is_empty() {
                break;
            }
            let mut trail = self.input;
            let parsed = full_line.parse_next(&mut trail);
            return if let Ok(val) = parsed {
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

fn full_line<'a>(input: &mut &'a str) -> winnow::Result<&'a str> {
    winnow::combinator::terminated(till_line_ending, line_ending).parse_next(input)
}

fn line<'a>(input: &mut &'a str) -> winnow::Result<Token<'a>> {
    alt((head, key_value, comment)).parse_next(input)
}

fn head<'a>(input: &mut &'a str) -> winnow::Result<Token<'a>> {
    let val = preceded('[', take_till(1.., ['\n', '\r', ';', '#'])).parse_next(input)?;

    // capture data until last ] to support brackets inside section head
    match val.rfind(']') {
        Some(ix) => Ok(Token::Head(&val[..ix])),
        None => Err(winnow::error::ContextError::new()),
    }
}

fn key_value<'a>(input: &mut &'a str) -> winnow::Result<Token<'a>> {
    let (k, v): (&str, &str) = separated_pair(
        take_till(1.., ['=', ';', '#']),
        '=',
        alt((take_till(0.., [';', '#']), eof.value(""))),
    )
    .parse_next(input)?;

    Ok(Token::Pair(k.trim(), v.trim()))
}

fn comment<'a>(input: &mut &'a str) -> winnow::Result<Token<'a>> {
    let source = *input;
    let _ = preceded(
        one_of(['#', ';']),
        alt((take_till(0.., ['\n', '\r']), eof.value(""))),
    )
    .parse_next(input)?;
    let len = source.len() - input.len();
    Ok(Token::Comment(&source[..len]))
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_case::test_case;

    #[test_case("", vec![] ; "Empty input")]
    #[test_case("[*.md]", vec![Token::Head("*.md")] ; "Only head with glob")]
    #[test_case("[*.md] ; test",vec![Token::Head("*.md"), Token::Comment("; test")] ; "Only head and comment after")]
    #[test_case("[*.[md]]", vec![Token::Head("*.[md]")] ; "Only head with squares inside")]
    #[test_case("[*.[md]", vec![Token::Head("*.[md")] ; "Only head with single square inside")]
    #[test_case("[ *.[md] ]", vec![Token::Head(" *.[md] ")] ; "Only head with spaces and squares")]
    #[test_case("[a]\n[b]", vec![Token::Head("a"), Token::Head("b")] ; "Two heads")]
    #[test_case("[a]\r\n[b]", vec![Token::Head("a"), Token::Head("b")] ; "Two heads win")]
    #[test_case("[a]\n\n[b]", vec![Token::Head("a"), Token::Head("b")] ; "Two heads with empty line in between")]
    #[test_case("[a]", vec![Token::Head("a")] ; "Only head plain")]
    #[test_case("[a]\r\n", vec![Token::Head("a")] ; "Only head and carriage return")]
    #[test_case("[a]\nk=v", vec![Token::Head("a"), Token::Pair("k", "v")] ; "Single section with one pair")]
    #[test_case("[a]\nk=", vec![Token::Head("a"), Token::Pair("k", "")] ; "Single section with one pair without value")]
    #[test_case("[a]\nk= ", vec![Token::Head("a"), Token::Pair("k", "")] ; "Single section with one pair without value with space after eq")]
    #[test_case("[a]\nk=\n[b]", vec![Token::Head("a"), Token::Pair("k", ""), Token::Head("b")] ; "Two sections with one pair without value")]
    #[test_case(
        "[a]\nk=v ; test",
        vec![
            Token::Head("a"),
            Token::Pair("k", "v"),
            Token::Comment("; test"),
        ] ; "Inlined comment"
    )]
    #[test_case(
        "[a]\nk=v#",
        vec![Token::Head("a"), Token::Pair("k", "v"), Token::Comment("#")] ; "Inlined empty comment"
    )]
    #[test_case(
        "[a]\nk=v#\n[b]\nk1=v1#",
        vec![
            Token::Head("a"),
            Token::Pair("k", "v"),
            Token::Comment("#"),
            Token::Head("b"),
            Token::Pair("k1", "v1"),
            Token::Comment("#"),
        ] ; "Two sections with comments"
    )]
    #[test_case(
        "[a]\nk=v ; test\n[b]",
        vec![
            Token::Head("a"),
            Token::Pair("k", "v"),
            Token::Comment("; test"),
            Token::Head("b"),
        ] ; "Two sections with comments second empty"
    )]
    #[test_case(
        "[a]\nk=v; test",
        vec![
            Token::Head("a"),
            Token::Pair("k", "v"),
            Token::Comment("; test"),
        ] ; "Section with pair and inlined comment"
    )]
    #[test_case(
        "[a]\nk=v\n[b]",
        vec![Token::Head("a"), Token::Pair("k", "v"), Token::Head("b")] ; "Two sections without comments"
    )]
    #[test_case(
        "[a]\n# test\nk=v\n[b]",
        vec![
            Token::Head("a"),
            Token::Comment("# test"),
            Token::Pair("k", "v"),
            Token::Head("b"),
        ] ; "Two sections first starts with comment and second is empty"
    )]
    #[test_case(
        "[a]\n; test\nk=v\n[b]",
        vec![
            Token::Head("a"),
            Token::Comment("; test"),
            Token::Pair("k", "v"),
            Token::Head("b"),
        ] ; "Section first starts with comment that defined by semicolon and second is empty"
    )]
    #[test_case(
        "[a]\n# test\nk = v \n[b]",
        vec![
            Token::Head("a"),
            Token::Comment("# test"),
            Token::Pair("k", "v"),
            Token::Head("b"),
        ] ; "Section first starts with comment that defined by hash and second is empty"
    )]
    #[test_case(
        "[a\n# test\nk = v \n[b]",
        vec![
            Token::Comment("# test"),
            Token::Pair("k", "v"),
            Token::Head("b"),
        ] ; "Two sections first has invalid header and second is empty"
    )]
    #[test_case(
        "[a\n# test\nk =  \n[b]",
        vec![
            Token::Comment("# test"),
            Token::Pair("k", ""),
            Token::Head("b"),
        ] ; "Two sections first has invalid header, no value in pair and second section is empty"
    )]
    #[test_case(
        "[a\n# test\nk = v \n[b]test",
        vec![
            Token::Comment("# test"),
            Token::Pair("k", "v"),
            Token::Head("b"),
        ] ; "Two sections first has invalid header, first line is comment starts with hash and second section is empty"
    )]
    #[test_case("#", vec![Token::Comment("#")] ; "Only comment hash")]
    #[test_case("# ", vec![Token::Comment("# ")] ; "Only comment hash and space after")]
    #[test_case("# a", vec![Token::Comment("# a")] ; "Only comment line")]
    fn tokenizing(input: &str, expected: Vec<Token>) {
        // Act
        let actual: Vec<Token> = tokenize(input).filter_map(|t| t.ok()).collect();

        // Assert
        assert_eq!(expected, actual);
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
