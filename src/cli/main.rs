// SPDX-License-Identifier: GPL-3.0-or-later

mod apply;
mod error;
mod merge;
mod show;
mod state;
mod wait;

use nm::NmClient;

pub(crate) use self::error::CliError;
use self::{
    apply::CommandApply, merge::CommandMerge, show::CommandShow,
    wait::CommandWait,
};

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), CliError> {
    let mut cli_cmd = clap::Command::new("nipc")
        .about("NetworkManager CLI")
        .arg_required_else_help(true)
        .subcommand_required(true)
        .arg(
            clap::Arg::new("quiet")
                .short('q')
                .action(clap::ArgAction::SetTrue)
                .help("Disable logging")
                .global(true),
        )
        .arg(
            clap::Arg::new("verbose")
                .short('v')
                .action(clap::ArgAction::Count)
                .help("Increase verbose level")
                .global(true),
        )
        .subcommand(clap::Command::new("ping").about("Check daemon connection"))
        .subcommand(CommandWait::new_cmd())
        .subcommand(CommandShow::new_cmd())
        .subcommand(CommandApply::new_cmd())
        .subcommand(CommandMerge::new_cmd());

    let matches = cli_cmd.get_matches_mut();

    let (log_groups, log_level) = match matches.get_count("verbose") {
        0 => (vec!["nm", "nmstate"], log::LevelFilter::Info),
        1 => (vec!["nm", "nmstate"], log::LevelFilter::Debug),
        2 => (vec!["nm", "nmstate"], log::LevelFilter::Trace),
        _ => (vec![""], log::LevelFilter::Trace),
    };

    if !matches.get_flag("quiet") {
        let mut log_builder = env_logger::Builder::new();
        if log_groups.is_empty() {
            log_builder.filter(None, log_level);
        } else {
            for log_group in log_groups {
                log_builder.filter(Some(log_group), log_level);
            }
        }
        log_builder.init();
    }

    log::info!("nmcli version: {}", clap::crate_version!());

    if let Err(e) = call_subcommand(&matches).await {
        eprintln!("{e}");
        std::process::exit(1);
    }

    Ok(())
}

async fn call_subcommand(matches: &clap::ArgMatches) -> Result<(), CliError> {
    if matches.subcommand_matches("ping").is_some() {
        let mut cli = NmClient::new().await?;
        println!("{}", cli.ping().await?);
        Ok(())
    } else if let Some(matches) = matches.subcommand_matches(CommandWait::CMD) {
        CommandWait::handle(matches).await?;
        Ok(())
    } else if let Some(matches) = matches.subcommand_matches(CommandShow::CMD) {
        CommandShow::handle(matches).await?;
        Ok(())
    } else if let Some(matches) = matches.subcommand_matches(CommandApply::CMD)
    {
        CommandApply::handle(matches).await?;
        Ok(())
    } else if let Some(matches) = matches.subcommand_matches(CommandMerge::CMD)
    {
        CommandMerge::handle(matches).await?;
        Ok(())
    } else {
        Err(CliError::from("Unknown command"))
    }
}
