use crate::{
    CompareItem, ComparisonFormatter, Errorer, ValidationFormatter, ValidationResult,
    ValidationState,
};
use console::style;

use prettytable::format::TableFormat;
use prettytable::{Cell, Row, Table, cell, format, row};
use std::collections::BTreeMap;

pub struct Formatter {
    only_problems: bool,
}

impl Formatter {
    #[must_use]
    pub fn new(only_problems: bool) -> Self {
        Self { only_problems }
    }
}

impl ValidationFormatter for Formatter {
    fn format(&self, result: ValidationResult) {
        let state = result.state();
        let msg = match state {
            ValidationState::Valid => style("valid").green(),
            ValidationState::Invalid => style("invalid").red(),
            ValidationState::SomeProblems => style("has some problems").yellow(),
        };

        if !self.only_problems || !state.is_ok() {
            println!(" {} {}", result.path, msg);
        }
        if state.is_ok() {
            return;
        }

        if !result.duplicate_sections.is_empty() {
            println!("   Duplicate sections:");
            for section in result.duplicate_sections {
                println!("     {section}");
            }
        }
        if !result.duplicate_properties.is_empty() {
            println!("   Duplicate properties:");
            for (section, duplicates) in result.duplicate_properties {
                println!("     [{section}]:");
                for property in duplicates {
                    println!("       {property}");
                }
            }
        }

        if !result.similar_properties.is_empty() {
            let mut table = Table::new();
            table.set_format(new_format(6));
            println!("   Similar properties:");
            for (section, sims) in result.similar_properties {
                println!("     [{section}]:");

                for (first, second) in sims {
                    table.add_row(row![first, second]);
                }
            }
            table.printstd();
        }

        if !result.ext_problems.is_empty() {
            for item in result.ext_problems {
                if !item.duplicates.is_empty() {
                    println!("   Duplicates related to {}:", item.ext);
                    for duplicate in item.duplicates {
                        println!("       {duplicate}");
                    }
                }

                if !item.similar.is_empty() {
                    let mut table = Table::new();
                    table.set_format(new_format(6));
                    println!("   Similar properties related to {}:", item.ext);
                    for (first, second) in item.similar {
                        table.add_row(row![first, second]);
                    }
                    table.printstd();
                }
            }
        }
        println!();
    }
}

pub struct Error {}

impl Errorer for Error {
    fn error(&self, path: &str, err: &str) {
        println!(" {path}");
        println!("  Error: {}", style(err).red());
        println!();
    }
}

pub struct Comparator {}

impl ComparisonFormatter for Comparator {
    fn format(&self, result: BTreeMap<&str, Vec<CompareItem>>) {
        let mut table = Table::new();
        table.set_format(new_format(0));
        table.set_titles(row![bF->"", bF->"FILE #1", bF->"FILE #2"]);

        for (sect, values) in result {
            if !sect.is_empty() {
                table.add_empty_row();
            }

            table.add_row(row![bFH3=>sect]);
            for value in values {
                let v1 = value.first_value.unwrap_or_default();
                let v2 = value.second_value.unwrap_or_default();
                let c1: Cell;
                let c2: Cell;
                if v1 == v2 {
                    // No color
                    c1 = cell!(v1);
                    c2 = cell!(v2);
                } else if v1.is_empty() || v2.is_empty() {
                    // Green because one is missing in other
                    c1 = cell!(Fg->v1);
                    c2 = cell!(Fg->v2);
                } else {
                    // Yellow because values are different
                    c1 = cell!(Fy->v1);
                    c2 = cell!(Fy->v2);
                }

                let r = Row::new(vec![cell!(value.key), c1, c2]);
                table.add_row(r);
            }
        }
        table.add_empty_row();
        table.printstd();
    }
}

fn new_format(ident: usize) -> TableFormat {
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
