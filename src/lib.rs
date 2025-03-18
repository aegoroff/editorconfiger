#![warn(unused_extern_crates)]
#![warn(clippy::unwrap_in_result)]
#![warn(clippy::unwrap_used)]
#![allow(clippy::missing_errors_doc)]
#[cfg(feature = "build-binary")]
pub mod console;
mod editorconfig;
mod enumerable;
pub mod glob;
pub mod similar;

use std::fs::File;
use std::io::BufReader;
use std::io::prelude::*;
use std::path::Path;

#[macro_use]
extern crate lalrpop_util;

#[cfg(test)] // <-- not needed in integration tests
extern crate rstest;

use editorconfig::Section;
use enumerable::IteratorExt;
use jwalk::{Parallelism, WalkDir};
use std::collections::{BTreeMap, BTreeSet, HashMap};

const EDITOR_CONFIG: &str = ".editorconfig";

/// A trait for reporting errors related to configuration validation.
///
/// This trait defines a single method, `error`, which is used to report errors encountered
/// during the validation of configuration files. Implementors of this trait can define
/// custom behavior for error reporting, such as logging the error to a file, printing it
/// to the console, or sending it to an external monitoring service.
///
/// # Methods
///
/// * `error` - Reports an error encountered during validation.
///
/// # Parameters
///
/// * `path` - A string slice that holds the path of the configuration file where the error occurred.
/// * `err` - A string slice that holds the error message.
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
    #[must_use]
    pub fn only_second(key: &'input str, second_value: &'input str) -> Self {
        CompareItem {
            key,
            first_value: None,
            second_value: Some(second_value),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum ValidationState {
    Valid,
    Invalid,
    SomeProblems,
}

/// Represents the result of validating a configuration file.
///
/// This struct holds various details about the validation process,
/// including paths, duplicate sections, duplicate properties,
/// external problems, and similar properties.
///
/// # Fields
///
/// * `path` - The path of the configuration file being validated.
/// * `duplicate_sections` - A list of sections that are duplicated within the file.
/// * `duplicate_properties` - A map where the keys are property names and the values are vectors of sections in which the properties are duplicated.
/// * `ext_problems` - A list of extended validation results containing details about duplicates and similar properties found in external files.
/// * `similar_properties` - A map where the keys are property names and the values are vectors of tuples, each containing a pair of similar properties.
///
/// # Example
///
/// ```
/// use std::collections::BTreeMap;
/// use editorconfiger::ValidationResult;
///
/// let validation_result = ValidationResult {
///     path: "path/to/config.file",
///     duplicate_sections: vec!["section1", "section2"],
///     duplicate_properties: BTreeMap::new(),
///     ext_problems: vec![],
///     similar_properties: BTreeMap::new(),
/// };
/// ```
pub struct ValidationResult<'input> {
    pub path: &'input str,
    pub duplicate_sections: Vec<&'input str>,
    pub duplicate_properties: BTreeMap<&'input str, Vec<&'input str>>,
    pub ext_problems: Vec<ExtValidationResult<'input>>,
    pub similar_properties: BTreeMap<&'input str, Vec<(&'input str, &'input str)>>,
}

/// Represents the result of an extensions validation process.
///
/// This struct holds details about duplicates and similar properties found for an extension
///
/// # Fields
///
/// * `ext` - A string representing the extension being validated.
/// * `duplicates` - A list of properties that are duplicated within extension section.
/// * `similar` - A list of tuples, each containing a pair of similar properties found.
///
/// # Example
///
/// ```
/// use editorconfiger::ExtValidationResult;
///
/// let ext_validation_result = ExtValidationResult {
///     ext: "extension".to_string(),
///     duplicates: vec!["property1", "property2"],
///     similar: vec![("property1", "property1_similar")],
/// };
/// ```
pub struct ExtValidationResult<'input> {
    pub ext: String,
    pub duplicates: Vec<&'input str>,
    pub similar: Vec<(&'input str, &'input str)>,
}

