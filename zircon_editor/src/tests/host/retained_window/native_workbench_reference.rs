use std::path::PathBuf;
use std::rc::Rc;

use crate::ui::layouts::views::load_preview_image;
use crate::ui::retained_host::callback_dispatch::BuiltinHostWindowTemplateBridge;
use crate::ui::retained_host::primitives::{Image, ModelRc, PhysicalSize, VecModel};
use crate::ui::retained_host::{
    HostWindowPresentationData, TemplateNodeFrameData, TemplatePaneNodeData, UiHostWindow,
};
use crate::ui::template_runtime::RetainedUiHostValue;
use zircon_runtime_interface::ui::layout::{UiFrame, UiSize};

const WORKBENCH_REFERENCE_IMAGE_CONTROL_ID: &str = "WorkbenchShellReferenceImage";
const WORKBENCH_REFERENCE_IMAGE_PATH: &str = "ui/editor/reference/workbench.png";
const WORKBENCH_REFERENCE_SOURCE_PATH: &str = "docs/ui-and-layout/workbench.png";
const WORKBENCH_REFERENCE_WIDTH: u32 = 1672;
const WORKBENCH_REFERENCE_HEIGHT: u32 = 941;

#[test]
fn host_window_template_bridge_projects_workbench_reference_overlay_node() {
    let bridge = BuiltinHostWindowTemplateBridge::new(UiSize::new(
        WORKBENCH_REFERENCE_WIDTH as f32,
        WORKBENCH_REFERENCE_HEIGHT as f32,
    ))
    .expect("builtin workbench host template should project");

    let node = bridge
        .host_projection()
        .node_by_control_id(WORKBENCH_REFERENCE_IMAGE_CONTROL_ID)
        .expect("host template should project the workbench reference image overlay node");

    assert_eq!(node.component, "Image");
    assert_eq!(
        node.frame,
        UiFrame::new(
            0.0,
            0.0,
            WORKBENCH_REFERENCE_WIDTH as f32,
            WORKBENCH_REFERENCE_HEIGHT as f32,
        )
    );
    assert_eq!(
        string_property(&node.properties, "image").as_deref(),
        Some(WORKBENCH_REFERENCE_IMAGE_PATH)
    );
    assert_eq!(
        string_property(&node.properties, "reference_source").as_deref(),
        Some(WORKBENCH_REFERENCE_SOURCE_PATH)
    );
}

#[test]
fn native_host_window_snapshot_draws_workbench_reference_overlay_pixels() {
    let ui = UiHostWindow::new().expect("workbench shell should instantiate");
    ui.show()
        .expect("workbench shell should show in test backend");
    ui.window().set_size(PhysicalSize::new(
        WORKBENCH_REFERENCE_WIDTH,
        WORKBENCH_REFERENCE_HEIGHT,
    ));

    let mut presentation = HostWindowPresentationData::default();
    presentation.root_template_nodes = model_rc(vec![reference_image_node()]);
    ui.set_host_presentation(presentation);

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
        "native host snapshot should exactly match docs/ui-and-layout/workbench.png when the root reference overlay is active"
    );
}

fn reference_image_node() -> TemplatePaneNodeData {
    TemplatePaneNodeData {
        node_id: "workbench_reference_image".into(),
        control_id: WORKBENCH_REFERENCE_IMAGE_CONTROL_ID.into(),
        role: "Image".into(),
        component_role: "image".into(),
        media_source: WORKBENCH_REFERENCE_IMAGE_PATH.into(),
        has_preview_image: true,
        preview_image: load_preview_image(WORKBENCH_REFERENCE_IMAGE_PATH, ""),
        frame: TemplateNodeFrameData {
            x: 0.0,
            y: 0.0,
            width: WORKBENCH_REFERENCE_WIDTH as f32,
            height: WORKBENCH_REFERENCE_HEIGHT as f32,
        },
        ..TemplatePaneNodeData::default()
    }
}

fn reference_png_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("zircon_editor manifest should live under the repository root")
        .join(WORKBENCH_REFERENCE_SOURCE_PATH)
}

fn string_property(
    properties: &std::collections::BTreeMap<String, RetainedUiHostValue>,
    key: &str,
) -> Option<String> {
    match properties.get(key) {
        Some(RetainedUiHostValue::String(value)) => Some(value.clone()),
        _ => None,
    }
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

fn model_rc<T: Clone + 'static>(values: Vec<T>) -> ModelRc<T> {
    ModelRc::from(Rc::new(VecModel::from(values)))
}
