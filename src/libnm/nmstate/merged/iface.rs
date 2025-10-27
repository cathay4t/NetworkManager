// SPDX-License-Identifier: Apache-2.0

// This file is based on the work of nmstate project(https://nmstate.io/) which
// is under license of Apache 2.0, authors of original file are:
//  * Gris Ge <fge@redhat.com>
//  * Fernando Fernandez Mancera <ffmancera@riseup.net>
//  * Wen Liang <liangwen12year@gmail.com>
//  * Íñigo Huguet <ihuguet@redhat.com>
//  * Quique Llorente <ellorent@redhat.com>

use serde::{Deserialize, Serialize};

use crate::{Interface, JsonDisplay, NmError, NmstateInterface};

#[derive(
    Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize, JsonDisplay,
)]
#[non_exhaustive]
pub struct MergedInterface {
    pub desired: Option<Interface>,
    pub current: Option<Interface>,
    pub merged: Interface,
    pub for_apply: Option<Interface>,
    pub for_verify: Option<Interface>,
}

impl MergedInterface {
    pub fn new(
        desired: Option<Interface>,
        current: Option<Interface>,
    ) -> Result<Self, NmError> {
        let merged = match (&desired, &current) {
            (Some(desired), Some(current)) => current.merge(desired)?,
            (Some(state), None) | (None, Some(state)) => state.clone(),
            _ => {
                log::warn!(
                    "BUG: MergedInterface:new() got both desired and current \
                     set to None"
                );
                Interface::default()
            }
        };
        let for_apply = if let Some(desired) = desired.as_ref() {
            let mut ret = desired.clone();
            ret.base_iface_mut().include_extra_for_apply(
                current.as_ref().map(|c| c.base_iface()),
            );
            Some(ret)
        } else {
            None
        };

        Ok(Self {
            for_apply,
            for_verify: desired.clone(),
            desired,
            current,
            merged,
        })
    }

    pub(crate) fn is_desired(&self) -> bool {
        self.desired.is_some()
    }

    pub fn is_changed(&self) -> bool {
        self.for_apply.is_some()
    }

    pub fn hide_secrets(&mut self) {
        for state in [
            self.desired.as_mut(),
            self.current.as_mut(),
            Some(&mut self.merged),
            self.for_apply.as_mut(),
            self.for_verify.as_mut(),
        ]
        .iter_mut()
        {
            if let Some(s) = state {
                s.hide_secrets()
            }
        }
    }
}