/// Property section assotiation, i.e. property and section that contain it
struct ExtendedProperty<'input> {
    pub name: &'input str,
    pub section: &'input str,
}

impl ValidationResult<'_> {
    #[must_use]
    pub fn state(&self) -> ValidationState {
        ValidationState::from(self)
    }

    fn is_ok(&self) -> bool {
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
    #[must_use]
    pub fn is_ok(&self) -> bool {
        matches!(self, ValidationState::Valid)
    }

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

/// Trait for formatting the results of a validation process.
///
/// This trait defines a method that must be implemented to format and display the results
/// of validating a configuration file. The formatting implementation can vary depending
/// on the use case, such as printing to the console, logging, or generating a report.
///
/// # Example
///
/// ```
/// use editorconfiger::{ValidationFormatter, ValidationResult};
///
/// struct MyFormatter;
///
/// impl ValidationFormatter for MyFormatter {
///     fn format(&self, result: ValidationResult) {
///         // ...
///     }
/// }
/// ```
///
/// # Method
///
/// * `format` - Formats the validation result.
///
/// # Parameters
///
/// * `result` - The `ValidationResult` struct containing the details of the validation process.
pub trait ValidationFormatter {
    fn format(&self, result: ValidationResult);
}

/// Trait for formatting the results of comparing two configuration files.
///
/// This trait defines a method that must be implemented to format and display the results
/// of comparing two configuration files. The formatting implementation can vary depending
/// on the use case, such as printing to the console, logging, or generating a report.
///
/// # Example
///
/// ```
/// use editorconfiger::{ComparisonFormatter, CompareItem};
/// use std::collections::BTreeMap;
///
/// struct MyComparisonFormatter;
///
/// impl ComparisonFormatter for MyComparisonFormatter {
///     fn format(&self, result: BTreeMap<&str, Vec<CompareItem>>) {
///         // ...
///     }
/// }
/// ```
///
/// # Method
///
/// * `format` - Formats the comparison result.
///
/// # Parameters
///
/// * `result` - A `BTreeMap` where the keys are section names and the values are vectors of `CompareItem`
///              structs, each containing details about the differences found during the comparison.
pub trait ComparisonFormatter {
    fn format(&self, result: BTreeMap<&str, Vec<CompareItem>>);
}

/// Validates all .editorconfig files in a given directory and its subdirectories.
///
/// This function traverses the directory specified by `path` and validates all files
/// that match the .editorconfig filename. It uses parallelism to speed up the process
/// by leveraging all available physical CPU cores. The function returns the number of
/// configuration files that were validated.
///
/// # Parameters
///
/// * `path` - A string slice that holds the path to the directory to be traversed.
/// * `formatter` - A reference to an implementation of the [`ValidationFormatter`] trait,
///                 which will be used to format the validation results.
/// * `err` - A reference to an implementation of the `Errorer` trait, which will be used
///           to handle any errors that occur during file reading or validation.
///
/// # Returns
///
/// * `usize` - The number of configuration files that were validated.
///
///
/// # Implementation Details
///
/// * The function uses the `WalkDir` crate to recursively traverse the directory.
/// * Files are filtered to only include those that match the .editorconfig filename.
/// * The [`validate_one`] function is called for each matching file to perform the validation.
/// * The `Rayon` crate is used to parallelize the file traversal and validation process.
pub fn validate_all<V: ValidationFormatter, E: Errorer>(
    path: &str,
    formatter: &V,
    err: &E,
) -> usize {
    let parallelism = Parallelism::RayonNewPool(num_cpus::get_physical());

    let root = decorate_path(path);

    let iter = WalkDir::new(root)
        .skip_hidden(false)
        .follow_links(false)
        .parallelism(parallelism);
    iter.into_iter()
        .filter_map(Result::ok)
        .filter(|f| f.file_type().is_file())
        .map(|f| f.path())
        .filter(|p| p.ends_with(EDITOR_CONFIG))
        .inspect(|p| {
            if let Some(p) = p.to_str() {
                if let Err(e) = validate_one(p, formatter, err) {
                    println!(" {p}: {e:?}");
                }
            }
        })
        .count()
}

/// Validates a single .editorconfig file.
///
/// This function reads the content of the configuration file specified by `path`,
/// and then validates it using the provided `formatter` to format the validation results.
/// If there is an error reading the file, the provided `err` handler will be used to handle it.
///
/// # Parameters
///
/// * `path` - A string slice that holds the path to the configuration file to be validated.
/// * `formatter` - A reference to an implementation of the `ValidationFormatter` trait,
///                 which will be used to format the validation results.
/// * `err` - A reference to an implementation of the [`Errorer`] trait, which will be used
///           to handle any errors that occur during file reading or validation.
pub fn validate_one<V: ValidationFormatter, E: Errorer>(
    path: &str,
    formatter: &V,
    err: &E,
) -> miette::Result<()> {
    if let Some(c) = read_from_file(path, err) {
        validate(&c, path, formatter)?;
    }
    Ok(())
}

/// Compares two .editorconfig files and formats the comparison results.
///
/// This function reads the contents of two configuration files specified by `path1` and `path2`,
/// and then compares them using the provided `formatter` to format the comparison results.
/// If there is an error reading either file, the provided `err` handler will be used to handle it.
///
/// # Parameters
///
/// * `path1` - A string slice that holds the path to the first .editorconfig file to be compared.
/// * `path2` - A string slice that holds the path to the second .editorconfig file to be compared.
/// * `err` - A reference to an implementation of the [`Errorer`] trait, which will be used
///           to handle any errors that occur during file reading.
/// * `formatter` - A reference to an implementation of the [`ComparisonFormatter`] trait,
///                 which will be used to format the comparison results.
pub fn compare_files<E: Errorer, F: ComparisonFormatter>(
    path1: &str,
    path2: &str,
    err: &E,
    formatter: &F,
) -> miette::Result<()> {
    if let Some(c1) = read_from_file(path1, err) {
        if let Some(c2) = read_from_file(path2, err) {
            compare(&c1, &c2, formatter)?;
        }
    }
    Ok(())
}

fn read_from_file<E: Errorer>(path: &str, err: &E) -> Option<String> {
    let conf = read_file_content(path);
    match conf {
        Ok(c) => return Some(c),
        Err(e) => err.error(
            path,
            &format!("Problem opening file or file syntax error - {e}"),
        ),
    }
    None
}

/// Reads whole file content into String
fn read_file_content<P: AsRef<Path>>(filename: P) -> Result<String, std::io::Error> {
    let file = File::open(filename)?;
    let mut reader = BufReader::new(file);

    // Check if file starts with a BOM marker
    // UTF-8: EF BB BF
    let mut bom = [0u8; 3];
    if reader.read_exact(&mut bom).is_ok() && &bom != b"\xEF\xBB\xBF" {
        // No BOM so reset file pointer back to start
        reader.rewind()?;
    }

    let mut contents = String::new();
    reader.read_to_string(&mut contents)?;
    Ok(contents)
}

/// Validates the content of an .editorconfig file.
///
/// This function parses the provided content of an .editorconfig file, checks for duplicate
/// and similar properties within sections, and validates properties with extended glob patterns.
/// The results of these validations are formatted using the provided `formatter`.
///
/// # Parameters
///
/// * `content` - A string slice that holds the content of the .editorconfig file to be validated.
/// * `path` - A string slice that holds the original file path, used for reporting purposes.
/// * `formatter` - A reference to an implementation of the `ValidationFormatter` trait,
///                 which will be used to format the validation results.
///
/// The function performs the following steps:
///
/// 1. Parses the content into sections.
/// 2. Iterates over each section to collect properties and their associated section titles.
/// 3. Checks for duplicate properties within each section and stores them.
/// 4. Checks for similar properties within each section and stores them.
/// 5. Validates properties with extended glob patterns and checks for duplicate and similar properties.
/// 6. Checks for duplicate section titles.
/// 7. Constructs a [`ValidationResult`] with all the gathered information.
/// 8. Uses the provided `formatter` to format the validation results.
///
/// The [`ValidationResult`] includes:
///
/// * The path of the validated file.
/// * A list of duplicate section titles.
/// * A map of duplicate properties by section.
/// * A map of similar properties by section.
/// * A list of problems with properties that have extended glob patterns.
pub fn validate<V: ValidationFormatter>(
    content: &str,
    path: &str,
    formatter: &V,
) -> miette::Result<()> {
    let mut dup_props = BTreeMap::new();
    let mut sim_props = BTreeMap::new();
    let mut all_ext_props = BTreeMap::new();

    let sections = editorconfig::parse(content)?;
    let mut section_heads = Vec::new();

    for sec in &sections {
        let props_fn = || {
            sec.properties.iter().map(|x| ExtendedProperty {
                name: x.name,
                section: sec.title,
            })
        };
        for e in glob::parse(sec.title) {
            all_ext_props
                .entry(e)
                .or_insert_with(Vec::new)
                .extend(props_fn());
        }
        section_heads.push(sec.title);

        let names_fn = || sec.properties.iter().map(|item| item.name);

        let mut duplicate_pops: Vec<&str> = names_fn().only_duplicates().collect();

        append_to_btree(&mut dup_props, sec.title, &mut duplicate_pops);

        let unique_props: Vec<&str> = names_fn().unique().collect();

        let mut similar = similar::find_suffix_pairs(&unique_props);
        append_to_btree(&mut sim_props, sec.title, &mut similar);
    }

    let ext_problems = all_ext_props
        .into_iter()
        .map(|(ext, props)| validate_extension(ext, props))
        .filter(|r| !r.duplicates.is_empty() || !r.similar.is_empty())
        .collect();

    let dup_sect = section_heads.into_iter().only_duplicates().collect();

    let result = ValidationResult {
        path,
        duplicate_sections: dup_sect,
        duplicate_properties: dup_props,
        similar_properties: sim_props,
        ext_problems,
    };

    formatter.format(result);
    Ok(())
}

fn append_to_btree<'a, T>(bree: &mut BTreeMap<&'a str, Vec<T>>, key: &'a str, data: &mut Vec<T>) {
    if !data.is_empty() {
        bree.entry(key).or_default().append(data);
    }
}

