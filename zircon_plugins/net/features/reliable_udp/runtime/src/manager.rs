use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex};

use zircon_runtime::core::framework::net::{
    ReliableDatagramAck, ReliableDatagramConfig, ReliableDatagramDeliveryReport,
    ReliableDatagramPacket, ReliableDatagramReceiveReport, ReliableDatagramReceiveStatus,
    ReliableDatagramRecoveryReport, ReliableDatagramRecoveryState, ReliableDatagramSendReport,
    ReliableDatagramSendStatus, ReliableDatagramSimulationProfile, ReliableDatagramStats,
};

const RESEND_ATTEMPT_CAP_DIAGNOSTIC: &str = "reliable datagram resend attempt cap exceeded";

#[derive(Clone, Debug)]
pub struct NetReliableUdpRuntimeManager {
    state: Arc<Mutex<NetReliableUdpRuntimeState>>,
}

#[derive(Debug)]
struct NetReliableUdpRuntimeState {
    config: ReliableDatagramConfig,
    next_sequence: u64,
    outbound: VecDeque<ReliableDatagramPacket>,
    resend_state: HashMap<u64, PendingResendState>,
    inbound_fragments: HashMap<u64, InboundFragmentAssembly>,
    completed_inbound_sequences: VecDeque<u64>,
    simulation_profile: ReliableDatagramSimulationProfile,
    simulated_packet_counter: u64,
    recovery_state: ReliableDatagramRecoveryState,
    dropped_packets_since_recovery: u64,
    recovery_diagnostic: Option<String>,
    stats: ReliableDatagramStats,
}

#[derive(Clone, Debug, Default)]
struct PendingResendState {
    last_sent_at_ms: u64,
    attempts: u8,
}

// Keeps partial datagrams in fragment-index order so out-of-order delivery can be
// reassembled without leaking runtime-owned buffers through the public contract.
#[derive(Debug)]
struct InboundFragmentAssembly {
    channel: String,
    fragment_count: u16,
    fragments: Vec<Option<Vec<u8>>>,
}

impl InboundFragmentAssembly {
    fn new(packet: &ReliableDatagramPacket) -> Self {
        Self {
            channel: packet.channel.clone(),
            fragment_count: packet.fragment_count,
            fragments: vec![None; packet.fragment_count as usize],
        }
    }

    fn insert(&mut self, packet: &ReliableDatagramPacket) -> ReliableDatagramReceiveStatus {
        if packet.fragment_count != self.fragment_count
            || packet.channel != self.channel
            || packet.fragment_index >= self.fragment_count
        {
            return ReliableDatagramReceiveStatus::InvalidFragment;
        }
        let fragment = &mut self.fragments[packet.fragment_index as usize];
        if fragment.is_some() {
            return ReliableDatagramReceiveStatus::DuplicateFragment;
        }
        *fragment = Some(packet.payload.clone());
        if self.fragments.iter().all(Option::is_some) {
            ReliableDatagramReceiveStatus::Reassembled
        } else {
            ReliableDatagramReceiveStatus::AcceptedFragment
        }
    }

    fn payload(&self) -> Vec<u8> {
        self.fragments
            .iter()
            .flat_map(|fragment| fragment.as_deref().unwrap_or_default())
            .copied()
            .collect()
    }
}

impl NetReliableUdpRuntimeManager {
    pub fn new(config: ReliableDatagramConfig) -> Self {
        Self {
            state: Arc::new(Mutex::new(NetReliableUdpRuntimeState {
                config,
                next_sequence: 1,
                outbound: VecDeque::new(),
                resend_state: HashMap::new(),
                inbound_fragments: HashMap::new(),
                completed_inbound_sequences: VecDeque::new(),
                simulation_profile: ReliableDatagramSimulationProfile::default(),
                simulated_packet_counter: 0,
                recovery_state: ReliableDatagramRecoveryState::Connected,
                dropped_packets_since_recovery: 0,
                recovery_diagnostic: None,
                stats: ReliableDatagramStats::default(),
            })),
        }
    }

    pub fn set_simulation_profile(&self, profile: ReliableDatagramSimulationProfile) {
        let mut state = self
            .state
            .lock()
            .expect("net reliable UDP state mutex poisoned");
        state.simulation_profile = profile;
        state.simulated_packet_counter = 0;
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
        state.resend_state.entry(sequence).or_default();
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
        state.resend_state.remove(&ack.sequence);
        let removed = before - state.outbound.len();
        state.stats.received_packets += removed as u64;
        removed
    }

