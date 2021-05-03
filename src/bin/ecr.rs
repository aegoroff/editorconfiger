use ansi_term::Colour::{Green, Red};
use clap::{App, Arg, ArgMatches, SubCommand};
use editorconfiger::{Errorer, Visitor};
use std::collections::BTreeMap;

#[macro_use]
extern crate clap;
extern crate ansi_term;

const PATH: &'static str = "PATH";

fn main() {
    let app = build_cli();
    let matches = app.get_matches();

    match matches.subcommand() {
        ("c", Some(cmd)) => compare(cmd),
        ("v", Some(cmd)) => validate_file(cmd),
        ("va", Some(cmd)) => validate_folder(cmd),
        _ => {}
    }
}

fn validate_file(cmd: &ArgMatches) {
    let path = cmd.value_of(PATH).unwrap();
    let visitor = ValidationVisitor::new(false);
    let err = ConsoleError {};
    editorconfiger::validate_one(path, &visitor, &err);
}

fn validate_folder(cmd: &ArgMatches) {
    let path = cmd.value_of(PATH).unwrap();
    let only_problems = cmd.is_present("problems");
    let visitor = ValidationVisitor::new(only_problems);
    let err = ConsoleError {};
    let results = editorconfiger::validate_all(path, &visitor, &err);
    println!();
    println!("  Total .editorconfig files found: {}", results);
}

fn compare(cmd: &ArgMatches) {
    // TODO: implement
}

struct ValidationVisitor {
    only_problems: bool,
}

impl ValidationVisitor {
    fn new(only_problems: bool) -> Self {
        Self { only_problems }
    }
}

impl Visitor for ValidationVisitor {
    fn visit(&self, path: &str, dup_sects: Vec<&str>, dup_props: BTreeMap<&str, Vec<&str>>) {
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

struct ConsoleError {}

impl Errorer for ConsoleError {
    fn error(&self, path: &str, err: &str) {
        println!(" {}", path);
        println!("  Error: {}", Red.paint(err));
        println!();
    }
}

fn build_cli() -> App<'static, 'static> {
    return App::new("ecr")
        .version(crate_version!())
        .author("egoroff <egoroff@gmail.com>")
        .about(".editorconfig files tool")
        .subcommand(
            SubCommand::with_name("v")
                .aliases(&["validate"])
                .about("Validate single .editorconfig file")
                .arg(
                    Arg::with_name(PATH)
                        .help("Path to .editorconfig file")
                        .required(true)
                        .index(1),
                ),
        )
        .subcommand(
            SubCommand::with_name("va")
                .aliases(&["validate-all"])
                .about("Validate all found .editorconfig files in a directory and all its children")
                .arg(
                    Arg::with_name(PATH)
                        .help("Path directory that contains .editorconfig files")
                        .required(true)
                        .index(1),
                )
                .arg(
                    Arg::with_name("problems")
                        .long("problems")
                        .short("p")
                        .takes_value(false)
                        .help("Show only files with problems. Correct files will not be shown.")
                        .required(false),
                ),
        )
        .subcommand(
            SubCommand::with_name("c")
                .aliases(&["compare"])
                .about("Compare two .editorconfig files")
                .arg(
                    Arg::with_name("FILE1")
                        .help("Path to the first .editorconfig file")
                        .required(true)
                        .index(1),
                )
                .arg(
                    Arg::with_name("FILE2")
                        .help("Path to the second .editorconfig file")
                        .required(true)
                        .index(2),
                ),
        );
}
