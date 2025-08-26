// SPDX-License-Identifier: Apache-2.0

use nm::{NmClient, NmNoDaemon, NmstateQueryOption};

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
            .arg(
                clap::Arg::new("SAVED")
                    .long("saved")
                    .short('s')
                    .action(clap::ArgAction::SetTrue)
                    .help("Show the daemon saved state only"),
            )
    }

    pub(crate) async fn handle(
        matches: &clap::ArgMatches,
    ) -> Result<(), CliError> {
        let net_state = if matches.get_flag("NO_DAEMON") {
            if matches.get_flag("SAVED") {
                return Err(
                    "--no-daemon cannot be used with --saved argument".into()
                );
            }
            NmNoDaemon::query_network_state(Default::default()).await?
        } else {
            let mut cli = NmClient::new().await?;
            let opt = if matches.get_flag("SAVED") {
                NmstateQueryOption::saved()
            } else {
                NmstateQueryOption::running()
            };
            cli.query_network_state(opt).await?
        };
        println!("{}", serde_yaml::to_string(&net_state)?);

        Ok(())
    }
}
