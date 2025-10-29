// SPDX-License-Identifier: GPL-3.0-or-later

use std::pin::Pin;

use nm::{NmError, NmNoDaemon};

use crate::CliError;

pub(crate) struct CommandWait;

impl CommandWait {
    pub(crate) const CMD: &str = "wait";

    pub(crate) fn new_cmd() -> clap::Command {
        clap::Command::new("wait")
            .alias("w")
            .about("Wait network interface reach expected state")
            .arg(
                clap::Arg::new("IFNAME")
                    .required(true)
                    .index(1)
                    .help("Wait specific interface only"),
            )
            .arg(
                clap::Arg::new("STATE")
                    .value_parser(clap::builder::PossibleValuesParser::new([
                        "up", "down",
                    ]))
                    .required(true)
                    .index(2)
                    .help("State to wait"),
            )
            .arg(
                clap::Arg::new("TIMEOUT_SEC")
                    .long("timeout")
                    .short('s')
                    .action(clap::ArgAction::Set)
                    .help("Maximum wait time in seconds"),
            )
    }

    pub(crate) async fn handle(
        matches: &clap::ArgMatches,
    ) -> Result<(), CliError> {
        let iface_name = matches
            .get_one::<String>("IFNAME")
            .ok_or(CliError::from("No interface defined".to_string()))?;

        let desired_state = matches
            .get_one::<String>("STATE")
            .ok_or(CliError::from("No state defined".to_string()))?;

        let future: Pin<Box<dyn Future<Output = Result<(), NmError>>>> =
            match desired_state.as_str() {
                "up" => Box::pin(NmNoDaemon::wait_link_carrier_up(iface_name)),
                "down" => {
                    Box::pin(NmNoDaemon::wait_link_carrier_down(iface_name))
                }
                state => {
                    return Err(CliError::from(format!(
                        "Unsupported state to wait: {state}"
                    )));
                }
            };
        log::info!(
            "Waiting {iface_name} to reach desired state {desired_state}"
        );
        match matches.get_one::<String>("TIMEOUT_SEC") {
            Some(tmo_sec_str) => {
                let tmo_sec = tmo_sec_str.parse::<u32>().map_err(|e| {
                    CliError::from(format!(
                        "Invalid timeout {tmo_sec_str}: {e}"
                    ))
                })?;
                match tokio::time::timeout(
                    std::time::Duration::from_secs(tmo_sec.into()),
                    future,
                )
                .await
                {
                    Ok(result) => {
                        result?;
                    }
                    Err(_) => {
                        return Err(CliError::from(format!(
                            "Timeout ({tmo_sec} secs) on waiting interface \
                             {iface_name} to reach desired state \
                             {desired_state}"
                        )));
                    }
                }
            }
            None => {
                future.await?;
            }
        }
        log::info!(
            "Interface {iface_name} reached desired state {desired_state}",
        );
        Ok(())
    }
}
