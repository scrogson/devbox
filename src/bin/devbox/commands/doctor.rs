use crate::prelude::*;

pub fn cli() -> App {
    subcommand("doctor").about("Check your system for potential problems")
}

pub fn exec(_args: &ArgMatches<'_>) -> CliResult {
    print_command_status("docker");
    print_command_status("docker-compose");

    Ok(())
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
