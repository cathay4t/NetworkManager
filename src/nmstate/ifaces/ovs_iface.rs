// SPDX-License-Identifier: Apache-2.0

use serde::{Deserialize, Serialize};

use crate::{BaseInterface, InterfaceType, NmstateError, NmstateInterface};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
#[non_exhaustive]
/// OpenvSwitch Internal Interface
pub struct OvsInterface {
    #[serde(flatten)]
    pub base: BaseInterface,
}

impl OvsInterface {
    pub fn new(base: BaseInterface) -> Self {
        Self { base }
    }
}

impl Default for OvsInterface {
    fn default() -> Self {
        Self {
            base: BaseInterface {
                iface_type: InterfaceType::OvsInterface,
                ..Default::default()
            },
        }
    }
}

impl NmstateInterface for OvsInterface {
    fn base_iface(&self) -> &BaseInterface {
        &self.base
    }

    fn base_iface_mut(&mut self) -> &mut BaseInterface {
        &mut self.base
    }

    fn is_virtual(&self) -> bool {
        true
    }

    fn is_userspace(&self) -> bool {
        false
    }

    fn hide_secrets_iface_specific(&mut self) {}

    fn sanitize_iface_specfic(
        &mut self,
        _is_desired: bool,
    ) -> Result<(), NmstateError> {
        Ok(())
    }

    fn include_diff_context_iface_specific(
        &mut self,
        _desired: &Self,
        _current: &Self,
    ) {
    }

    fn include_revert_context_iface_specific(
        &mut self,
        _desired: &Self,
        _pre_apply: &Self,
    ) {
    }
}
