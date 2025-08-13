// SPDX-License-Identifier: Apache-2.0

mod ovsdb;
mod plugin;
mod show;

use self::plugin::NmPluginOvs;
use nm::NmError;
use nm_plugin::NmPlugin;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), NmError> {
    NmPluginOvs::run().await
}
