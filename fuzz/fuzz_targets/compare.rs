#![no_main]

use std::collections::BTreeMap;
use libfuzzer_sys::fuzz_target;
use editorconfiger::{CompareItem, ComparisonFormatter};

extern crate editorconfiger;

fuzz_target!(|data: CompareInuput| {
    let f = Cmp {};
    editorconfiger::compare(data.file1, data.file2, &f);
});

#[derive(Clone, Debug, arbitrary::Arbitrary)]
pub struct CompareInuput<'a> {
    pub file1: &'a str,
    pub file2: &'a str,
}

struct Cmp {}

impl ComparisonFormatter for Cmp {
    fn format(&self, _result: BTreeMap<&str, Vec<CompareItem>>) {}
}