// SPDX-License-Identifier: Apache-2.0

use serde::{Deserialize, Serialize};

use crate::CUR_SCHEMA_VERSION;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct NmstateQueryOption {
    #[serde(default)]
    /// Schema version for output
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

#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize)]
#[non_exhaustive]
#[serde(rename_all = "kebab-case")]
pub enum NmstateStateKind {
    /// The current running network state
    #[default]
    RunningNetworkState,
    /// Network state stored in daemon
    SavedNetworkState,
}

impl std::fmt::Display for NmstateStateKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::RunningNetworkState => "running_network_state",
                Self::SavedNetworkState => "saved_network_state",
            }
        )
    }
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[non_exhaustive]
pub struct NmstateApplyOption {
    // Seconds to rollback desired state after applied.
    //pub revert_after: u32,
    /// Do not store desired state to persistent
    pub memory_only: bool,
    /// Do not verify whether post applied state matches with desired state.
    pub no_verify: bool,
    /// Indicate desire state is generate by diff between running state and
    /// saved state, when creating commit, the `pre_apply_state` should
    /// be post state after last commit
    pub is_diff: bool,
}
