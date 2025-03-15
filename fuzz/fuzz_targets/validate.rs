#![no_main]

use editorconfiger::{ValidationFormatter, ValidationResult};
use libfuzzer_sys::fuzz_target;

extern crate editorconfiger;

fuzz_target!(|data: ValidateInuput| {
    let f = Formatter {};
    editorconfiger::validate(data.content, data.path, &f);
});

#[derive(Clone, Debug, arbitrary::Arbitrary)]
pub struct ValidateInuput<'a> {
    pub content: &'a str,
    pub path: &'a str,
}

struct Formatter {}

impl ValidationFormatter for Formatter {
    fn format(&self, _result: ValidationResult) {}
}
