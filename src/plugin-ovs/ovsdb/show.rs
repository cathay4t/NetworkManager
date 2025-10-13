// SPDX-License-Identifier: Apache-2.0

// This file is based on the work of nmstate project(https://nmstate.io/) which
// is under license of Apache 2.0, authors of nmstate origin file are:
//  * Gris Ge <fge@redhat.com>
//  * Fernando Fernandez Mancera <ffmancera@riseup.net>
//  * Miguel Duarte Barroso <mdbarroso@redhat.com>

use std::collections::HashMap;

use nm::{
    BaseInterface, Interface, InterfaceType, Interfaces, NetworkState, NmError,
    NmstateInterface, OvsBridgeConfig, OvsBridgeInterface, OvsBridgePortConfig,
    OvsInterface, UnknownInterface,
};

use super::db::{OvsDbConnection, OvsDbEntry};

pub(crate) async fn ovsdb_is_running() -> bool {
    if let Ok(mut cli) = OvsDbConnection::new().await {
        cli.check_connection().await
    } else {
        false
    }
}

pub(crate) async fn ovsdb_retrieve() -> Result<NetworkState, NmError> {
    let mut ret = NetworkState::new();
    let mut cli = OvsDbConnection::new().await?;
    let ovsdb_ifaces = cli.get_ovs_ifaces().await?;
    let ovsdb_brs = cli.get_ovs_bridges().await?;
    let ovsdb_ports = cli.get_ovs_ports().await?;

    for ovsdb_br in ovsdb_brs.values() {
        let base_iface = BaseInterface::new(
            ovsdb_br.name.to_string(),
            InterfaceType::OvsBridge,
        );
        let bridge_conf =
            parse_ovs_bridge_conf(ovsdb_br, &ovsdb_ports, &ovsdb_ifaces);
        ret.ifaces.push(Interface::OvsBridge(Box::new(
            OvsBridgeInterface::new(base_iface, Some(bridge_conf)),
        )));
    }

    for ovsdb_iface in ovsdb_ifaces.values() {
        if let Some(iface) = ovsdb_iface_to_nmstate(ovsdb_iface, &ret.ifaces) {
            ret.ifaces.push(iface);
        }
    }

    Ok(ret)
}

fn parse_ovs_bridge_conf(
    ovsdb_br: &OvsDbEntry,
    ovsdb_ports: &HashMap<String, OvsDbEntry>,
    ovsdb_ifaces: &HashMap<String, OvsDbEntry>,
) -> OvsBridgeConfig {
    let mut ret = OvsBridgeConfig::default();
    let mut port_confs = Vec::new();
    for port_uuid in ovsdb_br.ports.as_slice() {
        if let Some(ovsdb_port) = ovsdb_ports.get(port_uuid) {
            let mut port_conf = OvsBridgePortConfig::default();
            if ovsdb_port.ports.len() == 1 {
                // The port name is not kernel interface name, so we use
                // Interface table for kernel interface name if found.
                if let Some(ovsdb_iface) =
                    ovsdb_ifaces.get(ovsdb_port.ports.first().unwrap())
                {
                    port_conf.name.clone_from(&ovsdb_iface.name);
                } else {
                    port_conf.name.clone_from(&ovsdb_port.name);
                }
            } else {
                log::warn!("Not supporting OVS Bond yet");
            }
            port_confs.push(port_conf);
        }
    }
    port_confs.sort_unstable_by(|a, b| a.name.as_str().cmp(b.name.as_str()));
    ret.ports = Some(port_confs);
    ret
}

fn ovsdb_iface_to_nmstate(
    ovsdb_iface: &OvsDbEntry,
    ifaces: &Interfaces,
) -> Option<Interface> {
    let mut port_to_ctrl = HashMap::new();
    for iface in ifaces
        .user_ifaces
        .values()
        .filter(|i| i.iface_type() == &InterfaceType::OvsBridge)
    {
        if let Some(ports) = iface.ports() {
            for port in ports {
                port_to_ctrl.insert(port, iface.name());
            }
        }
    }
    let base_iface = BaseInterface::new(
        ovsdb_iface.name.to_string(),
        InterfaceType::OvsInterface,
    );

    let mut iface = match ovsdb_iface.iface_type.as_str() {
        "system" => {
            Interface::Unknown(Box::new(UnknownInterface::new(base_iface)))
        }
        "internal" => {
            Interface::OvsInterface(Box::new(OvsInterface::new(base_iface)))
        }
        "patch" => {
            let ovs_iface = OvsInterface::new(base_iface);
            log::warn!("OVS patch is not supported yet");
            Interface::OvsInterface(Box::new(ovs_iface))
        }
        "dpdk" => {
            let ovs_iface = OvsInterface::new(base_iface);
            log::warn!("OVS DPDK is not supported yet");
            Interface::OvsInterface(Box::new(ovs_iface))
        }
        i => {
            log::debug!("Unknown OVS interface type '{i}'");
            return None;
        }
    };

    if let Some(ctrl) = port_to_ctrl.get(&iface.name()) {
        iface.base_iface_mut().controller = Some(ctrl.to_string());
        iface.base_iface_mut().controller_type = Some(InterfaceType::OvsBridge);
    }

    Some(iface)
}
