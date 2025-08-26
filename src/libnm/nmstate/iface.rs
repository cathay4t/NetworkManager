// SPDX-License-Identifier: Apache-2.0

use serde::{Deserialize, Deserializer, Serialize};

use super::value::get_json_value_difference;
use crate::{
    BaseInterface, ErrorKind, EthernetInterface, InterfaceState, InterfaceType,
    JsonDisplay, LoopbackInterface, NmError, NmstateController,
    NmstateInterface, OvsBridgeInterface, OvsInterface, UnknownInterface,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, JsonDisplay)]
#[serde(rename_all = "kebab-case", untagged)]
#[non_exhaustive]
/// Represent a kernel or user space network interface.
pub enum Interface {
    /// Ethernet interface.
    Ethernet(Box<EthernetInterface>),
    /// OVS Bridge
    OvsBridge(Box<OvsBridgeInterface>),
    /// OVS System Interface
    OvsInterface(Box<OvsInterface>),
    /// Loopback Interface
    Loopback(Box<LoopbackInterface>),
    /// Unknown interface.
    Unknown(Box<UnknownInterface>),
}

impl Default for Interface {
    fn default() -> Self {
        Self::Unknown(Box::default())
    }
}

impl<'de> Deserialize<'de> for Interface {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let mut v = serde_json::Value::deserialize(deserializer)?;

        // It is safe to do `v["state"]` here as serde_json will
        // return `json!(null)` for undefined property
        if matches!(
            Option::deserialize(&v["state"])
                .map_err(serde::de::Error::custom)?,
            Some(InterfaceState::Absent)
        ) {
            // Ignore all properties except type if state: absent
            let mut new_value = serde_json::map::Map::new();
            if let Some(n) = v.get("name") {
                new_value.insert("name".to_string(), n.clone());
            }
            if let Some(t) = v.get("type") {
                new_value.insert("type".to_string(), t.clone());
            }
            if let Some(s) = v.get("state") {
                new_value.insert("state".to_string(), s.clone());
            }
            v = serde_json::value::Value::Object(new_value);
        }

        match Option::deserialize(&v["type"])
            .map_err(serde::de::Error::custom)?
        {
            Some(InterfaceType::Ethernet) => {
                let inner = EthernetInterface::deserialize(v)
                    .map_err(serde::de::Error::custom)?;
                Ok(Interface::Ethernet(Box::new(inner)))
            }
            Some(InterfaceType::OvsBridge) => {
                let inner = OvsBridgeInterface::deserialize(v)
                    .map_err(serde::de::Error::custom)?;
                Ok(Interface::OvsBridge(Box::new(inner)))
            }
            Some(InterfaceType::OvsInterface) => {
                let inner = OvsInterface::deserialize(v)
                    .map_err(serde::de::Error::custom)?;
                Ok(Interface::OvsInterface(Box::new(inner)))
            }
            Some(InterfaceType::Loopback) => {
                let inner = LoopbackInterface::deserialize(v)
                    .map_err(serde::de::Error::custom)?;
                Ok(Interface::Loopback(Box::new(inner)))
            }
            _ => {
                let inner = UnknownInterface::deserialize(v)
                    .map_err(serde::de::Error::custom)?;
                Ok(Interface::Unknown(Box::new(inner)))
            }
        }
    }
}

macro_rules! gen_sanitize_iface_specfic {
    ( $desired:ident, $current:ident, $($variant:path,)+ ) => {
        match $desired {
            $(
                $variant(i) => {
                    let cur_iface = if let Some($variant(c)) = $current {
                        Some(c)
                    } else {
                        if $current.is_some() {
                            return Err(NmError::new(
                                ErrorKind::Bug,
                                format!(
                                    "current interface holding the same \
                                    interface type as as desired, current {}, \
                                    desired {}", i.iface_type(),
                                    $current.unwrap().iface_type(),
                                ),
                            ));
                        }
                        None
                    };
                    i.sanitize_iface_specfic(cur_iface.map(|v| &**v))
                }
            )+
        }
    };
}

macro_rules! gen_iface_fun {
    ( $self:ident, $func:ident, $($variant:path,)+ ) => {
        match $self {
            $(
                $variant(i) => i.$func(),
            )+
        }
    };
}

macro_rules! gen_iface_trait_impl {
    ( $(($func:ident, $return:ty),)+ ) => {
        $(
            fn $func(&self) -> $return {
                gen_iface_fun!(
                    self,
                    $func,
                    Self::Ethernet,
                    Self::OvsBridge,
                    Self::OvsInterface,
                    Self::Loopback,
                    Self::Unknown,
                )
            }
        )+
    }
}

macro_rules! gen_iface_trait_impl_mut {
    ( $(($func:ident, $return:ty),)+ ) => {
        $(
            fn $func(&mut self) -> $return {
                gen_iface_fun!(
                    self,
                    $func,
                    Self::Ethernet,
                    Self::OvsBridge,
                    Self::OvsInterface,
                    Self::Loopback,
                    Self::Unknown,
                )
            }
        )+
    }
}

impl NmstateInterface for Interface {
    gen_iface_trait_impl!(
        (is_virtual, bool),
        (is_userspace, bool),
        (base_iface, &BaseInterface),
    );

