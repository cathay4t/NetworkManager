// SPDX-License-Identifier: Apache-2.0

use nm::{
    Interface, NetworkState, NmError, NmIpcConnection, WifiLink,
    WifiPhyInterface,
};

use super::{
    dbus::WpaSupDbus, interface::WpaSupInterfaceState, plugin::NmPluginWifi,
};

impl NmPluginWifi {
    pub(crate) async fn query(
        &self,
        _conn: &mut NmIpcConnection,
    ) -> Result<NetworkState, NmError> {
        let mut ret = NetworkState::default();
        for (_, cfg_iface) in self.get_activated_cfgs()?.drain() {
            ret.ifaces.push(Interface::WifiCfg(Box::new(cfg_iface)));
        }

        let dbus = WpaSupDbus::new().await?;

        for wpa_iface in dbus.get_ifaces().await? {
            if let Ok(bss) =
                dbus.get_current_bss(wpa_iface.obj_path.as_str()).await
            {
                let mut wifi_link = WifiLink::from(bss);
                wifi_link.state = Some(wpa_iface.state.into());
                let iface =
                    WifiPhyInterface::new(wpa_iface.iface_name, wifi_link);
                ret.ifaces.push(Interface::WifiPhy(Box::new(iface)));
            }
        }

        Ok(ret)
    }
}
