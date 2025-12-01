// SPDX-License-Identifier: GPL-3.0-or-later

use nm::NmNoDaemon;

use crate::CliError;

pub(crate) struct CommandWifi;

impl CommandWifi {
    pub(crate) const CMD: &str = "wifi";

    pub(crate) fn new_cmd() -> clap::Command {
        clap::Command::new("wifi")
            .about("WIFI actions")
            .subcommand_required(true)
            .subcommand(
                clap::Command::new("scan").about("WIFI active scan").arg(
                    clap::Arg::new("IFACE")
                        .required(false)
                        .index(1)
                        .help("Scan on specified interface only"),
                ),
            )
    }

    pub(crate) async fn handle(
        matches: &clap::ArgMatches,
    ) -> Result<(), CliError> {
        if let Some(matches) = matches.subcommand_matches("scan") {
            let iface_name =
                matches.get_one::<String>("IFACE").map(|s| s.as_str());
            let mut wifi_cfgs = NmNoDaemon::wifi_scan(iface_name).await?;
            wifi_cfgs.sort_unstable_by_key(|wifi_cfg| wifi_cfg.signal_percent);
            wifi_cfgs.reverse();
            println!("{}", serde_yaml::to_string(&wifi_cfgs)?);
        }
        Ok(())
    }
}
