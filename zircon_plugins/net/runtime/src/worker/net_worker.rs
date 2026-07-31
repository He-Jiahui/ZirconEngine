use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{mpsc, Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::Duration;

use zircon_runtime::core::framework::net::{
    NetConnectionId, NetEndpoint, NetError, NetEvent, NetListenerId, NetPacket, NetSocketId,
};

use crate::poison_recovery::{lock_or_error, lock_recover, NetSharedState};

use super::egress::{AcceptedTcpConnection, NetEgress, TcpPollResult, WorkerReply};
use super::ingress::NetIngress;
use super::shutdown::NetWorkerShutdownReport;
use super::transport_runtime::run_worker;

const DEFAULT_EGRESS_CAPACITY: usize = 1024;
const DEFAULT_INGRESS_CAPACITY: usize = 1024;
const COMMAND_REPLY_TIMEOUT: Duration = Duration::from_secs(2);

#[derive(Debug)]
pub(crate) struct NetWorker {
    egress: mpsc::SyncSender<NetEgress>,
    ingress: Mutex<mpsc::Receiver<NetIngress>>,
    lifecycle: Mutex<NetWorkerLifecycle>,
    shutdown: AtomicBool,
    #[cfg(test)]
    fail_next_shutdown_after_submit: AtomicBool,
}

#[derive(Debug)]
struct NetWorkerLifecycle {
    thread: Option<JoinHandle<()>>,
    shutdown_reply: Option<mpsc::Receiver<Result<NetWorkerShutdownReport, NetError>>>,
}

impl NetWorker {
    pub(crate) fn spawn(next_connection_id: Arc<AtomicU64>) -> Result<Self, NetError> {
        let (egress, worker_egress) = mpsc::sync_channel(DEFAULT_EGRESS_CAPACITY);
        let (worker_ingress, ingress) = mpsc::sync_channel(DEFAULT_INGRESS_CAPACITY);
        let thread = thread::Builder::new()
            .name("zircon-net-worker".to_string())
            .spawn(move || run_worker(worker_egress, worker_ingress, next_connection_id))
            .map_err(|error| NetError::Io(error.to_string()))?;

        Ok(Self {
            egress,
            ingress: Mutex::new(ingress),
            lifecycle: Mutex::new(NetWorkerLifecycle {
                thread: Some(thread),
                shutdown_reply: None,
            }),
            shutdown: AtomicBool::new(false),
            #[cfg(test)]
            fail_next_shutdown_after_submit: AtomicBool::new(false),
        })
    }

    pub(crate) fn bind_udp(
        &self,
        socket: NetSocketId,
        bind: NetEndpoint,
    ) -> Result<NetEndpoint, NetError> {
        self.request(|reply| NetEgress::BindUdp {
            socket,
            bind,
            reply,
        })
    }

    pub(crate) fn send_udp(
        &self,
        socket: NetSocketId,
        destination: NetEndpoint,
        payload: Vec<u8>,
    ) -> Result<usize, NetError> {
        self.request(|reply| NetEgress::SendUdp {
            socket,
            destination,
            payload,
            reply,
        })
    }

    pub(crate) fn poll_udp(
        &self,
        socket: NetSocketId,
        max_packets: usize,
    ) -> Result<Vec<NetPacket>, NetError> {
        self.request(|reply| NetEgress::PollUdp {
            socket,
            max_packets,
            reply,
        })
    }

    pub(crate) fn close_udp(&self, socket: NetSocketId) -> Result<(), NetError> {
        self.request(|reply| NetEgress::CloseUdp { socket, reply })
    }

    pub(crate) fn listen_tcp(
        &self,
        listener: NetListenerId,
        bind: NetEndpoint,
    ) -> Result<NetEndpoint, NetError> {
        self.request(|reply| NetEgress::ListenTcp {
            listener,
            bind,
            reply,
        })
    }

    pub(crate) fn accept_tcp(
        &self,
        listener: NetListenerId,
        max_connections: usize,
    ) -> Result<Vec<AcceptedTcpConnection>, NetError> {
        self.request(|reply| NetEgress::AcceptTcp {
            listener,
            max_connections,
            reply,
        })
    }

    pub(crate) fn close_tcp_listener(&self, listener: NetListenerId) -> Result<(), NetError> {
        self.request(|reply| NetEgress::CloseTcpListener { listener, reply })
    }

    pub(crate) fn connect_tcp(
        &self,
        connection: NetConnectionId,
        remote: NetEndpoint,
    ) -> Result<(), NetError> {
        self.request(|reply| NetEgress::ConnectTcp {
            connection,
            remote,
            reply,
        })
    }

    pub(crate) fn send_tcp(
        &self,
        connection: NetConnectionId,
        payload: Vec<u8>,
    ) -> Result<usize, NetError> {
        self.request(|reply| NetEgress::SendTcp {
            connection,
            payload,
            reply,
        })
    }

    pub(crate) fn poll_tcp(
        &self,
        connection: NetConnectionId,
        max_bytes: usize,
    ) -> Result<TcpPollResult, NetError> {
        self.request(|reply| NetEgress::PollTcp {
            connection,
            max_bytes,
            reply,
        })
    }

    pub(crate) fn close_tcp(&self, connection: NetConnectionId) -> Result<(), NetError> {
        self.request(|reply| NetEgress::CloseTcp { connection, reply })
    }

    pub(crate) fn drain_ingress(&self, max_events: usize) -> Vec<NetEvent> {
        let mut events = Vec::new();
        let ingress = lock_recover(&self.ingress);
        while events.len() < max_events {
            match ingress.try_recv() {
                Ok(ingress) => events.push(ingress.into_event()),
                Err(_) => break,
            }
        }
        events
    }

    pub(crate) fn shutdown(&self) -> Result<NetWorkerShutdownReport, NetError> {
        if self.shutdown.load(Ordering::SeqCst) {
            return Ok(NetWorkerShutdownReport::default());
        }
        let mut lifecycle = lock_or_error(&self.lifecycle, NetSharedState::WorkerThread)?;
        if self.shutdown.load(Ordering::SeqCst) {
            return Ok(NetWorkerShutdownReport::default());
        }

        if lifecycle.shutdown_reply.is_none() {
            let (reply, receiver) = mpsc::sync_channel(1);
            self.egress
                .try_send(NetEgress::Shutdown { reply })
                .map_err(|error| {
                    NetError::Io(format!("net worker shutdown send failed: {error}"))
                })?;
            lifecycle.shutdown_reply = Some(receiver);

            #[cfg(test)]
            if self
                .fail_next_shutdown_after_submit
                .swap(false, Ordering::SeqCst)
            {
                return Err(NetError::Io(
                    "injected net worker shutdown failure after command submission".to_string(),
                ));
            }
        }

        let response = lifecycle
            .shutdown_reply
            .as_ref()
            .ok_or_else(|| NetError::Io("net worker shutdown reply is missing".to_string()))?
            .recv_timeout(COMMAND_REPLY_TIMEOUT);
        let report = match response {
            Ok(Ok(report)) => report,
            Ok(Err(error)) => {
                lifecycle.shutdown_reply.take();
                let join_result = join_worker_thread(&mut lifecycle);
                self.shutdown.store(true, Ordering::SeqCst);
                join_result?;
                return Err(error);
            }
            Err(mpsc::RecvTimeoutError::Timeout) => {
                return Err(NetError::Io(
                    "net worker shutdown response timed out".to_string(),
                ));
            }
            Err(mpsc::RecvTimeoutError::Disconnected) => {
                lifecycle.shutdown_reply.take();
                let join_result = join_worker_thread(&mut lifecycle);
                self.shutdown.store(true, Ordering::SeqCst);
                join_result?;
                return Err(NetError::Io(
                    "net worker shutdown response channel disconnected".to_string(),
                ));
            }
        };
        lifecycle.shutdown_reply.take();
        let join_result = join_worker_thread(&mut lifecycle);
        self.shutdown.store(true, Ordering::SeqCst);
        join_result?;
        Ok(report)
    }

    pub(crate) fn is_shutdown(&self) -> bool {
        self.shutdown.load(Ordering::SeqCst)
    }

    #[cfg(test)]
    pub(crate) fn poison_thread_for_test(&self) {
        let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let _guard = lock_recover(&self.lifecycle);
            panic!("poison net worker thread for typed-error coverage");
        }));
    }

    #[cfg(test)]
    pub(crate) fn fail_next_shutdown_after_submit_for_test(&self) {
        self.fail_next_shutdown_after_submit
            .store(true, Ordering::SeqCst);
    }

    fn request<T>(&self, build: impl FnOnce(WorkerReply<T>) -> NetEgress) -> Result<T, NetError> {
        if self.is_shutdown() {
            return Err(NetError::Io("net worker is shut down".to_string()));
        }

        let (reply, receiver) = mpsc::sync_channel(1);
        self.egress
            .try_send(build(reply))
            .map_err(|error| NetError::Io(format!("net worker egress send failed: {error}")))?;
        receiver
            .recv_timeout(COMMAND_REPLY_TIMEOUT)
            .map_err(|error| {
                NetError::Io(format!(
                    "net worker command response timed out or closed: {error}"
                ))
            })?
    }
}

impl Drop for NetWorker {
    fn drop(&mut self) {
        if self.shutdown().is_ok() {
            return;
        }

        let (replacement, replacement_receiver) = mpsc::sync_channel(1);
        let egress = std::mem::replace(&mut self.egress, replacement);
        drop(egress);
        drop(replacement_receiver);
        let lifecycle = match self.lifecycle.get_mut() {
            Ok(lifecycle) => lifecycle,
            Err(poisoned) => poisoned.into_inner(),
        };
        lifecycle.shutdown_reply.take();
        if let Some(thread) = lifecycle.thread.take() {
            let _ = thread.join();
        }
        self.shutdown.store(true, Ordering::SeqCst);
    }
}

fn join_worker_thread(lifecycle: &mut NetWorkerLifecycle) -> Result<(), NetError> {
    if let Some(thread) = lifecycle.thread.take() {
        thread
            .join()
            .map_err(|_| NetError::Io("net worker thread panicked during shutdown".into()))?;
    }
    Ok(())
}
