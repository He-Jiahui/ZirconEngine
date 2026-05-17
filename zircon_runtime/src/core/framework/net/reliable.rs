use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ReliableDatagramConfig {
    pub mtu_bytes: usize,
    pub resend_timeout_ms: u64,
    pub max_resend_attempts: u8,
    pub receive_window: u16,
}

impl Default for ReliableDatagramConfig {
    fn default() -> Self {
        Self {
            mtu_bytes: 1_200,
            resend_timeout_ms: 100,
            max_resend_attempts: 8,
            receive_window: 256,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ReliableDatagramStats {
    pub sent_packets: u64,
    pub received_packets: u64,
    pub resent_packets: u64,
    pub dropped_packets: u64,
    pub rtt_ms: f32,
}

impl Default for ReliableDatagramStats {
    fn default() -> Self {
        Self {
            sent_packets: 0,
            received_packets: 0,
            resent_packets: 0,
            dropped_packets: 0,
            rtt_ms: 0.0,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReliableDatagramSimulationProfile {
    pub drop_every_nth_packet: Option<u64>,
    pub reorder_window: usize,
    pub recovery_drop_threshold: Option<u64>,
}

impl ReliableDatagramSimulationProfile {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_drop_every_nth_packet(mut self, packet_interval: u64) -> Self {
        self.drop_every_nth_packet = (packet_interval > 0).then_some(packet_interval);
        self
    }

    pub fn with_reorder_window(mut self, packet_count: usize) -> Self {
        self.reorder_window = packet_count.max(1);
        self
    }

    pub fn with_recovery_drop_threshold(mut self, dropped_packets: u64) -> Self {
        self.recovery_drop_threshold = (dropped_packets > 0).then_some(dropped_packets);
        self
    }
}

impl Default for ReliableDatagramSimulationProfile {
    fn default() -> Self {
        Self {
            drop_every_nth_packet: None,
            reorder_window: 1,
            recovery_drop_threshold: None,
        }
    }
}

/// Copied state used by diagnostics to distinguish active recovery from a hard disconnect.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReliableDatagramRecoveryState {
    Connected,
    Recovering,
    Disconnected,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReliableDatagramRecoveryReport {
    pub state: ReliableDatagramRecoveryState,
    pub dropped_packets_since_recovery: u64,
    pub pending_packets: usize,
    pub diagnostic: Option<String>,
}

impl ReliableDatagramRecoveryReport {
    pub fn new(
        state: ReliableDatagramRecoveryState,
        dropped_packets_since_recovery: u64,
        pending_packets: usize,
    ) -> Self {
        Self {
            state,
            dropped_packets_since_recovery,
            pending_packets,
            diagnostic: None,
        }
    }

    pub fn with_diagnostic(mut self, diagnostic: impl Into<String>) -> Self {
        self.diagnostic = Some(diagnostic.into());
        self
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReliableDatagramPacket {
    pub sequence: u64,
    pub channel: String,
    pub fragment_index: u16,
    pub fragment_count: u16,
    pub payload: Vec<u8>,
}

impl ReliableDatagramPacket {
    pub fn new(sequence: u64, channel: impl Into<String>, payload: impl Into<Vec<u8>>) -> Self {
        Self {
            sequence,
            channel: channel.into(),
            fragment_index: 0,
            fragment_count: 1,
            payload: payload.into(),
        }
    }

    pub fn with_fragment(mut self, fragment_index: u16, fragment_count: u16) -> Self {
        self.fragment_index = fragment_index;
        self.fragment_count = fragment_count;
        self
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReliableDatagramDeliveryReport {
    pub delivered_packets: Vec<ReliableDatagramPacket>,
    pub dropped_packets: Vec<ReliableDatagramPacket>,
    pub recovery: ReliableDatagramRecoveryReport,
}

impl ReliableDatagramDeliveryReport {
    pub fn new(
        delivered_packets: impl IntoIterator<Item = ReliableDatagramPacket>,
        dropped_packets: impl IntoIterator<Item = ReliableDatagramPacket>,
        recovery: ReliableDatagramRecoveryReport,
    ) -> Self {
        Self {
            delivered_packets: delivered_packets.into_iter().collect(),
            dropped_packets: dropped_packets.into_iter().collect(),
            recovery,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReliableDatagramReceiveStatus {
    AcceptedFragment,
    DuplicateFragment,
    Reassembled,
    InvalidFragment,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReliableDatagramReceiveReport {
    pub sequence: u64,
    pub channel: String,
    pub status: ReliableDatagramReceiveStatus,
    pub ack: Option<ReliableDatagramAck>,
    pub payload: Option<Vec<u8>>,
    pub diagnostic: Option<String>,
}

impl ReliableDatagramReceiveReport {
    pub fn new(
        sequence: u64,
        channel: impl Into<String>,
        status: ReliableDatagramReceiveStatus,
    ) -> Self {
        Self {
            sequence,
            channel: channel.into(),
            status,
            ack: None,
            payload: None,
            diagnostic: None,
        }
    }

    pub fn with_ack(mut self, ack: ReliableDatagramAck) -> Self {
        self.ack = Some(ack);
        self
    }

    pub fn with_payload(mut self, payload: impl Into<Vec<u8>>) -> Self {
        self.payload = Some(payload.into());
        self
    }

    pub fn with_diagnostic(mut self, diagnostic: impl Into<String>) -> Self {
        self.diagnostic = Some(diagnostic.into());
        self
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReliableDatagramAck {
    pub sequence: u64,
}

impl ReliableDatagramAck {
    pub fn new(sequence: u64) -> Self {
        Self { sequence }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReliableDatagramSendStatus {
    Queued,
    Fragmented,
    PayloadTooLarge,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReliableDatagramSendReport {
    pub status: ReliableDatagramSendStatus,
    pub packets: Vec<ReliableDatagramPacket>,
}

impl ReliableDatagramSendReport {
    pub fn new(
        status: ReliableDatagramSendStatus,
        packets: impl IntoIterator<Item = ReliableDatagramPacket>,
    ) -> Self {
        Self {
            status,
            packets: packets.into_iter().collect(),
        }
    }
}
