// SPDX-License-Identifier: Apache-2.0

mod base;
mod dummy;
mod ethernet;
mod inter_ifaces;
mod loopback;
mod ovs_bridge;
mod ovs_iface;
mod unknown;
mod wifi;

pub use self::{
    base::BaseInterface,
    dummy::DummyInterface,
    ethernet::{EthernetConfig, EthernetDuplex, EthernetInterface, VethConfig},
    inter_ifaces::Interfaces,
    loopback::LoopbackInterface,
    ovs_bridge::{OvsBridgeConfig, OvsBridgeInterface, OvsBridgePortConfig},
    ovs_iface::OvsInterface,
    unknown::UnknownInterface,
    wifi::{
        WifiAuthType, WifiCfgInterface, WifiConfig, WifiPhyInterface, WifiState,
    },
};
