use crate::{CompareItem, ComparisonFormatter, Errorer, ValidationFormatter, ValidationResult};
use ansi_term::Colour::{Green, Red, Yellow};
use prettytable::format::TableFormat;
use prettytable::{format, Table};
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
    fn format(&self, result: ValidationResult) {
        if result.is_ok() {
            if !self.only_problems {
                println!(" {} {}", result.path, Green.paint("valid"));
            }
            return;
        }

        if result.is_invalid() {
            println!(" {} {}", result.path, Red.paint("invalid"));
        } else {
            println!(" {} {}", result.path, Yellow.paint("has some problems"));
        }

        if !result.duplicate_sections.is_empty() {
            println!("   Duplicate sections:");
            for section in result.duplicate_sections {
                println!("     {}", section);
            }
        }
        if !result.duplicate_properties.is_empty() {
            println!("   Duplicate properties:");
            for (section, duplicates) in result.duplicate_properties {
                println!("     [{}]:", section);
                for property in duplicates {
                    println!("       {}", property);
                }
            }
        }

        if !result.similar_properties.is_empty() {
            let mut table = Table::new();
            table.set_format(Comparator::new_compare_format(6));
            println!("   Similar properties:");
            for (section, sims) in result.similar_properties {
                println!("     [{}]:", section);

                for sim in sims {
                    table.add_row(row![sim.0, sim.1]);
                }
            }
            table.printstd();
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

pub struct Comparator {}

impl ComparisonFormatter for Comparator {
    fn format(&self, result: BTreeMap<&str, Vec<CompareItem>>) {
        let mut table = Table::new();
        table.set_format(Comparator::new_compare_format(0));
        table.set_titles(row![bF->"", bF->"FILE #1", bF->"FILE #2"]);

        for (sect, values) in result {
            if !sect.is_empty() {
                table.add_row(row![H3=>""]);
            }

            table.add_row(row![bFH3=>sect]);
            for value in values {
                let v1 = value.first_value.unwrap_or_default();
                let v2 = value.second_value.unwrap_or_default();
                if v1 != v2 && !v1.is_empty() && !v2.is_empty() {
                    table.add_row(row![value.key, Fy->v1, Fy->v2]);
                } else if v1 != v2 {
                    table.add_row(row![value.key, Fg->v1, Fg->v2]);
                } else {
                    table.add_row(row![value.key, v1, v2]);
                }
            }
        }
        table.add_row(row![H3=>""]);
        table.printstd();
    }
}

impl Comparator {
    fn new_compare_format(ident: usize) -> TableFormat {
        format::FormatBuilder::new()
            .column_separator(' ')
            .borders(' ')
            .separators(
                &[format::LinePosition::Title],
                format::LineSeparator::new('-', ' ', ' ', ' '),
            )
            .indent(ident)
            .padding(0, 0)
            .build()
    }
}
