use crate::prelude::*;

pub fn builtins() -> Vec<App> {
    vec![
        build::cli(),
        completions::cli(),
        doctor::cli(),
        logs::cli(),
        new::cli(),
        ps::cli(),
        start::cli(),
        stop::cli(),
        tasks::cli(),
        update::cli(),
    ]
}

pub fn builtin_exec(cmd: &str) -> Option<fn(&ArgMatches<'_>) -> CliResult> {
    let f = match cmd {
        "build" => build::exec,
        "completions" => completions::exec,
        "doctor" => doctor::exec,
        "logs" => logs::exec,
        "new" => new::exec,
        "ps" => ps::exec,
        "start" => start::exec,
        "stop" => stop::exec,
        "tasks" => tasks::exec,
        "update" => update::exec,
        _ => return None,
    };
    Some(f)
}

pub mod build;
pub mod completions;
pub mod doctor;
pub mod logs;
pub mod new;
pub mod ps;
pub mod start;
pub mod stop;
pub mod tasks;
pub mod update;
