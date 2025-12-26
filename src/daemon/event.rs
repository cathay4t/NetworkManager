// SPDX-License-Identifier: Apache-2.0

use std::time::SystemTime;

use nm::{
    ErrorKind, Interface, InterfaceState, InterfaceType, MergedNetworkState,
    NetworkState, NmError, NmNoDaemon, NmstateApplyOption, NmstateInterface,
    NmstateQueryOption, WifiPhyInterface,
};

use super::commander::NmCommander;

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct NmLinkEvent {
    pub iface_name: String,
    pub iface_type: InterfaceType,
    pub event_type: NmLinkEventType,
    pub time_stamp: SystemTime,
}

impl NmLinkEvent {
    pub(crate) fn new(
        iface_name: String,
        iface_type: InterfaceType,
        event_type: NmLinkEventType,
    ) -> Self {
        Self {
            iface_name,
            iface_type,
            event_type,
            time_stamp: SystemTime::now(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum NmLinkEventType {
    CarrierUp,
    CarrierDown,
}

impl NmLinkEvent {
    pub(crate) fn is_carrier_up(&self) -> bool {
        self.event_type == NmLinkEventType::CarrierUp
    }

    pub(crate) fn is_carrier_down(&self) -> bool {
        self.event_type == NmLinkEventType::CarrierDown
    }
}

impl NmCommander {
    pub(crate) async fn handle_link_event(
        &mut self,
        event: NmLinkEvent,
    ) -> Result<(), NmError> {
        let iface_name = event.iface_name.as_str();
        let saved_state = self.conf_manager.query_state().await?;
        let cur_state =
            NmNoDaemon::query_network_state(NmstateQueryOption::running())
                .await?;

        if let Some(cur_iface) = cur_state.ifaces.kernel_ifaces.get(iface_name)
        {
            match cur_iface {
                Interface::WifiPhy(wifi_phy_iface) => {
                    self.handle_wifi_phy_iface(
                        &event,
                        wifi_phy_iface,
                        &saved_state,
                        &cur_state,
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
        &mut self,
        event: &NmLinkEvent,
        cur_iface: &WifiPhyInterface,
        saved_state: &NetworkState,
        cur_state: &NetworkState,
    ) -> Result<(), NmError> {
        if let Some(ssid) = cur_iface.wifi.as_ref().map(|w| w.ssid.as_str()) {
            if let Some(wifi_cfg_iface) =
                saved_state.ifaces.user_ifaces.values().find_map(|i| {
                    if let Interface::WifiCfg(wifi_cfg_iface) = i {
                        if wifi_cfg_iface.wifi.as_ref().map(|w| w.ssid.as_str())
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
                new_iface.base_iface_mut().state = InterfaceState::Up;
                if event.is_carrier_up() {
                    new_iface.base.ipv4 = wifi_cfg_iface.base.ipv4.clone();
                    new_iface.base.ipv6 = wifi_cfg_iface.base.ipv6.clone();
                } else if event.is_carrier_down() {
                    new_iface.base.ipv4 = Some(Default::default());
                    new_iface.base.ipv6 = Some(Default::default());
                } else {
                    return Err(NmError::new(
                        ErrorKind::Bug,
                        format!("Unsupported link event {event:?}"),
                    ));
                }
                new_iface.wifi = None;
                let mut new_state = NetworkState::default();
                new_state
                    .ifaces
                    .push(Interface::WifiPhy(Box::new(new_iface)));
                let merged_state = MergedNetworkState::new(
                    new_state,
                    cur_state.clone(),
                    NmstateApplyOption::new().no_verify().memory_only(),
                )?;

                NmNoDaemon::apply_merged_state(&merged_state).await?;
                self.dhcpv4_manager
                    .apply_dhcp_config(None, &merged_state)
                    .await?;
            }
        } else {
            // New wifi NIC found
            let mut new_state = NetworkState::default();
            for wifi_cfg_iface in
                saved_state.ifaces.user_ifaces.values().filter_map(|i| {
                    if let Interface::WifiCfg(wifi_cfg_iface) = i {
                        Some(wifi_cfg_iface)
                    } else {
                        None
                    }
                })
            {
                if wifi_cfg_iface.parent().is_none()
                    || wifi_cfg_iface.parent() == Some(&event.iface_name)
                {
                    new_state.ifaces.push(Interface::WifiCfg(Box::new(
                        *wifi_cfg_iface.clone(),
                    )));
                }
            }
            if !new_state.is_empty() {
                let merged_state = MergedNetworkState::new(
                    new_state,
                    cur_state.clone(),
                    NmstateApplyOption::new().no_verify().memory_only(),
                )?;

                NmNoDaemon::apply_merged_state(&merged_state).await?;
            }
        }
        Ok(())
    }
}
