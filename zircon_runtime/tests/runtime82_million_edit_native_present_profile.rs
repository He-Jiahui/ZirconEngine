#![cfg(all(windows, feature = "ui", feature = "platform-winit"))]

use std::{
    ffi::c_void,
    mem::{size_of, MaybeUninit},
    panic,
    sync::{mpsc, Arc},
    thread,
    time::{Duration, Instant},
};

use winit::{
    application::ApplicationHandler,
    dpi::{PhysicalSize, Size},
    event::WindowEvent,
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    platform::windows::EventLoopBuilderExtWindows,
    raw_window_handle::{HasWindowHandle, RawWindowHandle},
    window::{Window, WindowAttributes, WindowId},
};
use zircon_runtime::{
    asset::pipeline::manager::ProjectAssetManager,
    core::{
        framework::render::{
            CapturedFrame, RenderFrameExtract, RenderFramework, RenderPipelineHandle,
            RenderQualityProfile, RenderViewportDescriptor, RenderViewportHandle,
            RenderViewportSurfaceDescriptor, RenderWorldSnapshotHandle, UiRenderSubmission,
        },
        math::UVec2,
    },
    graphics::WgpuRenderFramework,
    rhi::RenderNativeSurfaceTarget,
    scene::world::World,
    ui::{dispatch::UiInputManager, surface::UiSurface},
};
use zircon_runtime_interface::ui::{
    dispatch::{
        UiInputEvent, UiInputEventMetadata, UiInputSequence, UiInputTimestamp,
        UiKeyboardInputEvent, UiKeyboardInputState,
    },
    event_ui::{UiNodeId, UiNodePath, UiStateFlags, UiTreeId},
    layout::UiFrame,
    surface::UiRenderCommandKind,
    tree::{UiInputPolicy, UiTemplateNodeMetadata, UiTreeNode},
    widget::{UiWidgetBehavior, UiWidgetContract, UiWidgetEvent},
};

mod support;

const DOCUMENT_CHARACTERS: usize = 1_000_000;
const WARMUP_SAMPLES: usize = 5;
const MEASURED_SAMPLES: usize = 31;
const TEXT_NODE: UiNodeId = UiNodeId::new(2);
const TEXT_FRAME: UiFrame = UiFrame::new(8.0, 8.0, 600.0, 44.0);
const WINDOW_EVENT_TIMEOUT: Duration = Duration::from_secs(10);
const PROFILE_THREAD_TIMEOUT: Duration = Duration::from_secs(900);
const MARKER: &str = "RUNTIME82_NATIVE_PRESENT_MILLION_EDIT_SCALE_V1";

#[test]
#[ignore = "managed Windows Release WGPU diagnostic with a live Win32 surface"]
fn million_character_edit_to_present_and_readback_scale_profile() {
    assert!(!cfg!(debug_assertions), "run this profile with --release");
    let (sender, receiver) = mpsc::sync_channel(1);
    thread::Builder::new()
        .name("runtime82-native-scale".to_string())
        .spawn(move || {
            let result = panic::catch_unwind(run_native_scale_profile);
            let _ = sender.send(result);
        })
        .expect("spawn bounded native scale profile thread");
    match receiver.recv_timeout(PROFILE_THREAD_TIMEOUT) {
        Ok(Ok(())) => {}
        Ok(Err(payload)) => panic::resume_unwind(payload),
        Err(mpsc::RecvTimeoutError::Timeout) => {
            panic!("native scale profile exceeded its 900-second test-thread deadline")
        }
        Err(mpsc::RecvTimeoutError::Disconnected) => {
            panic!("native scale profile thread exited without reporting a result")
        }
    }
}

fn run_native_scale_profile() {
    let mut builder = EventLoop::builder();
    builder.with_any_thread(true);
    let event_loop = builder.build().expect("create Windows test event loop");
    let (result_sender, result_receiver) = mpsc::sync_channel(1);
    let app = NativeScaleProfile::new(result_sender);
    event_loop.run_app(app).expect("run native scale profile");
    let (samples, adapter_name, adapter_device_type) = result_receiver
        .try_recv()
        .expect("native surface or redraw was unavailable before the event-loop deadline");
    assert_eq!(samples.len(), MEASURED_SAMPLES);
    report_samples(&samples, &adapter_name, &adapter_device_type);
}

