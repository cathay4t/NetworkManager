// SPDX-License-Identifier: Apache-2.0

use nm::{NmClient, NmNoDaemon, NmstateApplyOption};

use super::{CliError, state::state_from_file};

pub(crate) struct CommandApply;

impl CommandApply {
    pub(crate) const CMD: &str = "apply";

    pub(crate) fn new_cmd() -> clap::Command {
        clap::Command::new("apply")
            .alias("a")
            .about("Apply network state")
            .arg(
                clap::Arg::new("STATE_FILE")
                    .required(false)
                    .index(1)
                    .help("Network state file"),
            )
            .arg(
                clap::Arg::new("NO_VERIFY")
                    .long("no-verify")
                    .action(clap::ArgAction::SetTrue)
                    .help(
                        "Do not verify that the state was completely set and \
                         disable rollback to previous state.",
                    ),
            )
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
        let mut opt = NmstateApplyOption::default();
        opt.no_verify = matches.get_flag("NO_VERIFY");

        let desired_state =
            if let Some(file_path) = matches.get_one::<String>("STATE_FILE") {
                state_from_file(file_path)?
            } else {
                state_from_file("-")?
            };

        let diff_net_state = if matches.get_flag("NO_DAEMON") {
            NmNoDaemon::apply_network_state(desired_state, opt).await?
        } else {
            let mut cli = NmClient::new().await?;
            cli.apply_network_state(desired_state, opt).await?
        };
        if diff_net_state.is_empty() {
            println!("Nothing changed");
        } else {
            println!(
                "Changed state:\n---\n{}",
                serde_yaml::to_string(&diff_net_state)?
            );
        }

        Ok(())
    }
}
