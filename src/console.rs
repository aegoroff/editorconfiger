#[cfg(feature = "build-binary")]
pub mod console_impl {
    use crate::{CompareItem, ComparisonFormatter, Errorer, ValidationFormatter, ValidationResult};
    use ansi_term::ANSIGenericString;
    use ansi_term::Colour::{Green, Red, Yellow};
    use prettytable::format::TableFormat;
    use prettytable::{cell, format, row, Table};
    use std::collections::BTreeMap;

    pub struct Formatter {
        only_problems: bool,
    }

    enum ValidationState {
        Valid,
        Invalid,
        SomeProblems,
    }

    impl Formatter {
        pub fn new(only_problems: bool) -> Self {
            Self { only_problems }
        }
    }

    impl ValidationFormatter for Formatter {
        fn format(&self, result: ValidationResult) {
            let state: ValidationState;

            if result.is_ok() {
                state = ValidationState::Valid;
            } else if result.is_invalid() {
                state = ValidationState::Invalid;
            } else {
                state = ValidationState::SomeProblems;
            }
            let msg: ANSIGenericString<str>;
            match state {
                ValidationState::Valid => msg = Green.paint("valid"),
                ValidationState::Invalid => msg = Red.paint("invalid"),
                ValidationState::SomeProblems => msg = Yellow.paint("has some problems"),
            }
            if !self.only_problems || !result.is_ok() {
                println!(" {} {}", result.path, msg);
            }
            if result.is_ok() {
                return;
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
                table.set_format(new_format(6));
                println!("   Similar properties:");
                for (section, sims) in result.similar_properties {
                    println!("     [{}]:", section);

                    for sim in sims {
                        table.add_row(row![sim.0, sim.1]);
                    }
                }
                table.printstd();
            }

            if !result.ext_problems.is_empty() {
                for item in result.ext_problems {
                    if !item.duplicates.is_empty() {
                        println!("   Duplicates related to {}:", item.ext);
                        for duplicate in item.duplicates {
                            println!("       {}", duplicate);
                        }
                    }

                    if !item.similar.is_empty() {
                        let mut table = Table::new();
                        table.set_format(new_format(6));
                        println!("   Similar properties related to {}:", item.ext);
                        for sim in item.similar {
                            table.add_row(row![sim.0, sim.1]);
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
            println!(" {}", path);
            println!("  Error: {}", Red.paint(err));
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
}
