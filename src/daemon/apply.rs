// SPDX-License-Identifier: GPL-3.0-or-later

use std::collections::HashSet;

use nm::{
    Interface, InterfaceType, MergedInterfaces, MergedNetworkState,
    NetworkState, NmError, NmIpcConnection, NmNoDaemon, NmstateApplyOption,
    NmstateInterface,
};

use super::commander::NmCommander;

const RETRY_COUNT: usize = 10;
const RETRY_INTERVAL_MS: u64 = 500;

impl NmCommander {
    pub(crate) async fn apply_network_state(
        &mut self,
        mut conn: Option<&mut NmIpcConnection>,
        mut desired_state: NetworkState,
        opt: NmstateApplyOption,
    ) -> Result<NetworkState, NmError> {
        if desired_state.is_empty() {
            if let Some(conn) = conn.as_deref_mut() {
                conn.log_info(
                    "Desired state is empty, no action required".to_string(),
                )
                .await;
            } else {
                log::info!("Desired state is empty, no action required");
            }
        }
        if let Some(conn) = conn.as_deref_mut() {
            conn.log_trace(format!("Apply {desired_state} with option {opt}"))
                .await;
        } else {
            log::trace!("Apply {desired_state} with option {opt}");
        }

        let mut state_to_save = self.conf_manager.query_state().await?;

        desired_state.ifaces.unify_veth_and_ethernet();

        state_to_save.merge(&desired_state)?;
        let mut state_to_apply = state_to_save.clone();
        remove_undesired_ifaces(&mut state_to_apply, &desired_state);

        if let Some(ref mut conn) = conn {
            conn.log_info(format!(
                "Merged desired with previous saved state, state to apply \
                 {state_to_apply}"
            ))
            .await;
        } else {
            log::info!(
                "Merged desired with previous saved state, state to apply \
                 {state_to_apply}"
            );
        }
        let mut pre_apply_current_state = self
            .query_network_state(conn.as_deref_mut(), Default::default())
            .await?;

        pre_apply_current_state.ifaces.unify_veth_and_ethernet();

        if let Some(conn) = conn.as_deref_mut() {
            conn.log_debug(format!(
                "Pre-apply current state {pre_apply_current_state}"
            ))
            .await;
        } else {
            log::debug!("Pre-apply current state {pre_apply_current_state}");
        }

        let revert_state =
            state_to_apply.generate_revert(&pre_apply_current_state)?;

        let merged_state = MergedNetworkState::new(
            state_to_apply,
            pre_apply_current_state.clone(),
            opt.clone(),
        )?;

        // TODO(Gris Ge): discard auto IPs

        // Suppress the monitor during applying
        self.monitor_manager.pause().await?;
        if let Err(e) = self
            .apply_merged_state(conn.as_deref_mut(), &merged_state, &opt)
            .await
        {
            if let Some(conn) = conn.as_deref_mut() {
                conn.log_warn(format!("Failed to apply desired state: {e}"))
                    .await;
                conn.log_warn(format!(
                    "Failed to apply merged state: {merged_state}"
                ))
                .await;
                conn.log_warn("Rollback to state before apply".to_string())
                    .await;
                conn.log_trace(format!(
                    "Rollback to state before apply {revert_state}"
                ))
                .await;
            } else {
                log::warn!("Failed to apply desired state: {e}");
                log::warn!("Failed to apply merged state: {merged_state}");
                log::warn!("Rollback to state before apply");
                log::trace!("Rollback to state before apply {revert_state}");
            }
            if let Err(e) =
                self.rollback(conn.as_deref_mut(), revert_state).await
            {
                if let Some(conn) = conn.as_deref_mut() {
                    conn.log_error(format!("Failed to rollback: {e}")).await;
                } else {
                    log::error!("Failed to rollback: {e}");
                }
            }
            return Err(e);
        }

        if let Err(e) =
            self.conf_manager.save_state(state_to_save.clone()).await
        {
            if let Some(conn) = conn {
                conn.log_warn(format!(
                    "BUG: Failed to persistent desired state {state_to_save}: \
                     {e}"
                ))
                .await;
            } else {
                log::warn!(
                    "BUG: Failed to persistent desired state {state_to_save}: \
                     {e}"
                );
            }
        }

        let (ifaces_start_monitor, ifaces_stop_monitor) =
            gen_iface_monitor_list(&merged_state.ifaces);

        {
            for iface in ifaces_stop_monitor.iter() {
                self.monitor_manager.del_iface_from_monitor(iface).await?;
            }
            for iface in ifaces_start_monitor.iter() {
                self.monitor_manager.add_iface_to_monitor(iface).await?;
            }
        }

        self.monitor_manager.resume().await?;

        let mut diff_state = match merged_state
            .gen_state_for_apply()
            .gen_diff(&pre_apply_current_state)
        {
            Ok(s) => s,
            Err(e) => {
                log::warn!("Returning full state instead of diff state: {e}");
                merged_state.gen_state_for_apply()
            }
        };
        diff_state.hide_secrets();

        Ok(diff_state)
    }

