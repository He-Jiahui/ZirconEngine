use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

use zircon_runtime::core::framework::net::{
    ReliableDatagramAck, ReliableDatagramConfig, ReliableDatagramPacket,
    ReliableDatagramSendReport, ReliableDatagramSendStatus, ReliableDatagramStats,
};

#[derive(Clone, Debug)]
pub struct NetReliableUdpRuntimeManager {
    state: Arc<Mutex<NetReliableUdpRuntimeState>>,
}

#[derive(Debug)]
struct NetReliableUdpRuntimeState {
    config: ReliableDatagramConfig,
    next_sequence: u64,
    outbound: VecDeque<ReliableDatagramPacket>,
    stats: ReliableDatagramStats,
}

impl NetReliableUdpRuntimeManager {
    pub fn new(config: ReliableDatagramConfig) -> Self {
        Self {
            state: Arc::new(Mutex::new(NetReliableUdpRuntimeState {
                config,
                next_sequence: 1,
                outbound: VecDeque::new(),
                stats: ReliableDatagramStats::default(),
            })),
        }
    }

    pub fn enqueue_reliable_datagram(
        &self,
        channel: impl Into<String>,
        payload: impl Into<Vec<u8>>,
    ) -> ReliableDatagramSendReport {
        let channel = channel.into();
        let payload = payload.into();
        let mut state = self
            .state
            .lock()
            .expect("net reliable UDP state mutex poisoned");
        let mtu = state.config.mtu_bytes;
        if mtu == 0 || payload.len() > mtu.saturating_mul(u16::MAX as usize) {
            return ReliableDatagramSendReport::new(
                ReliableDatagramSendStatus::PayloadTooLarge,
                Vec::new(),
            );
        }

        let sequence = state.next_sequence;
        state.next_sequence += 1;
        let packets = if payload.len() <= mtu {
            vec![ReliableDatagramPacket::new(sequence, channel, payload)]
        } else {
            let fragment_count = payload.len().div_ceil(mtu) as u16;
            payload
                .chunks(mtu)
                .enumerate()
                .map(|(index, chunk)| {
                    ReliableDatagramPacket::new(sequence, channel.clone(), chunk.to_vec())
                        .with_fragment(index as u16, fragment_count)
                })
                .collect::<Vec<_>>()
        };
        state.stats.sent_packets += packets.len() as u64;
        state.outbound.extend(packets.iter().cloned());
        let status = if packets.len() > 1 {
            ReliableDatagramSendStatus::Fragmented
        } else {
            ReliableDatagramSendStatus::Queued
        };
        ReliableDatagramSendReport::new(status, packets)
    }

    pub fn acknowledge(&self, ack: ReliableDatagramAck) -> usize {
        let mut state = self
            .state
            .lock()
            .expect("net reliable UDP state mutex poisoned");
        let before = state.outbound.len();
        state
            .outbound
            .retain(|packet| packet.sequence != ack.sequence);
        before - state.outbound.len()
    }

    pub fn pending_packets(&self) -> Vec<ReliableDatagramPacket> {
        self.state
            .lock()
            .expect("net reliable UDP state mutex poisoned")
            .outbound
            .iter()
            .cloned()
            .collect()
    }

    pub fn stats(&self) -> ReliableDatagramStats {
        self.state
            .lock()
            .expect("net reliable UDP state mutex poisoned")
            .stats
            .clone()
    }
}

impl Default for NetReliableUdpRuntimeManager {
    fn default() -> Self {
        Self::new(ReliableDatagramConfig::default())
    }
}

pub fn net_reliable_udp_runtime_manager() -> NetReliableUdpRuntimeManager {
    NetReliableUdpRuntimeManager::default()
}
