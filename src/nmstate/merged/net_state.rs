// SPDX-License-Identifier: Apache-2.0

use serde::{Deserialize, Serialize};

use crate::{
    JsonDisplay, MergedInterfaces, NetworkState, NmstateApplyOption,
    NmstateError,
};

#[derive(
    Clone, Debug, Default, PartialEq, Deserialize, Serialize, JsonDisplay,
)]
#[non_exhaustive]
pub struct MergedNetworkState {
    pub version: Option<u32>,
    pub description: Option<String>,
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
            version: desired.version,
            description: desired.description.clone(),
            ifaces: MergedInterfaces::new(desired.ifaces, current.ifaces)?,
            option,
        })
    }

    pub fn verify(&self, current: &NetworkState) -> Result<(), NmstateError> {
        self.ifaces.verify(&current.ifaces)
    }

    pub fn gen_state_for_apply(&self) -> NetworkState {
        NetworkState {
            ifaces: self.ifaces.gen_state_for_apply(),
            version: self.version,
            description: self.description.clone(),
        }
    }
}

impl NetworkState {
    pub fn merge(&mut self, new_state: &Self) -> Result<(), NmstateError> {
        self.ifaces.merge(&new_state.ifaces)?;
        Ok(())
    }
}
