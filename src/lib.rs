extern crate ini;
extern crate jwalk;

use ini::Ini;
use jwalk::WalkDir;
use std::collections::{BTreeMap, HashMap};

pub type AnyError = Box<dyn std::error::Error>;

const EDITOR_CONFIG: &str = ".editorconfig";

pub struct ValidationResult {
    pub duplicate_sections: Vec<String>,
    pub duplicate_properties: HashMap<String, Vec<String>>,
}

pub fn validate_all(path: &str) -> BTreeMap<String, ValidationResult> {
    let iter = WalkDir::new(path).skip_hidden(false).follow_links(false);
    let results: BTreeMap<String, ValidationResult> = iter
        .into_iter()
        .filter(Result::is_ok)
        .map(Result::unwrap)
        .filter(|f| f.file_type().is_file())
        .map(|f| f.path().to_str().unwrap_or("").to_string())
        .filter(|p| p.ends_with(EDITOR_CONFIG))
        .map(|p| (String::from(p.as_str()), validate_one(p.as_str())))
        .filter(|(_, res)| res.is_ok())
        .map(|(p, res)| (p, res.unwrap()))
        .collect();
    results
}

pub fn validate_one(path: &str) -> Result<ValidationResult, AnyError> {
    let conf = Ini::load_from_file(path)?;
    Ok(validate(conf))
}

fn validate(conf: Ini) -> ValidationResult {
    let mut sect_count = HashMap::new();
    let mut dup_props = HashMap::new();
    for (sec, prop) in &conf {
        let sk = sec.unwrap_or("root");
        *sect_count.entry(sk).or_insert(0) += 1;

        let mut duplicate_pops: Vec<String> = prop
            .iter()
            .map(|(k, _)| k)
            .fold(HashMap::new(), |mut h, s| {
                *h.entry(s).or_insert(0) += 1;
                h
            })
            .iter()
            .filter(|(_, v)| **v > 1)
            .map(|(k, _)| String::from(*k))
            .collect();

        if !duplicate_pops.is_empty() {
            let v = dup_props
                .entry(String::from(sk))
                .or_insert(Vec::<String>::new());
            v.append(&mut duplicate_pops)
        }
    }

    let dup_sect: Vec<String> = sect_count
        .iter()
        .filter(|(_, v)| **v > 1)
        .map(|(k, _)| String::from(*k))
        .collect();

    ValidationResult {
        duplicate_sections: dup_sect,
        duplicate_properties: dup_props,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validate_success() {
        // Arrange
        let config = r###"# Editor configuration
root = true

[*]
charset = utf-8
indent_style = space
indent_size = 2
insert_final_newline = true
trim_trailing_whitespace = true

[*.md]
max_line_length = off
trim_trailing_whitespace = false"###;
        let conf = Ini::load_from_str(config).unwrap();

        // Act
        let valid = validate(conf);

        // Assert
        assert!(valid.duplicate_properties.is_empty());
        assert!(valid.duplicate_sections.is_empty());
    }

    #[test]
    fn validate_fail_duplicate_keys_in_not_root() {
        // Arrange
        let config = r###"# Editor configuration
root = true

[*]
charset = utf-8
indent_style = space
indent_size = 2
insert_final_newline = true
trim_trailing_whitespace = true
trim_trailing_whitespace = false

[*.md]
max_line_length = off
trim_trailing_whitespace = false"###;
        let conf = Ini::load_from_str(config).unwrap();

        // Act
        let valid = validate(conf);

        // Assert
        assert!(!valid.duplicate_properties.is_empty());
        assert!(valid.duplicate_sections.is_empty());
    }

    #[test]
    fn validate_fail_duplicate_keys_in_root() {
        // Arrange
        let config = r###"# Editor configuration
root = true
root = false

[*.md]
max_line_length = off
trim_trailing_whitespace = false"###;
        let conf = Ini::load_from_str(config).unwrap();

        // Act
        let valid = validate(conf);

        // Assert
        assert!(!valid.duplicate_properties.is_empty());
        assert!(valid.duplicate_sections.is_empty());
    }

    #[test]
    fn validate_fail_duplicate_sections() {
        // Arrange
        let config = r###"# Editor configuration
root = true

[*.md]
max_line_length = off
trim_trailing_whitespace = false

[*.md]
max_line_length = on
trim_trailing_whitespace = true"###;
        let conf = Ini::load_from_str(config).unwrap();

        // Act
        let valid = validate(conf);

        // Assert
        assert!(valid.duplicate_properties.is_empty());
        assert!(!valid.duplicate_sections.is_empty());
    }

    #[test]
    fn parse() {
        // Arrange
        let config = r###"# Editor configuration
root = true

[*]
charset = utf-8
indent_style = space
indent_size = 2
insert_final_newline = true
trim_trailing_whitespace = true

[*.md]
max_line_length = off
trim_trailing_whitespace = false"###;

        // Act
        let conf = Ini::load_from_str(config).unwrap();

        // Assert
        let star = conf.section(Some("*")).unwrap();
        assert!(star.contains_key("charset"));
        let md = conf.section(Some("*.md")).unwrap();
        assert!(md.contains_key("max_line_length"));

        for (sec, prop) in &conf {
            println!("Section: {:?}", sec);
            for (key, value) in prop.iter() {
                println!("{:?}:{:?}", key, value);
            }
        }
    }

    #[test]
    fn parse_with_duplicate_values() {
        // Arrange
        let config = r###"
[*]
charset = utf-8
charset = utf-7
"###;

        // Act
        let conf = Ini::load_from_str(config).unwrap();

        // Assert
        let star = conf.section(Some("*")).unwrap();
        assert!(star.contains_key("charset"));
        assert_eq!(2, star.get_all("charset").count());

        for (sec, prop) in &conf {
            println!("Section: {:?}", sec);
            for (key, value) in prop.iter() {
                println!("{:?}:{:?}", key, value);
            }
        }
    }

    #[test]
    fn parse_complex_section_name() {
        // Arrange
        let config = r###"
[*.{js,jsx,ts,tsx,vue}]
indent_style = space
indent_size = 2
trim_trailing_whitespace = true
insert_final_newline = true
"###;

        // Act
        let conf = Ini::load_from_str(config).unwrap();

        // Assert
        for (sec, prop) in &conf {
            println!("Section: {:?}", sec);
            for (key, value) in prop.iter() {
                println!("{:?}:{:?}", key, value);
            }
        }
    }

    #[test]
    fn parse_invalid_syntax() {
        // Arrange
        let config = r###"
[*
charset = utf-8
charset = utf-7
"###;

        // Act
        let conf = Ini::load_from_str(config);

        // Assert
        assert!(conf.is_err());
    }
}
