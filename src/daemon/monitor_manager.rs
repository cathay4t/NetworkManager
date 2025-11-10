// SPDX-License-Identifier: Apache-2.0

use nm::NmError;

use super::{
    monitor_worker::{NmMonitorCmd, NmMonitorReply, NmMonitorWorker},
    worker::NmManager,
};

#[derive(Debug, Clone)]
pub(crate) struct NmMonitorManager {
    mgr: NmManager<NmMonitorCmd, NmMonitorReply>,
}

impl NmMonitorManager {
    pub(crate) async fn new() -> Result<Self, NmError> {
        Ok(Self {
            mgr: NmManager::new::<NmMonitorWorker>("monitor").await?,
        })
    }

    pub(crate) async fn pause(&mut self) -> Result<(), NmError> {
        self.mgr.exec(NmMonitorCmd::Pause).await?;
        Ok(())
    }

    pub(crate) async fn resume(&mut self) -> Result<(), NmError> {
        self.mgr.exec(NmMonitorCmd::Resume).await?;
        Ok(())
    }

    /// Will start NmMonitorWorker if not exists yet
    pub(crate) async fn add_iface_to_monitor(
        &mut self,
        iface_name: &str,
    ) -> Result<(), NmError> {
        self.mgr
            .exec(NmMonitorCmd::AddIface(iface_name.to_string()))
            .await?;
        Ok(())
    }

    /// Will stop NmMonitorWorker if no interfaces need to monitor
    pub(crate) async fn del_iface_from_monitor(
        &mut self,
        iface_name: &str,
    ) -> Result<(), NmError> {
        self.mgr
            .exec(NmMonitorCmd::DelIface(iface_name.to_string()))
            .await?;
        Ok(())
    }
}
