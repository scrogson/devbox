use crate::prelude::*;

pub fn cli() -> App {
    subcommand("logs")
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

pub fn exec(args: &ArgMatches) -> CliResult {
    let mut project = args.project()?;

    let compose_file = &project.docker_compose_file.clone();

    let mut cmd = Command::new("docker-compose");

    // Set the devbox compose file
    cmd.arg("-f").arg(compose_file.clone());

    if let Some(names) = args.values_of("SERVICE") {
        for name in names {
            maybe_append_docker_compose_override(&mut cmd, name, &mut project);
        }
    }

    cmd.arg("logs");

    if args.is_present("follow") {
        cmd.arg("-f");
    }

    if let Some(tail) = args.value_of("tail") {
        cmd.arg("--tail").arg(tail);
    }

    if let Some(names) = args.values_of("SERVICE") {
        names.for_each(|name| {
            cmd.arg(name);
        });
    }

    let _ = cmd.spawn()?.wait();

    Ok(())
}

fn maybe_append_docker_compose_override<'a>(
    cmd: &'a mut Command,
    name: &str,
    project: &'a mut Project,
) -> &'a mut Command {
    match project.find_service(name) {
        Ok(ref mut service) => match service.devbox_compose_file().to_str() {
            Some(override_file) => cmd.args(&["-f", override_file]),
            None => cmd,
        },
        Err(_) => cmd,
    }
}
