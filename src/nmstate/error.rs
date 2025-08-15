// SPDX-License-Identifier: Apache-2.0

use std::error::Error;

use crate::JsonDisplay;
use serde::{Deserialize, Serialize};

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonDisplay,
)]
#[non_exhaustive]
#[serde(rename_all = "kebab-case")]
pub enum ErrorKind {
    InvalidArgument,
    PluginFailure,
    Bug,
    VerificationError,
    NotImplementedError,
    NotSupportedError,
    KernelIntegerRoundedError,
    DependencyError,
    PolicyError,
    PermissionError,
    SrIovVfNotFound,
}

impl Default for ErrorKind {
    fn default() -> Self {
        Self::Bug
    }
}

#[derive(
    Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize, JsonDisplay,
)]
#[non_exhaustive]
#[serde(rename_all = "kebab-case")]
pub struct NmstateError {
    kind: ErrorKind,
    msg: String,
    line: String,
    position: usize,
}

impl Error for NmstateError {}

impl NmstateError {
    pub fn new(kind: ErrorKind, msg: String) -> Self {
        Self {
            kind,
            msg,
            ..Default::default()
        }
    }

    pub fn kind(&self) -> ErrorKind {
        self.kind
    }

    pub fn msg(&self) -> &str {
        self.msg.as_str()
    }

    pub fn line(&self) -> &str {
        self.line.as_str()
    }

    /// The position of character in line which cause the PolicyError, the
    /// first character is position 0.
    pub fn position(&self) -> usize {
        self.position
    }
}

impl From<serde_json::Error> for NmstateError {
    fn from(e: serde_json::Error) -> Self {
        NmstateError::new(
            ErrorKind::InvalidArgument,
            format!("Invalid property : {e}"),
        )
    }
}

impl From<std::net::AddrParseError> for NmstateError {
    fn from(e: std::net::AddrParseError) -> Self {
        NmstateError::new(
            ErrorKind::InvalidArgument,
            format!("Invalid IP address : {e}"),
        )
    }
}
