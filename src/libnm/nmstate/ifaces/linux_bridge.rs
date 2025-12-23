// SPDX-License-Identifier: Apache-2.0

// This file is based on the work of nmstate project(https://nmstate.io/) which
// is under license of Apache 2.0, authors of original file are:
//  * Gris Ge <fge@redhat.com>
//  * Fernando Fernandez Mancera <ffmancera@riseup.net>
//  * Quique Llorente <ellorent@redhat.com>

use serde::{Deserialize, Serialize};

use crate::{
    BaseInterface, BridgePortVlanConfig, ErrorKind, InterfaceType, JsonDisplay,
    NmError, NmstateInterface, VlanProtocol,
};

/// Bridge interface provided by linux kernel.
///
/// When serializing or deserializing, the [BaseInterface] will
/// be flatted and [LinuxBridgeConfig] stored as `bridge` section. The yaml
/// output [crate::NetworkState] containing an example linux bridge interface:
/// ```yml
/// interfaces:
/// - name: br0
///   type: linux-bridge
///   state: up
///   mac-address: 9A:91:53:6C:67:DA
///   mtu: 1500
///   min-mtu: 68
///   max-mtu: 65535
///   wait-ip: any
///   ipv4:
///     enabled: false
///   ipv6:
///     enabled: false
///   bridge:
///     options:
///       gc-timer: 29594
///       group-addr: 01:80:C2:00:00:00
///       group-forward-mask: 0
///       group-fwd-mask: 0
///       hash-max: 4096
///       hello-timer: 46
///       mac-ageing-time: 300
///       multicast-last-member-count: 2
///       multicast-last-member-interval: 100
///       multicast-membership-interval: 26000
///       multicast-querier: false
///       multicast-querier-interval: 25500
///       multicast-query-interval: 12500
///       multicast-query-response-interval: 1000
///       multicast-query-use-ifaddr: false
///       multicast-router: auto
///       multicast-snooping: true
///       multicast-startup-query-count: 2
///       multicast-startup-query-interval: 3125
///       stp:
///         enabled: true
///         forward-delay: 15
///         hello-time: 2
///         max-age: 20
///         priority: 32768
///       vlan-protocol: 802.1q
///     port:
///     - name: eth1
///       stp-hairpin-mode: false
///       stp-path-cost: 100
///       stp-priority: 32
///     - name: eth2
///       stp-hairpin-mode: false
///       stp-path-cost: 100
///       stp-priority: 32
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonDisplay)]
#[non_exhaustive]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct LinuxBridgeInterface {
    #[serde(flatten)]
    pub base: BaseInterface,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bridge: Option<LinuxBridgeConfig>,
}

impl LinuxBridgeInterface {
    pub fn new(name: String, bridge: LinuxBridgeConfig) -> Self {
        Self {
            base: BaseInterface {
                name: name.to_string(),
                iface_type: InterfaceType::LinuxBridge,
                ..Default::default()
            },
            bridge: Some(bridge),
        }
    }
}

impl Default for LinuxBridgeInterface {
    fn default() -> Self {
        Self {
            base: BaseInterface {
                iface_type: InterfaceType::LinuxBridge,
                ..Default::default()
            },
            bridge: None,
        }
    }
}

impl NmstateInterface for LinuxBridgeInterface {
    fn base_iface(&self) -> &BaseInterface {
        &self.base
    }

    fn base_iface_mut(&mut self) -> &mut BaseInterface {
        &mut self.base
    }

    fn is_virtual(&self) -> bool {
        true
    }

    fn is_controller(&self) -> bool {
        true
    }

    fn sanitize_iface_specfic(
        &mut self,
        _current: Option<&Self>,
    ) -> Result<(), NmError> {
        if let Some(opts) =
            self.bridge.as_ref().and_then(|b| b.options.as_ref())
        {
            opts.validate_vlan_default_pvid(self)?;
        }
        if let Some(opts) =
            self.bridge.as_mut().and_then(|b| b.options.as_mut())
        {
            opts.sanitize_group_fwd_mask(&self.base)?;
        }
        self.sort_ports();
        self.sanitize_stp_opts()?;
        self.use_upper_case_of_mac_address();
        self.flatten_port_vlan_ranges();
        self.sort_port_vlans();
        self.remove_runtime_only_timers();
        if let Some(port_confs) = self
            .bridge
            .as_ref()
            .and_then(|br_conf| br_conf.ports.as_ref())
        {
            for port_conf in port_confs {
                if let Some(vlan_conf) = port_conf.vlan.as_ref() {
                    vlan_conf.sanitize()?;
                }
            }
        }
        Ok(())
    }

    fn ports(&self) -> Option<Vec<&str>> {
        self.bridge
            .as_ref()
            .and_then(|br_conf| br_conf.ports.as_ref())
            .map(|ports| {
                ports.as_slice().iter().map(|p| p.name.as_str()).collect()
            })
    }

