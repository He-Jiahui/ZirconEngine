use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{mpsc, Arc};
use std::thread::{self, JoinHandle};
use std::time::Duration;

use zircon_runtime::core::framework::sound::{SoundError, SoundOutputDeviceDescriptor};

use super::callback::drain_callback_output;
use super::error::backend_unavailable;
use super::selection::{select_output_device, select_stream_config};
use super::shared_state::CpalOutputSharedState;

pub(in crate::output::cpal) fn spawn_device_thread(
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
