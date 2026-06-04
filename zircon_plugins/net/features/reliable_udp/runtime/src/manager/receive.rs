use zircon_runtime::core::framework::net::{
    ReliableDatagramAck, ReliableDatagramPacket, ReliableDatagramReceiveReport,
    ReliableDatagramReceiveStatus,
};

use super::assembly::InboundFragmentAssembly;
use super::NetReliableUdpRuntimeManager;

impl NetReliableUdpRuntimeManager {
    pub(in crate::manager) fn receive_packet_impl(
        &self,
        packet: ReliableDatagramPacket,
    ) -> ReliableDatagramReceiveReport {
        let mut state = self
            .state
            .lock()
            .expect("net reliable UDP state mutex poisoned");
        if state.completed_inbound_sequences.contains(&packet.sequence) {
            return ReliableDatagramReceiveReport::new(
                packet.sequence,
                packet.channel,
                ReliableDatagramReceiveStatus::DuplicateFragment,
            );
        }
        if packet.fragment_count == 0 || packet.fragment_index >= packet.fragment_count {
            return ReliableDatagramReceiveReport::new(
                packet.sequence,
                packet.channel,
                ReliableDatagramReceiveStatus::InvalidFragment,
            )
            .with_diagnostic("fragment index outside fragment count");
        }
        if packet.fragment_count == 1 {
            state.stats.received_packets += 1;
            state.completed_inbound_sequences.push_back(packet.sequence);
            state.trim_completed_inbound_sequences();
            return ReliableDatagramReceiveReport::new(
                packet.sequence,
                packet.channel,
                ReliableDatagramReceiveStatus::Reassembled,
            )
            .with_ack(ReliableDatagramAck::new(packet.sequence))
            .with_payload(packet.payload);
        }

        let sequence = packet.sequence;
        let channel = packet.channel.clone();
        let status = state
            .inbound_fragments
            .entry(sequence)
            .or_insert_with(|| InboundFragmentAssembly::new(&packet))
            .insert(&packet);
        match status {
            ReliableDatagramReceiveStatus::AcceptedFragment => {
                state.stats.received_packets += 1;
                ReliableDatagramReceiveReport::new(sequence, channel, status)
            }
            ReliableDatagramReceiveStatus::DuplicateFragment => {
                ReliableDatagramReceiveReport::new(sequence, channel, status)
            }
            ReliableDatagramReceiveStatus::InvalidFragment => {
                ReliableDatagramReceiveReport::new(sequence, channel, status)
                    .with_diagnostic("fragment does not match existing assembly")
            }
            ReliableDatagramReceiveStatus::Reassembled => {
                let payload = state
                    .inbound_fragments
                    .remove(&sequence)
                    .expect("reassembled fragment sequence should exist")
                    .payload();
                state.stats.received_packets += 1;
                state.completed_inbound_sequences.push_back(sequence);
                state.trim_completed_inbound_sequences();
                ReliableDatagramReceiveReport::new(sequence, channel, status)
                    .with_ack(ReliableDatagramAck::new(sequence))
                    .with_payload(payload)
            }
        }
    }
}
