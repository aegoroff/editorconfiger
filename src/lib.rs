pub mod console;
mod sect;
mod similar;

#[macro_use]
extern crate lalrpop_util;
#[macro_use]
extern crate prettytable;
extern crate aho_corasick;
extern crate ini;
extern crate jwalk;

use crate::similar::Similar;
use ini::{Ini, Properties};
use jwalk::WalkDir;
use std::collections::{BTreeMap, HashMap};

pub type AnyError = Box<dyn std::error::Error>;

const EDITOR_CONFIG: &str = ".editorconfig";

pub trait Errorer {
    fn error(&self, path: &str, err: &str);
}

#[derive(Debug, Clone)]
pub struct CompareItem<'input> {
    pub key: &'input str,
    pub first_value: Option<&'input str>,
    pub second_value: Option<&'input str>,
}

pub struct ValidationResult<'input> {
    pub path: &'input str,
    pub duplicate_sections: Vec<&'input str>,
    pub duplicate_properties: BTreeMap<&'input str, Vec<&'input str>>,
    pub ext_problems: Vec<ExtValidationResult<'input>>,
    pub similar_properties: BTreeMap<&'input str, Vec<(&'input str, &'input str)>>,
}

pub struct ExtValidationResult<'input> {
    pub ext: String,
    pub duplicates: Vec<&'input str>,
    pub similar: Vec<(&'input str, &'input str)>,
}

pub struct Property<'input> {
    pub name: &'input str,
    pub value: &'input str,
    pub section: &'input str,
}

pub struct Extension<'input> {
    pub value: String,
    pub section: &'input str,
}

impl<'input> ValidationResult<'input> {
    pub fn is_ok(&self) -> bool {
        self.duplicate_properties.is_empty()
            && self.duplicate_sections.is_empty()
            && self.similar_properties.is_empty()
            && self.ext_problems.is_empty()
    }

    pub fn is_invalid(&self) -> bool {
        !self.duplicate_properties.is_empty()
            || !self.duplicate_sections.is_empty()
            || self.ext_problems.iter().any(|e| !e.duplicates.is_empty())
    }
}

pub trait ValidationFormatter {
    fn format(&self, result: ValidationResult);
}

pub trait ComparisonFormatter {
    fn format(&self, result: BTreeMap<&str, Vec<CompareItem>>);
}

pub fn validate_all<V: ValidationFormatter, E: Errorer>(
    path: &str,
    formatter: &V,
    err: &E,
) -> usize {
    let iter = WalkDir::new(path).skip_hidden(false).follow_links(false);
    iter.into_iter()
        .filter(Result::is_ok)
        .map(Result::unwrap)
        .filter(|f| f.file_type().is_file())
        .map(|f| f.path().to_str().unwrap_or("").to_string())
        .filter(|p| p.ends_with(EDITOR_CONFIG))
        .inspect(|p| validate_one(&p, formatter, err))
        .count()
}

pub fn validate_one<V: ValidationFormatter, E: Errorer>(path: &str, formatter: &V, err: &E) {
    if let Some(c) = read_from_file(path, err) {
        validate(&c, path, formatter)
    }
}

pub fn compare<E: Errorer, F: ComparisonFormatter>(
    path1: &str,
    path2: &str,
    err: &E,
    formatter: &F,
) {
    if let Some(c1) = read_from_file(path1, err) {
        if let Some(c2) = read_from_file(path2, err) {
            compare_files(&c1, &c2, formatter);
        }
    }
}

fn read_from_file<E: Errorer>(path: &str, err: &E) -> Option<Ini> {
    let conf = Ini::load_from_file(path);
    match conf {
        Ok(c) => return Some(c),
        Err(e) => err.error(path, &e.to_string()),
    }
    None
}

