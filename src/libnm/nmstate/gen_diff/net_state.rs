// SPDX-License-Identifier: Apache-2.0

use crate::{MergedNetworkState, NetworkState, NmError};

impl NetworkState {
    /// Generate NetworkState containing only the properties changed comparing
    /// to `old_state`.
    pub fn gen_diff(&self, old: &Self) -> Result<Self, NmError> {
        let mut ret = Self::default();
        let old_version = old.version;
        let old_description = old.description.clone();

        let mut old = old.clone();
        old.ifaces.sanitize_for_diff();

        let mut desired = self.clone();
        desired.ifaces.sanitize_for_diff();

        let merged_state =
            MergedNetworkState::new(desired, old, Default::default())?;

        if self.description != old_description {
            ret.description.clone_from(&self.description);
        }
        if self.version != old_version {
            ret.version.clone_from(&self.version);
        } else {
            ret.version = None;
        }

        ret.ifaces = merged_state.ifaces.gen_diff()?;
        Ok(ret)
    }
}