struct NativeScaleProfile {
    window: Option<Arc<dyn Window>>,
    surfaces_available: bool,
    event_deadline: Instant,
    sampled: bool,
    result_sender: mpsc::SyncSender<(Vec<Sample>, String, String)>,
}

impl NativeScaleProfile {
    fn new(result_sender: mpsc::SyncSender<(Vec<Sample>, String, String)>) -> Self {
        Self {
            window: None,
            surfaces_available: false,
            event_deadline: Instant::now() + WINDOW_EVENT_TIMEOUT,
            sampled: false,
            result_sender,
        }
    }

    fn ensure_window(&mut self, event_loop: &dyn ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }
        let attributes = WindowAttributes::default()
            .with_title("Runtime82 million text edit native scale profile")
            .with_surface_size(Size::Physical(PhysicalSize::new(640, 360)))
            .with_resizable(false);
        self.window = Some(Arc::from(
            event_loop
                .create_window(attributes)
                .expect("create live profile window"),
        ));
    }
}

impl ApplicationHandler for NativeScaleProfile {
    fn resumed(&mut self, event_loop: &dyn ActiveEventLoop) {
        self.ensure_window(event_loop);
    }

    fn can_create_surfaces(&mut self, event_loop: &dyn ActiveEventLoop) {
        self.ensure_window(event_loop);
        self.surfaces_available = true;
        self.window
            .as_ref()
            .expect("profile window exists")
            .request_redraw();
    }

    fn about_to_wait(&mut self, event_loop: &dyn ActiveEventLoop) {
        if Instant::now() >= self.event_deadline {
            event_loop.exit();
        } else {
            event_loop.set_control_flow(ControlFlow::WaitUntil(self.event_deadline));
        }
    }

    fn window_event(
        &mut self,
        event_loop: &dyn ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::RedrawRequested if self.surfaces_available && !self.sampled => {
                let window = self.window.as_ref().expect("profile window exists");
                self.result_sender
                    .send(run_samples(window.as_ref()))
                    .expect("publish native scale profile samples");
                self.sampled = true;
                event_loop.exit();
            }
            WindowEvent::CloseRequested | WindowEvent::Destroyed => event_loop.exit(),
            _ => {}
        }
    }
}

#[derive(Clone, Copy)]
struct PresentedFrame {
    generation: u64,
    present_enqueue_ns: u128,
    finish_and_readback_ns: u128,
}

#[derive(Clone, Copy)]
struct Sample {
    generation_before: u64,
    generation_after: u64,
    edit_ns: u128,
    rebuild_to_extract_ns: u128,
    present_enqueue_ns: u128,
    finish_and_readback_ns: u128,
    edit_to_finished_present_and_readback_ns: u128,
    rss_before_bytes: usize,
    rss_after_bytes: usize,
    changed_pixels: usize,
}

