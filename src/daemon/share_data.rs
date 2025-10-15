// SPDX-License-Identifier: GPL-3.0-or-later

use std::sync::{Arc, Mutex, MutexGuard};

use nm::{ErrorKind, NmError};

use super::dhcp::NmDhcpV4Manager;

/// Share data among all threads of NM daemon
///
/// Clone of this object does not create new share data. The internal data is
/// shared among all threads of NM daemon
#[derive(Debug, Clone)]
pub(crate) struct NmDaemonShareData {
    dhcpv4_manager: Arc<Mutex<NmDhcpV4Manager>>,
}

impl NmDaemonShareData {
    pub(crate) fn new() -> Self {
        Self {
            dhcpv4_manager: Arc::new(Mutex::new(NmDhcpV4Manager::new())),
        }
    }

    /// Lock access to NmDhcpV4Manager
    pub(crate) fn dhcpv4_manager<'a>(
        &'a mut self,
    ) -> Result<MutexGuard<'a, NmDhcpV4Manager>, NmError> {
        self.dhcpv4_manager.lock().map_err(|e| {
            NmError::new(
                ErrorKind::Bug,
                format!(
                    "Failed to lock NmDhcpV4Manager of NmDaemonShareData: {e}"
                ),
            )
        })
    }
}
