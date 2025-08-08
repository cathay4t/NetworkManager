// SPDX-License-Identifier: Apache-2.0

use nm::{NmClient, NmNoDaemon};

use crate::CliError;

pub(crate) struct CommandShow;

impl CommandShow {
    pub(crate) const CMD: &str = "show";

    pub(crate) fn new_cmd() -> clap::Command {
        clap::Command::new("show")
            .alias("s")
            .about("Query network state")
            .arg(
                clap::Arg::new("NO_DAEMON")
                    .long("no-daemon")
                    .short('n')
                    .action(clap::ArgAction::SetTrue)
                    .help("Do not connect to NetworkManager daemon"),
            )
    }

    pub(crate) async fn handle(
        matches: &clap::ArgMatches,
    ) -> Result<(), CliError> {
        let net_state = if matches.get_flag("NO_DAEMON") {
            NmNoDaemon::query_network_state(Default::default()).await?
        } else {
            let mut cli = NmClient::new().await?;
            cli.query_network_state(Default::default()).await?
        };
        println!("{}", serde_yaml::to_string(&net_state)?);

        Ok(())
    }
}
