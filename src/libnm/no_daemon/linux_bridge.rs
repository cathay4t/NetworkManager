// SPDX-License-Identifier: Apache-2.0

// This file is based on the work of nmstate project(https://nmstate.io/) which
// is under license of Apache 2.0, authors of original file
// (rust/src/lib/nispor/linux_bridge.rs) are:
//  * Gris Ge <fge@redhat.com>
//  * Fernando Fernandez Mancera <ffmancera@riseup.net>
//  * Íñigo Huguet <ihuguet@redhat.com>

use super::linux_bridge_port_vlan::parse_port_vlan_conf;
use crate::{
    BaseInterface, LinuxBridgeConfig, LinuxBridgeInterface,
    LinuxBridgeMulticastRouterType, LinuxBridgeOptions, LinuxBridgePortConfig,
    LinuxBridgeStpOptions, NmError, VlanProtocol,
};

impl From<&nispor::BridgeVlanProtocol> for VlanProtocol {
    fn from(v: &nispor::BridgeVlanProtocol) -> Self {
        match v {
            nispor::BridgeVlanProtocol::Ieee8021Q => VlanProtocol::Ieee8021Q,
            nispor::BridgeVlanProtocol::Ieee8021AD => VlanProtocol::Ieee8021Ad,
            _ => {
                log::debug!("Unsupported linux bridge vlan protocol {v:?}");
                VlanProtocol::Unknown
            }
        }
    }
}

impl From<&nispor::BridgePortMulticastRouterType>
    for LinuxBridgeMulticastRouterType
{
    fn from(v: &nispor::BridgePortMulticastRouterType) -> Self {
        match v {
            nispor::BridgePortMulticastRouterType::Disabled => {
                LinuxBridgeMulticastRouterType::Disabled
            }
            nispor::BridgePortMulticastRouterType::TempQuery => {
                LinuxBridgeMulticastRouterType::Auto
            }
            nispor::BridgePortMulticastRouterType::Perm => {
                LinuxBridgeMulticastRouterType::Enabled
            }
            _ => {
                log::debug!("Unsupported linux bridge multicast router {v:?}");
                LinuxBridgeMulticastRouterType::Unknown
            }
        }
    }
}

impl From<&nispor::BridgeInfo> for LinuxBridgeConfig {
    fn from(np_bridge: &nispor::BridgeInfo) -> Self {
        Self {
            options: Some(LinuxBridgeOptions {
                gc_timer: np_bridge.gc_timer,
                group_addr: np_bridge
                    .group_addr
                    .as_ref()
                    .map(|s| s.to_uppercase()),
                group_forward_mask: np_bridge.group_fwd_mask,
                group_fwd_mask: np_bridge.group_fwd_mask,
                hash_max: np_bridge.multicast_hash_max,
                hello_timer: np_bridge.hello_timer,
                mac_ageing_time: np_bridge.ageing_time.map(devide_by_user_hz),
                multicast_last_member_count: np_bridge
                    .multicast_last_member_count,
                multicast_last_member_interval: np_bridge
                    .multicast_last_member_interval,
                multicast_membership_interval: np_bridge
                    .multicast_membership_interval,
                multicast_querier: np_bridge.multicast_querier,
                multicast_querier_interval: np_bridge
                    .multicast_querier_interval,
                multicast_query_interval: np_bridge.multicast_query_interval,
                multicast_query_response_interval: np_bridge
                    .multicast_query_response_interval,
                multicast_query_use_ifaddr: np_bridge
                    .multicast_query_use_ifaddr,
                multicast_router: np_bridge
                    .multicast_router
                    .as_ref()
                    .map(|r| r.into()),
                multicast_snooping: np_bridge.multicast_snooping,
                multicast_startup_query_count: np_bridge
                    .multicast_startup_query_count,
                multicast_startup_query_interval: np_bridge
                    .multicast_startup_query_interval,
                stp: Some(get_stp_options(np_bridge)),
                vlan_protocol: np_bridge
                    .vlan_protocol
                    .as_ref()
                    .map(|p| p.into()),
                vlan_default_pvid: np_bridge.default_pvid,
            }),
            ports: Some(
                np_bridge
                    .ports
                    .as_slice()
                    .iter()
                    .map(|iface_name| LinuxBridgePortConfig {
                        name: iface_name.to_string(),
                        ..Default::default()
                    })
                    .collect(),
            ),
        }
    }
}

