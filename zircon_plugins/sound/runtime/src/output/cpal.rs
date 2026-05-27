#[cfg(feature = "cpal-backend")]
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
#[cfg(feature = "cpal-backend")]
use std::sync::{mpsc, Arc, Mutex, Weak};
#[cfg(feature = "cpal-backend")]
use std::thread::{self, JoinHandle};
#[cfg(feature = "cpal-backend")]
use std::time::Duration;

use zircon_runtime::core::framework::sound::{
    SoundBackendCapability, SoundError, SoundOutputDeviceInfo,
};
#[cfg(feature = "cpal-backend")]
use zircon_runtime::core::framework::sound::{SoundOutputDeviceDescriptor, SoundOutputDeviceId};

#[cfg(feature = "cpal-backend")]
use crate::engine::SoundEngineState;
use crate::SoundConfig;

#[cfg(feature = "cpal-backend")]
use super::SoundOutputRingBuffer;

pub(crate) const CPAL_BACKEND: &str = "cpal";
#[cfg(feature = "cpal-backend")]
const CPAL_DEFAULT_OUTPUT_DEVICE_ID: &str = "sound.output.cpal.default";
#[cfg(feature = "cpal-backend")]
const CPAL_OUTPUT_DEVICE_ID_PREFIX: &str = "sound.output.cpal.device.";

#[cfg(feature = "cpal-backend")]
pub(crate) fn cpal_backend_capabilities() -> Vec<SoundBackendCapability> {
    vec![SoundBackendCapability {
        backend: CPAL_BACKEND.to_string(),
        display_name: "CPAL Default Output".to_string(),
        realtime_capable: true,
        deterministic: false,
        min_sample_rate_hz: 8_000,
        max_sample_rate_hz: 384_000,
        min_channel_count: 1,
        max_channel_count: 64,
        min_block_size_frames: 1,
        max_block_size_frames: 65_536,
        notes: vec![
            "uses the platform default output device through CPAL".to_string(),
            "availability depends on host audio devices and OS permissions".to_string(),
        ],
    }]
}

#[cfg(not(feature = "cpal-backend"))]
pub(crate) fn cpal_backend_capabilities() -> Vec<SoundBackendCapability> {
    Vec::new()
}

#[cfg(feature = "cpal-backend")]
pub(crate) fn cpal_output_devices(config: &SoundConfig) -> Vec<SoundOutputDeviceInfo> {
    use cpal::traits::HostTrait;

    let host = cpal::default_host();
    let mut devices = Vec::new();
    match host.default_output_device() {
        Some(device) => devices.push(output_device_info(
            SoundOutputDeviceId::new(CPAL_DEFAULT_OUTPUT_DEVICE_ID),
            &device,
            true,
            config,
        )),
        None => devices.push(unavailable_default_output_device(config)),
    }

    match host.output_devices() {
        Ok(output_devices) => {
            for (index, device) in output_devices.enumerate() {
                devices.push(output_device_info(
                    SoundOutputDeviceId::new(format!("{CPAL_OUTPUT_DEVICE_ID_PREFIX}{index}")),
                    &device,
                    false,
                    config,
                ));
            }
        }
        Err(error) => {
            if let Some(default) = devices.first_mut() {
                default.available = false;
                default.diagnostic =
                    Some(format!("cpal output device enumeration failed: {error}"));
            }
        }
    }
    devices
}

#[cfg(not(feature = "cpal-backend"))]
pub(crate) fn cpal_output_devices(_config: &SoundConfig) -> Vec<SoundOutputDeviceInfo> {
    Vec::new()
}

#[cfg(feature = "cpal-backend")]
pub(crate) fn validate_cpal_backend_supported() -> Result<(), SoundError> {
    Ok(())
}

#[cfg(not(feature = "cpal-backend"))]
pub(crate) fn validate_cpal_backend_supported() -> Result<(), SoundError> {
    Err(SoundError::BackendUnavailable {
        detail: cpal_backend_unavailable_detail(),
    })
}

#[cfg(not(feature = "cpal-backend"))]
pub(crate) fn cpal_backend_unavailable_detail() -> String {
    "sound output backend `cpal` requires the `cpal-backend` feature".to_string()
}

#[cfg(feature = "cpal-backend")]
pub(crate) struct CpalOutputSession {
    stop_flag: Arc<AtomicBool>,
    device_thread: Option<JoinHandle<()>>,
    producer_thread: Option<JoinHandle<()>>,
    shared: Arc<CpalOutputSharedState>,
}

