extern crate ini;
extern crate jwalk;

use ini::Ini;
use jwalk::WalkDir;
use std::collections::{BTreeMap, HashMap};

pub type AnyError = Box<dyn std::error::Error>;

const EDITOR_CONFIG: &str = ".editorconfig";

pub trait Errorer {
    fn error(&self, path: &str, err: &str);
}

pub trait Visitor {
    fn visit(&self, path: &str, dup_sect: Vec<&str>, dup_props: BTreeMap<&str, Vec<&str>>);
}

pub fn validate_all<V: Visitor, E: Errorer>(path: &str, visitor: &V, err: &E) -> usize {
    let iter = WalkDir::new(path).skip_hidden(false).follow_links(false);
    let results = iter
        .into_iter()
        .filter(Result::is_ok)
        .map(Result::unwrap)
        .filter(|f| f.file_type().is_file())
        .map(|f| f.path().to_str().unwrap_or("").to_string())
        .filter(|p| p.ends_with(EDITOR_CONFIG))
        .inspect(|p| validate_one(&p, visitor, err))
        .count();
    results
}

pub fn validate_one<V: Visitor, E: Errorer>(path: &str, visitor: &V, err: &E) {
    let conf = Ini::load_from_file(path);
    match conf {
        Ok(c) => validate(&c, path, visitor),
        Err(e) => err.error(path, &e.to_string()),
    }
}

fn validate<V: Visitor>(conf: &Ini, path: &str, visitor: &V) {
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
            let v = dup_props.entry(sk).or_insert(Vec::<&str>::new());
            v.append(&mut duplicate_pops)
        }
    }

    let dup_sect: Vec<&str> = sect_count
        .iter()
        .filter(|(_, v)| **v > 1)
        .map(|(k, _)| *k)
        .collect();

    visitor.visit(path, dup_sect, dup_props);
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestVisitor<F>
    where
        F: Fn(Vec<&str>, BTreeMap<&str, Vec<&str>>),
    {
        assert: F,
    }

    impl<F> TestVisitor<F>
    where
        F: Fn(Vec<&str>, BTreeMap<&str, Vec<&str>>),
    {
        fn new(assert: F) -> Self {
            Self { assert }
        }
    }

    impl<F> Visitor for TestVisitor<F>
    where
        F: Fn(Vec<&str>, BTreeMap<&str, Vec<&str>>),
    {
        fn visit(&self, _: &str, dup_sect: Vec<&str>, dup_props: BTreeMap<&str, Vec<&str>>) {
            (self.assert)(dup_sect, dup_props);
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
        let visitor = TestVisitor::new(|sect: Vec<&str>, props: BTreeMap<&str, Vec<&str>>| {
            assert!(props.is_empty());
            assert!(sect.is_empty());
        });

        // Act
        validate(&conf, "", &visitor);
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
        let visitor = TestVisitor::new(|sect: Vec<&str>, props: BTreeMap<&str, Vec<&str>>| {
            assert!(props.is_empty());
            assert!(sect.is_empty());
        });

        // Act
        validate(&conf, "", &visitor);
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
        let visitor = TestVisitor::new(|sect: Vec<&str>, props: BTreeMap<&str, Vec<&str>>| {
            assert!(!props.is_empty());
            assert!(sect.is_empty());
        });

        // Act
        validate(&conf, "", &visitor);
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
        let visitor = TestVisitor::new(|sect: Vec<&str>, props: BTreeMap<&str, Vec<&str>>| {
            assert!(!props.is_empty());
            assert!(sect.is_empty());
        });

        // Act
        validate(&conf, "", &visitor);
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
        let visitor = TestVisitor::new(|sect: Vec<&str>, props: BTreeMap<&str, Vec<&str>>| {
            assert!(props.is_empty());
            assert!(!sect.is_empty());
        });

        // Act
        validate(&conf, "", &visitor);
    }
}
