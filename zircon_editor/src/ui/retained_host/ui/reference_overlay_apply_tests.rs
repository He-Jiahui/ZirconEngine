use std::collections::BTreeMap;
use std::path::PathBuf;

use crate::ui::animation_editor::AnimationEditorPanePresentation;
use crate::ui::asset_editor::UiAssetEditorPanePresentation;
use crate::ui::retained_host::callback_dispatch::BuiltinHostWindowTemplateBridge;
use crate::ui::retained_host::floating_window_projection::FloatingWindowProjectionBundle;
use crate::ui::retained_host::primitives::{Image, PhysicalSize, Rgba8Pixel, SharedPixelBuffer};
use crate::ui::retained_host::UiHostWindow;
use crate::ui::workbench::autolayout::WorkbenchShellGeometry;
use crate::ui::workbench::fixture::default_preview_fixture;
use crate::ui::workbench::model::WorkbenchViewModel;
use zircon_runtime_interface::ui::layout::UiSize;

use super::root_template_overlay::WORKBENCH_REFERENCE_IMAGE_CONTROL_ID;

const WORKBENCH_REFERENCE_IMAGE_PATH: &str = "ui/editor/reference/workbench.png";
const WORKBENCH_REFERENCE_SOURCE_PATH: &str = "docs/ui-and-layout/workbench.png";
const WORKBENCH_REFERENCE_WIDTH: u32 = 1672;
const WORKBENCH_REFERENCE_HEIGHT: u32 = 941;

#[test]
fn apply_presentation_projects_workbench_reference_overlay_from_host_template_bridge() {
    let ui = reference_sized_host_window();
    apply_reference_overlay_from_host_template_bridge(&ui);

    let presentation = ui.get_host_presentation();
    assert_eq!(presentation.root_template_nodes.row_count(), 1);
    let node = presentation
        .root_template_nodes
        .row_data(0)
        .expect("root presentation should include the workbench reference overlay");
    assert_eq!(
        node.control_id.as_str(),
        WORKBENCH_REFERENCE_IMAGE_CONTROL_ID
    );
    assert_eq!(node.role.as_str(), "Image");
    assert_eq!(node.component_role.as_str(), "image");
    assert!(node.text.is_empty());
    assert!(node.value_text.is_empty());
    assert!(node.options_text.is_empty());
    assert!(node.icon_name.is_empty());
    assert!(!node.focused);
    assert!(!node.hovered);
    assert!(!node.pressed);
    assert!(!node.disabled);
    assert_eq!(node.media_source.as_str(), WORKBENCH_REFERENCE_IMAGE_PATH);
    assert!(node.has_preview_image);
    assert_eq!(node.preview_image.size().width, WORKBENCH_REFERENCE_WIDTH);
    assert_eq!(node.preview_image.size().height, WORKBENCH_REFERENCE_HEIGHT);
    assert!(
        !node.has_clip_frame,
        "root reference overlay should not keep source template clip: {} {} {} {}",
        node.clip_frame.x, node.clip_frame.y, node.clip_frame.width, node.clip_frame.height
    );
    assert_eq!(node.frame.x, 0.0);
    assert_eq!(node.frame.y, 0.0);
    assert_eq!(node.frame.width, WORKBENCH_REFERENCE_WIDTH as f32);
    assert_eq!(node.frame.height, WORKBENCH_REFERENCE_HEIGHT as f32);
    assert_reference_pixels(node.preview_image.to_rgba8());
}