#[cfg(feature = "cpal-backend")]
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

#[cfg(feature = "cpal-backend")]
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

#[cfg(feature = "cpal-backend")]
#[derive(Debug)]
struct CpalOutputSharedState {
    ring_buffer: Mutex<SoundOutputRingBuffer>,
    producer_rendered_blocks: AtomicU64,
    producer_rendered_frames: AtomicU64,
    callback_count: AtomicU64,
    last_callback_sequence: AtomicU64,
    underrun_count: AtomicU64,
    last_error: Mutex<Option<String>>,
}

#[cfg(feature = "cpal-backend")]
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

#[cfg(feature = "cpal-backend")]
impl Drop for CpalOutputSession {
    fn drop(&mut self) {
        self.stop();
    }
}

#[cfg(feature = "cpal-backend")]
impl CpalOutputSharedState {
    fn new(ring_buffer: SoundOutputRingBuffer) -> Self {
        Self {
            ring_buffer: Mutex::new(ring_buffer),
            producer_rendered_blocks: AtomicU64::new(0),
            producer_rendered_frames: AtomicU64::new(0),
            callback_count: AtomicU64::new(0),
            last_callback_sequence: AtomicU64::new(0),
            underrun_count: AtomicU64::new(0),
            last_error: Mutex::new(None),
        }
    }

    fn record_error(&self, detail: String) {
        if let Ok(mut last_error) = self.last_error.lock() {
            *last_error = Some(detail);
        }
    }

    fn clear_error(&self) {
        if let Ok(mut last_error) = self.last_error.lock() {
            *last_error = None;
        }
    }

    fn last_error(&self) -> Option<String> {
        self.last_error.lock().ok().and_then(|error| error.clone())
    }

    fn ring_buffer_status(&self) -> (Option<usize>, Option<usize>) {
        self.ring_buffer
            .lock()
            .map(|buffer| {
                (
                    Some(buffer.available_samples()),
                    Some(buffer.capacity_samples()),
                )
            })
            .unwrap_or((None, None))
    }
}

#[cfg(feature = "cpal-backend")]
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

#[cfg(feature = "cpal-backend")]
fn spawn_device_thread(
    descriptor: SoundOutputDeviceDescriptor,
    shared: Arc<CpalOutputSharedState>,
    stop_flag: Arc<AtomicBool>,
) -> Result<JoinHandle<()>, SoundError> {
    let (ready_sender, ready_receiver) = mpsc::sync_channel(1);
    let device_stop_flag = Arc::clone(&stop_flag);
    let thread = thread::Builder::new()
        .name("zircon-sound-cpal-device".to_string())
        .spawn(move || {
            run_device_thread(descriptor, shared, device_stop_flag, ready_sender);
        })
        .map_err(|error| backend_unavailable(format!("cpal device thread failed: {error}")))?;
    match ready_receiver.recv() {
        Ok(Ok(())) => Ok(thread),
        Ok(Err(detail)) => {
            stop_flag.store(true, Ordering::Release);
            let _ = thread.join();
            Err(backend_unavailable(detail))
        }
        Err(error) => {
            stop_flag.store(true, Ordering::Release);
            let _ = thread.join();
            Err(backend_unavailable(format!(
                "cpal device thread stopped before startup completed: {error}"
            )))
        }
    }
}

#[cfg(feature = "cpal-backend")]
fn run_device_thread(
    descriptor: SoundOutputDeviceDescriptor,
    shared: Arc<CpalOutputSharedState>,
    stop_flag: Arc<AtomicBool>,
    ready_sender: mpsc::SyncSender<Result<(), String>>,
) {
    use cpal::traits::{DeviceTrait, StreamTrait};

    let host = cpal::default_host();
    let device = match select_output_device(&host, &descriptor) {
        Ok(device) => device,
        Err(error) => {
            let _ = ready_sender.send(Err(error.to_string()));
            return;
        }
    };
    let stream_config = match select_stream_config(&device, &descriptor) {
        Ok(stream_config) => stream_config,
        Err(error) => {
            let _ = ready_sender.send(Err(error.to_string()));
            return;
        }
    };
    let callback_shared = Arc::clone(&shared);
    let error_shared = Arc::clone(&shared);
    let stream = match device.build_output_stream(
        &stream_config,
        move |output: &mut [f32], _| drain_callback_output(&callback_shared, output),
        move |error| {
            error_shared.record_error(format!("cpal stream error: {error}"));
        },
        None,
    ) {
        Ok(stream) => stream,
        Err(error) => {
            let _ = ready_sender.send(Err(format!("cpal output stream build failed: {error}")));
            return;
        }
    };
    if let Err(error) = stream.play() {
        let _ = ready_sender.send(Err(format!("cpal output stream play failed: {error}")));
        return;
    }
    let _ = ready_sender.send(Ok(()));
    while !stop_flag.load(Ordering::Acquire) {
        thread::sleep(Duration::from_millis(1));
    }
}

