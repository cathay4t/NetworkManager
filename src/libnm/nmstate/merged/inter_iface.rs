// SPDX-License-Identifier: Apache-2.0

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::{
    ErrorKind, Interface, InterfaceType, Interfaces, JsonDisplay,
    MergedInterface, NmError, NmstateInterface,
};

#[derive(
    Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize, JsonDisplay,
)]
#[non_exhaustive]
pub struct MergedInterfaces {
    pub kernel_ifaces: HashMap<String, MergedInterface>,
    pub user_ifaces: HashMap<(String, InterfaceType), MergedInterface>,
}

impl MergedInterfaces {
    pub fn new(
        desired: Interfaces,
        current: Interfaces,
    ) -> Result<Self, NmError> {
        let mut desired = desired;
        let mut current = current;

        desired.unify_veth_and_ethernet();
        current.unify_veth_and_ethernet();

        let mut kernel_ifaces: HashMap<String, MergedInterface> =
            HashMap::new();
        let mut user_ifaces: HashMap<(String, InterfaceType), MergedInterface> =
            HashMap::new();
        // TODO: Remove ignore interface
        // TODO: Resolve `type: unknown` in desired based on current state
        for mut des_iface in desired.drain() {
            let cur_iface =
                current.remove(des_iface.name(), Some(des_iface.iface_type()));
            des_iface.sanitize(cur_iface.as_ref())?;
            if des_iface.is_userspace() {
                user_ifaces.insert(
                    (
                        des_iface.name().to_string(),
                        des_iface.iface_type().clone(),
                    ),
                    MergedInterface::new(Some(des_iface), cur_iface)?,
                );
            } else {
                kernel_ifaces.insert(
                    des_iface.name().to_string(),
                    MergedInterface::new(Some(des_iface), cur_iface)?,
                );
            }
        }

        for cur_iface in current.drain() {
            if cur_iface.is_userspace() {
                user_ifaces.insert(
                    (
                        cur_iface.name().to_string(),
                        cur_iface.iface_type().clone(),
                    ),
                    MergedInterface::new(None, Some(cur_iface))?,
                );
            } else {
                kernel_ifaces.insert(
                    cur_iface.name().to_string(),
                    MergedInterface::new(None, Some(cur_iface))?,
                );
            }
        }
        let mut ret = Self {
            kernel_ifaces,
            user_ifaces,
        };

        ret.post_merge_sanitize()?;

        Ok(ret)
    }

    pub fn iter(&self) -> impl Iterator<Item = &MergedInterface> {
        self.user_ifaces.values().chain(self.kernel_ifaces.values())
    }

    pub fn gen_state_for_apply(&self) -> Interfaces {
        let kernel_ifaces: HashMap<String, Interface> = self
            .kernel_ifaces
            .iter()
            .filter_map(|(name, iface)| {
                iface
                    .for_apply
                    .as_ref()
                    .map(|i| (name.to_string(), i.clone()))
            })
            .collect();

        let user_ifaces: HashMap<(String, InterfaceType), Interface> = self
            .user_ifaces
            .iter()
            .filter_map(|((name, iface_type), iface)| {
                iface.for_apply.as_ref().map(|i| {
                    ((name.to_string(), iface_type.clone()), i.clone())
                })
            })
            .collect();

        Interfaces {
            kernel_ifaces,
            user_ifaces,
            ..Default::default()
        }
    }

    pub(crate) fn iter_mut(
        &mut self,
    ) -> impl Iterator<Item = &mut MergedInterface> {
        self.user_ifaces
            .values_mut()
            .chain(self.kernel_ifaces.values_mut())
    }

    pub(crate) fn verify(&self, current: &Interfaces) -> Result<(), NmError> {
        let mut merged = self.clone();
        let mut current = current.clone();

        current.unify_veth_and_ethernet();

        current.sanitize_for_verify();

        for des_iface in merged.iter_mut().filter(|i| i.is_desired()) {
            let iface = if let Some(i) = des_iface.for_verify.as_mut() {
                i
            } else {
                continue;
            };
            if iface.is_absent() || (iface.is_virtual() && iface.is_down()) {
                if let Some(cur_iface) =
                    current.get(iface.name(), Some(iface.iface_type()))
                {
                    verify_desire_absent_but_found_in_current(
                        iface, cur_iface,
                    )?;
                }
            } else if let Some(cur_iface) =
                current.get(iface.name(), Some(iface.iface_type()))
            {
                // Do not verify physical interface with state:down
                if iface.is_up() {
                    iface.verify(cur_iface)?;
                }
            } else if iface.is_up() {
                return Err(NmError::new(
                    ErrorKind::VerificationError,
                    format!(
                        "Failed to find desired interface {} {:?}",
                        iface.name(),
                        iface.iface_type()
                    ),
                ));
            }
        }
        Ok(())
    }

    fn post_merge_sanitize(&mut self) -> Result<(), NmError> {
        for iface in self
            .kernel_ifaces
            .values_mut()
            .chain(self.user_ifaces.values_mut())
        {
            if iface.merged.iface_type() == &InterfaceType::Loopback {
                iface.post_merge_sanitize_loopback();
            }
        }
        Ok(())
    }
}

fn verify_desire_absent_but_found_in_current(
    des_iface: &Interface,
    cur_iface: &Interface,
) -> Result<(), NmError> {
    if cur_iface.is_virtual() {
        // Virtual interface should be deleted by absent action
        Err(NmError::new(
            ErrorKind::VerificationError,
            format!(
                "Absent/Down interface {}/{} still found as {:?}",
                des_iface.name(),
                des_iface.iface_type(),
                cur_iface
            ),
        ))
    } else {
        // Hard to predict real hardware state due to backend variety.
        Ok(())
    }
}

impl Interfaces {
    pub(crate) fn unify_veth_and_ethernet(&mut self) {
        for iface in self
            .kernel_ifaces
            .values_mut()
            .filter(|i| i.iface_type() == &InterfaceType::Veth)
        {
            iface.base_iface_mut().iface_type = InterfaceType::Ethernet;
        }
    }

    pub(crate) fn sanitize_for_verify(&mut self) {
        for iface in self.iter_mut() {
            iface.sanitize_for_verify();
        }
    }
}
