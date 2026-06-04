use zircon_runtime::core::framework::net::{
    ReliableDatagramAck, ReliableDatagramPacket, ReliableDatagramRecoveryState,
};

use super::{NetReliableUdpRuntimeManager, RESEND_ATTEMPT_CAP_DIAGNOSTIC};

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
        let mut state = self
            .state
            .lock()
            .expect("net reliable UDP state mutex poisoned");
        let mut due_sequences = state.due_resend_sequences(now_ms);
        due_sequences.sort_unstable();
        if due_sequences.is_empty() {
            return Vec::new();
        }

        let mut due_packets = Vec::new();
        let mut capped_sequences = Vec::new();
        let max_attempts = state.config.max_resend_attempts;
        for sequence in due_sequences {
            let resend_state = state.resend_state.entry(sequence).or_default();
            if resend_state.attempts >= max_attempts {
                capped_sequences.push(sequence);
                continue;
            }
            resend_state.attempts += 1;
            resend_state.last_sent_at_ms = now_ms;
            due_packets.extend(
                state
                    .outbound
                    .iter()
                    .filter(|packet| packet.sequence == sequence)
                    .cloned(),
            );
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
