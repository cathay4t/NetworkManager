// SPDX-License-Identifier: Apache-2.0

mod plugin;

use nm::NmError;
use nm_plugin::NmPlugin;

use self::plugin::NmPluginDemo;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), NmError> {
    NmPluginDemo::run().await
}
