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
mod bounded_channel_drain_tests {
    use std::{collections::VecDeque, hint::black_box, time::Instant};

    use super::{RpcChannelMessage, drain_bounded_channel_messages};

    const BENCHMARK_MESSAGE_COUNT: usize = 8_192;
    const BENCHMARK_SAMPLE_COUNT: usize = 21;

    #[test]
    fn bounded_channel_drain_preserves_fifo_limit_and_tail() {
        let mut queue = benchmark_queue(5);

        assert!(drain_bounded_channel_messages(&mut queue, 0).is_empty());
        assert_eq!(message_sequences(&queue), vec![0, 1, 2, 3, 4]);

        let first = drain_bounded_channel_messages(&mut queue, 3);
        assert_eq!(message_sequences(&first), vec![0, 1, 2]);
        assert_eq!(message_sequences(&queue), vec![3, 4]);

        let tail = drain_bounded_channel_messages(&mut queue, usize::MAX);
        assert_eq!(message_sequences(&tail), vec![3, 4]);
        assert!(queue.is_empty());
    }

    #[test]
    #[ignore = "release-only performance evidence"]
    fn bounded_channel_drain_release_benchmark_evidence() {
        let seed = benchmark_queue(BENCHMARK_MESSAGE_COUNT);
        let mut legacy_equivalence = seed.clone();
        let mut optimized_equivalence = seed.clone();
        assert_eq!(
            legacy_drain(&mut legacy_equivalence, BENCHMARK_MESSAGE_COUNT),
            drain_bounded_channel_messages(&mut optimized_equivalence, BENCHMARK_MESSAGE_COUNT,)
        );
        assert_eq!(legacy_equivalence, optimized_equivalence);

        let legacy_capacity_growths = legacy_capacity_growths(seed.clone());
        let optimized_capacity_growths = optimized_capacity_growths(seed.clone());
        assert!(legacy_capacity_growths > 1);
        assert_eq!(optimized_capacity_growths, 0);

        let mut legacy_samples = Vec::with_capacity(BENCHMARK_SAMPLE_COUNT);
        let mut optimized_samples = Vec::with_capacity(BENCHMARK_SAMPLE_COUNT);
        for sample_index in 0..BENCHMARK_SAMPLE_COUNT {
            if sample_index % 2 == 0 {
                legacy_samples.push(measure_legacy(&seed));
                optimized_samples.push(measure_optimized(&seed));
            } else {
                optimized_samples.push(measure_optimized(&seed));
                legacy_samples.push(measure_legacy(&seed));
            }
        }

        let legacy_p50 = percentile(&legacy_samples, 50);
        let legacy_p95 = percentile(&legacy_samples, 95);
        let optimized_p50 = percentile(&optimized_samples, 50);
        let optimized_p95 = percentile(&optimized_samples, 95);
        println!(
            "PERF_RESULT task=plugins10_bounded_rpc_channel_drain messages={BENCHMARK_MESSAGE_COUNT} sample_pairs={BENCHMARK_SAMPLE_COUNT} order=alternating_legacy_first_even legacy_first_pairs=11 optimized_first_pairs=10 percentile_method=nearest_rank legacy_capacity_growths_per_sample={legacy_capacity_growths} optimized_capacity_growths_per_sample={optimized_capacity_growths} threshold_percent=15 legacy_p50_ns={legacy_p50} legacy_p95_ns={legacy_p95} optimized_p50_ns={optimized_p50} optimized_p95_ns={optimized_p95} legacy_raw_ns={} optimized_raw_ns={}",
            raw_samples(&legacy_samples),
            raw_samples(&optimized_samples),
        );
        assert!(
            optimized_p95.saturating_mul(100) <= legacy_p95.saturating_mul(85),
            "bounded drain P95 {optimized_p95}ns did not improve legacy {legacy_p95}ns by 15%"
        );
    }

    fn benchmark_queue(count: usize) -> VecDeque<RpcChannelMessage> {
        (0..count)
            .map(|sequence| RpcChannelMessage::new(7, 1, sequence as u64, vec![sequence as u8; 64]))
            .collect()
    }

    fn message_sequences<'a>(
        messages: impl IntoIterator<Item = &'a RpcChannelMessage>,
    ) -> Vec<u64> {
        messages
            .into_iter()
            .map(|message| message.sequence)
            .collect()
    }

    fn legacy_drain(
        queue: &mut VecDeque<RpcChannelMessage>,
        max_messages: usize,
    ) -> Vec<RpcChannelMessage> {
        let mut drained = Vec::new();
        while drained.len() < max_messages {
            let Some(message) = queue.pop_front() else {
                break;
            };
            drained.push(message);
        }
        drained
    }

    fn legacy_capacity_growths(mut queue: VecDeque<RpcChannelMessage>) -> usize {
        let mut drained = Vec::new();
        let mut growths = 0;
        while let Some(message) = queue.pop_front() {
            let previous_capacity = drained.capacity();
            drained.push(message);
            growths += usize::from(drained.capacity() != previous_capacity);
        }
        black_box(drained);
        growths
    }

    fn optimized_capacity_growths(mut queue: VecDeque<RpcChannelMessage>) -> usize {
        let drain_count = queue.len();
        let mut drained = Vec::with_capacity(drain_count);
        let initial_capacity = drained.capacity();
        drained.extend(queue.drain(..drain_count));
        let growths = usize::from(drained.capacity() != initial_capacity);
        black_box(drained);
        growths
    }

    fn measure_legacy(seed: &VecDeque<RpcChannelMessage>) -> u128 {
        let mut queue = seed.clone();
        let start = Instant::now();
        let drained = legacy_drain(black_box(&mut queue), BENCHMARK_MESSAGE_COUNT);
        let elapsed = start.elapsed().as_nanos();
        black_box(drained);
        elapsed
    }

    fn measure_optimized(seed: &VecDeque<RpcChannelMessage>) -> u128 {
        let mut queue = seed.clone();
        let start = Instant::now();
        let drained =
            drain_bounded_channel_messages(black_box(&mut queue), BENCHMARK_MESSAGE_COUNT);
        let elapsed = start.elapsed().as_nanos();
        black_box(drained);
        elapsed
    }

    fn percentile(samples: &[u128], percentile: usize) -> u128 {
        let mut sorted = samples.to_vec();
        sorted.sort_unstable();
        let rank = sorted.len().saturating_mul(percentile).div_ceil(100);
        sorted[rank.saturating_sub(1)]
    }

    fn raw_samples(samples: &[u128]) -> String {
        format!(
            "[{}]",
            samples
                .iter()
                .map(u128::to_string)
                .collect::<Vec<_>>()
                .join(",")
        )
    }
}
