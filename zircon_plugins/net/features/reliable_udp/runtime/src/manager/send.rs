use zircon_runtime::core::framework::net::{
    ReliableDatagramPacket, ReliableDatagramSendReport, ReliableDatagramSendStatus,
};

use super::NetReliableUdpRuntimeManager;

impl NetReliableUdpRuntimeManager {
    pub(in crate::manager) fn enqueue_reliable_datagram_impl(
        &self,
        channel: String,
        payload: Vec<u8>,
    ) -> ReliableDatagramSendReport {
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
        state.resend_state.entry(sequence).or_default();
        let status = if packets.len() > 1 {
            ReliableDatagramSendStatus::Fragmented
        } else {
            ReliableDatagramSendStatus::Queued
        };
        ReliableDatagramSendReport::new(status, packets)
    }
}
