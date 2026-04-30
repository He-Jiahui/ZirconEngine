use std::collections::BTreeMap;
use std::path::Path;

use slint::PhysicalSize;

use crate::ui::animation_editor::AnimationEditorPanePresentation;
use crate::ui::asset_editor::UiAssetEditorPanePresentation;
use crate::ui::layouts::windows::workbench_host_window::ModulePluginsPaneViewData;
use crate::ui::slint_host::floating_window_projection::build_floating_window_projection_bundle;
use crate::ui::slint_host::{apply_presentation, HostMenuStateData, UiHostContext, UiHostWindow};
use crate::ui::workbench::autolayout::{
    compute_workbench_shell_geometry, ShellSizePx, WorkbenchChromeMetrics,
};
use crate::ui::workbench::fixture::default_preview_fixture;
use crate::ui::workbench::model::WorkbenchViewModel;

const SCROLLED_WINDOW_POPUP_SCREENSHOT: &str =
    "editor-window-20260429-window-popup-scrolled-900x620.png";

#[test]
#[ignore = "writes visual screenshot artifact for manual popup closeout"]
fn capture_scrolled_window_popup_visual_artifact() {
    std::env::set_var("SLINT_BACKEND", "software");

    let shell_size = ShellSizePx::new(900.0, 620.0);
    let metrics = WorkbenchChromeMetrics::default();
    let fixture = default_preview_fixture();
    let chrome = fixture.build_chrome();
    let model = WorkbenchViewModel::build(&chrome);
    let geometry = compute_workbench_shell_geometry(
        &model,
        &chrome,
        &fixture.layout,
        &fixture.descriptors,
        shell_size,
        &metrics,
        None,
    );
    let floating_window_projection_bundle =
        build_floating_window_projection_bundle(&model, &geometry, &metrics, &[]);
    let ui_asset_panes: BTreeMap<String, UiAssetEditorPanePresentation> = BTreeMap::new();
    let animation_panes: BTreeMap<String, AnimationEditorPanePresentation> = BTreeMap::new();
    let module_plugins = ModulePluginsPaneViewData::default();
    let preset_names = (0..24)
        .map(|index| format!("Preset {index:02}"))
        .collect::<Vec<_>>();
    let ui = UiHostWindow::new().expect("workbench shell should instantiate for screenshot");

    ui.show()
        .expect("workbench shell should show for screenshot capture");
    ui.window().set_size(PhysicalSize::new(900, 620));
    apply_presentation(
        &ui,
        &model,
        &chrome,
        &geometry,
        &preset_names,
        Some("Preset 03"),
        &ui_asset_panes,
        &animation_panes,
        None,
        &module_plugins,
        None,
        &floating_window_projection_bundle,
        None,
    );
    ui.global::<UiHostContext>()
        .set_menu_state(HostMenuStateData {
            open_menu_index: 4,
            hovered_menu_index: -1,
            hovered_menu_item_index: 17,
            window_menu_scroll_px: 360.0,
            window_menu_popup_height_px: 192.0,
        });

    let snapshot = ui
        .window()
        .take_snapshot()
        .expect("software renderer should capture the scrolled Window popup");
    let output_dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("editor crate should live under the repository root")
        .join("target")
        .join("visual-layout");
    std::fs::create_dir_all(&output_dir).expect("visual-layout output directory should exist");
    let output_path = output_dir.join(SCROLLED_WINDOW_POPUP_SCREENSHOT);

    image::save_buffer_with_format(
        &output_path,
        snapshot.as_bytes(),
        snapshot.width(),
        snapshot.height(),
        image::ColorType::Rgba8,
        image::ImageFormat::Png,
    )
    .expect("scrolled Window popup screenshot should be written as PNG");

    assert!(
        output_path.exists(),
        "expected visual screenshot at {}",
        output_path.display()
    );
}
