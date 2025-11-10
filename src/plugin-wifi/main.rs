// SPDX-License-Identifier: Apache-2.0

#[macro_use]
mod dbus_macros;

mod apply;
mod bss;
mod dbus;
mod interface;
mod network;
mod plugin;
mod show;

use nm::NmError;
use nm_plugin::NmPlugin;

use self::plugin::NmPluginWifi;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), NmError> {
    NmPluginWifi::run().await
}
