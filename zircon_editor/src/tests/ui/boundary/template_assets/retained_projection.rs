use std::fs;
use std::path::Path;

use super::super::support::collect_rust_files;
use super::support::source;

#[test]
fn rust_owned_template_node_contract_keeps_retained_widget_state() {
    let template_nodes = source("src/ui/retained_host/host_contract/data/template_nodes/node.rs");

    for required in [
        "pub(crate) struct TemplatePaneNodeData",
        "pub component_role: SharedString",
        "pub component_category: SharedString",
        "pub component_layout_role: SharedString",
        "pub component_variant: SharedString",
        "pub label_text: SharedString",
        "pub label_color: Color",
        "pub label_brightness: f32",
        "pub layout_offset_x: f32",
        "pub layout_offset_y: f32",
        "pub layout_icon_size: f32",
        "pub layout_first_cell_offset_x: f32",
        "pub layout_second_cell_offset_x: f32",
        "pub layout_third_cell_offset_x: f32",
        "pub layout_fourth_cell_offset_x: f32",
        "pub value_number: f32",
        "pub value_percent: f32",
        "pub value_color: Color",
        "pub icon_color: Color",
        "pub icon_stroke_width: f32",
        "pub media_source: SharedString",
        "pub icon_name: SharedString",
        "pub has_preview_image: bool",
        "pub vector_components: ModelRc<f32>",
        "pub structured_options: ModelRc<TemplatePaneOptionData>",
        "pub collection_fields: ModelRc<TemplatePaneCollectionFieldData>",
        "pub structured_menu_items: ModelRc<TemplatePaneMenuItemData>",
        "pub actions: ModelRc<TemplatePaneActionData>",
        "pub surface_variant: SharedString",
        "pub text_tone: SharedString",
        "pub button_variant: SharedString",
        "pub button_style: ResolvedButtonStyle",
        "pub font_size: f32",
        "pub font_weight: i32",
        "pub text_align: SharedString",
        "pub overflow: SharedString",
        "pub corner_radius: f32",
        "pub border_width: f32",
    ] {
        assert!(
            template_nodes.contains(required),
            "template node DTO missing `{required}`"
        );
    }
}

#[test]
fn workbench_projection_uses_editor_assets_without_generated_host_dto_imports() {
    let root =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("src/ui/layouts/windows/workbench_host_window");

    for path in collect_rust_files(&root) {
        let text =
            fs::read_to_string(&path).unwrap_or_else(|error| panic!("read {:?}: {error}", path));
        assert!(
            !text.contains("crate::ui::retained_host::{FrameRect")
                && !text.contains("crate::ui::retained_host::{PaneData")
                && !text.contains("crate::ui::retained_host::HostWindowPresentationData"),
            "workbench projection internals should not import generated host DTOs: {:?}",
            path.file_name().expect("file name")
        );
    }
}
