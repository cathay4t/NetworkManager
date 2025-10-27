// SPDX-License-Identifier: Apache-2.0

mod apply;
mod base_iface;
mod error;
mod ethernet;
mod iface;
mod inter_ifaces;
mod ip;
mod query;
mod wifi;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct NmNoDaemon {}
