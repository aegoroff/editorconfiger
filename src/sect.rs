use std::path::PathBuf;

lalrpop_mod!(
    #[allow(clippy::all)]
    #[allow(unused)]
    pub section
);

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
    fn parse_success() {
        // Arrange

        // Assert
        assert!(!parse("22").is_empty());
        assert!(!parse("*.e1").is_empty());
        assert!(!parse("**.e1").is_empty());
        assert!(!parse("*.{e1}").is_empty());
        assert!(!parse("*.[ch]").is_empty());
        assert!(!parse("f.e1").is_empty());
        assert!(!parse("f1").is_empty());
        assert!(!parse("*.*").is_empty());
        assert!(!parse("*").is_empty());
        assert!(!parse("**").is_empty());
        assert!(!parse("*.{e1,e2}").is_empty());
        assert!(!parse("*.{e1,e2,f1.e1}").is_empty());
        assert!(!parse("{*.e1,*.e2}").is_empty());
        assert!(!parse("{f1,f2}.e1").is_empty());
        assert!(!parse("{f1,f2}").is_empty());
        assert!(!parse("{f1,.f2}").is_empty());
        assert!(!parse("{f1.e1,*.e1}").is_empty());
        assert!(!parse("{f1.e1,*.f1.e1}").is_empty());
        assert!(!parse("{f1.e1,.f1.e1}").is_empty());
        assert!(!parse("test/*").is_empty());
        assert!(!parse("test/**/*").is_empty());
        assert!(!parse("test/{p1,p2}/*").is_empty());
    }

    #[test]
    fn parse_get_data_single() {
        // Arrange

        // Act
        let result = parse("*.{e1}");

        // Assert
        assert_eq!(1, result.len());
        assert_eq!("*.e1", result[0]);
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
    fn parse_path_get_data_many() {
        // Arrange

        // Act
        let result = parse("test/*.{e1, e2}");

        // Assert
        assert_eq!(2, result.len());
        assert_eq!("test/*.e1", result[0]);
        assert_eq!("test/*.e2", result[1]);
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

    #[test]
    fn parse_path_get_data_many_mixed() {
        // Arrange

        // Act
        let result = parse("{f1.e1,*.f1.e1}");

        // Assert
        assert_eq!(2, result.len());
        assert_eq!("f1.e1", result[0]);
        assert_eq!("*.f1.e1", result[1]);
    }

    #[test]
    fn parse_path_get_data_many_only_list_full_wild() {
        // Arrange

        // Act
        let result = parse("{*.e1,*.e2}");

        // Assert
        assert_eq!(2, result.len());
        assert_eq!("*.e1", result[0]);
        assert_eq!("*.e2", result[1]);
    }

    #[test]
    fn parse_path_get_data_many_only_list_fulls() {
        // Arrange

        // Act
        let result = parse("{f1.e1,.f1.e1}");

        // Assert
        assert_eq!(2, result.len());
        assert_eq!("f1.e1", result[0]);
        assert_eq!(".f1.e1", result[1]);
    }

    #[test]
    fn parse_path_get_data_many_composite_fulls() {
        // Arrange

        // Act
        let result = parse("{f1,f2}.e1");

        // Assert
        assert_eq!(2, result.len());
        assert_eq!("f1.e1", result[0]);
        assert_eq!("f2.e1", result[1]);
    }
}
