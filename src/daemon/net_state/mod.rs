// SPDX-License-Identifier: Apache-2.0

mod apply;
mod query;

pub(crate) use self::apply::apply_network_state;
pub(crate) use self::query::query_network_state;