    /// Include port_name if port config changed
    fn include_diff_context_iface_specific(&mut self, current: &Self) {
        if let Some(ports) = self
            .bridge
            .as_mut()
            .and_then(|br_conf| br_conf.ports.as_mut())
            && let Some(cur_ports) = current
                .bridge
                .as_ref()
                .and_then(|br_conf| br_conf.ports.as_ref())
        {
            for (index, port_conf) in ports.iter_mut().enumerate() {
                if port_conf.name.is_empty()
                    && let Some(cur_port_conf) = cur_ports.get(index)
                {
                    port_conf.name = cur_port_conf.name.clone();
                }
            }
        }
    }

    fn sanitize_current_for_verify_iface_specfic(&mut self) {
        self.sort_ports();
    }
}

impl LinuxBridgeInterface {
    fn use_upper_case_of_mac_address(&mut self) {
        if let Some(address) = self
            .bridge
            .as_mut()
            .and_then(|br_conf| br_conf.options.as_mut())
            .and_then(|br_opts| br_opts.group_addr.as_mut())
        {
            address.make_ascii_uppercase()
        }
    }

    fn flatten_port_vlan_ranges(&mut self) {
        if let Some(port_confs) = self
            .bridge
            .as_mut()
            .and_then(|br_conf| br_conf.ports.as_mut())
        {
            for port_conf in port_confs {
                port_conf
                    .vlan
                    .as_mut()
                    .map(BridgePortVlanConfig::flatten_vlan_ranges);
            }
        }
    }

    fn sort_port_vlans(&mut self) {
        if let Some(port_confs) = self
            .bridge
            .as_mut()
            .and_then(|br_conf| br_conf.ports.as_mut())
        {
            for port_conf in port_confs {
                port_conf
                    .vlan
                    .as_mut()
                    .map(BridgePortVlanConfig::sort_trunk_tags);
            }
        }
    }

    fn sort_ports(&mut self) {
        if let Some(br_conf) = self.bridge.as_mut() {
            if let Some(port_confs) = br_conf.ports.as_mut() {
                port_confs.sort_unstable_by_key(|p| p.name.clone())
            }
        }
    }

    fn remove_runtime_only_timers(&mut self) {
        if let Some(br_conf) = self.bridge.as_mut() {
            if let Some(opts) = br_conf.options.as_mut() {
                opts.gc_timer = None;
                opts.hello_timer = None;
            }
        }
    }

    fn sanitize_stp_opts(&self) -> Result<(), NmError> {
        if let Some(stp_opts) = self
            .bridge
            .as_ref()
            .and_then(|b| b.options.as_ref())
            .and_then(|o| o.stp.as_ref())
        {
            stp_opts.sanitize()?;
        }
        Ok(())
    }

    pub(crate) fn vlan_filtering_is_enabled(&self) -> bool {
        if let Some(ports) = self.bridge.as_ref().and_then(|b| b.ports.as_ref())
        {
            ports.as_slice().iter().any(|port_conf| {
                if let Some(vlan_conf) = port_conf.vlan.as_ref() {
                    vlan_conf != &BridgePortVlanConfig::default()
                } else {
                    false
                }
            })
        } else {
            false
        }
    }
}

#[derive(
    Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize, JsonDisplay,
)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
#[non_exhaustive]
/// Linux bridge specific configuration.
pub struct LinuxBridgeConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    /// Linux bridge options. When applying, existing options will merged into
    /// desired.
    pub options: Option<LinuxBridgeOptions>,
    #[serde(
        skip_serializing_if = "Option::is_none",
        alias = "port",
        rename = "port"
    )]
    /// Linux bridge ports. When applying, desired port list will __override__
    /// current port list.
    pub ports: Option<Vec<LinuxBridgePortConfig>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
#[non_exhaustive]
pub struct LinuxBridgePortConfig {
    #[serde(default)]
    /// The kernel interface name of this bridge port.
    pub name: String,
    #[serde(
        skip_serializing_if = "Option::is_none",
        default,
        deserialize_with = "crate::deserializer::option_bool_or_string"
    )]
    /// Controls whether traffic may be send back out of the port on which it
    /// was received.
    pub stp_hairpin_mode: Option<bool>,
    #[serde(
        skip_serializing_if = "Option::is_none",
        default,
        deserialize_with = "crate::deserializer::option_u32_or_string"
    )]
    /// The STP path cost of the specified port.
    pub stp_path_cost: Option<u32>,
    #[serde(
        skip_serializing_if = "Option::is_none",
        default,
        deserialize_with = "crate::deserializer::option_u16_or_string"
    )]
    /// The STP port priority. The priority value is an unsigned 8-bit quantity
    /// (number between 0 and 255). This metric is used in the designated port
    /// an droot port selec‐ tion algorithms.
    pub stp_priority: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    /// Linux bridge VLAN filtering configure. If not defined, current VLAN
    /// filtering is preserved for the specified port.
    pub vlan: Option<BridgePortVlanConfig>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
