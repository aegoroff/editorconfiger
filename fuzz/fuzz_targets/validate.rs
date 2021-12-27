#![no_main]

use libfuzzer_sys::fuzz_target;
use editorconfiger::{Errorer, ValidationFormatter, ValidationResult};

extern crate editorconfiger;

fuzz_target!(|data: &[u8]| {
    let f = Formatter{};
    let e = Error{};
    if let Ok(s) = std::str::from_utf8(data) {
        editorconfiger::validate_one(s, &f, &e)
    }
});

struct Formatter {}

impl ValidationFormatter for Formatter
{
    fn format(&self, _result: ValidationResult) {}
}

pub struct Error {}

impl Errorer for Error {
    fn error(&self, _path: &str, _err: &str) {}
}