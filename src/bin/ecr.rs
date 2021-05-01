use clap::{App, Arg, ArgMatches, SubCommand};

#[macro_use]
extern crate clap;

fn main() {
    let app = build_cli();
    let matches = app.get_matches();

    match matches.subcommand() {
        ("c", Some(cmd)) => compare(cmd),
        ("v", Some(cmd)) => validate(cmd),
        _ => {}
    }
}

fn validate(cmd: &ArgMatches) {
    // TODO: implement
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
            SubCommand::with_name("c")
                .aliases(&["compare"])
                .about("Compare two .editorconfig files")
                .arg(
                    Arg::with_name("FILE1")
                        .help("Path to the first .editorconfig file")
                        .required(true)
                        .index(1),
                ).arg(
                Arg::with_name("FILE2")
                    .help("Path to the second .editorconfig file")
                    .required(true)
                    .index(2),
            ),
        );
}