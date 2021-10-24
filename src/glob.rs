use std::path::PathBuf;

lalrpop_mod!(
    #[allow(clippy::all)]
    #[allow(unused)]
    pub section
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
pub fn parse(string: &str) -> Vec<String> {
    let path = PathBuf::from(string);
    let file = path.file_name().unwrap_or_default().to_str().unwrap_or("*");
    let dir = path.parent();
    if let Some(dir) = dir {
        if let Some(dir) = dir.to_str() {
            parse_file(file)
                .into_iter()
                .map(|s| {
                    let mut p = PathBuf::from(dir);
                    p.push(s);
                    String::from(p.to_str().unwrap())
                })
                .collect()
        } else {
            parse_file(file)
        }
    } else {
        parse_file(file)
    }
}

fn parse_file(file: &str) -> Vec<String> {
    let parser = section::DefinesParser::new();
    return match parser.parse(file) {
        Ok(r) => r,
        Err(_e) => vec![],
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_tests() {
        // Arrange
        let cases = vec![
            ("22", vec!["22"]),
            ("*.e1", vec!["*.e1"]),
            ("**.e1", vec!["*.e1"]),
            ("*.{e1}", vec!["*.e1"]),
            ("*.[ch]", vec!["*.c", "*.h"]),
            ("f.e1", vec!["f.e1"]),
            ("f1", vec!["f1"]),
            ("*.*", vec!["*"]),
            ("*", vec!["*"]),
            ("**", vec!["*"]),
            ("*.{e1,e2}", vec!["*.e1", "*.e2"]),
            ("*.{e1,e2,f1.e1}", vec!["*.e1", "*.e2", "*.f1.e1"]),
            ("{f1.e1,{f1.e2, f1.e3}}", vec!["f1.e1", "f1.e2", "f1.e3"]),
            ("{f1.e1,f1.[ch]}", vec!["f1.e1", "f1.c", "f1.h"]),
            ("{*.e1,*.e2}", vec!["*.e1", "*.e2"]),
            ("{f1,f2}.e1", vec!["f1.e1", "f2.e1"]),
            ("{f1,f2}", vec!["f1", "f2"]),
            ("{f1,.f2}", vec!["f1", ".f2"]),
            ("{f1.e1,*.e1}", vec!["f1.e1", "*.e1"]),
            ("{f1.e1,*.f1.e1}", vec!["f1.e1", "*.f1.e1"]),
            ("{f1.e1,.f1.e1}", vec!["f1.e1", ".f1.e1"]),
            ("test/*.{e1, e2}", vec!["test/*.e1", "test/*.e2"]),
            ("test/*", vec!["test/*"]),
            ("test/**/*", vec!["test/**/*"]),
            ("test/{p1,p2}/*", vec!["test/p1/*", "test/p2/*"]),
        ];

        // Act & Assert
        for (validator, input, expected) in table_test!(cases) {
            let actual = parse(input);
            let actual = actual.iter().map(|s| &**s).collect();

            validator
                .given(&format!("{}", input))
                .when("parse")
                .then(&format!("it should be {:#?}", expected))
                .assert_eq(expected, actual);
        }
    }
}
