// SPDX-License-Identifier: Apache-2.0

mod ovsdb;
mod plugin;
mod show;

use nm::NmError;
use nm_plugin::NmPlugin;

use self::plugin::NmPluginOvs;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), NmError> {
    NmPluginOvs::run().await
}
