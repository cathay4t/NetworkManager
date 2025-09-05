// SPDX-License-Identifier: Apache-2.0

mod gen_diff;
mod iface;
mod iface_state;
mod iface_trait;
mod iface_type;
mod ifaces;
mod ip;
mod merged;
mod net_state;
mod revert;
mod state_options;
mod value;
mod version;

#[allow(dead_code)]
pub(crate) mod deserializer;
#[allow(dead_code)]
pub(crate) mod serializer;

pub use self::{
    iface::Interface,
    iface_state::InterfaceState,
    iface_trait::NmstateInterface,
    iface_type::InterfaceType,
    ifaces::{
        BaseInterface, EthernetConfig, EthernetDuplex, EthernetInterface,
        Interfaces, LoopbackInterface, OvsBridgeConfig, OvsBridgeInterface,
        OvsBridgePortConfig, OvsInterface, UnknownInterface, VethConfig,
    },
    ip::{InterfaceIpAddr, InterfaceIpv4, InterfaceIpv6},
    merged::{MergedInterface, MergedInterfaces, MergedNetworkState},
    net_state::NetworkState,
    state_options::{NmstateApplyOption, NmstateQueryOption, NmstateStateKind},
    version::CUR_SCHEMA_VERSION,
};

#[cfg(test)]
mod unit_tests;
