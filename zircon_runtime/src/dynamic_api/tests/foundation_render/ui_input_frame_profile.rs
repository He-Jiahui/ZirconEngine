use std::time::Instant;

use crate::asset::ProjectManifest;
use zircon_runtime_interface::{
    ZrRuntimeHostRequestBatchV1, ZrRuntimeHostRequestV1, ZR_RUNTIME_TOUCH_PHASE_ENDED_V1,
    ZR_RUNTIME_TOUCH_PHASE_STARTED_V1,
};

use super::*;

const MARKER: &str = "RUNTIME200_UI_INPUT_TO_CAPTURE_CYCLE_PROFILE_V1";
const WARMUP_SAMPLES: usize = 8;
const MEASURED_SAMPLES: usize = 31;
const PROFILE_VIEW: &str = r#"
[asset]
kind = "view"
id = "product.runtime.input_frame_profile"
version = 2
display_name = "Input Frame Profile"

[root]
node = "root"

[nodes.action]
component = "Button"
control_id = "ProfileAction"
props = { text = "Run" }
layout = { width = { min = 112.0, preferred = 128.0, max = 160.0, stretch = "Fixed" }, height = { min = 44.0, preferred = 44.0, max = 44.0, stretch = "Fixed" } }
events = [{ id = "Profile/Action", event = "Click", route = "runtime.profile.action" }]

[nodes.root]
component = "VerticalBox"
control_id = "ProfileRoot"
layout = { container = { kind = "VerticalBox" }, padding = { left = 16.0, right = 16.0, top = 16.0, bottom = 16.0 }, width = { stretch = "Stretch" }, height = { stretch = "Stretch" } }
children = [{ node = "action" }]
"#;

#[test]
fn product_pointer_action_reaches_host_output_and_render_extract() {
    let project = profile_project();
    let mut session = profile_session(&project);

    dispatch_clicks(&mut session, 1);
    let bytes = drain_host_output(&mut session);
    assert_profile_actions(&bytes, 1);
    let submission = session
        .current_ui_submission()
        .expect("extract product UI after pointer action")
        .expect("project UI must have a render submission");
    assert!(submission
        .commands()
        .any(|command| command.text.as_deref() == Some("Run")));

    let drained = drain_host_output(&mut session);
    assert!(
        drained.is_empty(),
        "host output must not replay a committed action"
    );
    drop(session);
    project.assert_removable_after_sessions_drop();
}

#[test]
#[ignore = "managed Windows WGPU product input-to-frame profiling evidence"]
fn product_pointer_action_to_wgpu_capture_cycle_profiles_dispatch_host_and_frame() {
    let project = profile_project();
    for clicks_per_sample in [1, 16] {
        let mut session = profile_session(&project);
        for _ in 0..WARMUP_SAMPLES {
            let _ = capture_input_cycle(&mut session, clicks_per_sample);
        }

        let mut dispatch = Vec::with_capacity(MEASURED_SAMPLES);
        let mut host = Vec::with_capacity(MEASURED_SAMPLES);
        let mut frame = Vec::with_capacity(MEASURED_SAMPLES);
        let mut total = Vec::with_capacity(MEASURED_SAMPLES);
        for _ in 0..MEASURED_SAMPLES {
            let sample = capture_input_cycle(&mut session, clicks_per_sample);
            dispatch.push(sample.dispatch_ns);
            host.push(sample.host_ns);
            frame.push(sample.frame_ns);
            total.push(sample.total_ns);
        }

        let stats = collect_runtime_diagnostics(&session.runtime.handle())
            .render
            .stats
            .expect("product frame must publish render statistics");
        assert!(stats.last_ui_command_count > 0);
        print_percentiles(clicks_per_sample, "dispatch", dispatch);
        print_percentiles(clicks_per_sample, "host", host);
        print_percentiles(clicks_per_sample, "wgpu_frame", frame);
        print_percentiles(clicks_per_sample, "input_to_capture_cycle", total);
        drop(session);
    }
    project.assert_removable_after_sessions_drop();
}

struct InputCycleSample {
    dispatch_ns: u128,
    host_ns: u128,
    frame_ns: u128,
    total_ns: u128,
}

