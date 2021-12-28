#![no_main]
use libfuzzer_sys::fuzz_target;

extern crate editorconfiger;

fuzz_target!(|data: &str| {
    editorconfiger::glob::parse(data);
});