fn validate_extension(ext: String, props: Vec<ExtendedProperty>) -> ExtValidationResult {
    let props_sections = props.into_iter().map(|p| (p.name, p.section)).fold(
        HashMap::new(),
        |mut h: HashMap<&str, BTreeSet<&str>>, (prop, sect)| {
            h.entry(prop).or_default().insert(sect);
            h
        },
    );

    let duplicates: Vec<&str> = props_sections
        .iter()
        .filter_map(|(p, sections)| {
            if (*sections).len() > 1 {
                Some(*p)
            } else {
                None
            }
        })
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

/// Compares the properties of two .editorconfig files contents and formats the comparison result.
///
/// # Arguments
///
/// * `content1` - A string slice holding the first .editorconfig content.
/// * `content2` - A string slice holding the second .editorconfig content.
/// * `formatter` - A reference to an implementation of the [`ComparisonFormatter`] trait,
///                 which will be used to format the comparison results.
///
/// The function performs the following steps:
///
/// 1. Parses the two configuration contents into sections.
/// 2. Maps the sections to their properties for both contents.
/// 3. Iterates over the sections of the first content and compares each property with the corresponding section in the second content.
/// 4. Collects the comparison results, including properties that are only in the first content, only in the second content, or in both with different values.
/// 5. Identifies sections that are missing in the first content but present in the second content and includes their properties in the result.
/// 6. Constructs a `BTreeMap` where the key is the section title and the value is a list of [`]CompareItem`] representing the comparison results for each property.
/// 7. Uses the provided `formatter` to format the comparison results.
///
/// The resulting `BTreeMap` includes:
///
/// * Section titles as keys.
/// * Lists of [`CompareItem`] for each section, representing the property comparisons.
///   - Each [`CompareItem`] includes the property key, its value in the first content (if any), and its value in the second content (if any).
pub fn compare<F: ComparisonFormatter>(
    content1: &str,
    content2: &str,
    formatter: &F,
) -> miette::Result<()> {
    let empty = BTreeMap::<&str, &str>::new();

    let f1 = editorconfig::parse(content1)?;
    let f2 = editorconfig::parse(content2)?;

    let s1_props = map_sections(&f1);
    let s2_props = map_sections(&f2);

    let result: BTreeMap<&str, Vec<CompareItem>> = s1_props
        .iter()
        .map(|s1| {
            let props1 = s1.1;
            let props2 = s2_props.get(s1.0).unwrap_or(&empty);
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
            (*s1.0, items)
        })
        .chain(
            // Sections missing in the first
            s2_props
                .iter()
                .filter(|s| !s1_props.contains_key(s.0))
                .map(|s| {
                    let items: Vec<CompareItem> =
                        s.1.iter()
                            .map(|p| CompareItem::only_second(p.0, p.1))
                            .collect();
                    (*s.0, items)
                }),
        )
        .collect();

    formatter.format(result);
    Ok(())
}

fn map_properties<'a>(s1: &'a Section<'a>) -> BTreeMap<&'a str, &'a str> {
    s1.properties.iter().map(|p| (p.name, p.value)).collect()
}