    pub fn simulate_outbound_delivery(
        &self,
        packets: impl IntoIterator<Item = ReliableDatagramPacket>,
    ) -> ReliableDatagramDeliveryReport {
        let mut state = self
            .state
            .lock()
            .expect("net reliable UDP state mutex poisoned");
        let mut delivered = Vec::new();
        let mut dropped = Vec::new();
        for packet in packets {
            state.simulated_packet_counter += 1;
            if state.should_drop_simulated_packet() {
                state.stats.dropped_packets += 1;
                state.dropped_packets_since_recovery += 1;
                dropped.push(packet);
            } else {
                delivered.push(packet);
            }
        }
        reorder_delivered_packets(&mut delivered, state.simulation_profile.reorder_window);
        state.update_recovery_after_delivery();
        ReliableDatagramDeliveryReport::new(delivered, dropped, state.recovery_report())
    }

    pub fn receive_packet(&self, packet: ReliableDatagramPacket) -> ReliableDatagramReceiveReport {
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

    pub fn recovery_state(&self) -> ReliableDatagramRecoveryReport {
        self.state
            .lock()
            .expect("net reliable UDP state mutex poisoned")
            .recovery_report()
    }

    pub fn mark_disconnected(
        &self,
        diagnostic: impl Into<String>,
    ) -> ReliableDatagramRecoveryReport {
        let mut state = self
            .state
            .lock()
            .expect("net reliable UDP state mutex poisoned");
        state.recovery_state = ReliableDatagramRecoveryState::Disconnected;
        state.recovery_diagnostic = Some(diagnostic.into());
        state.recovery_report()
    }

    pub fn mark_recovered(&self) -> ReliableDatagramRecoveryReport {
        let mut state = self
            .state
            .lock()
            .expect("net reliable UDP state mutex poisoned");
        state.recovery_state = ReliableDatagramRecoveryState::Connected;
        state.dropped_packets_since_recovery = 0;
        state.recovery_diagnostic = None;
        state.recovery_report()
    }

    pub fn resend_pending(&self, max_packets: usize) -> Vec<ReliableDatagramPacket> {
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

    pub fn resend_due(&self, now_ms: u64) -> Vec<ReliableDatagramPacket> {
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

    pub fn record_dropped_packet(&self) {
        let mut state = self
            .state
            .lock()
            .expect("net reliable UDP state mutex poisoned");
        state.stats.dropped_packets += 1;
        state.dropped_packets_since_recovery += 1;
        state.update_recovery_after_delivery();
    }

    pub fn record_rtt_ms(&self, rtt_ms: f32) {
        self.state
            .lock()
            .expect("net reliable UDP state mutex poisoned")
            .stats
            .rtt_ms = rtt_ms;
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

impl NetReliableUdpRuntimeState {
    fn should_drop_simulated_packet(&self) -> bool {
        self.simulation_profile
            .drop_every_nth_packet
            .is_some_and(|packet_interval| self.simulated_packet_counter % packet_interval == 0)
    }

    fn update_recovery_after_delivery(&mut self) {
        if self.recovery_state == ReliableDatagramRecoveryState::Disconnected {
            return;
        }
        self.recovery_state = match self.simulation_profile.recovery_drop_threshold {
            Some(threshold) if self.dropped_packets_since_recovery >= threshold => {
                ReliableDatagramRecoveryState::Recovering
            }
            _ => ReliableDatagramRecoveryState::Connected,
        };
        self.recovery_diagnostic = (self.recovery_state
            == ReliableDatagramRecoveryState::Recovering)
            .then(|| "drop threshold reached".to_string());
    }

    fn recovery_report(&self) -> ReliableDatagramRecoveryReport {
        let report = ReliableDatagramRecoveryReport::new(
            self.recovery_state,
            self.dropped_packets_since_recovery,
            self.outbound.len(),
        );
        match &self.recovery_diagnostic {
            Some(diagnostic) => report.with_diagnostic(diagnostic.clone()),
            None => report,
        }
    }

    fn trim_completed_inbound_sequences(&mut self) {
        let receive_window = self.config.receive_window as usize;
        if receive_window == 0 {
            self.completed_inbound_sequences.clear();
            return;
        }
        while self.completed_inbound_sequences.len() > receive_window {
            self.completed_inbound_sequences.pop_front();
        }
    }

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

fn reorder_delivered_packets(packets: &mut Vec<ReliableDatagramPacket>, reorder_window: usize) {
    if reorder_window <= 1 {
        return;
    }
    for chunk in packets.chunks_mut(reorder_window) {
        chunk.reverse();
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
