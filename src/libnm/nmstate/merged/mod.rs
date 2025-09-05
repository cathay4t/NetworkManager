// SPDX-License-Identifier: Apache-2.0

mod base_iface;
mod ethernet;
mod iface;
mod inter_iface;
mod ip;
mod loopback;
mod net_state;

pub use self::{
    iface::MergedInterface, inter_iface::MergedInterfaces,
    net_state::MergedNetworkState,
};
