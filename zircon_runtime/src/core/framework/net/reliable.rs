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
