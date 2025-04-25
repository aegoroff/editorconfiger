lalrpop_mod!(
    #[allow(clippy::all)]
    #[allow(unused)]
    #[allow(clippy::unwrap_in_result)]
    #[allow(clippy::unwrap_used)]
    #[allow(clippy::no_effect_underscore_binding)]
    #[allow(clippy::trivially_copy_pass_by_ref)]
    #[allow(clippy::cloned_instead_of_copied)]
    #[allow(clippy::cast_sign_loss)]
    #[allow(clippy::too_many_lines)]
    #[allow(clippy::match_same_arms)]
    #[allow(clippy::uninlined_format_args)]
    #[allow(clippy::unused_self)]
    #[allow(clippy::needless_pass_by_value)]
    #[allow(clippy::unnested_or_patterns)]
    #[allow(clippy::needless_raw_string_hashes)]
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
    parser.parse(string).unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_case::test_case;

    #[test_case("22", vec!["22"]; "case 1")]
    #[test_case("*.e1", vec!["*.e1"]; "case 2")]
    #[test_case("**.e1", vec!["**.e1"]; "case 3")]
    #[test_case("*.{e1}", vec!["*.e1"]; "case 4")]
    #[test_case("*.[ch]", vec!["*.c", "*.h"]; "case 5")]
    #[test_case("f.e1", vec!["f.e1"]; "case 6")]
    #[test_case("f1", vec!["f1"]; "case 7")]
    #[test_case("*.*", vec!["*.*"]; "case 8")]
    #[test_case("*", vec!["*"]; "case 9")]
    #[test_case("**", vec!["**"]; "case 10")]
    #[test_case("*.{e1,e2}", vec!["*.e1", "*.e2"]; "case 11")]
    #[test_case("*.{e1,e2,f1.e1}", vec!["*.e1", "*.e2", "*.f1.e1"]; "case 12")]
    #[test_case("{f1.e1,{f1.e2, f1.e3}}", vec!["f1.e1", "f1.e2", "f1.e3"]; "case 13")]
    #[test_case("{f1.e1,{f1.e2, {f1.e3, f1.e4}}}", vec!["f1.e1", "f1.e2", "f1.e3", "f1.e4"]; "case 14")]
    #[test_case("{f1.e1,{f1.e2, *.{f1.e3, f1.e4}}}", vec!["f1.e1", "f1.e2", "*.f1.e3", "*.f1.e4"]; "case 15")]
    #[test_case("{f1.e1,f1.[ch]}", vec!["f1.e1", "f1.c", "f1.h"]; "case 16")]
    #[test_case("{*.e1,*.e2}", vec!["*.e1", "*.e2"]; "case 17")]
    #[test_case("{f1,f2}.e1", vec!["f1.e1", "f2.e1"]; "case 18")]
    #[test_case("{f1,f2}", vec!["f1", "f2"]; "case 19")]
    #[test_case("{f1,.f2}", vec!["f1", ".f2"]; "case 20")]
    #[test_case("{f1.e1,*.e1}", vec!["f1.e1", "*.e1"]; "case 21")]
    #[test_case("{f1.e1,*.f1.e1}", vec!["f1.e1", "*.f1.e1"]; "case 22")]
    #[test_case("{f1.e1,.f1.e1}", vec!["f1.e1", ".f1.e1"]; "case 23")]
    #[test_case("test/*.{e1, e2}", vec!["test/*.e1", "test/*.e2"]; "case 24")]
    #[test_case("test/*", vec!["test/*"]; "case 25")]
    #[test_case("test/**/*", vec!["test/**/*"]; "case 26")]
    #[test_case("test/{p1,p2}/*", vec!["test/p1/*", "test/p2/*"]; "case 27")]
    fn parse_cases(input_str: &str, expected: Vec<&str>) {
        // Act
        let actual = parse(input_str);
        let actual: Vec<&str> = actual.iter().map(|s| &**s).collect();

        // Assert
        assert_eq!(actual, expected);
    }
}
