// SPDX-License-Identifier: Apache-2.0

mod base;
mod ethernet;
mod inter_ifaces;
mod ovs_bridge;
mod ovs_iface;
mod unknown;

pub use self::base::BaseInterface;
pub use self::ethernet::{EthernetConfig, EthernetDuplex, EthernetInterface};
pub use self::inter_ifaces::Interfaces;
pub use self::ovs_bridge::{
    OvsBridgeConfig, OvsBridgeInterface, OvsBridgePortConfig,
};
pub use self::ovs_iface::OvsInterface;
pub use self::unknown::UnknownInterface;
