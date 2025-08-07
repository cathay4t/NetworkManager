// SPDX-License-Identifier: Apache-2.0

use serde::{Deserialize, Serialize};

use crate::{NmError, NmLogEntry};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NmMessage {
    #[serde(rename = "type")]
    pub kind: String,
    pub data: String,
}

impl NmResult {
    pub fn is_err(&self) -> bool {
        self.kind == "error"
    }

    pub fn is_log(&self) -> bool {
        self.kind == "log"
    }
}

pub trait Emitable: serde::de::Serialize {
    fn kind() -> String;
}

pub trait Parseable: serde::de::DeserializeOwned {}