#[non_exhaustive]
pub struct LinuxBridgeOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gc_timer: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub group_addr: Option<String>,
    #[serde(
        skip_serializing_if = "Option::is_none",
        default,
        deserialize_with = "crate::deserializer::option_u16_or_string"
    )]
    /// Alias of [LinuxBridgeOptions.group_fwd_mask], not preferred, please
    /// use [LinuxBridgeOptions.group_fwd_mask] instead.
    pub group_forward_mask: Option<u16>,
    #[serde(
        skip_serializing_if = "Option::is_none",
        default,
        deserialize_with = "crate::deserializer::option_u16_or_string"
    )]
    pub group_fwd_mask: Option<u16>,
    #[serde(
        skip_serializing_if = "Option::is_none",
        default,
        deserialize_with = "crate::deserializer::option_u32_or_string"
    )]
    pub hash_max: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hello_timer: Option<u64>,
    #[serde(
        skip_serializing_if = "Option::is_none",
        default,
        deserialize_with = "crate::deserializer::option_u32_or_string"
    )]
    pub mac_ageing_time: Option<u32>,
    #[serde(
        skip_serializing_if = "Option::is_none",
        default,
        deserialize_with = "crate::deserializer::option_u32_or_string"
    )]
    pub multicast_last_member_count: Option<u32>,
    #[serde(
        skip_serializing_if = "Option::is_none",
        default,
        deserialize_with = "crate::deserializer::option_u64_or_string"
    )]
    pub multicast_last_member_interval: Option<u64>,
    #[serde(
        skip_serializing_if = "Option::is_none",
        default,
        deserialize_with = "crate::deserializer::option_u64_or_string"
    )]
    pub multicast_membership_interval: Option<u64>,
    #[serde(
        skip_serializing_if = "Option::is_none",
        default,
        deserialize_with = "crate::deserializer::option_bool_or_string"
    )]
    pub multicast_querier: Option<bool>,
    #[serde(
        skip_serializing_if = "Option::is_none",
        default,
        deserialize_with = "crate::deserializer::option_u64_or_string"
    )]
    pub multicast_querier_interval: Option<u64>,
    #[serde(
        skip_serializing_if = "Option::is_none",
        default,
        deserialize_with = "crate::deserializer::option_u64_or_string"
    )]
    pub multicast_query_interval: Option<u64>,
    #[serde(
        skip_serializing_if = "Option::is_none",
        default,
        deserialize_with = "crate::deserializer::option_u64_or_string"
    )]
    pub multicast_query_response_interval: Option<u64>,
    #[serde(
        skip_serializing_if = "Option::is_none",
        default,
        deserialize_with = "crate::deserializer::option_bool_or_string"
    )]
    pub multicast_query_use_ifaddr: Option<bool>,
    #[serde(
        skip_serializing_if = "Option::is_none",
        default,
        deserialize_with = "crate::deserializer::option_enum_string_or_integer"
    )]
    pub multicast_router: Option<LinuxBridgeMulticastRouterType>,
    #[serde(
        skip_serializing_if = "Option::is_none",
        default,
        deserialize_with = "crate::deserializer::option_bool_or_string"
    )]
    pub multicast_snooping: Option<bool>,
    #[serde(
        skip_serializing_if = "Option::is_none",
        default,
        deserialize_with = "crate::deserializer::option_u32_or_string"
    )]
    pub multicast_startup_query_count: Option<u32>,
    #[serde(
        skip_serializing_if = "Option::is_none",
        default,
        deserialize_with = "crate::deserializer::option_u64_or_string"
    )]
    pub multicast_startup_query_interval: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stp: Option<LinuxBridgeStpOptions>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vlan_protocol: Option<VlanProtocol>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vlan_default_pvid: Option<u16>,
}

impl LinuxBridgeOptions {
    pub(crate) fn sanitize_group_fwd_mask(
        &mut self,
        base_iface: &BaseInterface,
    ) -> Result<(), NmError> {
        match (self.group_forward_mask, self.group_fwd_mask) {
            (Some(v1), Some(v2)) => {
                if v1 != v2 {
                    return Err(NmError::new(
                        ErrorKind::InvalidArgument,
                        format!(
                            "Linux bridge {} has different \
                             group_forward_mask: {v1}, group_fwd_mask: {v2}, \
                             these two property is the same, hence conflicting",
                            base_iface.name.as_str()
                        ),
                    ));
                } else {
                    self.group_fwd_mask = Some(v1);
                    self.group_forward_mask = None;
                }
            }
            (Some(v), None) => {
                self.group_fwd_mask = Some(v);
                self.group_forward_mask = None;
            }
            (None, Some(v)) => {
                self.group_fwd_mask = Some(v);
                self.group_forward_mask = None;
            }
            _ => (),
        }

        Ok(())
    }

