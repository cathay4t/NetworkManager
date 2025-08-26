// SPDX-License-Identifier: Apache-2.0

use crate::{ErrorKind, MergedInterfaces, NmError, NmstateInterface};

use super::{iface::apply_iface_link_changes, ip::apply_iface_ip_changes};

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

    // TODO(Gris Ge): Sort by apply_order
    for merged_iface in merged_ifaces
        .kernel_ifaces
        .values()
        .filter(|i| i.for_apply.is_some())
    {
        // It is safe to unwrap here as it is checked by filter()
        let apply_iface = merged_iface.for_apply.as_ref().unwrap();

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
