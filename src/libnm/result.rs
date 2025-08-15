// SPDX-License-Identifier: Apache-2.0

use serde::{Deserialize, Serialize};

use crate::{NmError, NmLogEntry};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonDisplay)]
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

    pub fn to_log(&self) -> Option<NmLogEntry> {
        if self.is_log() {
            match serde_json::from_str::<NmLogEntry>(self.data.as_str()) {
                Ok(l) => Some(l),
                Err(e) => {
                    log::warn!(
                        "Ignoring failure on converting NmMessage to \
                         NmLogEntry: {e}"
                    );
                    None
                }
            }
        } else {
            None
        }
    }
}
