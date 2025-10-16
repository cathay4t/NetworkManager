// SPDX-License-Identifier: Apache-2.0

use std::time::Duration;

use serde::{Serialize, Serializer, de::DeserializeOwned, ser::SerializeMap};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::UnixStream,
};

use crate::{ErrorKind, NmError, NmLogEntry};

#[derive(Debug)]
/// IPC communication between:
///  * client and daemon
///  * daemon and plugin
///
/// The communication is based UnixStream, the data the format is `size+data`.
/// The size is u32 in big endian. The value should be in JSON format.
pub struct NmIpcConnection {
    /// Timeout in milliseconds.
    pub(crate) timeout_ms: u32,
    pub(crate) socket: UnixStream,
    pub(crate) log_prefix: String,
    pub(crate) log_target: String,
}

impl std::os::fd::AsFd for NmIpcConnection {
    fn as_fd(&self) -> std::os::fd::BorrowedFd<'_> {
        self.socket.as_fd()
    }
}

impl NmIpcConnection {
    const DEFAULT_TIMEOUT_MS: u32 = 30000;

    // Only accept size smaller than 10 MiB
    // TODO(Gris Ge): Provide a mechanism to change this limitation
    const IPC_MAX_SIZE: usize = 1024 * 1024 * 10;

    pub fn set_timeout(&mut self, timeout_ms: u32) {
        self.timeout_ms = timeout_ms;
    }

    pub async fn new_with_path(
        socket_path: &str,
        src_name: &str,
        dst_name: &str,
    ) -> Result<Self, NmError> {
        Ok(Self::new_with_stream(
            UnixStream::connect(socket_path).await.map_err(|e| {
                NmError::new(
                    ErrorKind::Bug,
                    format!("Failed to connect socket {}: {}", socket_path, e),
                )
            })?,
            src_name,
            dst_name,
        ))
    }

    pub fn new_with_stream(
        stream: UnixStream,
        src_name: &str,
        dst_name: &str,
    ) -> Self {
        Self {
            socket: stream,
            timeout_ms: Self::DEFAULT_TIMEOUT_MS,
            log_prefix: format!("{src_name}<->{dst_name}: "),
            log_target: format!("nm.{src_name}"),
        }
    }

    pub async fn send<T>(
        &mut self,
        data: Result<T, NmError>,
    ) -> Result<(), NmError>
    where
        T: NmCanIpc,
    {
        let msg = NmMessage::<T>::from(data);
        let json_str = serde_json::to_string(&msg).map_err(|e| {
            NmError::new(
                ErrorKind::Bug,
                format!("Failed to generate JSON string for {msg:?}: {e}",),
            )
        })?;
        log::trace!("{}sending json: {json_str}", self.log_prefix,);
        let data = json_str.as_bytes();
        if data.len() > Self::IPC_MAX_SIZE {
            return Err(NmError::new(
                ErrorKind::IpcMessageTooLarge,
                format!(
                    "{}Size({}) of IPC message exceeded the maximum \
                     support({}): {}",
                    self.log_prefix,
                    data.len(),
                    Self::IPC_MAX_SIZE,
                    json_str,
                ),
            ));
        }
        let len_bytes = (data.len() as u32).to_be_bytes();

        self.socket.write_all(&len_bytes).await.map_err(|e| {
            if e.kind() == std::io::ErrorKind::BrokenPipe {
                NmError::new(
                    ErrorKind::IpcFailure,
                    format!("{}Connection is closed", self.log_prefix),
                )
            } else {
                NmError::new(
                    ErrorKind::IpcFailure,
                    format!(
                        "{}Failed to send data size to UnixStream: {e}",
                        self.log_prefix
                    ),
                )
            }
        })?;
        self.socket.write_all(data).await.map_err(|e| {
            NmError::new(
                ErrorKind::IpcFailure,
                format!(
                    "{}Failed to send data to UnixStream: {e}",
                    self.log_prefix
                ),
            )
        })?;
        Ok(())
    }

    pub async fn log(&mut self, log: NmLogEntry) -> Result<(), NmError> {
        self.send(Ok(log)).await
    }

    // TODO (Gris Ge): Support redirecting plugin log to user
    pub async fn recv<T>(&mut self) -> Result<T, NmError>
    where
        T: NmCanIpc,
    {
        let mut remain_time = Duration::from_millis(self.timeout_ms.into());
        while remain_time > Duration::ZERO {
            let now = std::time::Instant::now();
            match tokio::time::timeout(remain_time, self._recv::<T>()).await {
                Ok(Ok(msg)) => {
                    let elapsed = now.elapsed();
                    if elapsed >= remain_time {
                        remain_time = Duration::ZERO;
                    } else {
                        remain_time -= elapsed;
                    }
                    match msg {
                        NmMessage::Log(l) => l.emit(),
                        NmMessage::Error(e) => return Err(e),
                        NmMessage::Data(d) => return Ok(d),
                    }
                }
                Ok(Err(e)) => {
                    return Err(e);
                }
                Err(_) => {
                    break;
                }
            }
        }
        Err(NmError::new(
            ErrorKind::Timeout,
            format!("{}Timeout on waiting reply", self.log_prefix),
        ))
    }

