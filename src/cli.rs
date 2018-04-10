use clap::{App, AppSettings, Arg, SubCommand};

pub fn new() -> App<'static, 'static> {
    App::new("devbox")
        .version(crate_version!())
        .author(crate_authors!())
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .setting(AppSettings::VersionlessSubcommands)
        .setting(AppSettings::ColoredHelp)
        .about("Control your local infrastructure and services")
        .subcommand(build())
        .subcommand(completions())
        .subcommand(doctor())
        .subcommand(logs())
        .subcommand(new_project())
        .subcommand(ps())
        .subcommand(start())
        .subcommand(stop())
        .subcommand(tasks())
        .subcommand(update())
}

fn project() -> Arg<'static, 'static> {
    Arg::with_name("PROJECT")
        .help("Project name")
        .short("p")
        .long("project")
        .env("DEVBOX_PROJECT")
        .required(true)
}

fn build() -> App<'static, 'static> {
    SubCommand::with_name("build")
        .about("Build infrastructure")
        .arg(Arg::with_name("SERVICE").help("The name of the service to build"))
        .arg(project())
}

fn completions() -> App<'static, 'static> {
    SubCommand::with_name("completions")
        .about("Generates completions for your shell")
        .arg(
            Arg::with_name("SHELL")
                .required(true)
                .possible_values(&["bash", "fish", "zsh"])
                .help("The shell to generate the script for"),
        )
}

fn doctor() -> App<'static, 'static> {
    SubCommand::with_name("doctor").about("Check your system for potential problems")
}

fn logs() -> App<'static, 'static> {
    SubCommand::with_name("logs")
        .about("Display logs for running services")
        .arg(
            Arg::with_name("follow")
                .short("f")
                .long("follow")
                .help("Follow log output"),
        )
        .arg(
            Arg::with_name("tail")
                .short("t")
                .long("tail")
                .takes_value(true)
                .default_value("10")
                .value_name("integer or \"all\"")
                .help("Number of lines to show from the end of the logs for each container."),
        )
        .arg(
            Arg::with_name("SERVICE")
                .multiple(true)
                .value_name("SERVICE")
                .help("The name of the service(s) to log"),
        )
        .arg(project())
}

fn new_project() -> App<'static, 'static> {
    SubCommand::with_name("new")
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

fn ps() -> App<'static, 'static> {
    SubCommand::with_name("ps").about("Display running services")
}

fn start() -> App<'static, 'static> {
    SubCommand::with_name("start")
        .about("Start infrastructure or service")
        .arg(Arg::with_name("SERVICE").help("The name of the service to start"))
        .arg(project())
}

fn stop() -> App<'static, 'static> {
    SubCommand::with_name("stop")
        .about("Stop infrastructure or service")
        .arg(Arg::with_name("SERVICE").help("The name of the service to stop"))
        .arg(project())
}

fn tasks() -> App<'static, 'static> {
    SubCommand::with_name("tasks")
        .about("List and execute tasks for a service")
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .subcommand(
            SubCommand::with_name("list")
                .about("List tasks for a service")
                .arg(
                    Arg::with_name("SERVICE")
                        .required(true)
                        .help("The name of the service"),
                ),
        )
        .subcommand(
            SubCommand::with_name("exec")
                .about("Execute tasks for an service")
                .arg(
                    Arg::with_name("SERVICE")
                        .required(true)
                        .help("The name of the service"),
                )
                .arg(
                    Arg::with_name("TASKS")
                        .required(true)
                        .multiple(true)
                        .help("The name(s) of the task(s)"),
                ),
        )
        .arg(project())
}

fn update() -> App<'static, 'static> {
    SubCommand::with_name("update")
        .about("Update a service")
        .arg(
            Arg::with_name("SERVICE")
                .required(true)
                .help("The name of the service to update"),
        )
        .arg(project())
}
