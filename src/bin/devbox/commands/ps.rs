use crate::prelude::*;

pub fn cli() -> App {
    subcommand("ps").about("Display running services")
}

pub fn exec(_args: &ArgMatches) -> CliResult {
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
