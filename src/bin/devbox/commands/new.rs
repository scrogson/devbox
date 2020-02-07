use crate::prelude::*;

pub fn cli() -> App {
    subcommand("new")
        .about("Generates a new devbox project template")
        .arg(
            Arg::with_name("PROJECT")
                .required(true)
                .help("The project name"),
        )
        .arg(
            Arg::with_name("git")
                .long("git")
                .takes_value(true)
                .help("A URL to a git repository containing configuration for this project"),
        )
}

pub fn exec(args: &ArgMatches) -> CliResult {
    let name = args.value_of("PROJECT")
        .ok_or_else(|| format_err!("Missing project name"))?;
    match args.value_of("git") {
        Some(repo) => Project::init_from_git(name, repo),
        None => Project::init(name),
    }
}
