use ansi_term::Colour::{Green, Red};
use clap::{App, Arg, ArgMatches, SubCommand};
use editorconfiger::ValidationResult;

#[macro_use]
extern crate clap;
extern crate ansi_term;

fn main() {
    let app = build_cli();
    let matches = app.get_matches();

    match matches.subcommand() {
        ("c", Some(cmd)) => compare(cmd),
        ("v", Some(cmd)) => validate_file(cmd),
        ("va", Some(cmd)) => validate_path(cmd),
        _ => {}
    }
}

fn validate_file(cmd: &ArgMatches) {
    let path = cmd.value_of("PATH").unwrap();
    let result = editorconfiger::validate_one(path);
    match result {
        Ok(res) => print_validation_result(path, res, false),
        Err(err) => println!(" Error: {}", Red.paint(err.to_string()))
    }
}

fn validate_path(cmd: &ArgMatches) {
    let path = cmd.value_of("PATH").unwrap();
    let only_problems = cmd.is_present("problems");
    let results = editorconfiger::validate_all(path);
    for (f, r) in results {
        print_validation_result(&f, r, only_problems)
    }
}

fn print_validation_result(f: &str, r: ValidationResult, only_problems: bool) {
    if r.duplicate_properties.is_empty() && r.duplicate_sections.is_empty() {
        if !only_problems {
            println!(" {} {}", f, Green.paint("valid"));
        }
    } else {
        println!(" {} {}", f, Red.paint("invalid"));
    }

    if !r.duplicate_sections.is_empty() {
        println!("   Duplicate sections:");
        for section in r.duplicate_sections {
            println!("     {}", section);
        }
    }
    if !r.duplicate_properties.is_empty() {
        println!("   Duplicate properties:");
        for (section, duplicates) in r.duplicate_properties {
            println!("     [{}]:", section);
            for property in duplicates {
                println!("       {}", property);
            }
        }
    }
}

fn compare(cmd: &ArgMatches) {
    // TODO: implement
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
                    Arg::with_name("PATH")
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
                    Arg::with_name("PATH")
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
