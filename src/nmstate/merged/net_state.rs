// SPDX-License-Identifier: Apache-2.0

use serde::{Deserialize, Serialize};

use crate::{MergedInterfaces, NetworkState, NmstateApplyOption, NmstateError};

#[derive(Clone, Debug, Default, PartialEq, Deserialize, Serialize)]
#[non_exhaustive]
pub struct MergedNetworkState {
    pub ifaces: MergedInterfaces,
    pub option: NmstateApplyOption,
}

impl MergedNetworkState {
    pub fn new(
        desired: NetworkState,
        current: NetworkState,
        option: NmstateApplyOption,
    ) -> Result<Self, NmstateError> {
        Ok(Self {
            ifaces: MergedInterfaces::new(desired.ifaces, current.ifaces)?,
            option,
        })
    }

    pub fn verify(&self, current: &NetworkState) -> Result<(), NmstateError> {
        self.ifaces.verify(&current.ifaces)
    }
}