pub(crate) fn apply_bridge_conf(
    _np_iface: nispor::IfaceConf,
    _iface: &LinuxBridgeInterface,
    _cur_iface: Option<&LinuxBridgeInterface>,
) -> Result<Vec<nispor::IfaceConf>, NmError> {
    todo!()
}

impl LinuxBridgeInterface {
    pub(crate) fn new_from_nispor(
        base_iface: BaseInterface,
        np_iface: &nispor::Iface,
    ) -> Self {
        if let Some(np_bridge_conf) = np_iface.bridge.as_ref() {
            Self {
                base: base_iface,
                bridge: Some(np_bridge_conf.into()),
            }
        } else {
            Self {
                base: base_iface,
                ..Default::default()
            }
        }
    }

    pub(crate) fn append_br_port_config(
        &mut self,
        port_np_ifaces: Vec<&nispor::Iface>,
    ) {
        let mut port_confs: Vec<LinuxBridgePortConfig> = Vec::new();
        for port_np_iface in port_np_ifaces {
            let mut port_conf = LinuxBridgePortConfig {
                name: port_np_iface.name.to_string(),
                stp_hairpin_mode: port_np_iface
                    .bridge_port
                    .as_ref()
                    .map(|i| i.hairpin_mode),
                stp_path_cost: port_np_iface
                    .bridge_port
                    .as_ref()
                    .map(|i| i.stp_path_cost),
                stp_priority: port_np_iface
                    .bridge_port
                    .as_ref()
                    .map(|i| i.stp_priority),
                ..Default::default()
            };

            if self.vlan_filtering_is_enabled()
                && let Some(np_port_info) = port_np_iface.bridge_port.as_ref()
            {
                port_conf.vlan = np_port_info.vlans.as_ref().and_then(|v| {
                    parse_port_vlan_conf(
                        v.as_slice(),
                        self.bridge
                            .as_ref()
                            .and_then(|br_conf| br_conf.options.as_ref())
                            .and_then(|br_opts| br_opts.vlan_default_pvid),
                    )
                });
            }
            port_confs.push(port_conf);
        }

        if let Some(br_conf) = self.bridge.as_mut() {
            br_conf.ports = Some(port_confs);
        }
    }
}

const DEFAULT_USER_HZ: u32 = 100;

// The kernel is multiplying these bridge properties by USER_HZ, we should
// divide into seconds:
//   * forward_delay
//   * ageing_time
//   * hello_time
//   * max_age
fn devide_by_user_hz(v: u32) -> u32 {
    if let Ok(Some(user_hz)) =
        nix::unistd::sysconf(nix::unistd::SysconfVar::CLK_TCK)
        && user_hz > 0
    {
        v / user_hz as u32
    } else {
        v / DEFAULT_USER_HZ
    }
}

fn get_stp_options(np_bridge: &nispor::BridgeInfo) -> LinuxBridgeStpOptions {
    LinuxBridgeStpOptions {
        enabled: Some(
            [
                Some(nispor::BridgeStpState::KernelStp),
                Some(nispor::BridgeStpState::UserStp),
            ]
            .contains(&np_bridge.stp_state),
        ),
        forward_delay: np_bridge.forward_delay.map(devide_by_user_hz).map(
            |v| {
                u8::try_from(v)
                    .unwrap_or(LinuxBridgeStpOptions::FORWARD_DELAY_MAX)
            },
        ),
        max_age: np_bridge.max_age.map(devide_by_user_hz).map(|v| {
            u8::try_from(v).unwrap_or(LinuxBridgeStpOptions::MAX_AGE_MAX)
        }),

        hello_time: np_bridge.hello_time.map(devide_by_user_hz).map(|v| {
            u8::try_from(v).unwrap_or(LinuxBridgeStpOptions::HELLO_TIME_MAX)
        }),
        priority: np_bridge.priority,
    }
}
