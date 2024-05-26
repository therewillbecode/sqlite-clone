mod metacommand;

use crate::repl::metacommand::handle_metacommand;
use anyhow::{anyhow, bail, Context, Result};
use clap::Command;
use clap::{Parser, Subcommand};
use std::io::Write;

pub fn repl_loop() -> Result<()> {
    loop {
        let line: String = readline()?;
        let line: &str = line.trim();
        if line.is_empty() {
            continue;
        }

        match respond(line) {
            Ok(quit) => {
                if quit {
                    break;
                }
            }
            Err(err) => {
                write!(std::io::stdout(), "{err}").context("failed to write err to std out")?;

                std::io::stdout()
                    .flush()
                    .context("failed to flush std out")?;
            }
        }
    }

    Ok(())
}

fn readline() -> Result<String> {
    write!(std::io::stdout(), "\n$ ").context("failed to write $ to stdout")?;
    std::io::stdout()
        .flush()
        .context("failed to flush std out")?;
    let mut buffer: String = String::new();
    std::io::stdin()
        .read_line(&mut buffer)
        .context("failed to read line from stdin")?;
    Ok(buffer)
}

fn respond(line: &str) -> Result<bool> {
    let args: Vec<String> = shlex::split(line)
        //.ok_or("error: Invalid quoting")
        .context("invalid quoting on args")?;
    let matches = cli()
        .try_get_matches_from(args)
        .context("failed to get matches for cli from args")?;

    match matches.subcommand() {
        Some(("ping", _matches)) => {
            write!(std::io::stdout(), "Pong").context("failed to write to std out")?;
            std::io::stdout()
                .flush()
                .context("failed to flush std out")?;
        }
        Some((".exit", _matches)) => {
            write!(std::io::stdout(), "Exiting ...").context("failed to write to std out")?;
            std::io::stdout()
                .flush()
                .context("failed to flush std out")?;
            return Ok(true);
        }
        Some((cmd, _matches)) if cmd.starts_with('.') => {
            write!(std::io::stdout(), "calling metacommand: \n")
                .context("failed to write to std out")?;
            let res = &handle_metacommand(cmd)?;
            write!(std::io::stdout(), "{res}").context("failed to flush std out")?;
            std::io::stdout()
                .flush()
                .context("failed to flush std out")?;
        }
        Some((name, _matches)) => unimplemented!("{name}"),
        None => unreachable!("subcommand required"),
    }

    Ok(false)
}

fn cli() -> Command {
    // strip out usage
    const PARSER_TEMPLATE: &str = "\
        {all-args}
    ";
    // strip out name/version
    const APPLET_TEMPLATE: &str = "\
        {about-with-newline}\n\
        {usage-heading}\n    {usage}\n\
        \n\
        {all-args}{after-help}\
    ";

    Command::new("repl")
        .multicall(true)
        .arg_required_else_help(true)
        .subcommand_required(true)
        .subcommand_value_name("APPLET")
        .subcommand_help_heading("APPLETS")
        .help_template(PARSER_TEMPLATE)
        .subcommand(
            Command::new("ping")
                .about("Get a response")
                .help_template(APPLET_TEMPLATE),
        )
        .subcommand(
            Command::new(".tables")
                .about("Get tables")
                .help_template(APPLET_TEMPLATE),
        )
        .subcommand(
            Command::new(".schema")
                .about("Get Schema")
                .help_template(APPLET_TEMPLATE),
        )
        .subcommand(
            Command::new(".indexes")
                .about("Get Indexes")
                .help_template(APPLET_TEMPLATE),
        )
        .subcommand(
            Command::new(".exit")
                .alias("exit")
                .about("Quit the REPL")
                .help_template(APPLET_TEMPLATE),
        )
}
