use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, Weak};
use std::thread::{self, JoinHandle};
use std::time::Duration;

use zircon_runtime::core::framework::sound::{SoundError, SoundOutputDeviceDescriptor};

use crate::engine::SoundEngineState;
use crate::SoundConfig;

use super::error::backend_unavailable;
use super::shared_state::CpalOutputSharedState;

pub(in crate::output::cpal) fn spawn_producer_thread(
    descriptor: SoundOutputDeviceDescriptor,
    engine_state: Weak<Mutex<SoundEngineState>>,
    config: Arc<Mutex<SoundConfig>>,
    shared: Arc<CpalOutputSharedState>,
    stop_flag: Arc<AtomicBool>,
) -> Result<JoinHandle<()>, SoundError> {
    thread::Builder::new()
        .name("zircon-sound-cpal-producer".to_string())
        .spawn(move || {
            let samples_per_block = descriptor
                .block_size_frames
                .saturating_mul(descriptor.channel_count as usize);
            let block_backoff = block_backoff_duration(&descriptor);
            while !stop_flag.load(Ordering::Acquire) {
                if ring_buffer_near_capacity(&shared, samples_per_block) {
                    thread::sleep(block_backoff);
                    continue;
                }
                let Ok(config) = config.try_lock().map(|config| config.clone()) else {
                    thread::sleep(Duration::from_millis(1));
                    continue;
                };
                let Some(engine_state) = engine_state.upgrade() else {
                    break;
                };
                let Ok(mut engine_state) = engine_state.try_lock() else {
                    thread::sleep(Duration::from_millis(1));
                    continue;
                };
                match engine_state.render_mix(&config, descriptor.block_size_frames) {
                    Ok(block) => {
                        shared.clear_error();
                        shared
                            .producer_rendered_blocks
                            .fetch_add(1, Ordering::Relaxed);
                        let channels = block.channel_count.max(1) as usize;
                        let frames = block.samples.len() / channels;
                        shared
                            .producer_rendered_frames
                            .fetch_add(frames as u64, Ordering::Relaxed);
                        push_producer_samples(&shared, &block.samples);
                    }
                    Err(error) => {
                        shared.record_error(error.to_string());
                        push_producer_samples(&shared, &vec![0.0; samples_per_block]);
                        thread::sleep(block_backoff);
                    }
                }
            }
        })
        .map_err(|error| backend_unavailable(format!("cpal producer thread failed: {error}")))
}

fn ring_buffer_near_capacity(shared: &CpalOutputSharedState, incoming_samples: usize) -> bool {
    shared
        .ring_buffer
        .lock()
        .map(|buffer| {
            buffer.available_samples().saturating_add(incoming_samples) > buffer.capacity_samples()
        })
        .unwrap_or(true)
}

fn push_producer_samples(shared: &CpalOutputSharedState, samples: &[f32]) {
    if let Ok(mut buffer) = shared.ring_buffer.lock() {
        buffer.push_samples(samples);
    }
}

fn block_backoff_duration(descriptor: &SoundOutputDeviceDescriptor) -> Duration {
    let sample_rate = descriptor.sample_rate_hz.max(1) as u64;
    let block_millis =
        ((descriptor.block_size_frames as u64).saturating_mul(1_000) / sample_rate).clamp(1, 10);
    Duration::from_millis(block_millis)
}