fn map_sections<'a>(sections: &'a [Section<'a>]) -> HashMap<&'a str, BTreeMap<&'a str, &'a str>> {
    let mut result = HashMap::new();
    for s in sections {
        result
            .entry(s.title)
            .or_insert(map_properties(s))
            .extend(map_properties(s));
    }
    result
}

/// On Windows added trailing back slash \ if volume and colon passed so as to paths look more pleasant
#[cfg(target_os = "windows")]
fn decorate_path(path: &str) -> String {
    if path.len() == 2 && path.ends_with(':') {
        format!("{path}\\")
    } else {
        String::from(path)
    }
}

/// On Unix just passthrough as is
#[cfg(not(target_os = "windows"))]
fn decorate_path(path: &str) -> String {
    String::from(path)
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_in_result)]
    #![allow(clippy::unwrap_used)]
    use super::*;
    use rstest::rstest;

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
        let config = r#"
root = true
[*]
a = b
c = d

[*.md]
e = f"#;
        let formatter = TestFormatter::new(|result: ValidationResult| {
            assert!(result.is_ok());
            assert_eq!(result.state(), ValidationState::Valid);
        });

        // Act
        let r = validate(config, "", &formatter);

        // Assert
        assert!(r.is_ok());
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
            assert_eq!(result.is_ok(), expected);
            assert_eq!(result.state(), ValidationState::SomeProblems);
        });

        // Act
        let r = validate(content, path, &formatter);

        // Assert
        assert!(r.is_ok());
    }

    #[rstest]
    #[case(ValidationState::Valid, true)]
    #[case(ValidationState::Invalid, false)]
    #[case(ValidationState::SomeProblems, false)]
    #[trace]
    fn is_ok_tests(#[case] state: ValidationState, #[case] expected: bool) {
        // Arrange

        // Act
        assert_eq!(state.is_ok(), expected);
    }

    #[test]
    fn validate_success_brackets_in_section_name() {
        // Arrange
        let config = r#"
[[*]]
a = b
c = d
"#;
        let formatter = TestFormatter::new(|result: ValidationResult| {
            assert!(result.is_ok());
            assert_eq!(result.state(), ValidationState::Valid);
        });

        // Act
        let r = validate(config, "", &formatter);

        // Assert
        assert!(r.is_ok());
    }

    #[test]
    fn validate_success_inline_comments() {
        // Arrange
        let config = r#"
[*]
a = b # comment 1
c = d # comment 2
"#;
        let formatter = TestFormatter::new(|result: ValidationResult| {
            assert!(result.is_ok());
            assert_eq!(result.state(), ValidationState::Valid);
        });

        // Act
        let r = validate(config, "", &formatter);

        // Assert
        assert!(r.is_ok());
    }

    #[test]
    fn validate_fail_duplicate_keys_in_not_root() {
        // Arrange
        let config = r#"
root = true
[*]
a = b
a = e
c = d

[*.md]
e = f"#;
        let formatter = TestFormatter::new(|result: ValidationResult| {
            assert!(!result.duplicate_properties.is_empty());
            assert!(result.duplicate_sections.is_empty());
            assert!(result.similar_properties.is_empty());
            assert_eq!(result.state(), ValidationState::Invalid);
        });

        // Act
        let r = validate(config, "", &formatter);

        // Assert
        assert!(r.is_ok());
    }

    #[test]
    fn validate_fail_similar_keys_in_not_root() {
        // Arrange
        let config = r#"
root = true
[*]
ab = b
dab = e
c = d

[*.md]
e = f"#;
        let formatter = TestFormatter::new(|result: ValidationResult| {
            assert!(result.duplicate_properties.is_empty());
            assert!(result.duplicate_sections.is_empty());
            assert!(!result.similar_properties.is_empty());
            assert!(result.ext_problems.is_empty());
            assert_eq!(result.state(), ValidationState::SomeProblems);
        });

        // Act
        let r = validate(config, "", &formatter);

        // Assert
        assert!(r.is_ok());
    }

    #[test]
    fn validate_fail_duplicate_keys_in_root() {
        // Arrange
        let config = r#"
root = true
root = false

[*]
a = b
c = d

[*.md]
e = f"#;
        let formatter = TestFormatter::new(|result: ValidationResult| {
            assert!(!result.duplicate_properties.is_empty());
            assert!(result.duplicate_sections.is_empty());
            assert!(result.similar_properties.is_empty());
            assert!(result.ext_problems.is_empty());
            assert_eq!(result.state(), ValidationState::Invalid);
        });

        // Act
        let r = validate(config, "", &formatter);

        // Assert
        assert!(r.is_ok());
    }

    #[test]
    fn validate_fail_duplicate_keys_ext_across_different_sections() {
        // Arrange
        let config = r#"
[*.{md,txt}]
a = b
c = d

[*.md]
a = d
"#;
        let formatter = TestFormatter::new(|result: ValidationResult| {
            assert!(result.duplicate_properties.is_empty());
            assert!(result.duplicate_sections.is_empty());
            assert!(result.similar_properties.is_empty());
            assert_eq!(result.ext_problems.len(), 1);
            assert_eq!(result.state(), ValidationState::Invalid);
        });

        // Act
        let r = validate(config, "", &formatter);

        // Assert
        assert!(r.is_ok());
    }

    #[test]
    fn validate_fail_similar_keys_ext_across_different_sections() {
        // Arrange
        let config = r#"
[*.{md,txt}]
a_b_c = b
x = d

[*.md]
d_a_b_c = d
"#;
        let formatter = TestFormatter::new(|result: ValidationResult| {
            assert!(result.duplicate_properties.is_empty());
            assert!(result.duplicate_sections.is_empty());
            assert!(result.similar_properties.is_empty());
            assert_eq!(result.ext_problems.len(), 1);
            assert_eq!(result.state(), ValidationState::SomeProblems);
        });

        // Act
        let r = validate(config, "", &formatter);

        // Assert
        assert!(r.is_ok());
    }

    #[test]
    fn validate_fail_duplicate_sections() {
        // Arrange
        let config = r#"
root = true

[*]
a = b
c = d

[*]
e = f"#;
        let formatter = TestFormatter::new(|result: ValidationResult| {
            assert!(result.duplicate_properties.is_empty());
            assert!(!result.duplicate_sections.is_empty());
            assert!(result.similar_properties.is_empty());
            assert_eq!(result.state(), ValidationState::Invalid);
        });

        // Act
        let r = validate(config, "", &formatter);

        // Assert
        assert!(r.is_ok());
    }

    #[test]
    fn compare_plain() {
        // Arrange
        let config1 = r#"
[*]
a = b
c = d
"#;
        let config2 = r#"
[*]
a = b1
c = d2
"#;

        let formatter = TestCompareFormatter::new(|res: BTreeMap<&str, Vec<CompareItem>>| {
            assert_eq!(1, res.len());
            assert_eq!(res.get("*").unwrap().len(), 2);
        });

        // Act
        let r = compare(config1, config2, &formatter);

        // Assert
        assert!(r.is_ok());
    }

    #[test]
    fn compare_plain_with_general() {
        // Arrange
        let config1 = r#"
root = true

[*]
a = b
c = d
"#;
        let config2 = r#"
root = true

[*]
a = b1
c = d2
"#;

        let formatter = TestCompareFormatter::new(|res: BTreeMap<&str, Vec<CompareItem>>| {
            assert_eq!(2, res.len());
            assert!(res.contains_key("*"));
            assert_eq!(res.get("*").unwrap().len(), 2);
            assert!(res.contains_key(""));
            assert_eq!(res.get("").unwrap().len(), 1);
        });

        // Act
        let r = compare(config1, config2, &formatter);

        // Assert
        assert!(r.is_ok());
    }

    #[test]
    fn compare_keys_different() {
        // Arrange
        let config1 = r#"
[*]
a = b
c = d
"#;
        let config2 = r#"
[*]
a = b1
d = d2
"#;
        let formatter = TestCompareFormatter::new(|res: BTreeMap<&str, Vec<CompareItem>>| {
            assert_eq!(1, res.len());
            assert_eq!(res.get("*").unwrap().len(), 3);
        });

        // Act
        let r = compare(config1, config2, &formatter);

        // Assert
        assert!(r.is_ok());
    }

    #[test]
    fn compare_sections_different() {
        // Arrange
        let config1 = r#"
[x]
a = b
c = d
"#;
        let config2 = r#"
[y]
a = b1
d = d2
"#;
        let formatter = TestCompareFormatter::new(|res: BTreeMap<&str, Vec<CompareItem>>| {
            assert_eq!(2, res.len());
            assert_eq!(res.get("x").unwrap().len(), 2);
            assert_eq!(res.get("y").unwrap().len(), 2);
        });

        // Act
        let r = compare(config1, config2, &formatter);

        // Assert
        assert!(r.is_ok());
    }

    #[test]
    fn compare_several_sections_with_same_name() {
        // Arrange
        let config1 = r#"
[*]
a = 1
c = 2

[*]
b = 3
d = 4
"#;
        let config2 = r#"
[*]
a = 5
c = 6

[*]
b = 7
d = 8
"#;
        let formatter = TestCompareFormatter::new(|res: BTreeMap<&str, Vec<CompareItem>>| {
            assert_eq!(1, res.len());
            assert_eq!(res.get("*").unwrap().len(), 4);
        });

        // Act
        let r = compare(config1, config2, &formatter);

        // Assert
        assert!(r.is_ok());
    }

    #[cfg(not(target_os = "windows"))]
    #[rstest]
    #[case("", "")]
    #[case("/", "/")]
    #[case("/home", "/home")]
    #[case("d:", "d:")]
    #[trace]
    fn decorate_path_tests(#[case] raw_path: &str, #[case] expected: &str) {
        // Arrange

        // Act
        let actual = decorate_path(raw_path);

        // Assert
        assert_eq!(actual, expected);
    }

    #[cfg(target_os = "windows")]
    #[rstest]
    #[case("", "")]
    #[case("/", "/")]
    #[case("d:", "d:\\")]
    #[case("dd:", "dd:")]
    #[trace]
    fn decorate_path_tests(#[case] raw_path: &str, #[case] expected: &str) {
        // Arrange

        // Act
        let actual = decorate_path(raw_path);

        // Assert
        assert_eq!(actual, expected);
    }
}
