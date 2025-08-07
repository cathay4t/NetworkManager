// SPDX-License-Identifier: Apache-2.0

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum ErrorKind {
    Bug,
    IpcFailure,
    IpcMessageTooLarge,
    InvalidLogLevel,
    InvalidUuid,
    Timeout,
}

impl std::fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

// Try not implement From for NmError here unless you are sure this
// error should always convert to certain type of ErrorKind.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[non_exhaustive]
pub struct NmError {
    pub kind: ErrorKind,
    pub msg: String,
}

impl NmError {
    pub fn new(kind: ErrorKind, msg: String) -> Self {
        Self { kind, msg }
    }
}

impl std::error::Error for NmError {}

impl std::fmt::Display for NmError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.kind, self.msg)
    }
}

impl From<serde_json::Error> for NmError {
    fn from(e: serde_json::Error) -> Self {
        Self::new(ErrorKind::Bug, format!("serde_json::Error: {e}"))
    }
}
