// SPDX-License-Identifier: Apache-2.0

use std::collections::HashMap;

use nm::{ErrorKind, NmError};
use zvariant::{ObjectPath, OwnedObjectPath};

use super::{interface::WpaSupInterface, network::WpaSupNetwork};

const WPA_SUP_DBUS_IFACE_ROOT: &str = "fi.w1.wpa_supplicant1";
const WPA_SUP_DBUS_IFACE_IFACE: &str = "fi.w1.wpa_supplicant1.Interface";
const WPA_SUP_DBUS_IFACE_NETWORK: &str = "fi.w1.wpa_supplicant1.Network";

// These proxy() macros only generate private struct, hence it should be
// sit with its consumer.
#[zbus::proxy(
    interface = "fi.w1.wpa_supplicant1",
    default_service = "fi.w1.wpa_supplicant1",
    default_path = "/fi/w1/wpa_supplicant1"
)]
trait WpaSupplicant {
    #[zbus(property)]
    fn interfaces(&self) -> zbus::Result<Vec<OwnedObjectPath>>;

    fn create_interface(
        &self,
        iface: HashMap<&str, zvariant::Value<'_>>,
    ) -> zbus::Result<OwnedObjectPath>;

    fn remove_interface(&self, obj_path: OwnedObjectPath) -> zbus::Result<()>;

    fn get_interface(&self, iface_name: &str) -> zbus::Result<OwnedObjectPath>;
}

pub(crate) struct WpaSupDbus<'a> {
    pub(crate) connection: zbus::Connection,
    proxy: WpaSupplicantProxy<'a>,
}

