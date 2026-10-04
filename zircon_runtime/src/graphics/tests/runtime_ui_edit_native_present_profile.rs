use std::{
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

use crate::{
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

const DOCUMENT_CHARACTERS: usize = 1_000_000;
const TEXT_NODE: UiNodeId = UiNodeId::new(2);
const TEXT_FRAME: UiFrame = UiFrame::new(8.0, 8.0, 600.0, 44.0);
const MARKER: &str = "RUNTIME82_NATIVE_PRESENT_MILLION_EDIT_DIAGNOSTIC_V1";
const WINDOW_EVENT_TIMEOUT: Duration = Duration::from_secs(10);
const DIAGNOSTIC_THREAD_TIMEOUT: Duration = Duration::from_secs(60);

#[test]
#[ignore = "managed Windows Release diagnostic requiring a live WGPU Win32 surface"]
fn million_character_visible_edit_reaches_native_present_and_changes_pixels() {
    assert!(
        !cfg!(debug_assertions),
        "run this diagnostic with --release"
    );
    let (sender, receiver) = mpsc::sync_channel(1);
    thread::Builder::new()
        .name("runtime82-native-present".to_string())
        .spawn(move || {
            let result = panic::catch_unwind(run_native_present_diagnostic);
            let _ = sender.send(result);
        })
        .expect("spawn bounded native present diagnostic thread");
    match receiver.recv_timeout(DIAGNOSTIC_THREAD_TIMEOUT) {
        Ok(Ok(())) => {}
        Ok(Err(payload)) => panic::resume_unwind(payload),
        Err(mpsc::RecvTimeoutError::Timeout) => {
            panic!("native present diagnostic exceeded its 60-second test-thread deadline")
        }
        Err(mpsc::RecvTimeoutError::Disconnected) => {
            panic!("native present diagnostic thread exited without reporting a result")
        }
    }
}

fn run_native_present_diagnostic() {
    let mut builder = EventLoop::builder();
    builder.with_any_thread(true);
    let event_loop = builder.build().expect("create Windows test event loop");
    let (result_sender, result_receiver) = mpsc::sync_channel(1);
    let app = NativePresentDiagnostic::new(result_sender);
    event_loop
        .run_app(app)
        .expect("run native present diagnostic");
    let (before, after, changed_pixels) = result_receiver
        .try_recv()
        .expect("native surface or redraw was unavailable before the event-loop deadline");
    assert!(after.generation > before.generation);
    assert!(
        changed_pixels > 4,
        "visible text region must change after the edit"
    );
}

struct NativePresentDiagnostic {
    window: Option<Arc<dyn Window>>,
    surfaces_available: bool,
    event_deadline: Instant,
    result: Option<(PresentedFrame, PresentedFrame, usize)>,
    result_sender: mpsc::SyncSender<(PresentedFrame, PresentedFrame, usize)>,
}

impl NativePresentDiagnostic {
    fn new(result_sender: mpsc::SyncSender<(PresentedFrame, PresentedFrame, usize)>) -> Self {
        Self {
            window: None,
            surfaces_available: false,
            event_deadline: Instant::now() + WINDOW_EVENT_TIMEOUT,
            result: None,
            result_sender,
        }
    }

    fn ensure_window(&mut self, event_loop: &dyn ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }
        let attributes = WindowAttributes::default()
            .with_title("Runtime82 native text present diagnostic")
            .with_surface_size(Size::Physical(PhysicalSize::new(640, 360)))
            .with_resizable(false);
        let window: Arc<dyn Window> = Arc::from(
            event_loop
                .create_window(attributes)
                .expect("create live diagnostic window"),
        );
        self.window = Some(window);
    }
}

impl ApplicationHandler for NativePresentDiagnostic {
    fn resumed(&mut self, event_loop: &dyn ActiveEventLoop) {
        self.ensure_window(event_loop);
    }

    fn can_create_surfaces(&mut self, event_loop: &dyn ActiveEventLoop) {
        self.ensure_window(event_loop);
        self.surfaces_available = true;
        self.window
            .as_ref()
            .expect("diagnostic window exists")
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
            WindowEvent::RedrawRequested if self.surfaces_available && self.result.is_none() => {
                let window = self.window.as_ref().expect("diagnostic window exists");
                let result = run_present_diagnostic(window.as_ref());
                self.result_sender
                    .send(result)
                    .expect("native present result receiver must remain alive");
                self.result = Some(result);
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

fn run_present_diagnostic(window: &dyn Window) -> (PresentedFrame, PresentedFrame, usize) {
    let physical = window.surface_size();
    let size = UVec2::new(physical.width, physical.height);
    assert!(
        size.x >= (TEXT_FRAME.x + TEXT_FRAME.width) as u32
            && size.y >= (TEXT_FRAME.y + TEXT_FRAME.height) as u32,
        "diagnostic window must contain the visible text field"
    );
    let RawWindowHandle::Win32(handle) = window
        .window_handle()
        .expect("live Win32 window handle")
        .as_raw()
    else {
        panic!("native present diagnostic requires a Win32 window");
    };
    let target = RenderNativeSurfaceTarget::Win32 {
        hwnd: handle.hwnd.get() as usize as u64,
        hinstance: handle.hinstance.map(|value| value.get() as usize as u64),
    };

    let framework = WgpuRenderFramework::new_for_test(Arc::new(ProjectAssetManager::default()))
        .expect("create WGPU render framework");
    let viewport = framework
        .create_viewport(RenderViewportDescriptor::new(size))
        .expect("create diagnostic viewport");
    framework
        .set_quality_profile(
            viewport,
            RenderQualityProfile::new("runtime82-native-text-edit")
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

    let mut surface = million_character_surface();
    let (before, before_capture) = present_and_capture(&framework, viewport, size, &surface);

    surface.focus_node(TEXT_NODE).expect("focus the text input");
    surface.rebuild();
    let mut input = UiInputManager::default();
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
        .expect("dispatch the real keyboard edit");
    let edit_ns = edit_start.elapsed().as_nanos();
    assert!(edit
        .widget_events
        .iter()
        .any(|event| matches!(event, UiWidgetEvent::TextEditChange { .. })));
    surface.clear_focus();
    let rebuild_start = Instant::now();
    surface.rebuild();
    let rebuild_ns = rebuild_start.elapsed().as_nanos();
    let changed_text = rendered_text(&surface);
    assert_eq!(changed_text.len(), DOCUMENT_CHARACTERS);
    assert!(changed_text.starts_with('i'));
    let (after, after_capture) = present_and_capture(&framework, viewport, size, &surface);

    assert_eq!(
        (before_capture.width, before_capture.height),
        (size.x, size.y)
    );
    assert_eq!(
        (after_capture.width, after_capture.height),
        (size.x, size.y)
    );
    let changed_pixels = changed_pixels_in_text_frame(&before_capture, &after_capture);
    println!(
        "{MARKER} document_characters={DOCUMENT_CHARACTERS} edit_ns={edit_ns} rebuild_to_extract_ns={rebuild_ns} present_enqueue_before_ns={} present_enqueue_after_ns={} finish_and_readback_before_ns={} finish_and_readback_after_ns={} generation_before={} generation_after={} text_region_changed_pixels={changed_pixels} os={} arch={} package_version={}",
        before.present_enqueue_ns,
        after.present_enqueue_ns,
        before.finish_and_readback_ns,
        after.finish_and_readback_ns,
        before.generation,
        after.generation,
        std::env::consts::OS,
        std::env::consts::ARCH,
        env!("CARGO_PKG_VERSION"),
    );
    framework
        .unbind_viewport_surface(viewport)
        .expect("release native surface before the window closes");
    (before, after, changed_pixels)
}

fn million_character_surface() -> UiSurface {
    let content = format!("W{}", "i".repeat(DOCUMENT_CHARACTERS));
    let mut surface = UiSurface::new(UiTreeId::new("runtime82.native_present.million_edit"));
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
        .expect("attach native present text field");
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
    let ui = UiRenderSubmission::single_frame(surface.render_frame_extract());
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
        "a scene submission or deferred surface acquire is not a native present"
    );
    assert!(stats.last_ui_command_count > 0);
    assert!(stats.last_ui_text_payload_count > 0);
    assert!(stats.last_ui_graph_executed_pass_count > 0);
    assert_eq!(captured.generation, receipt.frame_generation());
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
