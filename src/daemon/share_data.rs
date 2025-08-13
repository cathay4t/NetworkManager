// SPDX-License-Identifier: Apache-2.0

#[derive(Debug)]
pub(crate) struct NmDaemonShareData {
    // Place holder
    _foo: String,
}

impl NmDaemonShareData {
    pub(crate) fn new() -> Self {
        Self {
            _foo: "place_holder".into(),
        }
    }
}
