// SPDX-License-Identifier: Apache-2.0

mod iface;
mod inter_iface;
mod ip;
mod loopback;
mod net_state;

pub use self::iface::MergedInterface;
pub use self::inter_iface::MergedInterfaces;
pub use self::net_state::MergedNetworkState;
