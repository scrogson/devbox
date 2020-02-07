use crate::prelude::*;

pub fn cli() -> App {
    subcommand("tasks")
        .about("List and execute tasks for a service")
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .subcommand(
            subcommand("list").about("List tasks for a service").arg(
                Arg::with_name("SERVICE")
                    .required(true)
                    .help("The name of the service"),
            ),
        )
        .subcommand(
            subcommand("exec")
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

pub fn exec(matches: &ArgMatches) -> CliResult {
    let mut project = matches.project()?;

    let subcmd = matches
        .subcommand_name()
        .ok_or_else(|| format_err!("No subcommand found"))?;
    let args = matches
        .subcommand_matches(subcmd)
        .ok_or_else(|| format_err!("Error fetching argument for subcommand"))?;
    let service_name = args.value_of("SERVICE")
        .ok_or_else(|| format_err!("No `SERVICE` supplied"))?;
    let service = project.find_service(service_name)?;

    match subcmd {
        "list" => service.list_tasks(),
        "exec" => {
            let tasks = args.values_of_lossy("TASKS")
                .ok_or_else(|| format_err!("Error parsing `TASKS`"))?;
            service.exec_tasks(tasks)
        }
        _ => Ok(()),
    }
}
