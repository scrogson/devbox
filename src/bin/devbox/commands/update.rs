use crate::prelude::*;

pub fn cli() -> App {
    subcommand("update")
        .about("Update a service")
        .arg(
            Arg::with_name("SERVICE")
                .required(true)
                .help("The name of the service to update"),
        )
        .arg(project())
}

pub fn exec(args: &ArgMatches) -> CliResult {
    let mut project = args.project()?;
    let name = args.value_of("SERVICE")
        .ok_or_else(|| format_err!("Error parsing `SERVICE`"))?;
    let service = project.find_service(name)?;
    service.update()
}
