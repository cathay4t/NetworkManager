// SPDX-License-Identifier: Apache-2.0

use serde::{Deserialize, Serialize};

use crate::{ErrorKind, NmError};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(into = "String")]
#[serde(try_from = "String")]
#[non_exhaustive]
pub enum LinkEvent {
    LinkCarrierUp(String),
    LinkCarrierDown(String),
}

impl TryFrom<String> for LinkEvent {
    type Error = NmError;

    fn try_from(v: String) -> Result<Self, NmError> {
        if let Some(iface_name) = v.strip_prefix("link-carrier-up:") {
            Ok(Self::LinkCarrierUp(iface_name.to_string()))
        } else if let Some(iface_name) = v.strip_prefix("link-carrier-down:") {
            Ok(Self::LinkCarrierDown(iface_name.to_string()))
        } else {
            Err(NmError::new(
                ErrorKind::InvalidArgument,
                format!(
                    "Invalid LinkEvent, expecting \
                     `link-carrier-up:<iface_name>` or \
                     `link-carrier-down:<iface_name>`, but got {v}"
                ),
            ))
        }
    }
}

impl From<LinkEvent> for String {
    fn from(v: LinkEvent) -> String {
        v.to_string()
    }
}

impl std::fmt::Display for LinkEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::LinkCarrierUp(iface) => {
                write!(f, "link-carrier-up:{}", iface)
            }
            Self::LinkCarrierDown(iface) => {
                write!(f, "link-carrier-down:{}", iface)
            }
        }
    }
}

impl LinkEvent {
    pub fn iface_name(&self) -> &str {
        match self {
            Self::LinkCarrierUp(iface) | Self::LinkCarrierDown(iface) => iface,
        }
    }

    pub fn is_link_up(&self) -> bool {
        matches!(self, Self::LinkCarrierUp(_))
    }

    pub fn is_link_down(&self) -> bool {
        matches!(self, Self::LinkCarrierDown(_))
    }
}
