#[macro_use]
extern crate clap;
extern crate colored;
extern crate devbox;
#[macro_use]
extern crate failure;
extern crate rayon;

use devbox::*;

mod cli;
mod commands;
mod prelude;

fn run() -> Result<()> {
    cli::main()
}

fn main() {
    if let Err(ref err) = run() {
        use colored::*;

        eprintln!("{}", "ERROR:".red());
        err.causes()
            .for_each(|cause| eprintln!("{}", format!("{}", cause).red()));

        std::process::exit(1);
    }
}
