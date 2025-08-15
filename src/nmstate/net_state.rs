// SPDX-License-Identifier: Apache-2.0

use serde::{Deserialize, Serialize};

use crate::{ErrorKind, Interfaces, JsonDisplay, NmstateError};

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize, JsonDisplay)]
#[serde(deny_unknown_fields)]
#[non_exhaustive]
pub struct NetworkState {
    /// Please set it to 1 explicitly
    #[serde(default)]
    pub version: u32,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    /// Description for the whole desire state.
    pub description: String,
    /// Network interfaces
    #[serde(
        default,
        skip_serializing_if = "Interfaces::is_empty",
        rename = "interfaces"
    )]
    pub ifaces: Interfaces,
}

impl Default for NetworkState {
    fn default() -> Self {
        Self {
            version: 1,
            description: String::new(),
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

    pub fn merge(&mut self, new_state: &Self) -> Result<(), NmstateError> {
        self.ifaces.merge(&new_state.ifaces)?;
        Ok(())
    }

    /// Wrapping function of [serde_yaml::from_str()] with error mapped to
    /// [NmstateError].
    pub fn new_from_yaml(net_state_yaml: &str) -> Result<Self, NmstateError> {
        match serde_yaml::from_str(net_state_yaml) {
            Ok(s) => Ok(s),
            Err(e) => Err(NmstateError::new(
                ErrorKind::InvalidArgument,
                format!("Invalid YAML string: {e}"),
            )),
        }
    }
}