fn run_samples(window: &dyn Window) -> (Vec<Sample>, String, String) {
    let physical = window.surface_size();
    let size = UVec2::new(physical.width, physical.height);
    assert!(
        size.x >= (TEXT_FRAME.x + TEXT_FRAME.width) as u32
            && size.y >= (TEXT_FRAME.y + TEXT_FRAME.height) as u32,
        "profile window must contain the visible text field"
    );
    let RawWindowHandle::Win32(handle) = window
        .window_handle()
        .expect("live Win32 window handle")
        .as_raw()
    else {
        panic!("native scale profile requires a Win32 window");
    };
    let target = RenderNativeSurfaceTarget::Win32 {
        hwnd: handle.hwnd.get() as usize as u64,
        hinstance: handle.hinstance.map(|value| value.get() as usize as u64),
    };

    let asset_runtime =
        support::ProjectAssetTestRuntime::new(Arc::new(ProjectAssetManager::default()));
    let framework = WgpuRenderFramework::new(asset_runtime.access(), asset_runtime.worker_pool())
        .expect("create WGPU render framework");
    let framework = support::TestWgpuRenderFramework::new(asset_runtime, framework);
    let viewport = framework
        .create_viewport(RenderViewportDescriptor::new(size))
        .expect("create native profile viewport");
    framework
        .set_quality_profile(
            viewport,
            RenderQualityProfile::new("runtime82-native-scale")
                .with_pipeline_asset(RenderPipelineHandle::new(1))
                .with_clustered_lighting(false)
                .with_screen_space_ambient_occlusion(false)
                .with_temporal_history(false)
                .with_bloom(false)
                .with_color_grading(false),
        )
        .expect("configure stable native UI rendering");
    framework
        .bind_viewport_surface(viewport, RenderViewportSurfaceDescriptor::new(size, target))
        .expect("bind the live Win32 surface");
    let device = framework
        .query_stats()
        .expect("query native renderer device identity")
        .device_diagnostics
        .expect("native renderer must report its adapter identity");

    let mut samples = Vec::with_capacity(MEASURED_SAMPLES);
    for sample_index in 0..(WARMUP_SAMPLES + MEASURED_SAMPLES) {
        let mut surface = million_character_surface();
        let (before, before_capture) = present_and_capture(&framework, viewport, size, &surface);
        surface.focus_node(TEXT_NODE).expect("focus text input");
        surface.rebuild();
        let mut input = UiInputManager::default();
        let rss_before_bytes = current_rss_bytes();
        let sample_start = Instant::now();
        let edit_start = Instant::now();
        let edit = surface
            .dispatch_input_event_with_manager(
                &mut input,
                UiInputEvent::Keyboard(UiKeyboardInputEvent {
                    metadata: UiInputEventMetadata::new(
                        UiInputTimestamp::from_micros(1),
                        UiInputSequence::new(1),
                    ),
                    state: UiKeyboardInputState::Pressed,
                    key_code: 8,
                    scan_code: None,
                    physical_key: "Backspace".to_string(),
                    logical_key: "Backspace".to_string(),
                    text: None,
                }),
            )
            .expect("dispatch real keyboard edit");
        let edit_ns = edit_start.elapsed().as_nanos();
        assert!(edit
            .widget_events
            .iter()
            .any(|event| matches!(event, UiWidgetEvent::TextEditChange { .. })));
        surface.clear_focus();
        let rebuild_start = Instant::now();
        surface.rebuild();
        let rebuild_to_extract_ns = rebuild_start.elapsed().as_nanos();
        let changed_text = rendered_text(&surface);
        assert_eq!(changed_text.len(), DOCUMENT_CHARACTERS);
        assert!(changed_text.starts_with('i'));
        let (after, after_capture) = present_and_capture(&framework, viewport, size, &surface);
        let edit_to_finished_present_and_readback_ns = sample_start.elapsed().as_nanos();
        let rss_after_bytes = current_rss_bytes();
        assert!(after.generation > before.generation);
        let changed_pixels = changed_pixels_in_text_frame(&before_capture, &after_capture);
        assert!(
            changed_pixels > 4,
            "sample {sample_index} rendered no visible edit"
        );
        if sample_index >= WARMUP_SAMPLES {
            samples.push(Sample {
                generation_before: before.generation,
                generation_after: after.generation,
                edit_ns,
                rebuild_to_extract_ns,
                present_enqueue_ns: after.present_enqueue_ns,
                finish_and_readback_ns: after.finish_and_readback_ns,
                edit_to_finished_present_and_readback_ns,
                rss_before_bytes,
                rss_after_bytes,
                changed_pixels,
            });
        }
    }
    framework
        .unbind_viewport_surface(viewport)
        .expect("release native surface before window closes");
    (samples, device.adapter_name, device.adapter_device_type)
}

fn million_character_surface() -> UiSurface {
    let content = format!("W{}", "i".repeat(DOCUMENT_CHARACTERS));
    let mut surface = UiSurface::new(UiTreeId::new("runtime82.native_scale.million_edit"));
    surface.tree.insert_root(
        UiTreeNode::new(UiNodeId::new(1), UiNodePath::new("root"))
            .with_frame(UiFrame::new(0.0, 0.0, 640.0, 360.0))
            .with_state_flags(UiStateFlags {
                visible: true,
                ..UiStateFlags::default()
            }),
    );
    surface
        .tree
        .insert_child(
            UiNodeId::new(1),
            UiTreeNode::new(TEXT_NODE, UiNodePath::new("root/input"))
                .with_frame(TEXT_FRAME)
                .with_input_policy(UiInputPolicy::Receive)
                .with_state_flags(UiStateFlags {
                    visible: true,
                    enabled: true,
                    clickable: true,
                    hoverable: true,
                    focusable: true,
                    ..UiStateFlags::default()
                })
                .with_template_metadata(UiTemplateNodeMetadata {
                    component: "InputField".to_string(),
                    attributes: [
                        ("content".to_string(), toml::Value::String(content)),
                        ("caret_offset".to_string(), toml::Value::Integer(1)),
                    ]
                    .into_iter()
                    .collect(),
                    widget: UiWidgetContract {
                        behavior: UiWidgetBehavior::TextInput,
                        value_property: Some("content".to_string()),
                        ..UiWidgetContract::default()
                    },
                    ..UiTemplateNodeMetadata::default()
                }),
        )
        .expect("attach native scale text field");
    surface.rebuild();
    assert_eq!(rendered_text(&surface).len(), DOCUMENT_CHARACTERS + 1);
    assert!(rendered_text(&surface).starts_with('W'));
    surface
}

