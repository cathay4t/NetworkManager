// SPDX-License-Identifier: Apache-2.0

use futures_channel::mpsc::UnboundedSender;
use nm::{InterfaceType, NmError};

use super::{
    super::daemon::NmManagerCmd, NmMonitorCmd, NmMonitorReply, NmMonitorWorker,
};
use crate::TaskManager;

#[derive(Debug, Clone)]
pub(crate) struct NmMonitorManager {
    mgr: TaskManager<NmMonitorCmd, NmMonitorReply>,
    msg_to_commander: UnboundedSender<NmManagerCmd>,
}

impl NmMonitorManager {
    pub(crate) async fn new(
        msg_to_commander: UnboundedSender<NmManagerCmd>,
    ) -> Result<Self, NmError> {
        let mut ret = Self {
            mgr: TaskManager::new::<NmMonitorWorker>("monitor").await?,
            msg_to_commander,
        };
        ret.mgr
            .exec(NmMonitorCmd::SetCommanderSender(
                ret.msg_to_commander.clone(),
            ))
            .await?;
        Ok(ret)
    }

    pub(crate) async fn pause(&mut self) -> Result<(), NmError> {
        self.mgr.exec(NmMonitorCmd::Pause).await?;
        Ok(())
    }

    pub(crate) async fn resume(&mut self) -> Result<(), NmError> {
        self.mgr.exec(NmMonitorCmd::Resume).await?;
        Ok(())
    }

    /// Start monitoring on specified interface.
    pub(crate) async fn add_iface_to_monitor(
        &mut self,
        iface_name: &str,
    ) -> Result<(), NmError> {
        self.mgr
            .exec(NmMonitorCmd::AddIface(iface_name.to_string()))
            .await?;
        Ok(())
    }

    /// Stop monitoring on specified interface.
    pub(crate) async fn del_iface_from_monitor(
        &mut self,
        iface_name: &str,
    ) -> Result<(), NmError> {
        self.mgr
            .exec(NmMonitorCmd::DelIface(iface_name.to_string()))
            .await?;
        Ok(())
    }

    /// Start monitoring on specified interface types.
    pub(crate) async fn add_iface_type_to_monitor(
        &mut self,
        iface_type: InterfaceType,
    ) -> Result<(), NmError> {
        self.mgr
            .exec(NmMonitorCmd::AddIfaceType(iface_type))
            .await?;
        Ok(())
    }

    /// Stop monitoring on any WIFI NICs
    pub(crate) async fn del_iface_type_from_monitor(
        &mut self,
        iface_type: InterfaceType,
    ) -> Result<(), NmError> {
        self.mgr
            .exec(NmMonitorCmd::DelIfaceType(iface_type))
            .await?;
        Ok(())
    }
}
