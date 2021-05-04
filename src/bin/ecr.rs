use clap::{App, Arg, ArgMatches, SubCommand};
use editorconfiger::console::{Error, Formatter};

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
    editorconfiger::compare(path1, path2, &err);
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
