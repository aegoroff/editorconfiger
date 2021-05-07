pub mod console;

extern crate ini;
extern crate jwalk;

use ini::{Ini, Properties};
use jwalk::WalkDir;
use std::collections::{BTreeMap, HashMap};
#[macro_use]
extern crate prettytable;

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

pub trait ValidationFormatter {
    fn format(&self, path: &str, dup_sect: Vec<&str>, dup_props: BTreeMap<&str, Vec<&str>>);
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
    for (sec, prop) in conf {
        let sk = sec.unwrap_or("root");
        *sect_count.entry(sk).or_insert(0) += 1;

        let mut duplicate_pops: Vec<&str> = prop
            .iter()
            .map(|(k, _)| k)
            .fold(HashMap::new(), |mut h, s| {
                *h.entry(s).or_insert(0) += 1;
                h
            })
            .iter()
            .filter(|(_, v)| **v > 1)
            .map(|(k, _)| *k)
            .collect();

        if !duplicate_pops.is_empty() {
            dup_props
                .entry(sk)
                .or_insert_with(Vec::<&str>::new)
                .append(&mut duplicate_pops);
        }
    }

    let dup_sect: Vec<&str> = sect_count
        .iter()
        .filter(|(_, v)| **v > 1)
        .map(|(k, _)| *k)
        .collect();

    formatter.format(path, dup_sect, dup_props);
}

fn compare_files<F: ComparisonFormatter>(conf1: &Ini, conf2: &Ini, formatter: &F) {
    let empty = &Properties::new();

    let result: BTreeMap<&str, Vec<CompareItem>> =
        conf1
            .iter()
            .map(|(s1, props1)| {
                let props2 = conf2.section(s1).unwrap_or(empty);
                (s1, props1, props2)
            })
            .map(|(s1, props1, props2)| {
                let items: Vec<CompareItem> =
                    props1
                        .iter()
                        .map(|(k1, v1)| CompareItem {
                            key: k1,
                            first_value: Some(v1),
                            second_value: props2.get(k1),
                        })
                        .chain(props2.iter().filter(|(k, _)| !props1.contains_key(k)).map(
                            |(k, v)| CompareItem {
                                key: k,
                                first_value: None,
                                second_value: Some(v),
                            },
                        ))
                        .collect();
                (s1.unwrap_or_default(), items)
            })
            .chain(
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
        F: Fn(Vec<&str>, BTreeMap<&str, Vec<&str>>),
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
        F: Fn(Vec<&str>, BTreeMap<&str, Vec<&str>>),
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
        F: Fn(Vec<&str>, BTreeMap<&str, Vec<&str>>),
    {
        fn format(&self, _: &str, dup_sect: Vec<&str>, dup_props: BTreeMap<&str, Vec<&str>>) {
            (self.assert)(dup_sect, dup_props);
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
        let formatter = TestFormatter::new(|sect: Vec<&str>, props: BTreeMap<&str, Vec<&str>>| {
            assert!(props.is_empty());
            assert!(sect.is_empty());
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
        let formatter = TestFormatter::new(|sect: Vec<&str>, props: BTreeMap<&str, Vec<&str>>| {
            assert!(props.is_empty());
            assert!(sect.is_empty());
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
        let formatter = TestFormatter::new(|sect: Vec<&str>, props: BTreeMap<&str, Vec<&str>>| {
            assert!(props.is_empty());
            assert!(sect.is_empty());
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
        let formatter = TestFormatter::new(|sect: Vec<&str>, props: BTreeMap<&str, Vec<&str>>| {
            assert!(!props.is_empty());
            assert!(sect.is_empty());
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
        let formatter = TestFormatter::new(|sect: Vec<&str>, props: BTreeMap<&str, Vec<&str>>| {
            assert!(!props.is_empty());
            assert!(sect.is_empty());
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
        let formatter = TestFormatter::new(|sect: Vec<&str>, props: BTreeMap<&str, Vec<&str>>| {
            assert!(props.is_empty());
            assert!(!sect.is_empty());
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
