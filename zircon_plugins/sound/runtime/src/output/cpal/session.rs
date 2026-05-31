use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;

use zircon_runtime::core::framework::sound::{SoundError, SoundOutputDeviceDescriptor};

use crate::engine::SoundEngineState;
use crate::SoundConfig;

use super::super::ring_buffer::SoundOutputRingBuffer;
use super::device_thread::spawn_device_thread;
use super::producer_thread::spawn_producer_thread;
use super::shared_state::CpalOutputSharedState;

pub(crate) struct CpalOutputSession {
    stop_flag: Arc<AtomicBool>,
    device_thread: Option<JoinHandle<()>>,
    producer_thread: Option<JoinHandle<()>>,
    shared: Arc<CpalOutputSharedState>,
}

impl std::fmt::Debug for CpalOutputSession {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("CpalOutputSession")
            .field("stop_flag", &self.stop_flag.load(Ordering::Relaxed))
            .field("device_thread_active", &self.device_thread.is_some())
            .field("producer_thread_active", &self.producer_thread.is_some())
            .field("shared", &self.shared)
            .finish_non_exhaustive()
    }
}

#[derive(Debug)]
pub(crate) struct CpalOutputSessionStatus {
    pub(crate) rendered_blocks: u64,
    pub(crate) rendered_frames: u64,
    pub(crate) callback_count: u64,
    pub(crate) last_callback_sequence: Option<u64>,
    pub(crate) underrun_count: u64,
    pub(crate) last_error: Option<String>,
    pub(crate) queued_samples: Option<usize>,
    pub(crate) capacity_samples: Option<usize>,
}

impl CpalOutputSession {
    fn new(
        stop_flag: Arc<AtomicBool>,
        device_thread: JoinHandle<()>,
        producer_thread: JoinHandle<()>,
        shared: Arc<CpalOutputSharedState>,
    ) -> Self {
        Self {
            stop_flag,
            device_thread: Some(device_thread),
            producer_thread: Some(producer_thread),
            shared,
        }
    }

    pub(crate) fn status(&self) -> CpalOutputSessionStatus {
        let callback_count = self.shared.callback_count.load(Ordering::Relaxed);
        let (queued_samples, capacity_samples) = self.shared.ring_buffer_status();
        CpalOutputSessionStatus {
            rendered_blocks: self.shared.producer_rendered_blocks.load(Ordering::Relaxed),
            rendered_frames: self.shared.producer_rendered_frames.load(Ordering::Relaxed),
            callback_count,
            last_callback_sequence: (callback_count > 0)
                .then(|| self.shared.last_callback_sequence.load(Ordering::Relaxed)),
            underrun_count: self.shared.underrun_count.load(Ordering::Relaxed),
            last_error: self.shared.last_error(),
            queued_samples,
            capacity_samples,
        }
    }

    pub(crate) fn stop(&mut self) {
        self.stop_flag.store(true, Ordering::Release);
        if let Some(thread) = self.producer_thread.take() {
            if thread.join().is_err() {
                self.shared
                    .record_error("cpal producer thread panicked during shutdown".to_string());
            }
        }
        if let Some(thread) = self.device_thread.take() {
            if thread.join().is_err() {
                self.shared
                    .record_error("cpal device thread panicked during shutdown".to_string());
            }
        }
    }
}

impl Drop for CpalOutputSession {
    fn drop(&mut self) {
        self.stop();
    }
}

pub(crate) fn start_cpal_session(
    descriptor: &SoundOutputDeviceDescriptor,
    engine_state: Arc<Mutex<SoundEngineState>>,
    config: Arc<Mutex<SoundConfig>>,
) -> Result<CpalOutputSession, SoundError> {
    let capacity_samples = descriptor
        .block_size_frames
        .saturating_mul(descriptor.channel_count as usize)
        .saturating_mul(descriptor.latency_blocks);
    let shared = Arc::new(CpalOutputSharedState::new(SoundOutputRingBuffer::new(
        capacity_samples,
    )));
    let stop_flag = Arc::new(AtomicBool::new(false));
    let device_thread = spawn_device_thread(
        descriptor.clone(),
        Arc::clone(&shared),
        Arc::clone(&stop_flag),
    )?;
    let producer_thread = spawn_producer_thread(
        descriptor.clone(),
        Arc::downgrade(&engine_state),
        Arc::clone(&config),
        Arc::clone(&shared),
        Arc::clone(&stop_flag),
    )?;
    Ok(CpalOutputSession::new(
        stop_flag,
        device_thread,
        producer_thread,
        shared,
    ))
}
