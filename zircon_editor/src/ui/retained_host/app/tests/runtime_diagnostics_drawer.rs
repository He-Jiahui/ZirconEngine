use std::path::PathBuf;

use super::*;

const RUNTIME_DIAGNOSTICS_INSTANCE_ID: &str = "editor.runtime_diagnostics#1";
const RUNTIME_DIAGNOSTICS_VISUAL_ARTIFACT: &str =
    "editor-runtime-diagnostics-bottom-drawer-1280x720.png";

#[test]
fn native_root_bottom_drawer_header_click_activates_runtime_diagnostics_in_real_host() {
    let _guard = lock_env();

    let harness =
        ChildWindowHostHarness::new("zircon_retained_native_bottom_runtime_diag_dispatch");
    harness.activate_workbench_page();
    let baseline = harness.journal_len();

    dispatch_runtime_diagnostics_bottom_tab(&harness);

    assert_eq!(
        harness.delta_events_since(baseline),
        vec![EditorEvent::Layout(EventLayoutCommand::ActivateDrawerTab {
            slot: EventActivityDrawerSlot::Bottom,
            instance_id: EventViewInstanceId::new(RUNTIME_DIAGNOSTICS_INSTANCE_ID),
        })]
    );
    assert_runtime_diagnostics_bottom_drawer_visible(&harness);
}

#[test]
#[ignore = "captures a PNG visual artifact under target/editor-visual-check"]
fn capture_runtime_diagnostics_bottom_drawer_visual_artifact() {
    let _guard = lock_env();

    let harness = ChildWindowHostHarness::new("zircon_retained_runtime_diag_visual_artifact");
    harness
        .root_ui
        .window()
        .set_size(PhysicalSize::new(1280, 720));
    {
        let mut host = harness.host.borrow_mut();
        host.sync_shell_size();
        host.refresh_ui();
        host.recompute_if_dirty();
    }
    harness.activate_workbench_page();

    dispatch_runtime_diagnostics_bottom_tab(&harness);
    assert_runtime_diagnostics_bottom_drawer_visible(&harness);
    save_window_snapshot(&harness.root_ui, RUNTIME_DIAGNOSTICS_VISUAL_ARTIFACT);
}

fn dispatch_runtime_diagnostics_bottom_tab(harness: &ChildWindowHostHarness) {
    let presentation = harness.root_ui.get_host_presentation();
    let bottom = &presentation.host_scene_data.bottom_dock;
    let runtime_tab = (0..bottom.tab_frames.row_count())
        .filter_map(|row| bottom.tab_frames.row_data(row).map(|tab| (row, tab)))
        .find(|(_, tab)| tab.tab.id.as_str() == RUNTIME_DIAGNOSTICS_INSTANCE_ID)
        .expect("runtime diagnostics bottom tab should be projected");
    let tab = runtime_tab.1;
    let click_x =
        bottom.region_frame.x + bottom.header_frame.x + tab.frame.x + tab.frame.width * 0.5;
    let click_y =
        bottom.region_frame.y + bottom.header_frame.y + tab.frame.y + tab.frame.height * 0.5;

    let dispatch = harness
        .root_ui
        .dispatch_native_primary_press_for_test(click_x, click_y);

    assert!(dispatch.request_redraw());
}

fn assert_runtime_diagnostics_bottom_drawer_visible(harness: &ChildWindowHostHarness) {
    let layout = harness.host.borrow().runtime.current_layout();
    let drawer = layout.drawers.get(&ActivityDrawerSlot::Bottom).unwrap();
    assert_eq!(
        drawer.active_view.as_ref().map(|id| id.0.as_str()),
        Some(RUNTIME_DIAGNOSTICS_INSTANCE_ID)
    );
    drop(layout);

    harness.host.borrow_mut().recompute_if_dirty();
    let presentation = harness.root_ui.get_host_presentation();
    let active_bottom_tabs: Vec<_> = (0..presentation
        .host_scene_data
        .bottom_dock
        .tab_frames
        .row_count())
        .filter_map(|row| {
            presentation
                .host_scene_data
                .bottom_dock
                .tab_frames
                .row_data(row)
        })
        .filter(|tab| tab.tab.active)
        .map(|tab| tab.tab.id.to_string())
        .collect();
    assert_eq!(
        active_bottom_tabs,
        vec![RUNTIME_DIAGNOSTICS_INSTANCE_ID.to_string()]
    );
    assert_eq!(
        presentation.host_scene_data.bottom_dock.pane.kind.as_str(),
        "RuntimeDiagnostics"
    );
}

fn save_window_snapshot(ui: &UiHostWindow, filename: &str) -> PathBuf {
    let snapshot = ui
        .window()
        .take_snapshot()
        .unwrap_or_else(|error| panic!("software renderer should capture {filename}: {error}"));
    let output_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("editor crate should live under the repository root")
        .join("target")
        .join("editor-visual-check");
    std::fs::create_dir_all(&output_dir).expect("editor visual output directory should exist");
    let output_path = output_dir.join(filename);

    image::save_buffer_with_format(
        &output_path,
        snapshot.as_bytes(),
        snapshot.width(),
        snapshot.height(),
        image::ColorType::Rgba8,
        image::ImageFormat::Png,
    )
    .unwrap_or_else(|error| panic!("{filename} should be written as PNG: {error}"));

    assert!(
        output_path.exists(),
        "expected runtime diagnostics screenshot at {}",
        output_path.display()
    );
    output_path
}
