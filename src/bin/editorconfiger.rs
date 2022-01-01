use clap::{App, AppSettings, Arg, ArgMatches};
use editorconfiger::console::{Comparator, Error, Formatter};

#[macro_use]
extern crate clap;
extern crate ansi_term;

const PATH: &str = "PATH";

fn main() {
    let app = build_cli();
    let matches = app.get_matches();

    match matches.subcommand() {
        Some(("c", cmd)) => compare(cmd),
        Some(("vf", cmd)) => validate_file(cmd),
        Some(("vd", cmd)) => validate_folder(cmd),
        _ => {}
    }
}

fn validate_file(cmd: &ArgMatches) {
    let path = cmd.value_of(PATH).unwrap();
    let formatter = Formatter::new(false);
    let err = Error {};
    editorconfiger::validate_one(path, &formatter, &err);
}

fn validate_folder(cmd: &ArgMatches) {
    let path = cmd.value_of(PATH).unwrap();
    let only_problems = cmd.is_present("problems");
    let formatter = Formatter::new(only_problems);
    let err = Error {};
    let results = editorconfiger::validate_all(path, &formatter, &err);
    println!();
    println!("  Total .editorconfig files found: {}", results);
}

fn compare(cmd: &ArgMatches) {
    let path1 = cmd.value_of("FILE1").unwrap();
    let path2 = cmd.value_of("FILE2").unwrap();
    let err = Error {};
    println!(" FILE #1: {}", path1);
    println!(" FILE #2: {}", path2);
    let cmp = Comparator {};
    editorconfiger::compare_files(path1, path2, &err, &cmp);
}

fn build_cli() -> App<'static> {
    return App::new(crate_name!())
        .setting(AppSettings::ArgRequiredElseHelp)
        .version(crate_version!())
        .author("egoroff <egoroff@gmail.com>")
        .about(".editorconfig files tool")
        .subcommand(
            App::new("vf")
                .aliases(&["validate-file"])
                .about("Validate single .editorconfig file")
                .arg(
                    Arg::new(PATH)
                        .help("Path to .editorconfig file")
                        .required(true)
                        .index(1),
                ),
        )
        .subcommand(
            App::new("vd")
                .aliases(&["validate-dir"])
                .about("Validate all found .editorconfig files in a directory and all its children")
                .arg(
                    Arg::new(PATH)
                        .help("Path to the directory that contains .editorconfig files")
                        .required(true)
                        .index(1),
                )
                .arg(
                    Arg::new("problems")
                        .long("problems")
                        .short('p')
                        .takes_value(false)
                        .help(
                            "Show only files that have problems. Correct files will not be shown.",
                        )
                        .required(false),
                ),
        )
        .subcommand(
            App::new("c")
                .aliases(&["compare"])
                .about("Compare two .editorconfig files")
                .arg(
                    Arg::new("FILE1")
                        .help("Path to the first .editorconfig file")
                        .required(true)
                        .index(1),
                )
                .arg(
                    Arg::new("FILE2")
                        .help("Path to the second .editorconfig file")
                        .required(true)
                        .index(2),
                ),
        );
}
