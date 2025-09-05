// SPDX-License-Identifier: Apache-2.0

use super::{iface::apply_iface_link_changes, ip::apply_iface_ip_changes};
use crate::{
    ErrorKind, MergedInterface, MergedInterfaces, NmError, NmstateInterface,
};

pub(crate) async fn apply_ifaces(
    merged_ifaces: &MergedInterfaces,
) -> Result<(), NmError> {
    apply_ifaces_link_changes(merged_ifaces).await?;

    apply_ifaces_ip_changes(merged_ifaces).await?;

    Ok(())
}

/// Apply link level changes (e.g. state up/down, attach to controller)
async fn apply_ifaces_link_changes(
    merged_ifaces: &MergedInterfaces,
) -> Result<(), NmError> {
    let mut np_ifaces: Vec<nispor::IfaceConf> = Vec::new();

    let mut sorted_changed_mergd_ifaces: Vec<&MergedInterface> = merged_ifaces
        .kernel_ifaces
        .values()
        .filter(|i| i.for_apply.is_some())
        .collect();
    sorted_changed_mergd_ifaces.sort_unstable_by_key(|i| {
        i.for_apply
            .as_ref()
            .map(|i| i.base_iface().up_priority)
            .unwrap_or(u32::MAX)
    });

    for merged_iface in sorted_changed_mergd_ifaces {
        let apply_iface = if let Some(i) = merged_iface.for_apply.as_ref() {
            i
        } else {
            continue;
        };
        if let Some(np_iface) = apply_iface_link_changes(
            apply_iface,
            merged_iface.current.as_ref(),
            merged_ifaces,
        )? {
            np_ifaces.push(np_iface);
        }
    }
    if !np_ifaces.is_empty() {
        let mut net_conf = nispor::NetConf::default();
        net_conf.ifaces = Some(np_ifaces);

        log::debug!(
            "Pending nispor changes {}",
            serde_json::to_string(&net_conf).unwrap_or_default()
        );
        if let Err(e) = net_conf.apply_async().await {
            return Err(NmError::new(
                ErrorKind::Bug,
                format!("Failed to change link layer: {e}"),
            ));
        }
    }

    Ok(())
}

async fn apply_ifaces_ip_changes(
    merged_ifaces: &MergedInterfaces,
) -> Result<(), NmError> {
    let mut np_ifaces: Vec<nispor::IfaceConf> = Vec::new();

    for merged_iface in merged_ifaces
        .kernel_ifaces
        .values()
        .filter(|i| i.for_apply.is_some())
    {
        // It is safe to unwrap here as it is checked by filter()
        let apply_iface = merged_iface.for_apply.as_ref().unwrap();

        if let Some(np_iface) = apply_iface_ip_changes(
            apply_iface.base_iface(),
            merged_iface.current.as_ref().map(|c| c.base_iface()),
        )? {
            np_ifaces.push(np_iface);
        }
    }
    if !np_ifaces.is_empty() {
        let mut net_conf = nispor::NetConf::default();
        net_conf.ifaces = Some(np_ifaces);

        log::debug!(
            "Pending nispor changes {}",
            serde_json::to_string(&net_conf).unwrap_or_default()
        );

        if let Err(e) = net_conf.apply_async().await {
            return Err(NmError::new(
                ErrorKind::Bug,
                format!("Failed to change IP: {e}"),
            ));
        }
    }

    Ok(())
}
