// SPDX-License-Identifier: Apache-2.0

use nm::{
    ErrorKind, Interface, LinkEvent, MergedNetworkState, NetworkState, NmError,
    NmNoDaemon, NmstateApplyOption, NmstateQueryOption, WifiPhyInterface,
};

use super::share_data::NmDaemonShareData;

pub(crate) async fn handle_link_event(
    event: LinkEvent,
    mut share_data: NmDaemonShareData,
) -> Result<(), NmError> {
    let iface_name = event.iface_name();
    let saved_state = share_data.conf_manager.query_state().await?;
    let cur_state =
        NmNoDaemon::query_network_state(NmstateQueryOption::running()).await?;

    if let Some(cur_iface) = cur_state.ifaces.kernel_ifaces.get(iface_name) {
        match cur_iface {
            Interface::WifiPhy(wifi_phy_iface) => {
                handle_wifi_phy_iface(
                    &event,
                    wifi_phy_iface,
                    &saved_state,
                    &cur_state,
                    &mut share_data,
                )
                .await?;
            }
            _ => {
                log::warn!(
                    "handle_link_event: unsupported interface {cur_iface}"
                );
            }
        }
    }

    Ok(())
}

async fn handle_wifi_phy_iface(
    event: &LinkEvent,
    cur_iface: &WifiPhyInterface,
    saved_state: &NetworkState,
    cur_state: &NetworkState,
    share_data: &mut NmDaemonShareData,
) -> Result<(), NmError> {
    if let Some(ssid) =
        cur_iface.wifi_link.as_ref().and_then(|w| w.ssid.as_ref())
    {
        if let Some(wifi_cfg_iface) =
            saved_state.ifaces.user_ifaces.values().find_map(|i| {
                if let Interface::WifiCfg(wifi_cfg_iface) = i {
                    if wifi_cfg_iface
                        .wifi
                        .as_ref()
                        .and_then(|w| w.ssid.as_deref())
                        == Some(ssid)
                    {
                        Some(wifi_cfg_iface)
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
        {
            let mut new_iface = cur_iface.clone();
            if event.is_link_up() {
                new_iface.base.ipv4 = wifi_cfg_iface.base.ipv4.clone();
                new_iface.base.ipv6 = wifi_cfg_iface.base.ipv6.clone();
            } else if event.is_link_down() {
                new_iface.base.ipv4 = Some(Default::default());
                new_iface.base.ipv6 = Some(Default::default());
            } else {
                return Err(NmError::new(
                    ErrorKind::Bug,
                    format!("Unsupported link event {event}"),
                ));
            }
            new_iface.wifi_link = None;
            let mut new_state = NetworkState::default();
            new_state
                .ifaces
                .push(Interface::WifiPhy(Box::new(new_iface)));
            let merged_state = MergedNetworkState::new(
                new_state,
                cur_state.clone(),
                NmstateApplyOption::new().no_verify(),
            )?;

            NmNoDaemon::apply_merged_state(&merged_state).await?;
            share_data
                .dhcpv4_manager
                .apply_dhcp_config(None, &merged_state)
                .await?;
        }
    }
    Ok(())
}