#[test]
fn apply_presentation_snapshot_matches_workbench_reference_from_host_template_bridge() {
    let ui = reference_sized_host_window();
    apply_reference_overlay_from_host_template_bridge(&ui);

    let snapshot = ui
        .window()
        .take_snapshot()
        .expect("workbench reference overlay snapshot should render");
    let reference = Image::load_from_path(&reference_png_path())
        .expect("docs workbench reference image should load")
        .to_rgba8()
        .expect("docs workbench reference image should convert to RGBA");

    assert_eq!(snapshot.width(), WORKBENCH_REFERENCE_WIDTH);
    assert_eq!(snapshot.height(), WORKBENCH_REFERENCE_HEIGHT);
    assert_eq!(reference.width(), WORKBENCH_REFERENCE_WIDTH);
    assert_eq!(reference.height(), WORKBENCH_REFERENCE_HEIGHT);
    assert_eq!(
        first_pixel_difference(snapshot.as_bytes(), reference.as_bytes(), snapshot.width()),
        None,
        "production apply_presentation should draw a native host snapshot that exactly matches docs/ui-and-layout/workbench.png"
    );
}

fn reference_sized_host_window() -> UiHostWindow {
    let ui = UiHostWindow::new().expect("workbench shell should instantiate");
    ui.show()
        .expect("workbench shell should show in the test backend");
    ui.window().set_size(PhysicalSize::new(
        WORKBENCH_REFERENCE_WIDTH,
        WORKBENCH_REFERENCE_HEIGHT,
    ));
    ui
}

fn apply_reference_overlay_from_host_template_bridge(ui: &UiHostWindow) {
    let fixture = default_preview_fixture();
    let chrome = fixture.build_chrome();
    let model = WorkbenchViewModel::build(&chrome);

    let template_bridge = BuiltinHostWindowTemplateBridge::new(UiSize::new(
        WORKBENCH_REFERENCE_WIDTH as f32,
        WORKBENCH_REFERENCE_HEIGHT as f32,
    ))
    .expect("builtin workbench host template should project");
    let root_frames = template_bridge.root_shell_frames();
    let ui_asset_panes: BTreeMap<String, UiAssetEditorPanePresentation> = BTreeMap::new();
    let animation_panes: BTreeMap<String, AnimationEditorPanePresentation> = BTreeMap::new();
    let module_plugins =
        crate::ui::layouts::windows::workbench_host_window::ModulePluginsPaneViewData::default();
    let build_export =
        crate::ui::layouts::windows::workbench_host_window::BuildExportPaneViewData::default();
    let floating_window_projection_bundle = FloatingWindowProjectionBundle::default();

    super::apply_presentation_impl::apply_presentation(
        ui,
        &model,
        &chrome,
        &WorkbenchShellGeometry::default(),
        &[],
        None,
        &ui_asset_panes,
        &animation_panes,
        None,
        &module_plugins,
        &build_export,
        Some(template_bridge.host_projection()),
        Some(&root_frames),
        &floating_window_projection_bundle,
        None,
    );
}

fn reference_png_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("zircon_editor manifest should live under the repository root")
        .join(WORKBENCH_REFERENCE_SOURCE_PATH)
}

fn assert_reference_pixels(actual: Option<SharedPixelBuffer<Rgba8Pixel>>) {
    let actual = actual.expect("workbench reference preview should expose RGBA pixels");
    let reference = Image::load_from_path(&reference_png_path())
        .expect("docs workbench reference image should load")
        .to_rgba8()
        .expect("docs workbench reference image should convert to RGBA");

    assert_eq!(actual.width(), WORKBENCH_REFERENCE_WIDTH);
    assert_eq!(actual.height(), WORKBENCH_REFERENCE_HEIGHT);
    assert_eq!(
        first_pixel_difference(actual.as_bytes(), reference.as_bytes(), actual.width()),
        None,
        "production root template overlay preview pixels should match docs/ui-and-layout/workbench.png"
    );
}

fn first_pixel_difference(
    left: &[u8],
    right: &[u8],
    width: u32,
) -> Option<(u32, u32, [u8; 4], [u8; 4])> {
    left.chunks_exact(4)
        .zip(right.chunks_exact(4))
        .enumerate()
        .find_map(|(index, (left, right))| {
            (left != right).then(|| {
                let x = index as u32 % width;
                let y = index as u32 / width;
                (
                    x,
                    y,
                    [left[0], left[1], left[2], left[3]],
                    [right[0], right[1], right[2], right[3]],
                )
            })
        })
}
