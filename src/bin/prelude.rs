use clap::{self, SubCommand};
pub use std::process::{Command, Stdio};
pub use devbox::{Project, Result};
pub use clap::{AppSettings, ArgMatches};
pub use failure::ResultExt;

pub type App = clap::App<'static, 'static>;
pub type Arg = clap::Arg<'static, 'static>;
pub type CliResult = Result<()>;


pub trait ArgMatchesExt {
    fn project(&self) -> Result<Project> {
        let name = self._value_of("PROJECT")
            .ok_or_else(|| format_err!("Project name required"))?;
        Project::new(name)
    }

    fn _value_of(&self, name: &str) -> Option<&str>;
}

impl<'a> ArgMatchesExt for ArgMatches<'a> {
    fn _value_of(&self, name: &str) -> Option<&str> {
        self.value_of(name)
    }
}

pub fn project() -> Arg {
    Arg::with_name("PROJECT")
        .help("Project name")
        .short("p")
        .long("project")
        .env("DEVBOX_PROJECT")
        .required(true)
}

pub fn subcommand(name: &'static str) -> App {
    SubCommand::with_name(name).settings(&[
        AppSettings::UnifiedHelpMessage,
        AppSettings::DeriveDisplayOrder,
        AppSettings::DontCollapseArgsInUsage,
    ])
}

pub fn docker() -> Command {
    Command::new("docker")
}

pub fn docker_compose() -> Command {
    Command::new("docker-compose")
}