fn rendered_text(surface: &UiSurface) -> &str {
    surface
        .render_extract
        .list
        .commands
        .iter()
        .find(|command| command.node_id == TEXT_NODE && command.kind == UiRenderCommandKind::Text)
        .and_then(|command| command.text.as_deref())
        .expect("visible text field render command")
}

fn present_and_capture(
    framework: &WgpuRenderFramework,
    viewport: RenderViewportHandle,
    size: UVec2,
    surface: &UiSurface,
) -> (PresentedFrame, CapturedFrame) {
    let mut extract = RenderFrameExtract::from_snapshot(
        RenderWorldSnapshotHandle::new(1),
        World::new().to_render_snapshot(),
    );
    extract.apply_viewport_size(size);
    let ui = UiRenderSubmission::single_frame(Arc::clone(&surface.surface_frame().render_extract));
    let present_start = Instant::now();
    framework
        .present_frame_extract_with_ui(viewport, extract, Some(ui))
        .expect("present UI to the bound native window");
    let present_enqueue_ns = present_start.elapsed().as_nanos();
    let readback_start = Instant::now();
    let captured = framework
        .capture_frame(viewport)
        .expect("finish submission and read back the presented generation")
        .expect("presented frame remains available for diagnostic readback");
    let finish_and_readback_ns = readback_start.elapsed().as_nanos();
    let stats = framework
        .query_stats()
        .expect("query presented frame stats");
    let receipt = stats
        .last_frame_submission_receipt
        .expect("renderer must publish a frame submission receipt");
    assert!(
        receipt.present_submission().is_some(),
        "scene submission or deferred surface acquire is not native present"
    );
    assert!(stats.last_ui_command_count > 0);
    assert!(stats.last_ui_text_payload_count > 0);
    assert!(stats.last_ui_graph_executed_pass_count > 0);
    assert_eq!(captured.generation, receipt.frame_generation());
    assert_eq!((captured.width, captured.height), (size.x, size.y));
    assert_eq!(
        captured.rgba.len(),
        (captured.width * captured.height * 4) as usize
    );
    assert!(captured
        .rgba
        .chunks_exact(4)
        .any(|pixel| pixel != [0, 0, 0, 0]));
    (
        PresentedFrame {
            generation: receipt.frame_generation(),
            present_enqueue_ns,
            finish_and_readback_ns,
        },
        captured,
    )
}

fn changed_pixels_in_text_frame(before: &CapturedFrame, after: &CapturedFrame) -> usize {
    before
        .rgba
        .chunks_exact(4)
        .zip(after.rgba.chunks_exact(4))
        .enumerate()
        .filter(|(pixel_index, (old, new))| {
            let x = (*pixel_index as u32) % before.width;
            let y = (*pixel_index as u32) / before.width;
            x >= TEXT_FRAME.x as u32
                && x < (TEXT_FRAME.x + TEXT_FRAME.width) as u32
                && y >= TEXT_FRAME.y as u32
                && y < (TEXT_FRAME.y + TEXT_FRAME.height) as u32
                && old != new
        })
        .count()
}

