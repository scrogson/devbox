extern crate clap;
extern crate colored;
extern crate devbox;
#[macro_use]
extern crate failure;

use devbox::*;

fn run() -> Result<()> {
    let matches = cli::new().get_matches();

    let subcmd = matches
        .subcommand_name()
        .ok_or_else(|| format_err!("No subcommand found"))?;
    let args = matches
        .subcommand_matches(subcmd)
        .ok_or_else(|| format_err!("Error fetching argument for subcommand"))?;

    match subcmd {
        "build" => devbox::build(args),
        "completions" => devbox::completions(args),
        "doctor" => devbox::doctor(args),
        "logs" => devbox::logs(args),
        "new" => devbox::new_project(args),
        "ps" => devbox::ps(args),
        "start" => devbox::start(args),
        "stop" => devbox::stop(args),
        "tasks" => devbox::tasks(args),
        "update" => devbox::update(args),
        unknown => Err(UnimplementedSubcommand(unknown.to_owned()).into()),
    }
}

fn main() {
    if let Err(ref err) = run() {
        use colored::*;

        eprintln!("{}", "ERROR:".red());
        err.causes()
            .for_each(|cause| eprintln!("{}", format!("{}", cause).red()));

        std::process::exit(1);
    }
}
