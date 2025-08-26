// SPDX-License-Identifier: Apache-2.0

use serde::{Deserialize, Serialize};

use super::value::copy_undefined_value;
use crate::{BaseInterface, InterfaceType, NmError};

/// Trait implemented by all type of interfaces.
pub trait NmstateInterface:
    std::fmt::Debug + for<'a> Deserialize<'a> + Serialize + Default
{
    fn base_iface(&self) -> &BaseInterface;

    fn base_iface_mut(&mut self) -> &mut BaseInterface;

    fn is_virtual(&self) -> bool;

    fn is_userspace(&self) -> bool;

    fn name(&self) -> &str {
        self.base_iface().name.as_str()
    }

    fn iface_type(&self) -> &InterfaceType {
        &self.base_iface().iface_type
    }

    /// Invoke [BaseInterface::hide_secrets()] and interface specifics
    /// `hide_secrets_iface_specific()`.
    /// Will invoke `hide_secrets_iface_spec()` at the end.
    /// Please do not override this but implement
    /// `hide_secrets_iface_specific()` instead.
    fn hide_secrets(&mut self) {
        self.base_iface_mut().hide_secrets();
        self.hide_secrets_iface_specific();
    }

    fn hide_secrets_iface_specific(&mut self) {}

    fn is_ignore(&self) -> bool {
        self.base_iface().state.is_ignore()
    }

    fn is_up(&self) -> bool {
        self.base_iface().state.is_up()
    }

    fn is_down(&self) -> bool {
        self.base_iface().state.is_down()
    }

    fn is_absent(&self) -> bool {
        self.base_iface().state.is_absent()
    }

    /// Use properties defined in new_state to override Self without
    /// understanding the property meaning and limitation.
    /// Will invoke `merge_iface_specific()` at the end.
    /// Please do not override this function but implement
    /// `merge_iface_specific()` instead.
    fn merge(&mut self, new_state: &Self) -> Result<(), NmError>
    where
        for<'de> Self: Deserialize<'de>,
    {
        let mut new_value = serde_json::to_value(new_state)?;
        let old_value = serde_json::to_value(&self)?;
        copy_undefined_value(&mut new_value, &old_value);

        *self = serde_json::from_value(new_value)?;
        self.base_iface_mut().merge(new_state.base_iface());
        self.merge_iface_specific(new_state)?;

        Ok(())
    }

    /// Please implemented this function if special merge action required
    /// for certain interface type. Do not need to worry about the merge of
    /// [BaseInterface].
    fn merge_iface_specific(
        &mut self,
        _new_state: &Self,
    ) -> Result<(), NmError> {
        Ok(())
    }

    /// Invoke sanitize on the [BaseInterface] and `sanitize_iface_specfic()`.
    fn sanitize(&mut self, is_desired: bool) -> Result<(), NmError> {
        self.base_iface_mut().sanitize(is_desired)?;
        self.sanitize_iface_specfic(is_desired)
    }

    /// Invoke sanitize current for verify on the [BaseInterface] and
    /// `sanitize_iface_specfic()`
    fn sanitize_for_verify(&mut self) {
        self.base_iface_mut().sanitize_for_verify();
        self.sanitize_for_verify_iface_specfic();
    }

    /// Please implement this function if special sanitize action required
    /// for certain interface type. Do not include action for [BaseInterface].
    fn sanitize_iface_specfic(
        &mut self,
        _is_desired: bool,
    ) -> Result<(), NmError> {
        Ok(())
    }

    /// Please implement this function if special sanitize action required
    /// for certain interface type during verification. Do not include action
    /// for [BaseInterface]. Default implementation is empty.
    fn sanitize_for_verify_iface_specfic(&mut self) {}

    /// When generating difference between desired and current, certain value
    /// should be included as context in the output. For example, when
    /// VLAN ID changed, including base-iface as context seems reasonable.
    /// Default implementation does nothing.
    /// This function will invoke `include_diff_context()` against
    /// `BaseInterface`. For any interface specific task, please implement
    /// `include_diff_context_iface_specific()` instead.
    fn include_diff_context(&mut self, desired: &Self, current: &Self) {
        self.base_iface_mut()
            .include_diff_context(desired.base_iface(), current.base_iface());
        self.include_diff_context_iface_specific(desired, current)
    }

    fn include_diff_context_iface_specific(
        &mut self,
        _desired: &Self,
        _current: &Self,
    ) {
    }

    fn from_base(base_iface: BaseInterface) -> Self {
        let mut new = Self::default();
        *new.base_iface_mut() = base_iface;
        new
    }

    /// When generating revert state for desired state, certain value
    /// should be included as context in the output. For example, when
    /// reverting a IP disable action, we should include its original static
    /// IP addresses.
    /// This function will invoke `include_revert_context()` against
    /// `BaseInterface`. For any interface specific task, please implement
    /// `include_revert_context_iface_specific()` instead.
    fn include_revert_context(&mut self, desired: &Self, pre_apply: &Self) {
        self.base_iface_mut().include_revert_context(
            desired.base_iface(),
            pre_apply.base_iface(),
        );
        self.include_revert_context_iface_specific(desired, pre_apply);
    }

    fn include_revert_context_iface_specific(
        &mut self,
        _desired: &Self,
        _pre_apply: &Self,
    ) {
    }
}

pub trait NmstateController {
    fn is_controller(&self) -> bool;
    fn ports(&self) -> Option<Vec<&str>>;
}

impl<T> NmstateController for T
where
    T: NmstateControllerInterface,
{
    fn is_controller(&self) -> bool {
        true
    }

    fn ports(&self) -> Option<Vec<&str>> {
        self.ports()
    }
}

/// Controller Interface
///
/// E.g. Bond, Linux bridge, OVS bridge, VRF
pub trait NmstateControllerInterface: NmstateInterface {
    fn ports(&self) -> Option<Vec<&str>>;
}

pub trait NmstateChild {
    fn is_child(&self) -> bool;

    fn parent(&self) -> Option<&str>;
}

/// Interface depend on its parent interface
///
/// E.g VLAN, VxLAN, MacVlan
pub trait NmstateChildInterface: NmstateInterface {
    fn parent(&self) -> Option<&str>;
}

impl<T> NmstateChild for T
where
    T: NmstateChildInterface,
{
    fn is_child(&self) -> bool {
        true
    }

    fn parent(&self) -> Option<&str> {
        self.parent()
    }
}