    gen_iface_trait_impl_mut!(
        (base_iface_mut, &mut BaseInterface),
        (hide_secrets_iface_specific, ()),
        (sanitize_for_verify_iface_specfic, ()),
    );

    fn sanitize_iface_specfic(
        &mut self,
        current: Option<&Self>,
    ) -> Result<(), NmError> {
        gen_sanitize_iface_specfic!(
            self,
            current,
            Interface::Ethernet,
            Interface::OvsBridge,
            Interface::OvsInterface,
            Interface::Loopback,
            Interface::Unknown,
        )
    }

    fn include_diff_context_iface_specific(
        &mut self,
        desired: &Self,
        current: &Self,
    ) {
        match (self, desired, current) {
            (
                Self::Ethernet(i),
                Self::Ethernet(desired),
                Self::Ethernet(current),
            ) => i.include_diff_context_iface_specific(desired, current),
            (
                Self::OvsBridge(i),
                Self::OvsBridge(desired),
                Self::OvsBridge(current),
            ) => i.include_diff_context_iface_specific(desired, current),
            (
                Self::OvsInterface(i),
                Self::OvsInterface(desired),
                Self::OvsInterface(current),
            ) => i.include_diff_context_iface_specific(desired, current),
            (
                Self::Loopback(i),
                Self::Loopback(desired),
                Self::Loopback(current),
            ) => i.include_diff_context_iface_specific(desired, current),
            (
                Self::Unknown(i),
                Self::Unknown(desired),
                Self::Unknown(current),
            ) => i.include_diff_context_iface_specific(desired, current),
            _ => {
                log::error!(
                    "BUG: Interface::include_diff_context_iface_specific() \
                     Unexpected input desired {desired:?} current {current:?}",
                );
            }
        }
    }

    fn include_revert_context_iface_specific(
        &mut self,
        desired: &Self,
        pre_apply: &Self,
    ) {
        match (self, desired, pre_apply) {
            (
                Self::Ethernet(i),
                Self::Ethernet(desired),
                Self::Ethernet(pre_apply),
            ) => i.include_revert_context_iface_specific(desired, pre_apply),
            (
                Self::OvsBridge(i),
                Self::OvsBridge(desired),
                Self::OvsBridge(pre_apply),
            ) => i.include_revert_context_iface_specific(desired, pre_apply),
            (
                Self::OvsInterface(i),
                Self::OvsInterface(desired),
                Self::OvsInterface(pre_apply),
            ) => i.include_revert_context_iface_specific(desired, pre_apply),
            (
                Self::Loopback(i),
                Self::Loopback(desired),
                Self::Loopback(pre_apply),
            ) => i.include_revert_context_iface_specific(desired, pre_apply),
            (
                Self::Unknown(i),
                Self::Unknown(desired),
                Self::Unknown(pre_apply),
            ) => i.include_revert_context_iface_specific(desired, pre_apply),
            _ => {
                log::error!(
                    "BUG: Interface::include_revert_context_iface_specific() \
                     Unexpected input desired {desired:?} pre_apply \
                     {pre_apply:?}"
                );
            }
        }
    }
}

impl NmstateController for Interface {
    fn is_controller(&self) -> bool {
        match self {
            Self::OvsBridge(i) => i.is_controller(),
            _ => false,
        }
    }

    fn ports(&self) -> Option<Vec<&str>> {
        match self {
            Self::OvsBridge(i) => i.ports(),
            _ => None,
        }
    }
}

impl From<BaseInterface> for Interface {
    fn from(base_iface: BaseInterface) -> Self {
        match base_iface.iface_type {
            InterfaceType::Loopback => Interface::Loopback(Box::new(
                LoopbackInterface::from_base(base_iface),
            )),
            InterfaceType::Ethernet | InterfaceType::Veth => {
                Interface::Ethernet(Box::new(EthernetInterface::from_base(
                    base_iface,
                )))
            }
            InterfaceType::OvsBridge => Interface::OvsBridge(Box::new(
                OvsBridgeInterface::from_base(base_iface),
            )),
            InterfaceType::OvsInterface => Interface::OvsInterface(Box::new(
                OvsInterface::from_base(base_iface),
            )),
            InterfaceType::Unknown(_) => Interface::Unknown(Box::new(
                UnknownInterface::from_base(base_iface),
            )),
            _ => {
                log::warn!(
                    "Unsupported interface type {} for interface {}",
                    base_iface.iface_type,
                    base_iface.name
                );
                Interface::Unknown(Box::new(UnknownInterface::from_base(
                    base_iface,
                )))
            }
        }
    }
}

impl Interface {
    pub(crate) fn clone_name_type_only(&self) -> Self {
        self.base_iface().clone_name_type_only().into()
    }

    pub(crate) fn verify(&self, current: &Self) -> Result<(), NmError> {
        let self_value = serde_json::to_value(self.clone())?;
        let current_value = serde_json::to_value(current.clone())?;

        if let Some((reference, desire, current)) = get_json_value_difference(
            format!("{}.interface", self.name()),
            &self_value,
            &current_value,
        ) {
            Err(NmError::new(
                ErrorKind::VerificationError,
                format!(
                    "Verification failure: {reference} desire '{desire}', \
                     current '{current}'"
                ),
            ))
        } else {
            Ok(())
        }
    }
}
