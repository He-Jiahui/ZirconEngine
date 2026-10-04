//! 管理 ACK 移除、按超时/字节预算选择整条消息重发、达到上限后的断开状态。
//! wire ACK 只有 u16 序号，必须与本地 u64 序列的活动窗口对应；调用者需驱动时钟与真实发送。

use std::collections::{HashMap, HashSet, VecDeque};

use zircon_runtime::core::framework::net::{
    ReliableDatagramAck, ReliableDatagramPacket, ReliableDatagramRecoveryState,
};

use super::{NetReliableUdpRuntimeManager, RESEND_ATTEMPT_CAP_DIAGNOSTIC};
use crate::ReliableUdpWireHeader;

impl NetReliableUdpRuntimeManager {
    pub(in crate::manager) fn acknowledge_impl(&self, ack: ReliableDatagramAck) -> usize {
        let mut state = self
            .state
            .lock()
            .expect("net reliable UDP state mutex poisoned");
        let before = state.outbound.len();
        state
            .outbound
            .retain(|packet| packet.sequence != ack.sequence);
        state.resend_state.remove(&ack.sequence);
        let removed = before - state.outbound.len();
        state.stats.received_packets += removed as u64;
        removed
    }

    pub(in crate::manager) fn acknowledge_wire_header_impl(
        &self,
        header: ReliableUdpWireHeader,
    ) -> usize {
        let acked_sequences = header.acked_sequences().into_iter().collect::<HashSet<_>>();
        let mut state = self
            .state
            .lock()
            .expect("net reliable UDP state mutex poisoned");
        let before = state.outbound.len();
        let acknowledged = state
            .outbound
            .iter()
            // BUG: [CR-PLUGIN-NET-0020] 截断 u64 会同时确认相差 65536 的在途消息，活动窗口没有阻止旧/新序号别名。
            .filter(|packet| acked_sequences.contains(&(packet.sequence as u16)))
            .map(|packet| packet.sequence)
            .collect::<HashSet<_>>();
        state
            .outbound
            .retain(|packet| !acknowledged.contains(&packet.sequence));
        for sequence in &acknowledged {
            state.resend_state.remove(sequence);
        }
        let removed = before - state.outbound.len();
        state.stats.received_packets += removed as u64;
        removed
    }

    pub(in crate::manager) fn resend_pending_impl(
        &self,
        max_packets: usize,
    ) -> Vec<ReliableDatagramPacket> {
        let mut state = self
            .state
            .lock()
            .expect("net reliable UDP state mutex poisoned");
        let packets = state
            .outbound
            .iter()
            .take(max_packets)
            .cloned()
            .collect::<Vec<_>>();
        state.stats.resent_packets += packets.len() as u64;
        packets
    }

    pub(in crate::manager) fn resend_due_impl(&self, now_ms: u64) -> Vec<ReliableDatagramPacket> {
        self.resend_due_with_byte_budget_impl(now_ms, usize::MAX)
    }

    pub(in crate::manager) fn resend_due_with_byte_budget_impl(
        &self,
        now_ms: u64,
        max_payload_bytes: usize,
    ) -> Vec<ReliableDatagramPacket> {
        let mut state = self
            .state
            .lock()
            .expect("net reliable UDP state mutex poisoned");
        let mut due_sequences = state.due_resend_sequences(now_ms);
        due_sequences.sort_unstable();
        if due_sequences.is_empty() {
            return Vec::new();
        }

        let max_attempts = state.config.max_resend_attempts;
        let (capped_sequences, resend_sequences): (Vec<_>, Vec<_>) =
            due_sequences.into_iter().partition(|sequence| {
                state
                    .resend_state
                    .get(sequence)
                    .is_some_and(|resend_state| resend_state.attempts >= max_attempts)
            });
        let mut packets_by_sequence = due_packets_by_sequence(&state.outbound, &resend_sequences);
        let mut due_packets = Vec::new();
        let mut remaining_bytes = max_payload_bytes;
        for sequence in resend_sequences {
            let packets = packets_by_sequence.remove(&sequence).unwrap_or_default();
            let byte_cost = packets
                .iter()
                .map(|packet| packet.payload.len())
                .sum::<usize>();
            if byte_cost > remaining_bytes {
                continue;
            }

            let resend_state = state.resend_state.entry(sequence).or_default();
            resend_state.attempts += 1;
            resend_state.last_sent_at_ms = now_ms;
            remaining_bytes -= byte_cost;
            due_packets.extend(packets);
        }

        if !capped_sequences.is_empty() {
            state.drop_capped_sequences(&capped_sequences);
            state.recovery_state = ReliableDatagramRecoveryState::Disconnected;
            state.recovery_diagnostic = Some(RESEND_ATTEMPT_CAP_DIAGNOSTIC.to_string());
        }
        state.stats.resent_packets += due_packets.len() as u64;
        due_packets
    }
}

fn due_packets_by_sequence(
    outbound: &VecDeque<ReliableDatagramPacket>,
    due_sequences: &[u64],
) -> HashMap<u64, Vec<ReliableDatagramPacket>> {
    let due_sequences = due_sequences.iter().copied().collect::<HashSet<_>>();
    let mut packets_by_sequence = HashMap::with_capacity(due_sequences.len());
    for packet in outbound {
        if due_sequences.contains(&packet.sequence) {
            packets_by_sequence
                .entry(packet.sequence)
                .or_insert_with(Vec::new)
                .push(packet.clone());
        }
    }
    packets_by_sequence
}

impl super::state::NetReliableUdpRuntimeState {
    fn due_resend_sequences(&self, now_ms: u64) -> Vec<u64> {
        let resend_timeout_ms = self.config.resend_timeout_ms;
        if resend_timeout_ms == 0 {
            return self.resend_state.keys().copied().collect();
        }
        self.resend_state
            .iter()
            .filter_map(|(sequence, resend_state)| {
                now_ms
                    .saturating_sub(resend_state.last_sent_at_ms)
                    .ge(&resend_timeout_ms)
                    .then_some(*sequence)
            })
            .collect()
    }

    fn drop_capped_sequences(&mut self, sequences: &[u64]) {
        self.outbound
            .retain(|packet| !sequences.contains(&packet.sequence));
        for sequence in sequences {
            self.resend_state.remove(sequence);
        }
    }
}

#[cfg(test)]
#[path = "tests/resend_grouped_due_packet_tests.rs"]
mod grouped_due_packet_tests;
