// SPDX-License-Identifier: Apache-2.0

use crate::{
    ErrorKind, NmError,
    nmstate::{MergedInterfaces, NmstateInterface},
};

use super::{
    base_iface::apply_base_iface_link_changes, iface::init_np_iface,
    ip::apply_iface_ip_changes,
};

pub(crate) async fn apply_ifaces(
    merged_ifaces: &MergedInterfaces,
) -> Result<(), NmError> {
    apply_link_changes(merged_ifaces).await?;

    apply_ip_changes(merged_ifaces).await?;

    Ok(())
}

/// Apply link level changes (e.g. state up/down, attach to controller)
async fn apply_link_changes(
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

        if let Some(cur_iface) = merged_iface.current.as_ref() {
            let mut np_iface = init_np_iface(cur_iface);
            apply_base_iface_link_changes(
                &mut np_iface,
                apply_iface,
                cur_iface,
            )?;
            if np_iface != init_np_iface(cur_iface) {
                np_ifaces.push(np_iface);
            }
        } else {
            return Err(NmError::new(
                ErrorKind::NoSupport,
                format!("Not support create new interface yet: {apply_iface}"),
            ));
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

async fn apply_ip_changes(
    merged_ifaces: &MergedInterfaces,
) -> Result<(), NmError> {
    let mut np_ifaces: Vec<nispor::IfaceConf> = Vec::new();

    for (apply_iface, cur_iface) in
        merged_ifaces.kernel_ifaces.values().filter_map(|i| {
            if let (Some(apply_iface), Some(cur_iface)) =
                (i.for_apply.as_ref(), i.current.as_ref())
            {
                Some((apply_iface, cur_iface))
            } else {
                None
            }
        })
    {
        let mut np_iface = init_np_iface(cur_iface);

        apply_iface_ip_changes(
            &mut np_iface,
            apply_iface.base_iface(),
            cur_iface.base_iface(),
        )?;

        if np_iface != init_np_iface(cur_iface) {
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
