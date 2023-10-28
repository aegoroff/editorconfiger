lalrpop_mod!(
    #[allow(clippy::all)]
    #[allow(unused)]
    #[allow(clippy::unwrap_in_result)]
    #[allow(clippy::unwrap_used)]
    glob
);

/// Parses .editorconfig section title string and extracts all extensions into
/// Vec. Each extension as separate element if any
///
/// Examples:
///
/// ```
/// use editorconfiger::glob;
///
/// let result = glob::parse("*.{e1, e2}");
/// assert_eq!(2, result.len());
/// assert_eq!("*.e1", result[0]);
/// assert_eq!("*.e2", result[1]);
///
/// let result = glob::parse("*.[ch]");
/// assert_eq!(2, result.len());
/// assert_eq!("*.c", result[0]);
/// assert_eq!("*.h", result[1]);
///
/// let result = glob::parse("*");
/// assert_eq!(1, result.len());
/// assert_eq!("*", result[0]);
/// ```
#[must_use]
pub fn parse(string: &str) -> Vec<String> {
    let parser = glob::DefinesParser::new();
    return match parser.parse(string) {
        Ok(r) => r,
        Err(_e) => vec![],
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case("22", vec!["22"])]
    #[case("*.e1", vec!["*.e1"])]
    #[case("**.e1", vec!["**.e1"])]
    #[case("*.{e1}", vec!["*.e1"])]
    #[case("*.[ch]", vec!["*.c", "*.h"])]
    #[case("f.e1", vec!["f.e1"])]
    #[case("f1", vec!["f1"])]
    #[case("*.*", vec!["*.*"])]
    #[case("*", vec!["*"])]
    #[case("**", vec!["**"])]
    #[case("*.{e1,e2}", vec!["*.e1", "*.e2"])]
    #[case("*.{e1,e2,f1.e1}", vec!["*.e1", "*.e2", "*.f1.e1"])]
    #[case("{f1.e1,{f1.e2, f1.e3}}", vec!["f1.e1", "f1.e2", "f1.e3"])]
    #[case("{f1.e1,{f1.e2, {f1.e3, f1.e4}}}", vec!["f1.e1", "f1.e2", "f1.e3", "f1.e4"])]
    #[case("{f1.e1,{f1.e2, *.{f1.e3, f1.e4}}}", vec!["f1.e1", "f1.e2", "*.f1.e3", "*.f1.e4"])]
    #[case("{f1.e1,f1.[ch]}", vec!["f1.e1", "f1.c", "f1.h"])]
    #[case("{*.e1,*.e2}", vec!["*.e1", "*.e2"])]
    #[case("{f1,f2}.e1", vec!["f1.e1", "f2.e1"])]
    #[case("{f1,f2}", vec!["f1", "f2"])]
    #[case("{f1,.f2}", vec!["f1", ".f2"])]
    #[case("{f1.e1,*.e1}", vec!["f1.e1", "*.e1"])]
    #[case("{f1.e1,*.f1.e1}", vec!["f1.e1", "*.f1.e1"])]
    #[case("{f1.e1,.f1.e1}", vec!["f1.e1", ".f1.e1"])]
    #[case("test/*.{e1, e2}", vec!["test/*.e1", "test/*.e2"])]
    #[case("test/*", vec!["test/*"])]
    #[case("test/**/*", vec!["test/**/*"])]
    #[case("test/{p1,p2}/*", vec!["test/p1/*", "test/p2/*"])]
    #[trace]
    fn parse_cases(#[case] s: &str, #[case] expected: Vec<&str>) {
        // Act
        let actual = parse(s);
        let actual: Vec<&str> = actual.iter().map(|s| &**s).collect();

        // Assert
        assert_eq!(actual, expected);
    }
}