fn capture_input_cycle(session: &mut RuntimeDynamicSession, clicks: usize) -> InputCycleSample {
    let cycle_start = Instant::now();
    dispatch_clicks(session, clicks);
    let dispatch_ns = cycle_start.elapsed().as_nanos();

    let host_start = Instant::now();
    let bytes = drain_host_output(session);
    let host_ns = host_start.elapsed().as_nanos();

    let frame_start = Instant::now();
    session
        .tick_frame()
        .expect("advance product runtime before capture");
    let captured = session
        .capture_frame(ZrRuntimeFrameRequestV1::new(
            ZIRCON_RUNTIME_ABI_VERSION_V1,
            ZrRuntimeViewportHandle::new(1),
            ZrRuntimeViewportSizeV1::new(CAPTURE_WIDTH, CAPTURE_HEIGHT),
        ))
        .expect("capture product WGPU frame after pointer input");
    let frame_ns = frame_start.elapsed().as_nanos();
    let total_ns = cycle_start.elapsed().as_nanos();

    assert_profile_actions(&bytes, clicks);
    assert_eq!(
        (captured.width, captured.height),
        (CAPTURE_WIDTH, CAPTURE_HEIGHT)
    );
    assert_eq!(
        captured.rgba.len(),
        (CAPTURE_WIDTH * CAPTURE_HEIGHT * 4) as usize
    );
    let background = &captured.rgba[..4];
    let visible_pixels = captured
        .rgba
        .chunks_exact(4)
        .filter(|pixel| pixel[3] != 0)
        .count();
    let changed_pixels = captured
        .rgba
        .chunks_exact(4)
        .filter(|pixel| *pixel != background)
        .count();
    assert!(visible_pixels > 0 && changed_pixels > 100);
    let stats = collect_runtime_diagnostics(&session.runtime.handle())
        .render
        .stats
        .expect("captured product frame must publish render statistics");
    assert!(stats.last_graph_executed_pass_count > 0);
    assert!(stats.last_ui_command_count > 0);
    InputCycleSample {
        dispatch_ns,
        host_ns,
        frame_ns,
        total_ns,
    }
}

fn print_percentiles(clicks: usize, stage: &str, mut samples: Vec<u128>) {
    assert_eq!(samples.len(), MEASURED_SAMPLES);
    samples.sort_unstable();
    let percentile = |percent: usize| samples[(samples.len() * percent).div_ceil(100) - 1];
    println!(
        "{MARKER} clicks={clicks} stage={stage} samples={} p50_ns={} p95_ns={} p99_ns={}",
        samples.len(),
        percentile(50),
        percentile(95),
        percentile(99)
    );
}

fn profile_project() -> F2Project {
    let project = F2Project::create();
    let asset_path = project.root.join("assets/ui/input_frame_profile.zui");
    std::fs::create_dir_all(asset_path.parent().expect("UI asset has a parent"))
        .expect("create UI asset directory");
    std::fs::write(&asset_path, PROFILE_VIEW).expect("write product UI profile view");

    let paths = ProjectPaths::from_root(&project.root).expect("product UI project paths");
    let mut manifest = ProjectManifest::load(paths.manifest_path()).expect("load project manifest");
    manifest
        .ui_roots
        .push(AssetUri::parse("res://ui/input_frame_profile.zui").expect("profile view asset URI"));
    manifest
        .save(paths.manifest_path())
        .expect("persist profile UI root");
    project
}

fn profile_session(project: &F2Project) -> RuntimeDynamicSession {
    RuntimeDynamicSession::new(
        RuntimeDynamicSessionProfile::Runtime,
        Some(RuntimeProjectConfig::from_root(&project.root).expect("profile project config")),
    )
    .expect("load profile UI into product runtime")
}

fn dispatch_clicks(session: &mut RuntimeDynamicSession, clicks: usize) {
    for _ in 0..clicks {
        for phase in [
            ZR_RUNTIME_TOUCH_PHASE_STARTED_V1,
            ZR_RUNTIME_TOUCH_PHASE_ENDED_V1,
        ] {
            let status = session.handle_event(ZrRuntimeEventV1::touch(
                ZIRCON_RUNTIME_ABI_VERSION_V1,
                ZrRuntimeViewportHandle::new(1),
                41,
                phase,
                24.0,
                24.0,
            ));
            assert!(status.is_ok(), "product pointer input failed: {status:?}");
        }
    }
}

fn drain_host_output(session: &mut RuntimeDynamicSession) -> Vec<u8> {
    let bytes = session
        .prepare_host_request_output()
        .expect("encode product UI host requests");
    session.commit_host_request_output();
    bytes
}

fn assert_profile_actions(bytes: &[u8], expected: usize) {
    let batch: ZrRuntimeHostRequestBatchV1 =
        serde_json::from_slice(bytes).expect("decode typed product UI action batch");
    let actions = batch
        .requests
        .iter()
        .filter_map(|request| match request {
            ZrRuntimeHostRequestV1::UiAction(action) => Some(action),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(actions.len(), expected, "one host action per pointer click");
    assert!(actions.iter().all(|action| {
        action.target_surface == 1 && action.invocation.target_id() == "runtime.profile.action"
    }));
}
