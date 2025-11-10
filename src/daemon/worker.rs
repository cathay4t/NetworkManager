// SPDX-License-Identifier: Apache-2.0

use futures::{
    SinkExt, StreamExt,
    channel::{
        mpsc::{UnboundedReceiver, UnboundedSender, unbounded},
        oneshot::{Sender, channel},
    },
};
use nm::{ErrorKind, NmError};

pub(crate) trait NmWorker: Sized + Send {
    type Cmd: std::fmt::Display + Send;
    type Reply: Send;
    // Once `associated_type_defaults` feature is stable, we should use this:
    // type FromManager = (Self::Cmd, Sender<Result<Self::Result, NmError>>);

    #[allow(clippy::type_complexity)]
    fn new(
        receiver: UnboundedReceiver<(
            Self::Cmd,
            Sender<Result<Self::Reply, NmError>>,
        )>,
    ) -> impl Future<Output = Result<Self, NmError>> + Send;

    #[allow(clippy::type_complexity)]
    fn receiver(
        &mut self,
    ) -> &mut UnboundedReceiver<(Self::Cmd, Sender<Result<Self::Reply, NmError>>)>;

    fn process_cmd(
        &mut self,
        cmd: Self::Cmd,
    ) -> impl Future<Output = Result<Self::Reply, NmError>> + Send;

    #[allow(clippy::type_complexity)]
    fn recv_cmd(
        &mut self,
    ) -> impl Future<
        Output = Option<(Self::Cmd, Sender<Result<Self::Reply, NmError>>)>,
    > + Send {
        async { self.receiver().next().await }
    }

    /// Default implementation of this function should be invoked in tokio
    /// worker thread.
    /// Return only when sender all dropped(daemon quit).
    fn run(&mut self) -> impl Future<Output = ()> + Send {
        async {
            loop {
                let (cmd, sender) = match self.recv_cmd().await {
                    Some(c) => c,
                    None => break,
                };
                let cmd_str = cmd.to_string();
                let result = self.process_cmd(cmd).await;
                if sender.send(result).is_err() {
                    log::error!("Failed to send reply for command {cmd_str}");
                }
            }
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct NmManager<C, R>
where
    C: std::fmt::Display + Clone,
{
    name: &'static str,
    sender: UnboundedSender<(C, Sender<Result<R, NmError>>)>,
}

impl<C, R> NmManager<C, R>
where
    C: std::fmt::Display + Clone,
{
    pub(crate) async fn new<W>(name: &'static str) -> Result<Self, NmError>
    where
        W: NmWorker<Cmd = C, Reply = R> + 'static,
    {
        let (sender, receiver) = unbounded::<(C, Sender<Result<R, NmError>>)>();

        let mut worker = W::new(receiver).await?;

        tokio::spawn(async move { worker.run().await });

        Ok(Self { name, sender })
    }

    pub(crate) async fn exec(&mut self, cmd: C) -> Result<R, NmError> {
        let (result_sender, result_receiver) = channel::<Result<R, NmError>>();

        self.sender
            .send((cmd.clone(), result_sender))
            .await
            .map_err(|e| {
                NmError::new(
                    ErrorKind::Bug,
                    format!(
                        "Manager {}: failed to send {}: {e}",
                        cmd, self.name
                    ),
                )
            })?;

        result_receiver.await.map_err(|e| {
            NmError::new(
                ErrorKind::Bug,
                format!(
                    "Manager {}: failed to receive reply for {cmd}: {e}",
                    self.name
                ),
            )
        })?
    }
}
