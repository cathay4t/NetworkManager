// SPDX-License-Identifier: Apache-2.0

use super::{base_iface::np_iface_to_base_iface, error::np_error_to_nmstate};
use crate::{
    ErrorKind, EthernetInterface, Interface, InterfaceType, LoopbackInterface,
    NetworkState, NmError, NmNoDaemon, NmstateQueryOption, UnknownInterface,
};

impl NmNoDaemon {
    pub async fn query_network_state(
        option: NmstateQueryOption,
    ) -> Result<NetworkState, NmError> {
        if option.version != 1 {
            return Err(NmError::new(
                ErrorKind::InvalidSchemaVersion,
                format!(
                    "Only support version 1, but desired {}",
                    option.version
                ),
            ));
        }
        // TODO: check other property in NmstateQueryOption

        let mut net_state = NetworkState::default();
        let mut filter = nispor::NetStateFilter::default();
        // Do not query routes in order to prevent BGP routes consuming too much
        // CPU time, we let `get_routes()` do the query by itself.
        filter.route = None;
        let np_state = nispor::NetState::retrieve_with_filter_async(&filter)
            .await
            .map_err(np_error_to_nmstate)?;

        for (_, np_iface) in np_state.ifaces.iter() {
            // The `ovs-system` is reserved for OVS kernel datapath
            if np_iface.name == "ovs-system" {
                continue;
            }
            // The `ovs-netdev` is reserved for OVS netdev datapath
            if np_iface.name == "ovs-netdev" {
                continue;
            }
            // The vti interface is reserved for Ipsec
            if np_iface.iface_type == nispor::IfaceType::Other("Vti".into()) {
                continue;
            }

            let base_iface = np_iface_to_base_iface(np_iface);
            let iface = match &base_iface.iface_type {
                InterfaceType::Ethernet | InterfaceType::Veth => {
                    Interface::Ethernet(Box::new(EthernetInterface::new(
                        base_iface, None,
                    )))
                }
                InterfaceType::Loopback => Interface::Loopback(Box::new(
                    LoopbackInterface::new(base_iface),
                )),
                _ => {
                    log::debug!(
                        "Got unsupported interface {} type {:?}",
                        np_iface.name,
                        np_iface.iface_type
                    );
                    Interface::Unknown({
                        Box::new(UnknownInterface::new(base_iface))
                    })
                }
            };
            net_state.ifaces.push(iface);
        }
        Ok(net_state)
    }
}
