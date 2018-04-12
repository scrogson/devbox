use prelude::*;
use std::io::prelude::*;

pub fn cli() -> App {
    subcommand("stop")
        .about("Stop infrastructure or service")
        .arg(Arg::with_name("SERVICE").help("The name of the service to stop"))
        .arg(project())
}

pub fn exec(args: &ArgMatches) -> CliResult {
    let mut project = args.project()?;

    match args.value_of("SERVICE") {
        Some(name) => {
            let service = project.find_service(name)?;
            service.stop()
        }
        None => {
            let _ = destroy_environment();
            let _ = remove_images();
            Ok(())
        }
    }
}

fn destroy_environment() -> CliResult {
    let _ = docker_compose()
        .args(&["down", "-v", "--remove-orphans"])
        .spawn()?
        .wait();
    Ok(())
}

fn remove_images() -> CliResult {
    let images = docker()
        .args(&["images", "-q", "-f", "dangling=true"])
        .output()?;

    let command = Command::new("xargs")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .args(&["-I", "ARGS", "docker", "rmi", "-f", "ARGS"])
        .spawn()?;

    command
        .stdin
        .ok_or_else(|| format_err!("Failed to read stdin"))?
        .write_all(&images.stdout)?;

    Ok(())
}

