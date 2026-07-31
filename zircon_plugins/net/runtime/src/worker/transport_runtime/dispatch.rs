use std::sync::atomic::AtomicU64;
use std::sync::{mpsc, Arc};

use zircon_runtime::core::framework::net::NetError;

use super::WorkerCore;
use crate::worker::egress::{NetEgress, WorkerReply};
use crate::worker::ingress::NetIngress;

pub(in crate::worker) fn run_worker(
    egress: mpsc::Receiver<NetEgress>,
    ingress: mpsc::SyncSender<NetIngress>,
    next_connection_id: Arc<AtomicU64>,
) {
    let Ok(mut core) = WorkerCore::new(ingress, next_connection_id) else {
        return;
    };

    while let Ok(command) = egress.recv() {
        if handle_command(&mut core, command, &egress) {
            break;
        }
    }
}

fn handle_command(
    core: &mut WorkerCore,
    command: NetEgress,
    egress: &mpsc::Receiver<NetEgress>,
) -> bool {
    match command {
        NetEgress::BindUdp {
            socket,
            bind,
            reply,
        } => reply_result(reply, core.bind_udp(socket, bind)),
        NetEgress::SendUdp {
            socket,
            destination,
            payload,
            reply,
        } => reply_result(reply, core.send_udp(socket, destination, &payload)),
        NetEgress::PollUdp {
            socket,
            max_packets,
            reply,
        } => reply_result(reply, core.poll_udp(socket, max_packets)),
        NetEgress::CloseUdp { socket, reply } => reply_result(reply, core.close_udp(socket)),
        NetEgress::ListenTcp {
            listener,
            bind,
            reply,
        } => reply_result(reply, core.listen_tcp(listener, bind)),
        NetEgress::AcceptTcp {
            listener,
            max_connections,
            reply,
        } => reply_result(reply, core.accept_tcp(listener, max_connections)),
        NetEgress::CloseTcpListener { listener, reply } => {
            reply_result(reply, core.close_tcp_listener(listener));
        }
        NetEgress::ConnectTcp {
            connection,
            remote,
            reply,
        } => reply_result(reply, core.connect_tcp(connection, remote)),
        NetEgress::SendTcp {
            connection,
            payload,
            reply,
        } => reply_result(reply, core.send_tcp(connection, &payload)),
        NetEgress::PollTcp {
            connection,
            max_bytes,
            reply,
        } => reply_result(reply, core.poll_tcp(connection, max_bytes)),
        NetEgress::CloseTcp { connection, reply } => {
            reply_result(reply, core.close_tcp(connection));
        }
        NetEgress::Shutdown { reply } => {
            let report = core.shutdown_report(egress.try_iter().count());
            reply_result(reply, Ok(report));
            return true;
        }
    }
    false
}

fn reply_result<T>(reply: WorkerReply<T>, value: Result<T, NetError>) {
    let _ = reply.send(value);
}
