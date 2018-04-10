use std::env;
use std::ffi::OsStr;
use std::io::prelude::*;
use std::process::{Command, Stdio};

use clap::ArgMatches;
use failure::ResultExt;
use rayon::prelude::*;

use cli;
use errors::*;
use project::{self, Project};
use service::Service;

pub fn new<S: AsRef<OsStr>>(program: S, service: &Service) -> Command {
    let devbox_compose_file = env::var("COMPOSE_FILE").unwrap();
    let service_compose_file = service.devbox_compose_file();

    let mut cmd = Command::new(program);

    cmd.arg("-f")
        .arg(&devbox_compose_file)
        .arg("-f")
        .arg(&service_compose_file)
        .arg("--project-directory")
        .arg(&service_compose_file.parent().unwrap());

    cmd
}

pub fn new_project(args: &ArgMatches) -> Result<()> {
    let name = args.value_of("PROJECT")
        .ok_or_else(|| format_err!("Missing project name"))?;
    match args.value_of("git") {
        Some(repo) => project::init_from_git(name, repo),
        None => project::init(name)
    }
}

pub fn build(args: &ArgMatches) -> Result<()> {
    let mut project = project_from_args(args)?;

    if let Some(name) = args.value_of("SERVICE") {
        let mut service = project.find_service(name)?;
        let _ = service.clone_repo();
        service.build()
    } else {
        let _ = clone_services(&mut project);
        let _ = build_services(&mut project);
        let _ = create_network(&project);
        let _ = create_volumes(&project);
        let _ = pull_latest_images();
        let _ = build_images();

        Ok(())
    }
}

pub fn completions(args: &ArgMatches) -> Result<()> {
    let shell = args.value_of("SHELL")
        .ok_or_else(|| format_err!("Missing `SHELL` argument"))?
        .parse()
        .map_err(|err| format_err!("{}", err))
        .context("Unable to parse `SHELL` argument")?;

    cli::new().gen_completions_to("devbox", shell, &mut ::std::io::stdout());

    Ok(())
}

pub fn doctor(_args: &ArgMatches) -> Result<()> {
    print_command_status("docker");
    print_command_status("docker-compose");

    Ok(())
}

pub fn logs(args: &ArgMatches) -> Result<()> {
    let mut project = project_from_args(args)?;

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

pub fn ps(_args: &ArgMatches) -> Result<()> {
    let _ = docker()
        .args(&[
            "ps",
            "--format",
            "table {{.ID}}\t{{.Names}}\t{{.Status}}\t{{.Ports}}",
        ])
        .spawn()?
        .wait();
    Ok(())
}

pub fn start(args: &ArgMatches) -> Result<()> {
    let mut project = project_from_args(args)?;

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

pub fn stop(args: &ArgMatches) -> Result<()> {
    let mut project = project_from_args(args)?;

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

pub fn tasks(matches: &ArgMatches) -> Result<()> {
    let mut project = project_from_args(matches)?;

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

pub fn update(args: &ArgMatches) -> Result<()> {
    let mut project = project_from_args(args)?;
    let name = args.value_of("SERVICE")
        .ok_or_else(|| format_err!("Error parsing `SERVICE`"))?;
    let service = project.find_service(name)?;
    service.update()
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

fn clone_services(project: &mut Project) -> Result<()> {
    project
        .services
        .par_iter_mut()
        .map(|ref mut service| service.clone_repo())
        .collect::<Vec<Result<()>>>();
    Ok(())
}

fn build_services(project: &mut Project) -> Result<()> {
    project
        .services
        .par_iter_mut()
        .map(|ref mut service| service.build())
        .collect::<Vec<Result<()>>>();
    Ok(())
}

fn create_network(project: &Project) -> Result<()> {
    println!("\nCreating '{}' network", &project.name);

    let _ = docker()
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .args(&["network", "create", &project.name])
        .spawn()?
        .wait();
    Ok(())
}

fn create_volumes(project: &Project) -> Result<()> {
    println!("\nCreating volumes");
    project
        .volumes
        .par_iter()
        .map(|s| create_volume(s.as_str()))
        .collect::<Vec<Result<()>>>();
    Ok(())
}

fn create_volume(name: &str) -> Result<()> {
    println!("Creating volume: {}", name);
    let _ = docker()
        .stdout(Stdio::null())
        .args(&["volume", "create", "--name", name])
        .spawn()?
        .wait();
    Ok(())
}

fn pull_latest_images() -> Result<()> {
    println!("\nPulling latest images...");
    let _ = docker_compose().args(&["pull"]).spawn()?.wait();
    Ok(())
}

fn build_images() -> Result<()> {
    println!("\nBuilding images...");
    let _ = docker_compose().args(&["build"]).spawn()?.wait();
    Ok(())
}

fn destroy_environment() -> Result<()> {
    let _ = docker_compose()
        .args(&["down", "-v", "--remove-orphans"])
        .spawn()?
        .wait();
    Ok(())
}

fn remove_images() -> Result<()> {
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

fn project_from_args(args: &ArgMatches) -> Result<Project> {
    let name = args.value_of("PROJECT")
        .ok_or_else(|| format_err!("Project name required"))?;
    Project::new(name)
}

fn docker() -> Command {
    Command::new("docker")
}

fn docker_compose() -> Command {
    Command::new("docker-compose")
}

fn print_command_status(command: &str) {
    use colored::Colorize;

    if command_exists(command) {
        println!("{} {} was found in PATH", "✔".green(), command);
    } else {
        println!("{} {} was not found in PATH", "✘".red(), command);
    }
}

fn command_exists(command: &str) -> bool {
    match Command::new(command)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
    {
        Ok(_) => true,
        Err(_) => false,
    }
}