fn validate<V: ValidationFormatter>(conf: &Ini, path: &str, formatter: &V) {
    let mut sect_count = HashMap::new();
    let mut dup_props = BTreeMap::new();
    let mut sim_props = BTreeMap::new();
    let mut all_ext_props = BTreeMap::new();
    let mut all_extensions: Vec<Extension> = Vec::new();
    for (sec, prop) in conf {
        let sk = sec.unwrap_or("root");
        *sect_count.entry(sk).or_insert(0) += 1;
        let extensions = sect::parse(sk);

        extensions
            .into_iter()
            .inspect(|e| {
                let props: Vec<Property> = prop
                    .iter()
                    .map(|(k, v)| Property {
                        name: k,
                        value: v,
                        section: sk,
                    })
                    .collect();

                all_extensions.push(Extension {
                    value: e.clone(),
                    section: sk,
                });

                all_ext_props
                    .entry(e.clone())
                    .or_insert_with(Vec::new)
                    .extend(props);
            })
            .count();

        let unique_props: HashMap<&str, i32> =
            prop.iter()
                .map(|(k, _)| k)
                .fold(HashMap::new(), |mut h, s| {
                    *h.entry(s).or_insert(0) += 1;
                    h
                });

        let mut duplicate_pops = find_duplicates(&unique_props);

        if !duplicate_pops.is_empty() {
            dup_props
                .entry(sk)
                .or_insert_with(Vec::<&str>::new)
                .append(&mut duplicate_pops);
        }

        let props: Vec<&str> = unique_props.keys().copied().collect();
        let sim = Similar::new(&props);
        let mut similar = sim.find(&props);
        if !similar.is_empty() {
            sim_props
                .entry(sk)
                .or_insert_with(Vec::<(&str, &str)>::new)
                .append(&mut similar);
        }
    }

    let multi_section_extensions: HashMap<&str, i32> =
        all_extensions
            .iter()
            .map(|p| &p.value)
            .fold(HashMap::new(), |mut h, s| {
                *h.entry(&s).or_insert(0) += 1;
                h
            });

    let ext_problems: Vec<ExtValidationResult> = all_ext_props
        .into_iter()
        .map(|(ext, props)| {
            let unique_props: HashMap<&str, i32> =
                props
                    .iter()
                    .map(|p| p.name)
                    .filter(|e| {
                        if let Some((_e, c)) = multi_section_extensions.get_key_value(e) {
                            return *c > 1;
                        }
                        false
                    })
                    .fold(HashMap::new(), |mut h, s| {
                        *h.entry(s).or_insert(0) += 1;
                        h
                    });
            let duplicates = find_duplicates(&unique_props);

            let props: Vec<&str> = unique_props.keys().copied().collect();
            let sim = Similar::new(&props);
            let similar = sim.find(&props);

            ExtValidationResult {
                ext,
                duplicates,
                similar,
            }
        })
        .filter(|r| !r.duplicates.is_empty() || !r.similar.is_empty())
        .collect();

    let dup_sect: Vec<&str> = find_duplicates(&sect_count);

    let result = ValidationResult {
        path,
        duplicate_sections: dup_sect,
        duplicate_properties: dup_props,
        similar_properties: sim_props,
        ext_problems,
    };

    formatter.format(result);
}

fn find_duplicates<'a>(unique_props: &HashMap<&'a str, i32>) -> Vec<&'a str> {
    unique_props
        .iter()
        .filter(|(_, v)| **v > 1)
        .map(|(k, _)| *k)
        .collect()
}

