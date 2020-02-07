use crate::prelude::*;

pub fn cli() -> App {
    subcommand("start")
        .about("Start infrastructure or service")
        .arg(Arg::with_name("SERVICE").help("The name of the service to start"))
        .arg(project())
}

pub fn exec(args: &ArgMatches) -> CliResult {
    let mut project = args.project()?;

    match args.value_of("SERVICE") {
        Some(name) => {
            let service = project.find_service(name)?;
            service.start()
        }
        None => {
            let _ = docker_compose().args(&["up", "-d"]).spawn()?.wait();
            Ok(())
        }
    }
}
