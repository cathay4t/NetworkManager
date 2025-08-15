// SPDX-License-Identifier: Apache-2.0

use crate::nmstate::{
    Interface, InterfaceState, InterfaceType, NmstateInterface,
};

fn nmstate_iface_type_to_nispor(
    iface_type: &InterfaceType,
) -> nispor::IfaceType {
    match iface_type {
        InterfaceType::Ethernet => nispor::IfaceType::Ethernet,
        InterfaceType::Loopback => nispor::IfaceType::Loopback,
        InterfaceType::Veth => nispor::IfaceType::Veth,
        _ => {
            log::warn!(
                "BUG: Requesting unsupported interface type {iface_type}"
            );
            nispor::IfaceType::Unknown
        }
    }
}

pub(crate) fn nmstate_iface_state_to_nispor(
    iface_state: InterfaceState,
) -> nispor::IfaceState {
    match iface_state {
        InterfaceState::Up => nispor::IfaceState::Up,
        InterfaceState::Down => nispor::IfaceState::Down,
        InterfaceState::Absent => nispor::IfaceState::Absent,
        _ => {
            log::warn!(
                "BUG: Requesting unsupported interface state {iface_state}"
            );
            nispor::IfaceState::Unknown
        }
    }
}

pub(crate) fn init_np_iface(iface: &Interface) -> nispor::IfaceConf {
    let mut np_iface = nispor::IfaceConf::default();
    np_iface.name = iface.name().to_string();
    np_iface.iface_type =
        Some(nmstate_iface_type_to_nispor(iface.iface_type()));
    np_iface.state = nmstate_iface_state_to_nispor(iface.base_iface().state);
    np_iface
}
