use std::collections::{HashMap, VecDeque};

use zircon_runtime::core::framework::net::{
    ReliableDatagramConfig, ReliableDatagramPacket, ReliableDatagramRecoveryReport,
    ReliableDatagramRecoveryState, ReliableDatagramSimulationProfile, ReliableDatagramStats,
};

use super::assembly::InboundFragmentAssembly;

#[derive(Debug)]
pub(in crate::manager) struct NetReliableUdpRuntimeState {
    pub(in crate::manager) config: ReliableDatagramConfig,
    pub(in crate::manager) next_sequence: u64,
    pub(in crate::manager) outbound: VecDeque<ReliableDatagramPacket>,
    pub(in crate::manager) resend_state: HashMap<u64, PendingResendState>,
    pub(in crate::manager) inbound_fragments: HashMap<u64, InboundFragmentAssembly>,
    pub(in crate::manager) completed_inbound_sequences: VecDeque<u64>,
    pub(in crate::manager) simulation_profile: ReliableDatagramSimulationProfile,
    pub(in crate::manager) simulated_packet_counter: u64,
    pub(in crate::manager) recovery_state: ReliableDatagramRecoveryState,
    pub(in crate::manager) dropped_packets_since_recovery: u64,
    pub(in crate::manager) recovery_diagnostic: Option<String>,
    pub(in crate::manager) stats: ReliableDatagramStats,
}

#[derive(Clone, Debug, Default)]
pub(in crate::manager) struct PendingResendState {
    pub(in crate::manager) last_sent_at_ms: u64,
    pub(in crate::manager) attempts: u8,
}

impl NetReliableUdpRuntimeState {
    pub(in crate::manager) fn new(config: ReliableDatagramConfig) -> Self {
        Self {
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
        }
    }

    pub(in crate::manager) fn recovery_report(&self) -> ReliableDatagramRecoveryReport {
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

    pub(in crate::manager) fn trim_completed_inbound_sequences(&mut self) {
        let receive_window = self.config.receive_window as usize;
        if receive_window == 0 {
            self.completed_inbound_sequences.clear();
            return;
        }
        while self.completed_inbound_sequences.len() > receive_window {
            self.completed_inbound_sequences.pop_front();
        }
    }
}
