use prelude::*;

pub fn cli() -> App {
    subcommand("renew")
        .about("Re-generates an existing devbox project template from a git repository")
        .arg(project())
        .arg(
            Arg::with_name("git")
                .long("git")
                .takes_value(true)
                .required(true)
                .help("A URL to a git repository containing configuration for this project"),
        )
}

pub fn exec(args: &ArgMatches) -> CliResult {
    let name = args.value_of("PROJECT")
        .ok_or_else(|| format_err!("Missing project name"))?;
    let repo = args.value_of("git")
        .ok_or_else(|| format_err!("Missing git repository URL!"))?;
    Project::update_from_git(name, repo)
}
