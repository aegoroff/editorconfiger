#![allow(clippy::unwrap_in_result)]
#![allow(clippy::unwrap_used)]
use std::io;

use bugreport::{
    bugreport,
    collector::{CompileTimeInformation, EnvironmentVariables, OperatingSystem, SoftwareVersion},
    format::Markdown,
};

use clap::{
    arg, command, crate_authors, crate_description, crate_name, crate_version, value_parser,
    ArgAction, ArgMatches, Command,
};
use clap_complete::{generate, Shell};
use editorconfiger::console::{Comparator, Error, Formatter};

#[cfg(target_os = "linux")]
use mimalloc::MiMalloc;

#[cfg(target_os = "linux")]
#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

const PATH: &str = "PATH";
const FILE1: &str = "FILE1";
const FILE2: &str = "FILE2";
const PROBLEMS: &str = "problems";

fn main() -> miette::Result<()> {
    let app = build_cli();
    let matches = app.get_matches();

    match matches.subcommand() {
        Some(("c", cmd)) => compare(cmd)?,
        Some(("vf", cmd)) => validate_file(cmd)?,
        Some(("vd", cmd)) => validate_folder(cmd),
        Some(("completion", cmd)) => print_completions(cmd),
        Some(("bugreport", cmd)) => print_bugreport(cmd),
        _ => {}
    };
    Ok(())
}

fn validate_file(cmd: &ArgMatches) -> miette::Result<()> {
    let path = cmd.get_one::<String>(PATH).unwrap();
    let formatter = Formatter::new(false);
    let err = Error {};
    editorconfiger::validate_one(path, &formatter, &err)
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

fn compare(cmd: &ArgMatches) -> miette::Result<()> {
    let path1 = cmd.get_one::<String>(FILE1).unwrap();
    let path2 = cmd.get_one::<String>(FILE2).unwrap();
    let err = Error {};
    println!(" FILE #1: {path1}");
    println!(" FILE #2: {path2}");
    let comparator = Comparator {};
    editorconfiger::compare_files(path1, path2, &err, &comparator)
}

fn print_completions(matches: &ArgMatches) {
    let mut cmd = build_cli();
    let bin_name = cmd.get_name().to_string();
    if let Some(generator) = matches.get_one::<Shell>("generator") {
        generate(*generator, &mut cmd, bin_name, &mut io::stdout());
    }
}

fn print_bugreport(_matches: &ArgMatches) {
    bugreport!()
        .info(SoftwareVersion::default())
        .info(OperatingSystem::default())
        .info(EnvironmentVariables::list(&["SHELL", "TERM"]))
        .info(CompileTimeInformation::default())
        .print::<Markdown>();
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
                    arg!(-p - -problems).action(ArgAction::SetTrue).help(
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
        .subcommand(
            Command::new("completion")
                .about("Generate the autocompletion script for the specified shell")
                .arg(
                    arg!([generator])
                        .value_parser(value_parser!(Shell))
                        .required(true)
                        .index(1),
                ),
        )
        .subcommand(
            Command::new("bugreport")
                .about("Collect information about the system and the environment that users can send along with a bug report"),
        )
}
