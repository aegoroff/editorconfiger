mod editorconfig_lexer;
mod editorconfig_parser;
mod enumerable;
pub mod glob;
pub mod similar;
#[cfg(feature = "build-binary")]
pub mod console;

use std::fs::File;
use std::io::prelude::*;
use std::io::{BufReader, SeekFrom};
use std::path::Path;

#[macro_use]
extern crate lalrpop_util;
extern crate aho_corasick;
extern crate jwalk;
extern crate nom;

#[cfg(test)] // <-- not needed in integration tests
#[macro_use]
extern crate spectral;

#[cfg(test)] // <-- not needed in integration tests
#[macro_use]
extern crate table_test;

#[cfg(test)] // <-- not needed in integration tests
extern crate rstest;

use crate::editorconfig_parser::Section;
use jwalk::{Parallelism, WalkDir};
use std::collections::{BTreeMap, BTreeSet, HashMap};

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

impl<'input> CompareItem<'input> {
    pub fn only_second(key: &'input str, second_value: &'input str) -> Self {
        CompareItem {
            key,
            first_value: None,
            second_value: Some(second_value),
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum ValidationState {
    Valid,
    Invalid,
    SomeProblems,
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

impl<'input> ValidationResult<'input> {
    pub fn state(&self) -> ValidationState {
        ValidationState::from(self)
    }

    pub fn is_ok(&self) -> bool {
        self.duplicate_properties.is_empty()
            && self.duplicate_sections.is_empty()
            && self.similar_properties.is_empty()
            && self.ext_problems.is_empty()
    }

    fn is_invalid(&self) -> bool {
        !self.duplicate_properties.is_empty()
            || !self.duplicate_sections.is_empty()
            || self.ext_problems.iter().any(|e| !e.duplicates.is_empty())
    }
}

impl ValidationState {
    fn from(result: &ValidationResult) -> ValidationState {
        if result.is_ok() {
            ValidationState::Valid
        } else if result.is_invalid() {
            ValidationState::Invalid
        } else {
            ValidationState::SomeProblems
        }
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
    let parallelism = Parallelism::RayonNewPool(num_cpus::get_physical());

    let iter = WalkDir::new(path)
        .skip_hidden(false)
        .follow_links(false)
        .parallelism(parallelism);
    iter.into_iter()
        .filter(Result::is_ok)
        .map(Result::unwrap)
        .filter(|f| f.file_type().is_file())
        .map(|f| f.path())
        .filter(|p| p.ends_with(EDITOR_CONFIG))
        .map(|f| f.to_str().unwrap_or("").to_string())
        .inspect(|p| validate_one(p, formatter, err))
        .count()
}

pub fn validate_one<V: ValidationFormatter, E: Errorer>(path: &str, formatter: &V, err: &E) {
    if let Some(c) = read_from_file(path, err) {
        validate(&c, path, formatter)
    }
}

pub fn compare_files<E: Errorer, F: ComparisonFormatter>(
    path1: &str,
    path2: &str,
    err: &E,
    formatter: &F,
) {
    if let Some(c1) = read_from_file(path1, err) {
        if let Some(c2) = read_from_file(path2, err) {
            compare(&c1, &c2, formatter);
        }
    }
}

fn read_from_file<E: Errorer>(path: &str, err: &E) -> Option<String> {
    let conf = read_file_content(path);
    match conf {
        Ok(c) => return Some(c),
        Err(e) => err.error(
            path,
            &format!("Problem opening file or file syntax error - {}", e),
        ),
    }
    None
}

fn read_file_content<P: AsRef<Path>>(filename: P) -> Result<String, std::io::Error> {
    let file = File::open(filename)?;
    let mut reader = BufReader::new(file);

    // Check if file starts with a BOM marker
    // UTF-8: EF BB BF
    let mut bom = [0u8; 3];
    if let Ok(..) = reader.read_exact(&mut bom) {
        if &bom != b"\xEF\xBB\xBF" {
            // Reset file pointer
            reader.seek(SeekFrom::Start(0))?;
        }
    }

    let mut contents = String::new();
    reader.read_to_string(&mut contents)?;
    Ok(contents)
}

pub fn validate<V: ValidationFormatter>(content: &str, path: &str, formatter: &V) {
    let mut dup_props = BTreeMap::new();
    let mut sim_props = BTreeMap::new();
    let mut all_ext_props = BTreeMap::new();

    let sections = editorconfig_parser::parse(content);
    let mut section_heads = Vec::new();

    for sec in &sections {
        for e in &sec.extensions {
            all_ext_props
                .entry(e)
                .or_insert_with(Vec::new)
                .extend(&sec.properties);
        }
        section_heads.push(sec.title);

        let names_fn = || sec.properties.iter().map(|item| item.name);

        let mut duplicate_pops: Vec<&str> = enumerable::only_duplicates(names_fn()).collect();

        append_to_btree(&mut dup_props, sec.title, &mut duplicate_pops);

        let unique_props: Vec<&str> = enumerable::only_unique(names_fn()).collect();

        let mut similar = similar::find_suffix_pairs(&unique_props);
        append_to_btree(&mut sim_props, sec.title, &mut similar)
    }

    let ext_problems: Vec<ExtValidationResult> = all_ext_props
        .into_iter()
        .map(|(ext, props)| validate_extension(ext.to_string(), props))
        .filter(|r| !r.duplicates.is_empty() || !r.similar.is_empty())
        .collect();

    let dup_sect: Vec<&str> = enumerable::only_duplicates(section_heads.into_iter()).collect();

    let result = ValidationResult {
        path,
        duplicate_sections: dup_sect,
        duplicate_properties: dup_props,
        similar_properties: sim_props,
        ext_problems,
    };

    formatter.format(result);
}

fn append_to_btree<'a, T>(bree: &mut BTreeMap<&'a str, Vec<T>>, key: &'a str, data: &mut Vec<T>) {
    if !data.is_empty() {
        bree.entry(key).or_insert_with(Vec::<T>::new).append(data);
    }
}

fn validate_extension<'a>(ext: String, props: Vec<&'a Property>) -> ExtValidationResult<'a> {
    let props_sections = props.into_iter().map(|p| (p.name, p.section)).fold(
        HashMap::new(),
        |mut h, (prop, sect)| {
            h.entry(prop).or_insert_with(BTreeSet::new).insert(sect);
            h
        },
    );

    let duplicates: Vec<&str> = props_sections
        .iter()
        .filter(|(_, sections)| (*sections).len() > 1)
        .map(|(p, _)| *p)
        .collect();

    let props: Vec<&str> = props_sections.keys().copied().collect();
    let similar = similar::find_suffix_pairs(&props)
        .into_iter()
        .filter(|(first, second)| {
            let empty = BTreeSet::new();
            let first_sections = props_sections.get(first).unwrap_or(&empty);
            let second_sections = props_sections.get(second).unwrap_or(&empty);
            first_sections.intersection(second_sections).count() == 0
        })
        .collect();

    ExtValidationResult {
        ext,
        duplicates,
        similar,
    }
}

pub fn compare<F: ComparisonFormatter>(content1: &str, content2: &str, formatter: &F) {
    let empty = BTreeMap::<&str, &str>::new();

    let f1 = editorconfig_parser::parse(content1);
    let f2 = editorconfig_parser::parse(content2);

    let s1_props = map_sections(&f1);
    let s2_props = map_sections(&f2);

    let result: BTreeMap<&str, Vec<CompareItem>> = f1
        .iter()
        .map(|s1| {
            let props1 = map_properties(s1);
            let props2 = s2_props.get(s1.title).unwrap_or(&empty);
            (s1, props1, props2)
        })
        .map(|(s1, props1, props2)| {
            let items: Vec<CompareItem> = props1
                .iter()
                .map(|(k1, v1)| CompareItem {
                    key: k1,
                    first_value: Some(v1),
                    second_value: props2.get(k1).copied(),
                })
                .chain(
                    // Properties in the section that missing in the first
                    props2
                        .iter()
                        .filter(|(k, _)| !props1.contains_key(*k))
                        .map(|(k, v)| CompareItem::only_second(k, v)),
                )
                .collect();
            (s1.title, items)
        })
        .chain(
            // Sections missing in the first
            f2.iter()
                .filter(|s| s1_props.get(s.title).is_none())
                .map(|s| {
                    let items: Vec<CompareItem> = s
                        .properties
                        .iter()
                        .map(|p| CompareItem::only_second(p.name, p.value))
                        .collect();
                    (s.title, items)
                }),
        )
        .collect();

    formatter.format(result);
}

fn map_properties<'a>(s1: &'a Section<'a>) -> BTreeMap<&'a str, &'a str> {
    s1.properties.iter().map(|p| (p.name, p.value)).collect()
}

fn map_sections<'a>(sections: &'a [Section<'a>]) -> HashMap<&'a str, BTreeMap<&'a str, &'a str>> {
    sections
        .iter()
        .map(|s| (s.title, map_properties(s)))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::*;
    use spectral::prelude::*;

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
        let formatter =
            TestFormatter::new(|result: ValidationResult| {
                assert_that!(result.is_ok()).is_true();
                assert_that(&result.state()).is_equal_to(ValidationState::Valid);
            });

        // Act
        validate(config, "", &formatter);
    }

    #[rstest]
    #[case(
    "S=\u{1b}\u{1b}\u{1e}_=\u{1b}\n\u{1b},\u{1b}s=\u{1b}\u{0}\u{0}\u{1b}\u{1b}1L",
    "\n*\u{1b}\u{1b}",
    false
    )]
    #[trace]
    fn validate_arbitrary(#[case] content: &str, #[case] path: &str, #[case] expected: bool) {
        // Arrange
        let formatter = TestFormatter::new(|result: ValidationResult| {
            assert_that!(result.is_ok()).is_equal_to(expected);
            assert_that(&result.state()).is_equal_to(ValidationState::SomeProblems);
        });

        // Act
        validate(content, path, &formatter);
    }

    #[test]
    fn validate_success_brackets_in_section_name() {
        // Arrange
        let config = r###"
[[*]]
a = b
c = d
"###;
        let formatter =
            TestFormatter::new(|result: ValidationResult| {
                assert_that!(result.is_ok()).is_true();
                assert_that(&result.state()).is_equal_to(ValidationState::Valid);
            });

        // Act
        validate(config, "", &formatter);
    }

    #[test]
    fn validate_success_inline_comments() {
        // Arrange
        let config = r###"
[*]
a = b # comment 1
c = d # comment 2
"###;
        let formatter =
            TestFormatter::new(|result: ValidationResult| {
                assert_that!(result.is_ok()).is_true();
                assert_that(&result.state()).is_equal_to(ValidationState::Valid);
            });

        // Act
        validate(config, "", &formatter);
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
        let formatter = TestFormatter::new(|result: ValidationResult| {
            assert_that(&result.duplicate_properties.is_empty()).is_false();
            assert_that(&result.duplicate_sections.is_empty()).is_true();
            assert_that(&result.similar_properties.is_empty()).is_true();
            assert_that(&result.state()).is_equal_to(ValidationState::Invalid);
        });

        // Act
        validate(config, "", &formatter);
    }

    #[test]
    fn validate_fail_similar_keys_in_not_root() {
        // Arrange
        let config = r###"
root = true
[*]
ab = b
dab = e
c = d

[*.md]
e = f"###;
        let formatter = TestFormatter::new(|result: ValidationResult| {
            assert!(result.duplicate_properties.is_empty());
            assert!(result.duplicate_sections.is_empty());
            assert!(!result.similar_properties.is_empty());
            assert_that!(result.ext_problems).is_empty();
            assert_that(&result.state()).is_equal_to(ValidationState::SomeProblems);
        });

        // Act
        validate(config, "", &formatter);
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
        let formatter = TestFormatter::new(|result: ValidationResult| {
            assert!(!result.duplicate_properties.is_empty());
            assert!(result.duplicate_sections.is_empty());
            assert!(result.similar_properties.is_empty());
            assert_that!(result.ext_problems).is_empty();
            assert_that(&result.state()).is_equal_to(ValidationState::Invalid);
        });

        // Act
        validate(config, "", &formatter);
    }

    #[test]
    fn validate_fail_duplicate_keys_ext_across_different_sections() {
        // Arrange
        let config = r###"
[*.{md,txt}]
a = b
c = d

[*.md]
a = d
"###;
        let formatter = TestFormatter::new(|result: ValidationResult| {
            assert!(result.duplicate_properties.is_empty());
            assert!(result.duplicate_sections.is_empty());
            assert!(result.similar_properties.is_empty());
            assert_that!(result.ext_problems).has_length(1);
            assert_that(&result.state()).is_equal_to(ValidationState::Invalid);
        });

        // Act
        validate(config, "", &formatter);
    }

    #[test]
    fn validate_fail_similar_keys_ext_across_different_sections() {
        // Arrange
        let config = r###"
[*.{md,txt}]
a_b_c = b
x = d

[*.md]
d_a_b_c = d
"###;
        let formatter = TestFormatter::new(|result: ValidationResult| {
            assert!(result.duplicate_properties.is_empty());
            assert!(result.duplicate_sections.is_empty());
            assert!(result.similar_properties.is_empty());
            assert_that!(result.ext_problems).has_length(1);
            assert_that(&result.state()).is_equal_to(ValidationState::SomeProblems);
        });

        // Act
        validate(config, "", &formatter);
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
        let formatter = TestFormatter::new(|result: ValidationResult| {
            assert!(result.duplicate_properties.is_empty());
            assert!(!result.duplicate_sections.is_empty());
            assert!(result.similar_properties.is_empty());
            assert_that(&result.state()).is_equal_to(ValidationState::Invalid);
        });

        // Act
        validate(config, "", &formatter);
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

        let formatter = TestCompareFormatter::new(|res: BTreeMap<&str, Vec<CompareItem>>| {
            assert_eq!(1, res.len());
            assert_that(res.get("*").unwrap()).has_length(2);
        });

        // Act
        compare(config1, config2, &formatter);
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

        let formatter = TestCompareFormatter::new(|res: BTreeMap<&str, Vec<CompareItem>>| {
            assert_eq!(2, res.len());
            assert_that!(res.get("*")).is_some();
            assert_that(res.get("*").unwrap()).has_length(2);
            assert_that!(res.get("")).is_some();
            assert_that(res.get("").unwrap()).has_length(1);
        });

        // Act
        compare(config1, config2, &formatter);
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
        let formatter = TestCompareFormatter::new(|res: BTreeMap<&str, Vec<CompareItem>>| {
            assert_eq!(1, res.len());
            assert_that(res.get("*").unwrap()).has_length(3);
        });

        // Act
        compare(config1, config2, &formatter);
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
        let formatter = TestCompareFormatter::new(|res: BTreeMap<&str, Vec<CompareItem>>| {
            assert_eq!(2, res.len());
            assert_that(res.get("x").unwrap()).has_length(2);
            assert_that(res.get("y").unwrap()).has_length(2);
        });

        // Act
        compare(config1, config2, &formatter);
    }
}
