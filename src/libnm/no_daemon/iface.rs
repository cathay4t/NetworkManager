// SPDX-License-Identifier: Apache-2.0

use super::{
    base_iface::apply_base_iface_link_changes, ethernet::apply_ethernet_conf,
};
use crate::{
    BaseInterface, Interface, InterfaceState, InterfaceType, MergedInterfaces,
    NmError, NmstateInterface,
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

pub(crate) fn init_np_iface(iface: &BaseInterface) -> nispor::IfaceConf {
    let mut np_iface = nispor::IfaceConf::default();
    np_iface.name = iface.name.to_string();
    np_iface.iface_type = Some(nmstate_iface_type_to_nispor(&iface.iface_type));
    np_iface.state = nmstate_iface_state_to_nispor(iface.state);
    np_iface
}

/// Return None if no change required
pub(crate) fn apply_iface_link_changes(
    apply_iface: &Interface,
    cur_iface: Option<&Interface>,
    merged_ifaces: &MergedInterfaces,
) -> Result<Option<nispor::IfaceConf>, NmError> {
    if should_skip_link_change(apply_iface, cur_iface, merged_ifaces) {
        return Ok(None);
    }

    let mut np_conf = if let Some(cur_iface) = cur_iface {
        init_np_iface(cur_iface.base_iface())
    } else {
        init_np_iface(apply_iface.base_iface())
    };
    let init_np_conf = np_conf.clone();

    apply_base_iface_link_changes(&mut np_conf, apply_iface.base_iface())?;

    if let Interface::Ethernet(apply_iface) = apply_iface {
        apply_ethernet_conf(&mut np_conf, apply_iface, cur_iface)?;
    }

    if np_conf != init_np_conf || cur_iface.is_none() {
        Ok(Some(np_conf))
    } else {
        Ok(None)
    }
}

/// Skip link:
///  * loopback interface cannot be deleted
///  * Absent on non-exist interface
///  * Veth peer should be skipped when both end is marked as absent
fn should_skip_link_change(
    apply_iface: &Interface,
    cur_iface: Option<&Interface>,
    merged_ifaces: &MergedInterfaces,
) -> bool {
    if apply_iface.is_absent() {
        if apply_iface.iface_type() == &InterfaceType::Loopback {
            log::info!(
                "Skipping removing loopback interface because it cannot be \
                 deleted",
            );
            return true;
        }
        if cur_iface.is_none() {
            log::info!(
                "Skipping removing interface {}/{} because it does not exists",
                apply_iface.name(),
                apply_iface.iface_type()
            );
            return true;
        }
        if let Some(Interface::Ethernet(cur_iface)) = cur_iface {
            if let Some(peer) = cur_iface.veth.as_ref().map(|v| v.peer.as_str())
            {
                if peer > cur_iface.base.name.as_str()
                    && let Some(peer_iface) = merged_ifaces
                        .kernel_ifaces
                        .get(peer)
                        .and_then(|m| m.for_apply.as_ref())
                    && peer_iface.is_absent()
                {
                    log::info!(
                        "Skipping removing interface {}/{} because its veth \
                         peer is already marked as absent",
                        apply_iface.name(),
                        apply_iface.iface_type()
                    );
                    return true;
                }
            }
        }
    }
    false
}
