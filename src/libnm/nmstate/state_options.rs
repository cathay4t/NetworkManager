// SPDX-License-Identifier: Apache-2.0

use serde::{Deserialize, Serialize};

use crate::{CUR_SCHEMA_VERSION, JsonDisplay};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonDisplay)]
#[non_exhaustive]
pub struct NmstateQueryOption {
    /// Schema version for output
    #[serde(default)]
    pub version: u32,
    /// Which kind of NetworkState to query
    #[serde(default)]
    pub kind: NmstateStateKind,
}

impl Default for NmstateQueryOption {
    fn default() -> Self {
        Self {
            version: CUR_SCHEMA_VERSION,
            kind: NmstateStateKind::default(),
        }
    }
}

impl NmstateQueryOption {
    pub fn running() -> Self {
        Self {
            kind: NmstateStateKind::RunningNetworkState,
            ..Default::default()
        }
    }

    pub fn saved() -> Self {
        Self {
            kind: NmstateStateKind::SavedNetworkState,
            ..Default::default()
        }
    }
}

#[derive(
    Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize, JsonDisplay,
)]
#[non_exhaustive]
#[serde(rename_all = "kebab-case")]
pub enum NmstateStateKind {
    /// The current running network state
    #[default]
    RunningNetworkState,
    /// Network state stored in daemon
    SavedNetworkState,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonDisplay)]
#[non_exhaustive]
#[serde(rename_all = "kebab-case")]
pub struct NmstateApplyOption {
    /// Do not verify whether post applied state matches with desired state.
    pub no_verify: bool,
}

impl Default for NmstateApplyOption {
    fn default() -> Self {
        Self { no_verify: false }
    }
}

impl NmstateApplyOption {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn no_verify(mut self) -> Self {
        self.no_verify = true;
        self
    }
}
