// SPDX-License-Identifier: Apache-2.0

use crate::{
    BaseInterface, EthernetConfig, EthernetDuplex, EthernetInterface,
    Interface, NmError, VethConfig,
};

pub(crate) fn apply_ethernet_conf(
    np_iface: &mut nispor::IfaceConf,
    apply_iface: &EthernetInterface,
    cur_iface: Option<&Interface>,
) -> Result<(), NmError> {
    // TODO(Gris Ge): Change veth peer
    if cur_iface.is_none()
        && let Some(peer) =
            // Create new veth, already sanitized, so no need to validate
            apply_iface.veth.as_ref().map(|v| v.peer.as_str())
    {
        np_iface.iface_type = Some(nispor::IfaceType::Veth);
        let mut np_veth_conf = nispor::VethConf::default();
        np_veth_conf.peer = peer.to_string();
        np_iface.veth = Some(np_veth_conf);
    }
    Ok(())
}

impl EthernetInterface {
    pub(crate) fn new_from_nispor(
        base: BaseInterface,
        np_iface: &nispor::Iface,
    ) -> Self {
        Self {
            base,
            veth: get_veth_conf(np_iface),
            ethernet: get_eth_conf(np_iface),
        }
    }
}

fn get_veth_conf(np_iface: &nispor::Iface) -> Option<VethConfig> {
    np_iface
        .veth
        .as_ref()
        .map(|v| v.peer.as_str())
        .map(|peer| VethConfig {
            peer: peer.to_string(),
        })
}

fn get_eth_conf(np_iface: &nispor::Iface) -> Option<EthernetConfig> {
    if let Some(ethtool_info) = &np_iface.ethtool {
        if let Some(link_mode_info) = &ethtool_info.link_mode {
            let mut eth_conf = EthernetConfig::default();

            if link_mode_info.speed > 0 {
                eth_conf.speed = Some(link_mode_info.speed);
            }
            eth_conf.auto_neg = Some(link_mode_info.auto_negotiate);
            match link_mode_info.duplex {
                nispor::EthtoolLinkModeDuplex::Full => {
                    eth_conf.duplex = Some(EthernetDuplex::Full);
                }
                nispor::EthtoolLinkModeDuplex::Half => {
                    eth_conf.duplex = Some(EthernetDuplex::Half);
                }
                _ => (),
            }
            return Some(eth_conf);
        }
    }
    None
}