    pub(crate) fn validate_vlan_default_pvid(
        &self,
        linux_bridge: &LinuxBridgeInterface,
    ) -> Result<(), NmError> {
        if let Some(pvid) = self.vlan_default_pvid {
            if pvid != 1 && !linux_bridge.vlan_filtering_is_enabled() {
                return Err(NmError::new(
                    ErrorKind::InvalidArgument,
                    format!(
                        "Linux bridge {} has vlan-default-pvid different than \
                         1 but VLAN filtering is not enabled.",
                        linux_bridge.base.name.as_str()
                    ),
                ));
            }
        }

        Ok(())
    }
}

#[derive(
    Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize, JsonDisplay,
)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
#[non_exhaustive]
pub struct LinuxBridgeStpOptions {
    #[serde(
        skip_serializing_if = "Option::is_none",
        default,
        deserialize_with = "crate::deserializer::option_bool_or_string"
    )]
    /// If disabled during applying, the remaining STP options will be discard.
    pub enabled: Option<bool>,
    #[serde(
        skip_serializing_if = "Option::is_none",
        default,
        deserialize_with = "crate::deserializer::option_u8_or_string"
    )]
    pub forward_delay: Option<u8>,
    #[serde(
        skip_serializing_if = "Option::is_none",
        default,
        deserialize_with = "crate::deserializer::option_u8_or_string"
    )]
    pub hello_time: Option<u8>,
    #[serde(
        skip_serializing_if = "Option::is_none",
        default,
        deserialize_with = "crate::deserializer::option_u8_or_string"
    )]
    pub max_age: Option<u8>,
    #[serde(
        skip_serializing_if = "Option::is_none",
        default,
        deserialize_with = "crate::deserializer::option_u16_or_string"
    )]
    pub priority: Option<u16>,
}

impl LinuxBridgeStpOptions {
    pub const HELLO_TIME_MAX: u8 = 10;
    pub const HELLO_TIME_MIN: u8 = 1;
    pub const MAX_AGE_MAX: u8 = 40;
    pub const MAX_AGE_MIN: u8 = 6;
    pub const FORWARD_DELAY_MAX: u8 = 30;
    pub const FORWARD_DELAY_MIN: u8 = 2;

    pub(crate) fn sanitize(&self) -> Result<(), NmError> {
        if let Some(hello_time) = self.hello_time {
            if !(Self::HELLO_TIME_MIN..=Self::HELLO_TIME_MAX)
                .contains(&hello_time)
            {
                let e = NmError::new(
                    ErrorKind::InvalidArgument,
                    format!(
                        "Desired STP hello time {} is not in the valid range \
                         of [{},{}]",
                        hello_time,
                        Self::HELLO_TIME_MIN,
                        Self::HELLO_TIME_MAX
                    ),
                );
                log::error!("{e}");
                return Err(e);
            }
        }

        if let Some(max_age) = self.max_age {
            if !(Self::MAX_AGE_MIN..=Self::MAX_AGE_MAX).contains(&max_age) {
                let e = NmError::new(
                    ErrorKind::InvalidArgument,
                    format!(
                        "Desired STP max age {} is not in the range of [{},{}]",
                        max_age,
                        Self::MAX_AGE_MIN,
                        Self::MAX_AGE_MAX
                    ),
                );
                log::error!("{e}");
                return Err(e);
            }
        }
        if let Some(forward_delay) = self.forward_delay {
            if !(Self::FORWARD_DELAY_MIN..=Self::FORWARD_DELAY_MAX)
                .contains(&forward_delay)
            {
                let e = NmError::new(
                    ErrorKind::InvalidArgument,
                    format!(
                        "Desired STP forward delay {} is not in the range of \
                         [{},{}]",
                        forward_delay,
                        Self::FORWARD_DELAY_MIN,
                        Self::FORWARD_DELAY_MAX
                    ),
                );
                log::error!("{e}");
                return Err(e);
            }
        }
        Ok(())
    }
}

#[derive(
    Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize, JsonDisplay,
)]
#[serde(rename_all = "kebab-case")]
#[non_exhaustive]
#[repr(u64)]
pub enum LinuxBridgeMulticastRouterType {
    #[default]
    #[serde(alias = "1")]
    Auto = 1u64,
    #[serde(alias = "0")]
    Disabled = 0,
    #[serde(alias = "2")]
    Enabled = 2,
    Unknown = u64::MAX,
}
