// SPDX-License-Identifier: GPL-3.0-or-later

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
