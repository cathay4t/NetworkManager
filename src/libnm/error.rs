// SPDX-License-Identifier: Apache-2.0

use serde::{Deserialize, Serialize};

use crate::{JsonDisplay, NmCanIpc};

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonDisplay,
)]
#[serde(rename_all = "kebab-case")]
#[non_exhaustive]
pub enum ErrorKind {
    /// Please report this as bug to upstream
    Bug,
    /// Inter-process communication remote end closed
    IpcClosed,
    /// Inter-process communication failure
    IpcFailure,
    /// Data send through NmIpcConnection exceeded the maximum size
    IpcMessageTooLarge,
    /// Invalid log level
    InvalidLogLevel,
    /// Invalid UUID format
    InvalidUuid,
    /// Invalid schema version
    InvalidSchemaVersion,
    /// Invalid argument
    InvalidArgument,
    /// Timeout
    Timeout,
    /// Not supported
    NoSupport,
    /// Plugin failure
    PluginFailure,
    /// Daemon failure
    DaemonFailure,
    /// Post applied state does not match with desired state
    VerificationError,
    /// Permission deny
    PermissionDeny,
}

// Try not implement From for NmError here unless you are sure this
// error should always convert to certain type of ErrorKind.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonDisplay)]
#[non_exhaustive]
pub struct NmError {
    pub kind: ErrorKind,
    pub msg: String,
}

impl NmError {
    pub const IPC_KIND: &'static str = "error";

    pub fn new(kind: ErrorKind, msg: String) -> Self {
        Self { kind, msg }
    }
}

impl std::error::Error for NmError {}

impl From<serde_json::Error> for NmError {
    fn from(e: serde_json::Error) -> Self {
        Self::new(ErrorKind::Bug, format!("serde_json::Error: {e}"))
    }
}

impl NmCanIpc for NmError {
    fn ipc_kind(&self) -> String {
        Self::IPC_KIND.to_string()
    }
}

impl From<std::io::Error> for NmError {
    fn from(e: std::io::Error) -> Self {
        Self::new(ErrorKind::Bug, format!("std::io::Error: {e}"))
    }
}
