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
