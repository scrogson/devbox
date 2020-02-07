use crate::cli;
use crate::prelude::*;

pub fn cli() -> App {
    subcommand("completions")
        .about("Generates completions for your shell")
        .arg(
            Arg::with_name("SHELL")
                .required(true)
                .possible_values(&["bash", "fish", "zsh"])
                .help("The shell to generate the script for"),
        )
}

pub fn exec(args: &ArgMatches<'_>) -> CliResult {
    let shell = args.value_of("SHELL")
        .ok_or_else(|| format_err!("Missing `SHELL` argument"))?
        .parse()
        .map_err(|err| format_err!("{}", err))
        .context("Unable to parse `SHELL` argument")?;

    cli::cli().gen_completions_to("devbox", shell, &mut ::std::io::stdout());

    Ok(())
}
