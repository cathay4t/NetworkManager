// SPDX-License-Identifier: Apache-2.0

mod error;
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

pub use self::error::{ErrorKind, NmstateError};
pub use self::iface::Interface;
pub use self::iface_state::InterfaceState;
pub use self::iface_trait::{
    NmstateChild, NmstateChildInterface, NmstateController,
    NmstateControllerInterface, NmstateInterface,
};
pub use self::iface_type::InterfaceType;
pub use self::ifaces::{
    BaseInterface, EthernetConfig, EthernetDuplex, EthernetInterface,
    Interfaces, OvsBridgeConfig, OvsBridgeInterface, OvsBridgePortConfig,
    OvsInterface, UnknownInterface,
};
pub use self::ip::{InterfaceIpAddr, InterfaceIpv4, InterfaceIpv6};
pub use self::merged::{MergedInterface, MergedInterfaces, MergedNetworkState};
pub use self::net_state::NetworkState;
pub use self::state_options::{
    NmstateApplyOption, NmstateQueryOption, NmstateStateKind,
};
pub use self::version::CUR_SCHEMA_VERSION;
pub use nmstate_derive::JsonDisplay;
