use ansi_term::Colour::{Green, Red};
use clap::{App, Arg, ArgMatches, SubCommand};

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
    // TODO: implement
}

fn validate_path(cmd: &ArgMatches) {
    let path = cmd.value_of("PATH").unwrap();
    let results = editorconfiger::validate_all(path);
    for (f, r) in results {
        let result;
        if r {
            result = Green.paint("valid")
        } else {
            result = Red.paint("valid")
        }
        println!(" {} {}", f, result);
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
