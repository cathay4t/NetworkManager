// SPDX-License-Identifier: Apache-2.0

use std::collections::HashSet;

use futures::{
    channel::{mpsc::UnboundedReceiver, oneshot::Sender},
    stream::StreamExt,
};
use nm::{ErrorKind, LinkEvent, NmClient, NmError};
use rtnetlink::{
    MulticastGroup, new_multicast_connection,
    packet_core::{NetlinkMessage, NetlinkPayload},
    packet_route::{
        RouteNetlinkMessage,
        link::{LinkAttribute, State},
    },
    sys::SocketAddr,
};

use crate::TaskWorker;

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum NmMonitorCmd {
    AddIface(String),
    DelIface(String),
    /// Stop the monitoring but preserving the internal monitoring list
    Pause,
    /// Resume the monitoring, emit current status of monitoring
    /// interface list.
    Resume,
}

impl std::fmt::Display for NmMonitorCmd {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::AddIface(iface) => {
                write!(f, "start-iface-monitor:{iface}")
            }
            Self::DelIface(iface) => {
                write!(f, "stop-iface-monitor:{iface}")
            }
            Self::Pause => {
                write!(f, "pause-monitor")
            }
            Self::Resume => {
                write!(f, "resume-monitor")
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum NmMonitorReply {
    None,
}

type FromManager = (NmMonitorCmd, Sender<Result<NmMonitorReply, NmError>>);

#[derive(Debug)]
pub(crate) struct NmMonitorWorker {
    receiver: UnboundedReceiver<FromManager>,
    netlink_handle: Option<rtnetlink::Handle>,
    netlink_msg: Option<
        UnboundedReceiver<(NetlinkMessage<RouteNetlinkMessage>, SocketAddr)>,
    >,
    iface_monitor_list: HashSet<String>,
}

impl TaskWorker for NmMonitorWorker {
    type Cmd = NmMonitorCmd;
    type Reply = NmMonitorReply;

    async fn new(
        receiver: UnboundedReceiver<FromManager>,
    ) -> Result<Self, NmError> {
        Ok(Self {
            receiver,
            iface_monitor_list: HashSet::new(),
            netlink_handle: None,
            netlink_msg: None,
        })
    }

    fn receiver(&mut self) -> &mut UnboundedReceiver<FromManager> {
        &mut self.receiver
    }

    async fn process_cmd(
        &mut self,
        cmd: NmMonitorCmd,
    ) -> Result<NmMonitorReply, NmError> {
        log::debug!("Processing monitor command: {cmd}");
        match cmd {
            NmMonitorCmd::AddIface(iface) => {
                self.iface_monitor_list.insert(iface);
                if self.netlink_msg.is_none() {
                    self.resume().await?;
                }
            }
            NmMonitorCmd::DelIface(iface) => {
                self.iface_monitor_list.remove(&iface);
                if self.iface_monitor_list.is_empty() {
                    self.pause();
                }
            }
            NmMonitorCmd::Pause => {
                self.pause();
            }
            NmMonitorCmd::Resume => {
                self.resume().await?;
            }
        }
        Ok(NmMonitorReply::None)
    }

    async fn run(&mut self) {
        loop {
            if let Some(mut netlink_msg) = self.netlink_msg.take() {
                tokio::select! {
                    cmd_result = self.recv_cmd() => {
                        if let Some((cmd, sender)) = cmd_result {
                            let cmd_str = cmd.to_string();
                            let result = self.process_cmd(cmd).await;
                            if sender.send(result).is_err() {
                                log::error!(
                                    "Failed to send reply for command {cmd_str}"
                                );
                            }
                        } else {
                            break;
                        }
                    }
                    result = netlink_msg.next() => {
                        if let Some((nl_msg, _)) = result {
                            if let Err(e) = process_rtnl_message(
                                nl_msg,
                                &self.iface_monitor_list
                            ).await {
                                log::error!("{e}");
                            }
                        }
                    }
                }
                self.netlink_msg = Some(netlink_msg);
            } else if let Some((cmd, sender)) = self.recv_cmd().await {
                let cmd_str = cmd.to_string();
                let result = self.process_cmd(cmd).await;
                if sender.send(result).is_err() {
                    log::error!("Failed to send reply for command {cmd_str}");
                }
            } else {
                break;
            }
        }
    }
}

impl NmMonitorWorker {
    fn pause(&mut self) {
        self.netlink_handle = None;
        self.netlink_msg = None;
    }

    async fn resume(&mut self) -> Result<(), NmError> {
        let (conn, handle, msg) =
            new_multicast_connection(&[MulticastGroup::Link]).map_err(|e| {
                NmError::new(
                    ErrorKind::InvalidArgument,
                    format!(
                        "Failed to create netlink multicast socket for \
                         interface monitor: {e}"
                    ),
                )
            })?;
        tokio::spawn(conn);

        // Just send out link query request to kernel, no need to process reply
        // as the reply will be queued in `netlink_msg`.
        let mut link_handle = handle.link().get().execute();
        while let Some(Ok(link_msg)) = link_handle.next().await {
            if let Some(event) =
                parse_nl_attrs_to_link_event(link_msg.attributes.as_slice())
                && self.iface_monitor_list.contains(event.iface_name())
            {
                let mut client =
                    NmClient::new_with_name("daemon-monitor").await?;
                client.notify_link_event(event).await?;
            }
        }

        self.netlink_handle = Some(handle);
        self.netlink_msg = Some(msg);
        Ok(())
    }
}

async fn process_rtnl_message(
    nl_msg: NetlinkMessage<RouteNetlinkMessage>,
    iface_monitor_list: &HashSet<String>,
) -> Result<(), NmError> {
    if let Some(event) = parse_rtnl_message(nl_msg)
        && iface_monitor_list.contains(event.iface_name())
    {
        let mut client = NmClient::new_with_name("daemon-monitor").await?;
        client.notify_link_event(event).await?;
    }
    Ok(())
}

fn parse_nl_attrs_to_link_event(attrs: &[LinkAttribute]) -> Option<LinkEvent> {
    let iface_name = attrs.iter().find_map(|attr| {
        if let &LinkAttribute::IfName(iface_name) = &attr {
            Some(iface_name.to_string())
        } else {
            None
        }
    })?;

    if let Some(state) = attrs.iter().find_map(|attr| {
        if let &LinkAttribute::OperState(s) = attr {
            Some(s)
        } else {
            None
        }
    }) {
        match state {
            State::Up => {
                return Some(LinkEvent::LinkCarrierUp(iface_name.to_string()));
            }
            State::Down => {
                return Some(LinkEvent::LinkCarrierDown(
                    iface_name.to_string(),
                ));
            }
            _ => (),
        };
    }
    None
}

fn parse_rtnl_message(
    nl_msg: NetlinkMessage<RouteNetlinkMessage>,
) -> Option<LinkEvent> {
    if let NetlinkPayload::InnerMessage(RouteNetlinkMessage::NewLink(
        link_msg,
    )) = nl_msg.payload
    {
        parse_nl_attrs_to_link_event(link_msg.attributes.as_slice())
    } else {
        None
    }
}
