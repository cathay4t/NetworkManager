// SPDX-License-Identifier: Apache-2.0

use nm::{InterfaceType, NmCanIpc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
#[non_exhaustive]
pub struct NmPluginInfo {
    pub name: String,
    pub version: String,
    pub iface_types: Vec<InterfaceType>,
}

impl NmPluginInfo {
    pub fn new(
        name: String,
        version: String,
        iface_types: Vec<InterfaceType>,
    ) -> Self {
        Self {
            name,
            version,
            iface_types,
        }
    }
}

impl NmCanIpc for NmPluginInfo {
    fn ipc_kind(&self) -> String {
        "nm-plugin-info".to_string()
    }
}