fn report_samples(samples: &[Sample], adapter_name: &str, adapter_device_type: &str) {
    let generation_before: Vec<u64> = samples
        .iter()
        .map(|sample| sample.generation_before)
        .collect();
    let generation_after: Vec<u64> = samples
        .iter()
        .map(|sample| sample.generation_after)
        .collect();
    let total_ns: Vec<u128> = samples
        .iter()
        .map(|sample| sample.edit_to_finished_present_and_readback_ns)
        .collect();
    let edit_ns: Vec<u128> = samples.iter().map(|sample| sample.edit_ns).collect();
    let rebuild_ns: Vec<u128> = samples
        .iter()
        .map(|sample| sample.rebuild_to_extract_ns)
        .collect();
    let enqueue_ns: Vec<u128> = samples
        .iter()
        .map(|sample| sample.present_enqueue_ns)
        .collect();
    let finish_ns: Vec<u128> = samples
        .iter()
        .map(|sample| sample.finish_and_readback_ns)
        .collect();
    let rss_before: Vec<usize> = samples
        .iter()
        .map(|sample| sample.rss_before_bytes)
        .collect();
    let rss_after: Vec<usize> = samples
        .iter()
        .map(|sample| sample.rss_after_bytes)
        .collect();
    let rss_delta: Vec<i128> = samples
        .iter()
        .map(|sample| sample.rss_after_bytes as i128 - sample.rss_before_bytes as i128)
        .collect();
    let changed_pixels: Vec<usize> = samples.iter().map(|sample| sample.changed_pixels).collect();
    println!(
        "{MARKER} document_characters={DOCUMENT_CHARACTERS} warmups={WARMUP_SAMPLES} samples={MEASURED_SAMPLES} generation_before={generation_before:?} generation_after={generation_after:?} edit_to_finished_present_and_readback_ns={total_ns:?} total_p50_ns={} total_p95_ns={} total_p99_ns={} edit_ns={edit_ns:?} rebuild_to_extract_ns={rebuild_ns:?} present_enqueue_ns={enqueue_ns:?} finish_and_readback_ns={finish_ns:?} rss_before_bytes={rss_before:?} rss_after_bytes={rss_after:?} rss_delta_bytes={rss_delta:?} rss_delta_p50_bytes={} rss_delta_p95_bytes={} rss_delta_p99_bytes={} changed_pixels={changed_pixels:?} os={} arch={} package_version={} cpu_identifier={:?} adapter_name={adapter_name:?} adapter_device_type={adapter_device_type:?}",
        percentile(&total_ns, 50),
        percentile(&total_ns, 95),
        percentile(&total_ns, 99),
        percentile(&rss_delta, 50),
        percentile(&rss_delta, 95),
        percentile(&rss_delta, 99),
        std::env::consts::OS,
        std::env::consts::ARCH,
        env!("CARGO_PKG_VERSION"),
        std::env::var("PROCESSOR_IDENTIFIER").ok(),
    );
}

fn percentile<T: Copy + Ord>(samples: &[T], percentile: usize) -> T {
    assert!(!samples.is_empty());
    let mut ordered = samples.to_vec();
    ordered.sort_unstable();
    let rank = percentile.saturating_mul(ordered.len()).div_ceil(100);
    ordered[rank.saturating_sub(1).min(ordered.len() - 1)]
}

#[repr(C)]
struct ProcessMemoryCounters {
    cb: u32,
    page_fault_count: u32,
    peak_working_set_size: usize,
    working_set_size: usize,
    quota_peak_paged_pool_usage: usize,
    quota_paged_pool_usage: usize,
    quota_peak_non_paged_pool_usage: usize,
    quota_non_paged_pool_usage: usize,
    pagefile_usage: usize,
    peak_pagefile_usage: usize,
}

#[link(name = "kernel32")]
unsafe extern "system" {
    fn GetCurrentProcess() -> *mut c_void;
}

#[link(name = "psapi")]
unsafe extern "system" {
    fn GetProcessMemoryInfo(
        process: *mut c_void,
        counters: *mut ProcessMemoryCounters,
        size: u32,
    ) -> i32;
}

fn current_rss_bytes() -> usize {
    let mut counters = MaybeUninit::<ProcessMemoryCounters>::zeroed();
    let counters_ptr = counters.as_mut_ptr();
    // SAFETY: the ABI-sized struct and process handle remain valid throughout the OS call.
    unsafe {
        (*counters_ptr).cb = size_of::<ProcessMemoryCounters>() as u32;
        assert_ne!(
            GetProcessMemoryInfo(
                GetCurrentProcess(),
                counters_ptr,
                size_of::<ProcessMemoryCounters>() as u32,
            ),
            0,
            "GetProcessMemoryInfo failed"
        );
        counters.assume_init().working_set_size
    }
}