impl WpaSupDbus<'_> {
    pub(crate) async fn new() -> Result<Self, NmError> {
        let connection = zbus::Connection::system().await.map_err(|e| {
            NmError::new(
                ErrorKind::PluginFailure,
                format!("Failed to create system DBUS connection: {e}"),
            )
        })?;
        let proxy =
            WpaSupplicantProxy::new(&connection).await.map_err(|e| {
                NmError::new(
                    ErrorKind::PluginFailure,
                    format!(
                        "Failed to create DBUS proxy to wpa_supplicant: {e}"
                    ),
                )
            })?;

        Ok(Self { connection, proxy })
    }

    pub(crate) async fn get_iface_obj_paths(
        &self,
    ) -> Result<Vec<String>, NmError> {
        Ok(self
            .proxy
            .interfaces()
            .await
            .map_err(map_zbus_err)?
            .into_iter()
            .map(obj_path_to_string)
            .collect())
    }

    pub(crate) async fn get_iface_obj_path(
        &self,
        iface_name: &str,
    ) -> Result<Option<String>, NmError> {
        match self
            .proxy
            .get_interface(iface_name)
            .await
            .map(obj_path_to_string)
        {
            Ok(s) => Ok(Some(s)),
            Err(e) => {
                if let zbus::Error::MethodError(error_path, _, _) = &e
                    && error_path.as_str()
                        == "fi.w1.wpa_supplicant1.InterfaceUnknown"
                {
                    Ok(None)
                } else {
                    Err(map_zbus_err(e))
                }
            }
        }
    }

    pub(crate) async fn get_network_obj_paths(
        &self,
        iface_obj_path: &str,
    ) -> Result<Vec<String>, NmError> {
        let proxy = zbus::Proxy::new(
            &self.connection,
            WPA_SUP_DBUS_IFACE_ROOT,
            iface_obj_path,
            WPA_SUP_DBUS_IFACE_IFACE,
        )
        .await
        .map_err(map_zbus_err)?;
        Ok(proxy
            .get_property::<Vec<OwnedObjectPath>>("Networks")
            .await
            .map_err(map_zbus_err)?
            .into_iter()
            .map(obj_path_to_string)
            .collect())
    }

    pub(crate) async fn get_network(
        &self,
        network_obj_path: &str,
    ) -> Result<WpaSupNetwork, NmError> {
        let obj_path = str_to_obj_path(network_obj_path)?;
        let proxy = zbus::Proxy::new(
            &self.connection,
            WPA_SUP_DBUS_IFACE_ROOT,
            obj_path.as_str(),
            WPA_SUP_DBUS_IFACE_NETWORK,
        )
        .await
        .map_err(map_zbus_err)?;
        let value = proxy
            .get_property::<zvariant::OwnedValue>("Properties")
            .await
            .map_err(map_zbus_err)?;

        WpaSupNetwork::from_value(value, obj_path)
    }

    pub(crate) async fn get_networks(
        &self,
        iface_obj_path: &str,
    ) -> Result<Vec<WpaSupNetwork>, NmError> {
        let mut ret: Vec<WpaSupNetwork> = Vec::new();
        for network_obj_path in
            self.get_network_obj_paths(iface_obj_path).await?
        {
            ret.push(self.get_network(&network_obj_path).await?);
        }
        Ok(ret)
    }

    pub(crate) async fn add_iface(
        &self,
        iface_name: &str,
    ) -> Result<String, NmError> {
        Ok(self
            .proxy
            .create_interface(
                WpaSupInterface::new(iface_name.to_string()).to_value(),
            )
            .await
            .map(obj_path_to_string)
            .map_err(map_zbus_err)?)
    }

    pub(crate) async fn del_iface(
        &self,
        iface_name: &str,
    ) -> Result<(), NmError> {
        let iface_obj_path = self
            .proxy
            .get_interface(iface_name)
            .await
            .map_err(map_zbus_err)?;
        self.proxy
            .remove_interface(iface_obj_path)
            .await
            .map_err(map_zbus_err)
    }

    pub(crate) async fn add_network(
        &self,
        iface_obj_path: &str,
        network: &WpaSupNetwork,
    ) -> Result<String, NmError> {
        let proxy = zbus::Proxy::new(
            &self.connection,
            WPA_SUP_DBUS_IFACE_ROOT,
            iface_obj_path,
            WPA_SUP_DBUS_IFACE_IFACE,
        )
        .await
        .map_err(map_zbus_err)?;
        proxy
            .call::<&str, HashMap<&str, zvariant::Value<'_>>, OwnedObjectPath>(
                "AddNetwork",
                &network.to_value(),
            )
            .await
            .map(obj_path_to_string)
            .map_err(map_zbus_err)
    }

    pub(crate) async fn enable_network(
        &self,
        network_obj_path: &str,
    ) -> Result<(), NmError> {
        let proxy = zbus::Proxy::new(
            &self.connection,
            WPA_SUP_DBUS_IFACE_ROOT,
            network_obj_path,
            WPA_SUP_DBUS_IFACE_NETWORK,
        )
        .await
        .map_err(map_zbus_err)?;
        proxy
            .set_property::<bool>("Enabled", true)
            .await
            .map_err(map_zbus_fdo_err)
    }

    pub(crate) async fn del_network(
        &self,
        iface_obj_path: &str,
        network_obj_path: &str,
    ) -> Result<(), NmError> {
        let network_obj_path = str_to_obj_path(network_obj_path)?;
        let proxy = zbus::Proxy::new(
            &self.connection,
            WPA_SUP_DBUS_IFACE_ROOT,
            iface_obj_path,
            WPA_SUP_DBUS_IFACE_IFACE,
        )
        .await
        .map_err(map_zbus_err)?;
        proxy
            .call::<&str, ObjectPath, ()>("RemoveNetwork", &network_obj_path)
            .await
            .map_err(map_zbus_err)
    }
}

fn obj_path_to_string(obj_path: OwnedObjectPath) -> String {
    obj_path.into_inner().to_string()
}

fn str_to_obj_path(obj_path_str: &str) -> Result<OwnedObjectPath, NmError> {
    OwnedObjectPath::try_from(obj_path_str).map_err(|e| {
        NmError::new(
            ErrorKind::Bug,
            format!(
                "Failed to convert string {obj_path_str} to DBUS object path: \
                 {e}"
            ),
        )
    })
}

pub(crate) fn map_zbus_err(e: zbus::Error) -> NmError {
    NmError::new(
        ErrorKind::PluginFailure,
        format!("DBUS error of wpa_supplicant: {e}"),
    )
}

pub(crate) fn map_zbus_fdo_err(e: zbus::fdo::Error) -> NmError {
    NmError::new(
        ErrorKind::PluginFailure,
        format!("DBUS error of wpa_supplicant: {e}"),
    )
}