    async fn rollback(
        &mut self,
        mut conn: Option<&mut NmIpcConnection>,
        revert_state: NetworkState,
    ) -> Result<(), NmError> {
        let mut opt = NmstateApplyOption::default();
        opt.no_verify = true;

        let current_state = self
            .query_network_state(conn.as_deref_mut(), Default::default())
            .await?;
        let merged_state =
            MergedNetworkState::new(revert_state, current_state, opt.clone())?;

        let apply_state = merged_state.gen_state_for_apply();

        NmNoDaemon::apply_merged_state(&merged_state).await?;
        self.plugin_manager
            .apply_network_state(&apply_state, &opt)
            .await?;

        self.dhcpv4_manager
            .apply_dhcp_config(conn, &merged_state)
            .await?;

        Ok(())
    }

    async fn verify(
        &mut self,
        mut conn: Option<&mut NmIpcConnection>,
        merged_state: &MergedNetworkState,
    ) -> Result<(), NmError> {
        let post_apply_current_state = self
            .query_network_state(conn.as_deref_mut(), Default::default())
            .await?;
        if let Some(conn) = conn {
            conn.log_trace(format!(
                "Post apply network state: {post_apply_current_state}"
            ))
            .await;
        } else {
            log::trace!("Post apply network state: {post_apply_current_state}");
        }
        merged_state.verify(&post_apply_current_state)?;
        Ok(())
    }

    async fn apply_merged_state(
        &mut self,
        mut conn: Option<&mut NmIpcConnection>,
        merged_state: &MergedNetworkState,
        opt: &NmstateApplyOption,
    ) -> Result<(), NmError> {
        let apply_state = merged_state.gen_state_for_apply();

        if let Some(conn) = conn.as_deref_mut() {
            conn.log_trace(format!("apply_state {apply_state}")).await;
        } else {
            log::trace!("apply_state {apply_state}");
        }

        NmNoDaemon::apply_merged_state(merged_state).await?;
        self.plugin_manager
            .apply_network_state(&apply_state, opt)
            .await?;

        self.dhcpv4_manager
            .apply_dhcp_config(conn.as_deref_mut(), merged_state)
            .await?;

        let mut result: Result<(), NmError> = Ok(());
        if !opt.no_verify {
            for cur_retry_count in 1..(RETRY_COUNT + 1) {
                result = self.verify(conn.as_deref_mut(), merged_state).await;
                if let Err(e) = &result {
                    if let Some(conn) = conn.as_deref_mut() {
                        conn.log_info(format!(
                            "Retrying({cur_retry_count}/{RETRY_COUNT}) on \
                             verification error: {e}"
                        ))
                        .await;
                    } else {
                        log::info!(
                            "Retrying({cur_retry_count}/{RETRY_COUNT}) on \
                             verification error: {e}"
                        );
                    }
                    tokio::time::sleep(std::time::Duration::from_millis(
                        RETRY_INTERVAL_MS,
                    ))
                    .await;
                } else {
                    break;
                }
            }
        }
        result
    }
}

fn remove_undesired_ifaces(
    merged_desired_state: &mut NetworkState,
    desired_state: &NetworkState,
) {
    merged_desired_state
        .ifaces
        .kernel_ifaces
        .retain(|iface_name, _| {
            desired_state
                .ifaces
                .kernel_ifaces
                .contains_key(&iface_name.to_string())
        });
    merged_desired_state.ifaces.user_ifaces.retain(|key, _| {
        desired_state
            .ifaces
            .user_ifaces
            .contains_key(&(key.clone()))
    });
}

/// Return iface names to start and stop monitor
fn gen_iface_monitor_list(
    merged_ifaces: &MergedInterfaces,
) -> (HashSet<String>, HashSet<String>) {
    let mut ifaces_start_monitor = HashSet::new();
    let mut ifaces_stop_monitor = HashSet::new();

    let mut has_wifi_bind_to_any = false;

    for merged_iface in merged_ifaces
        .iter()
        .filter(|i| i.merged.iface_type() == &InterfaceType::WifiCfg)
    {
        let wifi_iface = if let Interface::WifiCfg(i) = &merged_iface.merged {
            i
        } else {
            continue;
        };
        if let Some(parent) = wifi_iface.parent() {
            if wifi_iface.is_up() {
                ifaces_start_monitor.insert(parent.to_string());
            } else if wifi_iface.is_absent() || wifi_iface.is_down() {
                ifaces_stop_monitor.insert(parent.to_string());
            }
        } else if wifi_iface.is_up() {
            has_wifi_bind_to_any = true;
        }
    }
    if has_wifi_bind_to_any {
        for iface_name in merged_ifaces.iter().filter_map(|merged_iface| {
            if !merged_iface.merged.is_absent()
                && !merged_iface.merged.is_ignore()
                && merged_iface.merged.iface_type() == &InterfaceType::WifiPhy
            {
                Some(merged_iface.merged.name())
            } else {
                None
            }
        }) {
            ifaces_start_monitor.insert(iface_name.to_string());
        }
    }

    (ifaces_start_monitor, ifaces_stop_monitor)
}
