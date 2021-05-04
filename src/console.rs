use crate::{Errorer, ValidationFormatter};
use ansi_term::Colour::{Green, Red};
use std::collections::BTreeMap;

pub struct Formatter {
    only_problems: bool,
}

impl Formatter {
    pub fn new(only_problems: bool) -> Self {
        Self { only_problems }
    }
}

impl ValidationFormatter for Formatter {
    fn format(&self, path: &str, dup_sects: Vec<&str>, dup_props: BTreeMap<&str, Vec<&str>>) {
        if dup_props.is_empty() && dup_sects.is_empty() {
            if !self.only_problems {
                println!(" {} {}", path, Green.paint("valid"));
            }
            return;
        }

        println!(" {} {}", path, Red.paint("invalid"));
        if !dup_sects.is_empty() {
            println!("   Duplicate sections:");
            for section in dup_sects {
                println!("     {}", section);
            }
        }
        if !dup_props.is_empty() {
            println!("   Duplicate properties:");
            for (section, duplicates) in dup_props {
                println!("     [{}]:", section);
                for property in duplicates {
                    println!("       {}", property);
                }
            }
        }
        println!();
    }
}

pub struct Error {}

impl Errorer for Error {
    fn error(&self, path: &str, err: &str) {
        println!(" {}", path);
        println!("  Error: {}", Red.paint(err));
        println!();
    }
}
