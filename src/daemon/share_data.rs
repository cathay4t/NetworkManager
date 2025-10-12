// SPDX-License-Identifier: GPL-3.0-or-later

use super::dhcp::NmDhcpV4Manager;

#[derive(Debug, Default)]
pub(crate) struct NmDaemonShareData {
    pub(crate) dhcpv4_manager: NmDhcpV4Manager,
}