#[cfg(feature = "cpal-backend")]
fn select_output_device(
    host: &cpal::Host,
    descriptor: &SoundOutputDeviceDescriptor,
) -> Result<cpal::Device, SoundError> {
    use cpal::traits::HostTrait;

    match cpal_picker_device_index(descriptor.id.as_str())? {
        Some(index) => host
            .output_devices()
            .map_err(|error| {
                backend_unavailable(format!("cpal output device enumeration failed: {error}"))
            })?
            .nth(index)
            .ok_or_else(|| {
                backend_unavailable(format!(
                    "cpal output device `{}` is not available",
                    descriptor.id.as_str()
                ))
            }),
        None => host.default_output_device().ok_or_else(|| {
            backend_unavailable("cpal default output device is not available".to_string())
        }),
    }
}

#[cfg(feature = "cpal-backend")]
fn cpal_picker_device_index(device_id: &str) -> Result<Option<usize>, SoundError> {
    if device_id == CPAL_DEFAULT_OUTPUT_DEVICE_ID {
        return Ok(None);
    }
    let Some(raw_index) = device_id.strip_prefix(CPAL_OUTPUT_DEVICE_ID_PREFIX) else {
        return Ok(None);
    };
    raw_index.parse::<usize>().map(Some).map_err(|_| {
        backend_unavailable(format!(
            "cpal output device id `{device_id}` has an invalid picker index"
        ))
    })
}

#[cfg(feature = "cpal-backend")]
fn select_stream_config(
    device: &cpal::Device,
    descriptor: &SoundOutputDeviceDescriptor,
) -> Result<cpal::StreamConfig, SoundError> {
    use cpal::traits::DeviceTrait;

    let supported_configs = device.supported_output_configs().map_err(|error| {
        backend_unavailable(format!("cpal output config query failed: {error}"))
    })?;
    for config in supported_configs {
        if config.sample_format() != cpal::SampleFormat::F32 {
            continue;
        }
        if config.channels() != descriptor.channel_count {
            continue;
        }
        let sample_rate = descriptor.sample_rate_hz;
        if sample_rate < config.min_sample_rate().0 || sample_rate > config.max_sample_rate().0 {
            continue;
        }
        return Ok(config
            .with_sample_rate(cpal::SampleRate(sample_rate))
            .config());
    }
    Err(backend_unavailable(format!(
        "cpal selected output device does not support f32 {} Hz / {} channel output",
        descriptor.sample_rate_hz, descriptor.channel_count
    )))
}

#[cfg(feature = "cpal-backend")]
fn output_device_info(
    id: SoundOutputDeviceId,
    device: &cpal::Device,
    is_default: bool,
    config: &SoundConfig,
) -> SoundOutputDeviceInfo {
    use cpal::traits::DeviceTrait;

    let display_name = device
        .name()
        .unwrap_or_else(|_| "CPAL Output Device".to_string());
    let mut descriptor = SoundOutputDeviceDescriptor {
        id,
        backend: CPAL_BACKEND.to_string(),
        display_name,
        sample_rate_hz: config.sample_rate_hz,
        channel_count: config.channel_count,
        block_size_frames: config.block_size_frames,
        latency_blocks: zircon_runtime::core::framework::sound::DEFAULT_SOUND_OUTPUT_LATENCY_BLOCKS,
    };
    if let Ok(default_config) = device.default_output_config() {
        descriptor.sample_rate_hz = default_config.sample_rate().0;
        descriptor.channel_count = default_config.channels();
    }
    let diagnostic = select_stream_config(device, &descriptor)
        .err()
        .map(|error| error.to_string());
    SoundOutputDeviceInfo {
        descriptor,
        is_default,
        available: diagnostic.is_none(),
        diagnostic,
    }
}

