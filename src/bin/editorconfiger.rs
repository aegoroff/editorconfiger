use clap::{
    arg, command, crate_authors, crate_description, crate_name, crate_version, ArgAction, ArgMatches, Command,
};
use editorconfiger::console::{Comparator, Error, Formatter};

const PATH: &str = "PATH";
const FILE1: &str = "FILE1";
const FILE2: &str = "FILE2";
const PROBLEMS: &str = "problems";

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
    let path = cmd.get_one::<String>(PATH).unwrap();
    let formatter = Formatter::new(false);
    let err = Error {};
    editorconfiger::validate_one(path, &formatter, &err);
}

fn validate_folder(cmd: &ArgMatches) {
    let path = cmd.get_one::<String>(PATH).unwrap();
    let only_problems = cmd.get_flag(PROBLEMS);
    let formatter = Formatter::new(only_problems);
    let err = Error {};
    let results = editorconfiger::validate_all(path, &formatter, &err);
    println!();
    println!("  Total .editorconfig files found: {results}");
}

fn compare(cmd: &ArgMatches) {
    let path1 = cmd.get_one::<String>(FILE1).unwrap();
    let path2 = cmd.get_one::<String>(FILE2).unwrap();
    let err = Error {};
    println!(" FILE #1: {path1}");
    println!(" FILE #2: {path2}");
    let cmp = Comparator {};
    editorconfiger::compare_files(path1, path2, &err, &cmp);
}

fn build_cli() -> Command {
    command!(crate_name!())
        .arg_required_else_help(true)
        .version(crate_version!())
        .author(crate_authors!("\n"))
        .about(crate_description!())
        .subcommand(
            Command::new("vf")
                .aliases(["validate-file"])
                .about("Validate single .editorconfig file")
                .arg(
                    arg!([PATH])
                        .help("Path to .editorconfig file")
                        .required(true)
                        .index(1),
                ),
        )
        .subcommand(
            Command::new("vd")
                .aliases(["validate-dir"])
                .about("Validate all found .editorconfig files in a directory and all its children")
                .arg(
                    arg!([PATH])
                        .help("Path to the directory that contains .editorconfig filese")
                        .required(true)
                        .index(1),
                )
                .arg(
                    arg!(-p --problems)
                        .action(ArgAction::SetTrue)
                        .help(
                            "Show only files that have problems. Correct files will not be shown.",
                        ),
                ),
        )
        .subcommand(
            Command::new("c")
                .aliases(["compare"])
                .about("Compare two .editorconfig files")
                .arg(
                    arg!([FILE1])
                        .help("Path to the first .editorconfig file")
                        .required(true)
                        .index(1),
                )
                .arg(
                    arg!([FILE2])
                        .help("Path to the second .editorconfig file")
                        .required(true)
                        .index(2),
                ),
        )
}