    async fn _recv<T>(&mut self) -> Result<NmMessage<T>, NmError>
    where
        T: NmCanIpc + std::fmt::Debug,
    {
        let mut message_size_bytes = 0u32.to_be_bytes();
        self.socket
            .read_exact(&mut message_size_bytes)
            .await
            .map_err(|e| {
                if e.kind() == std::io::ErrorKind::UnexpectedEof {
                    NmError::new(
                        ErrorKind::IpcClosed,
                        format!("{} closed", self.log_prefix),
                    )
                } else {
                    NmError::new(
                        ErrorKind::Bug,
                        format!(
                            "{}Failed to read socket message length: {e}",
                            self.log_prefix
                        ),
                    )
                }
            })?;
        let message_size = u32::from_be_bytes(message_size_bytes) as usize;
        if message_size == 0 {
            return Err(NmError::new(
                ErrorKind::IpcFailure,
                format!("{}Connection is closed by remote", self.log_prefix),
            ));
        }
        if message_size >= Self::IPC_MAX_SIZE {
            return Err(NmError::new(
                ErrorKind::IpcMessageTooLarge,
                format!(
                    "{}Received size({}) of IPC message exceeded the maximum \
                     support({})",
                    self.log_prefix,
                    message_size,
                    Self::IPC_MAX_SIZE
                ),
            ));
        }
        let mut buffer = vec![0u8; message_size];

        if let Err(e) = self.socket.read_exact(&mut buffer).await {
            if e.kind() == std::io::ErrorKind::UnexpectedEof {
                return Err(NmError::new(
                    ErrorKind::IpcFailure,
                    format!(
                        "{}connection closed by other end",
                        self.log_prefix
                    ),
                ));
            } else {
                return Err(NmError::new(
                    ErrorKind::IpcFailure,
                    format!(
                        "{}Failed to read message to buffer with size {}: {}",
                        self.log_prefix, message_size, e
                    ),
                ));
            }
        }
        let json_str = str::from_utf8(&buffer).map_err(|e| {
            NmError::new(
                ErrorKind::IpcFailure,
                format!(
                    "{}Invalid UTF-8 string error: {e}: content:{buffer:?}",
                    self.log_prefix
                ),
            )
        })?;
        log::trace!("{}Received {json_str}", self.log_prefix);
        let ret = NmMessage::from_json(json_str)?;
        Ok(ret)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum NmMessage<T> {
    Error(NmError),
    Log(NmLogEntry),
    Data(T),
}

impl<T> NmMessage<T> {
    pub(crate) fn from_json(json_str: &str) -> Result<Self, NmError>
    where
        T: NmCanIpc,
    {
        let value = serde_json::from_str::<serde_json::Value>(json_str)?;
        let map = if let Some(m) = value.as_object() {
            m
        } else {
            return Err(NmError::new(
                ErrorKind::IpcFailure,
                format!(
                    "Expecting map with 'kind' and 'data', but got: {json_str}"
                ),
            ));
        };

        if let (Some(kind), Some(data_value)) =
            (map.get("kind").and_then(|k| k.as_str()), map.get("data"))
        {
            match kind {
                NmError::IPC_KIND => {
                    Err(serde_json::from_value(data_value.clone())?)
                }
                NmLogEntry::IPC_KIND => Ok(NmMessage::Log(
                    serde_json::from_value(data_value.clone())?,
                )),
                _ => {
                    let data = T::deserialize(data_value)?;
                    if kind == data.ipc_kind() {
                        Ok(NmMessage::Data(T::deserialize(data_value)?))
                    } else {
                        Err(NmError::new(
                            ErrorKind::Bug,
                            format!(
                                "Expecting 'kind' set to {} but got {}",
                                data.ipc_kind(),
                                kind
                            ),
                        ))
                    }
                }
            }
        } else {
            Err(NmError::new(
                ErrorKind::Bug,
                format!(
                    "Expecting 'kind' and 'data', but not defined: {json_str}"
                ),
            ))
        }
    }
}

impl<T: NmCanIpc> Serialize for NmMessage<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut map = serializer.serialize_map(Some(2))?;
        match self {
            Self::Error(e) => {
                map.serialize_entry("kind", &e.ipc_kind())?;
                map.serialize_entry("data", e)?;
            }
            Self::Log(l) => {
                map.serialize_entry("kind", &l.ipc_kind())?;
                map.serialize_entry("data", l)?;
            }
            Self::Data(d) => {
                map.serialize_entry("kind", &d.ipc_kind())?;
                map.serialize_entry("data", d)?;
            }
        }
        map.end()
    }
}

pub trait NmCanIpc:
    Serialize + DeserializeOwned + std::fmt::Debug + Clone
{
    fn ipc_kind(&self) -> String;
}

impl NmCanIpc for String {
    fn ipc_kind(&self) -> String {
        self.to_string()
    }
}

impl NmCanIpc for () {
    fn ipc_kind(&self) -> String {
        "null".to_string()
    }
}

impl<T> NmCanIpc for Result<T, NmError>
where
    T: NmCanIpc,
{
    fn ipc_kind(&self) -> String {
        match self {
            Ok(o) => o.ipc_kind(),
            Err(e) => e.ipc_kind(),
        }
    }
}

impl<T> From<T> for NmMessage<T>
where
    T: NmCanIpc,
{
    fn from(v: T) -> Self {
        Self::Data(v)
    }
}

impl<T> From<Result<T, NmError>> for NmMessage<T>
where
    T: NmCanIpc,
{
    fn from(result: Result<T, NmError>) -> Self {
        match result {
            Ok(v) => Self::Data(v),
            Err(e) => Self::Error(e),
        }
    }
}
