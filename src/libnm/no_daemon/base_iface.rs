// SPDX-License-Identifier: Apache-2.0

use nmstate::{BaseInterface, InterfaceState, InterfaceType};

const SUPPORTED_LIST: [InterfaceType; 1] = [InterfaceType::Ethernet];

fn np_iface_type_to_nmstate(
    np_iface_type: &nispor::IfaceType,
) -> InterfaceType {
    match np_iface_type {
        nispor::IfaceType::Bond => InterfaceType::Bond,
        nispor::IfaceType::Bridge => InterfaceType::LinuxBridge,
        nispor::IfaceType::Dummy => InterfaceType::Dummy,
        nispor::IfaceType::Ethernet => InterfaceType::Ethernet,
        nispor::IfaceType::Hsr => InterfaceType::Hsr,
        nispor::IfaceType::Loopback => InterfaceType::Loopback,
        nispor::IfaceType::MacSec => InterfaceType::MacSec,
        nispor::IfaceType::MacVlan => InterfaceType::MacVlan,
        nispor::IfaceType::MacVtap => InterfaceType::MacVtap,
        nispor::IfaceType::OpenvSwitch => InterfaceType::OvsInterface,
        nispor::IfaceType::Veth => InterfaceType::Veth,
        nispor::IfaceType::Vlan => InterfaceType::Vlan,
        nispor::IfaceType::Vrf => InterfaceType::Vrf,
        nispor::IfaceType::Vxlan => InterfaceType::Vxlan,
        nispor::IfaceType::Ipoib => InterfaceType::InfiniBand,
        nispor::IfaceType::Tun => InterfaceType::Tun,
        nispor::IfaceType::Xfrm => InterfaceType::Xfrm,
        nispor::IfaceType::IpVlan => InterfaceType::IpVlan,
        nispor::IfaceType::Other(v) => InterfaceType::Unknown(v.to_lowercase()),
        _ => {
            InterfaceType::Unknown(format!("{np_iface_type:?}").to_lowercase())
        }
    }
}

fn np_iface_state_to_nmstate(
    state: &nispor::IfaceState,
    flags: &[nispor::IfaceFlag],
) -> InterfaceState {
    // nispor::IfaceState::Up means operational up.
    // Check also the Running flag with, according to [1], means operational
    // state Up or Unknown.
    // [1] https://www.kernel.org/doc/Documentation/networking/operstates.txt
    if *state == nispor::IfaceState::Up
        || flags.contains(&nispor::IfaceFlag::Running)
    {
        InterfaceState::Up
    } else if *state == nispor::IfaceState::Down {
        InterfaceState::Down
    } else {
        InterfaceState::Unknown
    }
}

pub(crate) fn np_iface_to_base_iface(
    np_iface: &nispor::Iface,
) -> BaseInterface {
    let mut base_iface = BaseInterface::default();
    base_iface.name = np_iface.name.to_string();
    base_iface.state =
        np_iface_state_to_nmstate(&np_iface.state, np_iface.flags.as_slice());
    base_iface.iface_type = np_iface_type_to_nmstate(&np_iface.iface_type);
    base_iface.mac_address = Some(np_iface.mac_address.to_uppercase());
    base_iface.permanent_mac_address = get_permanent_mac_address(np_iface);
    base_iface.controller = np_iface.controller.as_ref().map(|c| c.to_string());
    base_iface.mtu = if np_iface.mtu >= 0 {
        Some(np_iface.mtu as u64)
    } else {
        Some(0u64)
    };
    base_iface.min_mtu = if let Some(mtu) = np_iface.min_mtu {
        if mtu >= 0 { Some(mtu as u64) } else { None }
    } else {
        None
    };
    base_iface.max_mtu = if let Some(mtu) = np_iface.max_mtu {
        if mtu >= 0 { Some(mtu as u64) } else { None }
    } else {
        None
    };
    if !SUPPORTED_LIST.contains(&base_iface.iface_type) {
        log::debug!(
            "Got unsupported interface type {}: {}, ignoring",
            &base_iface.iface_type,
            &base_iface.name
        );
        base_iface.state = InterfaceState::Ignore;
    }

    base_iface
}

fn get_permanent_mac_address(iface: &nispor::Iface) -> Option<String> {
    if iface.permanent_mac_address.is_empty() {
        // Bond port also hold perm_hwaddr which is the mac address before
        // this interface been assgined to bond as subordinate.
        if let Some(bond_port_info) = &iface.bond_subordinate {
            if bond_port_info.perm_hwaddr.is_empty() {
                None
            } else {
                Some(bond_port_info.perm_hwaddr.as_str().to_uppercase())
            }
        } else {
            None
        }
    } else {
        Some(iface.permanent_mac_address.as_str().to_uppercase())
    }
}
