use prelude::*;
use rayon::prelude::*;

pub fn cli() -> App {
    subcommand("build")
        .about("Build infrastructure")
        .arg(Arg::with_name("SERVICE").help("The name of the service to build"))
        .arg(project())
}

pub fn exec(args: &ArgMatches) -> CliResult {
    let mut project = args.project()?;

    if let Some(name) = args.value_of("SERVICE") {
        let service = project.find_service(name)?;
        let _ = service.clone_repo();
        service.build()
    } else {
        let _ = create_network(&project);
        let _ = create_volumes(&project);
        let _ = pull_latest_images();
        let _ = build_images();
        let _ = clone_services(&mut project);
        let _ = build_services(&mut project);

        Ok(())
    }
}

fn create_network(project: &Project) -> CliResult {
    println!("\nCreating '{}' network", &project.name);

    let _ = docker()
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .args(&["network", "create", &project.name])
        .spawn()?
        .wait();
    Ok(())
}

fn create_volumes(project: &Project) -> CliResult {
    println!("\nCreating volumes");
    project
        .volumes
        .par_iter()
        .map(|s| create_volume(s.as_str()))
        .collect::<Vec<CliResult>>();
    Ok(())
}

fn create_volume(name: &str) -> CliResult {
    println!("Creating volume: {}", name);
    let _ = docker()
        .stdout(Stdio::null())
        .args(&["volume", "create", "--name", name])
        .spawn()?
        .wait();
    Ok(())
}

fn pull_latest_images() -> CliResult {
    println!("\nPulling latest images...");
    let _ = docker_compose().args(&["pull"]).spawn()?.wait();
    Ok(())
}

fn build_images() -> CliResult {
    println!("\nBuilding images...");
    let _ = docker_compose().args(&["build"]).spawn()?.wait();
    Ok(())
}

fn clone_services(project: &mut Project) -> CliResult {
    project
        .services
        .par_iter_mut()
        .map(|ref mut service| service.clone_repo())
        .collect::<Vec<CliResult>>();
    Ok(())
}

fn build_services(project: &mut Project) -> CliResult {
    project
        .services
        .par_iter_mut()
        .map(|ref mut service| service.build())
        .collect::<Vec<CliResult>>();
    Ok(())
}
