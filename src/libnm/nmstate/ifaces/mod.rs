// SPDX-License-Identifier: Apache-2.0

mod base;
mod bond;
mod dummy;
mod ethernet;
mod inter_ifaces;
mod loopback;
mod ovs_bridge;
mod ovs_iface;
mod unknown;
mod vlan;
mod wifi;

pub use self::{
    base::BaseInterface,
    bond::{
        BondAdSelect, BondAllPortsActive, BondArpAllTargets, BondArpValidate,
        BondConfig, BondFailOverMac, BondInterface, BondLacpRate, BondMode,
        BondOptions, BondPortConfig, BondPrimaryReselect, BondXmitHashPolicy,
    },
    dummy::DummyInterface,
    ethernet::{EthernetConfig, EthernetDuplex, EthernetInterface, VethConfig},
    inter_ifaces::Interfaces,
    loopback::LoopbackInterface,
    ovs_bridge::{OvsBridgeConfig, OvsBridgeInterface, OvsBridgePortConfig},
    ovs_iface::OvsInterface,
    unknown::UnknownInterface,
    vlan::{
        VlanConfig, VlanInterface, VlanProtocol, VlanQosMapping,
        VlanRegistrationProtocol,
    },
    wifi::{
        WifiAuthType, WifiCfgInterface, WifiConfig, WifiPhyInterface, WifiState,
    },
};