#[cfg(feature = "cpal-backend")]
fn unavailable_default_output_device(config: &SoundConfig) -> SoundOutputDeviceInfo {
    SoundOutputDeviceInfo {
        descriptor: SoundOutputDeviceDescriptor {
            id: SoundOutputDeviceId::new(CPAL_DEFAULT_OUTPUT_DEVICE_ID),
            backend: CPAL_BACKEND.to_string(),
            display_name: "CPAL Default Output".to_string(),
            sample_rate_hz: config.sample_rate_hz,
            channel_count: config.channel_count,
            block_size_frames: config.block_size_frames,
            latency_blocks:
                zircon_runtime::core::framework::sound::DEFAULT_SOUND_OUTPUT_LATENCY_BLOCKS,
        },
        is_default: true,
        available: false,
        diagnostic: Some("cpal default output device is not available".to_string()),
    }
}

#[cfg(feature = "cpal-backend")]
fn spawn_producer_thread(
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

#[cfg(feature = "cpal-backend")]
fn ring_buffer_near_capacity(shared: &CpalOutputSharedState, incoming_samples: usize) -> bool {
    shared
        .ring_buffer
        .lock()
        .map(|buffer| {
            buffer.available_samples().saturating_add(incoming_samples) > buffer.capacity_samples()
        })
        .unwrap_or(true)
}

#[cfg(feature = "cpal-backend")]
fn push_producer_samples(shared: &CpalOutputSharedState, samples: &[f32]) {
    if let Ok(mut buffer) = shared.ring_buffer.lock() {
        buffer.push_samples(samples);
    }
}

#[cfg(feature = "cpal-backend")]
fn drain_callback_output(shared: &CpalOutputSharedState, output: &mut [f32]) {
    let sequence_index = shared.callback_count.fetch_add(1, Ordering::Relaxed);
    shared
        .last_callback_sequence
        .store(sequence_index, Ordering::Relaxed);
    let underrun = match shared.ring_buffer.lock() {
        Ok(mut buffer) => buffer.drain_into_with_silence(output),
        Err(_) => {
            output.fill(0.0);
            output.len()
        }
    };
    if underrun > 0 {
        shared
            .underrun_count
            .fetch_add(underrun as u64, Ordering::Relaxed);
    }
}

#[cfg(feature = "cpal-backend")]
fn block_backoff_duration(descriptor: &SoundOutputDeviceDescriptor) -> Duration {
    let sample_rate = descriptor.sample_rate_hz.max(1) as u64;
    let block_millis =
        ((descriptor.block_size_frames as u64).saturating_mul(1_000) / sample_rate).clamp(1, 10);
    Duration::from_millis(block_millis)
}

#[cfg(feature = "cpal-backend")]
fn backend_unavailable(detail: String) -> SoundError {
    SoundError::BackendUnavailable { detail }
}

#[cfg(all(test, feature = "cpal-backend"))]
mod tests {
    use super::super::SoundOutputRingBuffer;
    use super::{
        cpal_picker_device_index, drain_callback_output, CpalOutputSharedState,
        CPAL_DEFAULT_OUTPUT_DEVICE_ID, CPAL_OUTPUT_DEVICE_ID_PREFIX,
    };

    #[test]
    fn output_device_cpal_callback_drain_zero_fills_underrun_and_counts_callback() {
        let shared = CpalOutputSharedState::new(SoundOutputRingBuffer::new(2));
        shared.ring_buffer.lock().unwrap().push_samples(&[0.25]);

        let mut output = [9.0, 9.0, 9.0];
        drain_callback_output(&shared, &mut output);

        assert_eq!(output, [0.25, 0.0, 0.0]);
        assert_eq!(
            shared
                .callback_count
                .load(std::sync::atomic::Ordering::Relaxed),
            1
        );
        assert_eq!(
            shared
                .underrun_count
                .load(std::sync::atomic::Ordering::Relaxed),
            2
        );
    }

    #[test]
    fn output_device_cpal_picker_ids_parse_default_and_indexed_devices() {
        assert_eq!(
            cpal_picker_device_index(CPAL_DEFAULT_OUTPUT_DEVICE_ID).unwrap(),
            None
        );
        assert_eq!(
            cpal_picker_device_index(&format!("{CPAL_OUTPUT_DEVICE_ID_PREFIX}7")).unwrap(),
            Some(7)
        );
        assert_eq!(
            cpal_picker_device_index("sound.output.cpal.manual").unwrap(),
            None
        );
        assert!(cpal_picker_device_index(&format!("{CPAL_OUTPUT_DEVICE_ID_PREFIX}bad")).is_err());
    }
}
