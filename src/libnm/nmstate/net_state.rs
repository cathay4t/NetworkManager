// SPDX-License-Identifier: Apache-2.0

use serde::{Deserialize, Serialize};

use crate::{CUR_SCHEMA_VERSION, ErrorKind, Interfaces, JsonDisplay, NmError};

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize, JsonDisplay)]
#[serde(deny_unknown_fields)]
#[non_exhaustive]
pub struct NetworkState {
    /// Please set it to 1 explicitly
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub version: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    /// Description for the whole desire state.
    pub description: Option<String>,
    /// Network interfaces
    #[serde(default, rename = "interfaces")]
    pub ifaces: Interfaces,
}

impl Default for NetworkState {
    fn default() -> Self {
        Self {
            version: Some(CUR_SCHEMA_VERSION),
            description: None,
            ifaces: Default::default(),
        }
    }
}

impl NetworkState {
    pub const HIDE_PASSWORD_STR: &str = "<_password_hidden_by_nmstate>";

    pub fn hide_secrets(&mut self) {
        log::debug!("Replacing secrets with {}", Self::HIDE_PASSWORD_STR);
        self.ifaces.hide_secrets();
    }

    pub fn is_empty(&self) -> bool {
        self == &Self {
            version: self.version,
            ..Default::default()
        }
    }

    pub fn new() -> Self {
        Self::default()
    }

    /// Wrapping function of [serde_yaml::from_str()] with error mapped to
    /// [NmError].
    pub fn new_from_yaml(net_state_yaml: &str) -> Result<Self, NmError> {
        match serde_yaml::from_str(net_state_yaml) {
            Ok(s) => Ok(s),
            Err(e) => Err(NmError::new(
                ErrorKind::InvalidArgument,
                format!("Invalid YAML string: {e}"),
            )),
        }
    }
}
