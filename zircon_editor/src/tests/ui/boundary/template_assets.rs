use std::fs;
use std::path::{Path, PathBuf};

use super::support::collect_rust_files;

fn source(relative: &str) -> String {
    fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join(relative))
        .unwrap_or_else(|error| panic!("read `{relative}`: {error}"))
}

fn assert_no_files_with_extension(root: PathBuf, extension: &str) {
    if !root.exists() {
        return;
    }

    let mut stack = vec![root];
    while let Some(path) = stack.pop() {
        if path.is_dir() {
            for entry in fs::read_dir(&path)
                .unwrap_or_else(|error| panic!("read `{}`: {error}", path.display()))
            {
                stack.push(
                    entry
                        .unwrap_or_else(|error| {
                            panic!("read entry under `{}`: {error}", path.display())
                        })
                        .path(),
                );
            }
            continue;
        }

        assert_ne!(
            path.extension().and_then(|value| value.to_str()),
            Some(extension),
            "active editor UI tree should not contain `{}`",
            path.display()
        );
    }
}

#[test]
fn build_script_tracks_editor_assets_not_deleted_ui_sources() {
    let build = source("build.rs");

    assert!(build.contains("emit_rerun_if_changed_recursive(\"assets\")"));
    assert!(!build.contains("emit_rerun_if_changed_recursive(\"ui\")"));
    assert!(!build.contains("compile_slint_ui"));
}

#[test]
fn active_editor_ui_tree_contains_no_deleted_source_files() {
    assert_no_files_with_extension(Path::new(env!("CARGO_MANIFEST_DIR")).join("ui"), "slint");
}

#[test]
fn host_template_assets_are_toml_authority_for_editor_shells() {
    for (relative, markers) in [
        (
            "assets/ui/editor/host/workbench_shell.ui.toml",
            &["UiHostWindow", "activity_rail", "document_host", "menu_bar"] as &[_],
        ),
        (
            "assets/ui/editor/host/workbench_drawer_source.ui.toml",
            &[
                "WorkbenchDrawerSource",
                "BottomDrawerHeaderRoot",
                "LeftDrawerPanelRoot",
            ],
        ),
        (
            "assets/ui/editor/host/floating_window_source.ui.toml",
            &["FloatingWindowSourceRoot", "FloatingWindowTopBarRoot"],
        ),
        (
            "assets/ui/editor/host/scene_viewport_toolbar.ui.toml",
            &["SceneViewportToolbarRoot", "FrameSelection"],
        ),
        (
            "assets/ui/editor/host/asset_surface_controls.ui.toml",
            &["AssetSurfaceControls", "OpenAssetBrowser"],
        ),
        (
            "assets/ui/editor/host/inspector_surface_controls.ui.toml",
            &["InspectorSurfaceControls", "DeleteSelected"],
        ),
        (
            "assets/ui/editor/host/startup_welcome_controls.ui.toml",
            &["CreateProject", "OpenExistingProject"],
        ),
    ] {
        let asset = source(relative);
        for marker in markers {
            assert!(asset.contains(marker), "{relative} missing `{marker}`");
        }
    }
}

#[test]
fn component_showcase_uses_material_meta_component_assets() {
    for (relative, markers) in [
        (
            "assets/ui/editor/material_meta_components.ui.toml",
            &[
                "MaterialStateLayer",
                "MaterialRipple",
                "MaterialButtonBase",
                "MaterialButton",
                "MaterialTextButton",
                "MaterialIconButton",
                "MaterialToggleButton",
                "MaterialCheckboxRow",
                "MaterialCheckBox",
                "MaterialOutlinedField",
                "MaterialSliderField",
                "MaterialListItem",
                "MaterialComboBox",
                "MaterialDatePickerPopup",
                "MaterialGroupBox",
                "MaterialLineEdit",
                "MaterialMenuBarItem",
                "MaterialMenuBar",
                "MaterialMenuFrame",
                "MaterialMenuItem",
                "MaterialProgressIndicator",
                "MaterialScrollView",
                "MaterialSlider",
                "MaterialSpinBox",
                "MaterialSpinner",
                "MaterialSwitch",
                "MaterialStandardTableView",
                "MaterialTabWidgetImpl",
                "MaterialTabImpl",
                "MaterialTabBarHorizontalImpl",
                "MaterialTabBarVerticalImpl",
                "MaterialTabWidget",
                "MaterialTextEdit",
                "MaterialTimePickerPopup",
            ] as &[_],
        ),
        (
            "assets/ui/editor/component_showcase.ui.toml",
            &[
                "material_meta_components.ui.toml#MaterialButton",
                "material_meta_components.ui.toml#MaterialSwitch",
                "material_meta_components.ui.toml#MaterialCheckBox",
                "material_meta_components.ui.toml#MaterialLineEdit",
                "material_meta_components.ui.toml#MaterialTextEdit",
                "material_meta_components.ui.toml#MaterialSpinBox",
                "material_meta_components.ui.toml#MaterialSlider",
                "material_meta_components.ui.toml#MaterialComboBox",
                "material_meta_components.ui.toml#MaterialGroupBox",
                "material_meta_components.ui.toml#MaterialListItem",
                "material_meta_components.ui.toml#MaterialStandardTableView",
                "material_meta_components.ui.toml#MaterialMenuFrame",
            ],
        ),
    ] {
        let asset = source(relative);
        for marker in markers {
            assert!(asset.contains(marker), "{relative} missing `{marker}`");
        }
    }
}

#[test]
fn material_meta_components_cover_slint_material_exports() {
    let asset = source("assets/ui/editor/material_meta_components.ui.toml");
    for component in [
        "ButtonBase",
        "Button",
        "TextButton",
        "IconButton",
        "CheckBox",
        "ComboBox",
        "Ripple",
        "StateLayer",
        "ListItem",
        "DatePickerPopup",
        "GroupBox",
        "LineEdit",
        "MenuBarItem",
        "MenuBar",
        "MenuFrame",
        "MenuItem",
        "ProgressIndicator",
        "ScrollView",
        "Slider",
        "SpinBox",
        "Spinner",
        "Switch",
        "StandardTableView",
        "TabWidgetImpl",
        "TabImpl",
        "TabBarHorizontalImpl",
        "TabBarVerticalImpl",
        "TabWidget",
        "TextEdit",
        "TimePickerPopup",
    ] {
        let marker = format!("[components.Material{component}]");
        assert!(
            asset.contains(&marker),
            "material_meta_components.ui.toml missing Slint Material export `{component}`"
        );
    }
}

#[test]
fn rust_owned_template_node_contract_keeps_retained_widget_state() {
    let template_nodes = source("src/ui/slint_host/host_contract/data/template_nodes.rs");

    for required in [
        "pub(crate) struct TemplatePaneNodeData",
        "pub component_role: SharedString",
        "pub value_number: f32",
        "pub value_percent: f32",
        "pub value_color: Color",
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
            !text.contains("crate::ui::slint_host::{FrameRect")
                && !text.contains("crate::ui::slint_host::{PaneData")
                && !text.contains("crate::ui::slint_host::HostWindowPresentationData"),
            "workbench projection internals should not import generated host DTOs: {:?}",
            path.file_name().expect("file name")
        );
    }
}
