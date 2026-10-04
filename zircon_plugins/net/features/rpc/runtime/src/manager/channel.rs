//! 按 channel ID 为内存消息分配独立序号并在队列中保序；flag 描述可靠/不可靠语义。
//! 此队列尚未绑定网络 transport，可靠性 flag 本身不会产生 ACK 或重发。

use std::collections::VecDeque;

use zircon_runtime::core::framework::net::NetError;

use super::NetRpcRuntimeManager;

pub const RPC_CHANNEL_RELIABLE_ORDERED: u8 = 0b0000_0001;
pub const RPC_CHANNEL_UNRELIABLE: u8 = 0b0000_0010;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RpcChannelMessage {
    pub channel_id: u8,
    pub flags: u8,
    pub sequence: u64,
    pub payload: Vec<u8>,
}

impl RpcChannelMessage {
    pub fn new(channel_id: u8, flags: u8, sequence: u64, payload: Vec<u8>) -> Self {
        Self {
            channel_id,
            flags,
            sequence,
            payload,
        }
    }

    pub fn is_reliable_ordered(&self) -> bool {
        self.flags & RPC_CHANNEL_RELIABLE_ORDERED != 0
    }
}

impl NetRpcRuntimeManager {
    pub fn enqueue_channel_message(
        &self,
        channel_id: u8,
        flags: u8,
        payload: Vec<u8>,
    ) -> Result<RpcChannelMessage, NetError> {
        if flags & !(RPC_CHANNEL_RELIABLE_ORDERED | RPC_CHANNEL_UNRELIABLE) != 0 {
            return Err(NetError::Io("unsupported RPC channel flags".to_string()));
        }

        let mut state = self.state.lock().expect("net RPC state mutex poisoned");
        let sequence = state.channel_sequences.entry(channel_id).or_insert(0);
        let message = RpcChannelMessage::new(channel_id, flags, *sequence, payload);
        *sequence += 1;
        state
            .channel_queues
            .entry(channel_id)
            .or_insert_with(VecDeque::new)
            .push_back(message.clone());
        Ok(message)
    }

    pub fn drain_channel_messages(
        &self,
        channel_id: u8,
        max_messages: usize,
    ) -> Vec<RpcChannelMessage> {
        let mut state = self.state.lock().expect("net RPC state mutex poisoned");
        let Some(queue) = state.channel_queues.get_mut(&channel_id) else {
            return Vec::new();
        };

        drain_bounded_channel_messages(queue, max_messages)
    }
}

fn drain_bounded_channel_messages(
    queue: &mut VecDeque<RpcChannelMessage>,
    max_messages: usize,
) -> Vec<RpcChannelMessage> {
    let drain_count = max_messages.min(queue.len());
    let mut drained = Vec::with_capacity(drain_count);
    drained.extend(queue.drain(..drain_count));
    drained
}

#[cfg(test)]
#[path = "tests/channel_bounded_channel_drain_tests.rs"]
mod bounded_channel_drain_tests;
