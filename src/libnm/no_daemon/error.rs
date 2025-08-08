// SPDX-License-Identifier: Apache-2.0

use crate::{ErrorKind, NmError};

// TODO: Properly handle cases like:
//  * Permission deny
//  * Invalid argument
pub(crate) fn np_error_to_nmstate(np_error: nispor::NisporError) -> NmError {
    NmError::new(
        ErrorKind::Bug,
        format!("{}: {}", np_error.kind, np_error.msg),
    )
}