fn compare_files<F: ComparisonFormatter>(conf1: &Ini, conf2: &Ini, formatter: &F) {
    let empty = &Properties::new();

    let result: BTreeMap<&str, Vec<CompareItem>> = conf1
        .iter()
        .map(|(s1, props1)| {
            let props2 = conf2.section(s1).unwrap_or(empty);
            (s1, props1, props2)
        })
        .map(|(s1, props1, props2)| {
            let items: Vec<CompareItem> = props1
                .iter()
                .map(|(k1, v1)| CompareItem {
                    key: k1,
                    first_value: Some(v1),
                    second_value: props2.get(k1),
                })
                .chain(
                    // Properties in the section that missing in the first
                    props2
                        .iter()
                        .filter(|(k, _)| !props1.contains_key(k))
                        .map(|(k, v)| CompareItem {
                            key: k,
                            first_value: None,
                            second_value: Some(v),
                        }),
                )
                .collect();
            (s1.unwrap_or_default(), items)
        })
        .chain(
            // Sections missing in the first
            conf2
                .iter()
                .filter(|(s, _)| conf1.section(*s).is_none())
                .map(|(s, p)| {
                    let items: Vec<CompareItem> = p
                        .iter()
                        .map(|(k, v)| CompareItem {
                            key: k,
                            first_value: None,
                            second_value: Some(v),
                        })
                        .collect();
                    (s.unwrap_or_default(), items)
                }),
        )
        .collect();

    formatter.format(result);
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestFormatter<F>
    where
        F: Fn(ValidationResult),
    {
        assert: F,
    }

    struct TestCompareFormatter<F>
    where
        F: Fn(BTreeMap<&str, Vec<CompareItem>>),
    {
        assert: F,
    }

    impl<F> TestFormatter<F>
    where
        F: Fn(ValidationResult),
    {
        fn new(assert: F) -> Self {
            Self { assert }
        }
    }

    impl<F> TestCompareFormatter<F>
    where
        F: Fn(BTreeMap<&str, Vec<CompareItem>>),
    {
        fn new(assert: F) -> Self {
            Self { assert }
        }
    }

    impl<F> ValidationFormatter for TestFormatter<F>
    where
        F: Fn(ValidationResult),
    {
        fn format(&self, result: ValidationResult) {
            (self.assert)(result);
        }
    }

    impl<F> ComparisonFormatter for TestCompareFormatter<F>
    where
        F: Fn(BTreeMap<&str, Vec<CompareItem>>),
    {
        fn format(&self, result: BTreeMap<&str, Vec<CompareItem>>) {
            (self.assert)(result);
        }
    }

    #[test]
    fn validate_success() {
        // Arrange
        let config = r###"
root = true
[*]
a = b
c = d

[*.md]
e = f"###;
        let conf = Ini::load_from_str(config).unwrap();
        let formatter = TestFormatter::new(|result: ValidationResult| {
            assert!(result.is_ok());
        });

        // Act
        validate(&conf, "", &formatter);
    }

    #[test]
    fn validate_success_brakets_in_section_name() {
        // Arrange
        let config = r###"
[[*]]
a = b
c = d
"###;
        let conf = Ini::load_from_str(config).unwrap();
        let formatter = TestFormatter::new(|result: ValidationResult| {
            assert!(result.is_ok());
        });

        // Act
        validate(&conf, "", &formatter);
    }

    #[test]
    fn validate_success_inline_comments() {
        // Arrange
        let config = r###"
[*]
a = b # comment 1
c = d # comment 2
"###;
        let conf = Ini::load_from_str(config).unwrap();
        let formatter = TestFormatter::new(|result: ValidationResult| {
            assert!(result.is_ok());
        });

        // Act
        validate(&conf, "", &formatter);
    }

    #[test]
    fn validate_fail_duplicate_keys_in_not_root() {
        // Arrange
        let config = r###"
root = true
[*]
a = b
a = e
c = d

[*.md]
e = f"###;
        let conf = Ini::load_from_str(config).unwrap();
        let formatter = TestFormatter::new(|result: ValidationResult| {
            assert!(!result.duplicate_properties.is_empty());
            assert!(result.duplicate_sections.is_empty());
            assert!(result.similar_properties.is_empty());
        });

        // Act
        validate(&conf, "", &formatter);
    }

    #[test]
    fn validate_fail_duplicate_keys_in_root() {
        // Arrange
        let config = r###"
root = true
root = false

[*]
a = b
c = d

[*.md]
e = f"###;
        let conf = Ini::load_from_str(config).unwrap();
        let formatter = TestFormatter::new(|result: ValidationResult| {
            assert!(!result.duplicate_properties.is_empty());
            assert!(result.duplicate_sections.is_empty());
            assert!(result.similar_properties.is_empty());
        });

        // Act
        validate(&conf, "", &formatter);
    }

    #[test]
    fn validate_fail_duplicate_sections() {
        // Arrange
        let config = r###"
root = true

[*]
a = b
c = d

[*]
e = f"###;
        let conf = Ini::load_from_str(config).unwrap();
        let formatter = TestFormatter::new(|result: ValidationResult| {
            assert!(result.duplicate_properties.is_empty());
            assert!(!result.duplicate_sections.is_empty());
            assert!(result.similar_properties.is_empty());
        });

        // Act
        validate(&conf, "", &formatter);
    }

    #[test]
    fn compare_plain() {
        // Arrange
        let config1 = r###"
[*]
a = b
c = d
"###;
        let config2 = r###"
[*]
a = b1
c = d2
"###;
        let conf1 = Ini::load_from_str(config1).unwrap();
        let conf2 = Ini::load_from_str(config2).unwrap();

        let formatter = TestCompareFormatter::new(|res: BTreeMap<&str, Vec<CompareItem>>| {
            assert_eq!(1, res.len());
            assert_eq!(2, res.get("*").unwrap().len());
        });

        // Act
        compare_files(&conf1, &conf2, &formatter);
    }

    #[test]
    fn compare_plain_with_general() {
        // Arrange
        let config1 = r###"
root = true

[*]
a = b
c = d
"###;
        let config2 = r###"
root = true

[*]
a = b1
c = d2
"###;
        let conf1 = Ini::load_from_str(config1).unwrap();
        let conf2 = Ini::load_from_str(config2).unwrap();

        let formatter = TestCompareFormatter::new(|res: BTreeMap<&str, Vec<CompareItem>>| {
            assert_eq!(2, res.len());
            assert_eq!(2, res.get("*").unwrap().len());
            assert_eq!(1, res.get("").unwrap().len());
        });

        // Act
        compare_files(&conf1, &conf2, &formatter);
    }

    #[test]
    fn compare_keys_different() {
        // Arrange
        let config1 = r###"
[*]
a = b
c = d
"###;
        let config2 = r###"
[*]
a = b1
d = d2
"###;
        let conf1 = Ini::load_from_str(config1).unwrap();
        let conf2 = Ini::load_from_str(config2).unwrap();
        let formatter = TestCompareFormatter::new(|res: BTreeMap<&str, Vec<CompareItem>>| {
            assert_eq!(1, res.len());
            assert_eq!(3, res.get("*").unwrap().len());
        });

        // Act
        compare_files(&conf1, &conf2, &formatter);
    }

    #[test]
    fn compare_sections_different() {
        // Arrange
        let config1 = r###"
[x]
a = b
c = d
"###;
        let config2 = r###"
[y]
a = b1
d = d2
"###;
        let conf1 = Ini::load_from_str(config1).unwrap();
        let conf2 = Ini::load_from_str(config2).unwrap();
        let formatter = TestCompareFormatter::new(|res: BTreeMap<&str, Vec<CompareItem>>| {
            assert_eq!(2, res.len());
            assert_eq!(2, res.get("x").unwrap().len());
            assert_eq!(2, res.get("x").unwrap().len());
        });

        // Act
        compare_files(&conf1, &conf2, &formatter);
    }
}
