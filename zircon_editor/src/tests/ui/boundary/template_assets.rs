use std::fs;
use std::path::{Path, PathBuf};

use super::support::collect_rust_files;
use zircon_runtime::ui::v2::UiZuiAssetLoader;

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

fn assert_no_legacy_ui_toml(root: PathBuf) {
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

        let Some(file_name) = path.file_name().and_then(|value| value.to_str()) else {
            continue;
        };
        assert!(
            !file_name.ends_with(".ui.toml") || file_name.ends_with(".v2.ui.toml"),
            "packaged UI asset tree must not contain legacy schema file `{}`",
            path.display()
        );
    }
}

#[test]
fn build_script_tracks_editor_assets_not_deleted_ui_sources() {
    let build = source("build.rs");

    assert!(build.contains("emit_rerun_if_changed_recursive(\"assets\")"));
    assert!(!build.contains("emit_rerun_if_changed_recursive(\"ui\")"));
    assert!(!build.contains("compile_retained_ui"));
}

#[test]
fn active_editor_ui_tree_contains_no_deleted_source_files() {
    assert_no_files_with_extension(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("ui"),
        &["sli", "nt"].concat(),
    );
}

#[test]
fn packaged_ui_asset_roots_contain_only_v2_schema_files() {
    let editor_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("assets/ui");
    let runtime_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("zircon_editor lives directly under workspace root")
        .join("zircon_runtime/assets/ui");

    assert_no_legacy_ui_toml(editor_root);
    assert_no_legacy_ui_toml(runtime_root);
}

#[test]
fn host_template_assets_are_toml_authority_for_editor_shells() {
    let assets: &[(&str, &[&str])] = &[
        (
            "assets/ui/editor/host/activity_drawer_window.v2.ui.toml",
            &[
                "ActivityDrawerWindowRoot",
                "ActivityDrawerWindowContentSlot",
            ],
        ),
        (
            "assets/ui/editor/host/workbench_shell.v2.ui.toml",
            &["UiHostWindow", "activity_rail", "document_host", "menu_bar"],
        ),
        (
            "assets/ui/editor/host/workbench_drawer_source.v2.ui.toml",
            &[
                "WorkbenchDrawerSource",
                "BottomDrawerHeaderRoot",
                "LeftDrawerPanelRoot",
            ],
        ),
        (
            "assets/ui/editor/host/floating_window_source.v2.ui.toml",
            &["FloatingWindowSourceRoot", "FloatingWindowTopBarRoot"],
        ),
        (
            "assets/ui/editor/host/scene_viewport_toolbar.v2.ui.toml",
            &["SceneViewportToolbarRoot", "FrameSelection"],
        ),
        (
            "assets/ui/editor/host/asset_surface_controls.v2.ui.toml",
            &["AssetSurfaceControls", "OpenAssetBrowser"],
        ),
        (
            "assets/ui/editor/host/inspector_surface_controls.v2.ui.toml",
            &["InspectorSurfaceControls", "DeleteSelected"],
        ),
        (
            "assets/ui/editor/host/startup_welcome_controls.v2.ui.toml",
            &["CreateProject", "OpenExistingProject"],
        ),
    ];

    for (relative, markers) in assets {
        let asset = source(relative);
        for &marker in *markers {
            assert!(asset.contains(marker), "{relative} missing `{marker}`");
        }
    }
}

#[test]
fn editor_v2_replacement_assets_do_not_keep_same_name_v1_sources() {
    for relative in [
        "assets/ui/editor/animation_editor.ui.toml",
        "assets/ui/editor/assets_activity.ui.toml",
        "assets/ui/editor/component_showcase.ui.toml",
        "assets/ui/editor/console.ui.toml",
        "assets/ui/editor/hierarchy.ui.toml",
        "assets/ui/editor/inspector.ui.toml",
        "assets/ui/editor/project_overview.ui.toml",
        "assets/ui/editor/welcome.ui.toml",
        "assets/ui/editor/host/activity_drawer_window.ui.toml",
        "assets/ui/editor/host/animation_graph_body.ui.toml",
        "assets/ui/editor/host/animation_sequence_body.ui.toml",
        "assets/ui/editor/host/asset_surface_controls.ui.toml",
        "assets/ui/editor/host/build_export_desktop_body.ui.toml",
        "assets/ui/editor/host/console_body.ui.toml",
        "assets/ui/editor/host/editor_main_frame.ui.toml",
        "assets/ui/editor/host/floating_window_source.ui.toml",
        "assets/ui/editor/host/hierarchy_body.ui.toml",
        "assets/ui/editor/host/inspector_body.ui.toml",
        "assets/ui/editor/host/inspector_surface_controls.ui.toml",
        "assets/ui/editor/host/module_plugins_body.ui.toml",
        "assets/ui/editor/host/pane_surface_controls.ui.toml",
        "assets/ui/editor/host/runtime_diagnostics_body.ui.toml",
        "assets/ui/editor/host/scene_viewport_toolbar.ui.toml",
        "assets/ui/editor/host/startup_welcome_controls.ui.toml",
        "assets/ui/editor/host/workbench_drawer_source.ui.toml",
        "assets/ui/editor/host/workbench_shell.ui.toml",
        "assets/ui/editor/windows/asset_window.ui.toml",
        "assets/ui/editor/windows/ui_layout_editor_window.ui.toml",
        "assets/ui/editor/windows/workbench_window.ui.toml",
        "assets/ui/editor/workbench_activity_rail.ui.toml",
        "assets/ui/editor/workbench_dock_header.ui.toml",
        "assets/ui/editor/workbench_menu_chrome.ui.toml",
        "assets/ui/editor/workbench_menu_popup.ui.toml",
        "assets/ui/editor/workbench_page_chrome.ui.toml",
        "assets/ui/editor/workbench_status_bar.ui.toml",
    ] {
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(relative);
        assert!(
            !path.exists(),
            "v2-replaced editor production asset must stay deleted: {relative}"
        );
    }
}

#[test]
fn critical_editor_shells_are_hard_cut_to_v2_assets() {
    let registry = source("src/ui/template_runtime/builtin/template_documents.rs");
    let runtime_host = source("src/ui/template_runtime/runtime/runtime_host.rs");
    for required in [
        "editor_main_frame.v2.ui.toml",
        "activity_drawer_window.v2.ui.toml",
        "workbench_window.v2.ui.toml",
        "asset_window.v2.ui.toml",
        "ui_layout_editor_window.v2.ui.toml",
        "component_showcase.v2.ui.toml",
        "material_demo_window.v2.ui.toml",
        "material_component_lab.v2.ui.toml",
        "workbench_shell.v2.ui.toml",
        "workbench_drawer_source.v2.ui.toml",
        "floating_window_source.v2.ui.toml",
        "scene_viewport_toolbar.v2.ui.toml",
        "console_body.v2.ui.toml",
        "inspector_body.v2.ui.toml",
        "hierarchy_body.v2.ui.toml",
        "animation_sequence_body.v2.ui.toml",
        "animation_graph_body.v2.ui.toml",
        "runtime_diagnostics_body.v2.ui.toml",
        "performance_timeline_body.v2.ui.toml",
        "module_plugins_body.v2.ui.toml",
        "build_export_desktop_body.v2.ui.toml",
        "asset_surface_controls.v2.ui.toml",
        "startup_welcome_controls.v2.ui.toml",
        "inspector_surface_controls.v2.ui.toml",
        "pane_surface_controls.v2.ui.toml",
    ] {
        assert!(
            registry.contains(required),
            "builtin template registry missing v2 asset `{required}`"
        );
    }
    for forbidden in [
        "editor_main_frame.ui.toml",
        "activity_drawer_window.ui.toml",
        "workbench_window.ui.toml",
        "asset_window.ui.toml",
        "ui_layout_editor_window.ui.toml",
        "component_showcase.ui.toml",
        "material_demo_window.ui.toml",
        "workbench_shell.ui.toml",
        "workbench_drawer_source.ui.toml",
        "floating_window_source.ui.toml",
        "scene_viewport_toolbar.ui.toml",
        "console_body.ui.toml",
        "inspector_body.ui.toml",
        "hierarchy_body.ui.toml",
        "animation_sequence_body.ui.toml",
        "animation_graph_body.ui.toml",
        "runtime_diagnostics_body.ui.toml",
        "module_plugins_body.ui.toml",
        "build_export_desktop_body.ui.toml",
        "asset_surface_controls.ui.toml",
        "startup_welcome_controls.ui.toml",
        "inspector_surface_controls.ui.toml",
        "pane_surface_controls.ui.toml",
    ] {
        assert!(
            !registry.contains(&format!("\"{forbidden}\"")),
            "builtin template registry should not route critical shell through old asset `{forbidden}`"
        );
    }

    for required in [
        "UiV2PrototypeStoreFileCache",
        "v2_template_file_cache()",
        ".load_store(std::iter::once(path.as_ref().to_path_buf()))",
        "Arc<UiV2AssetDocument>",
        "Arc<UiV2CompiledDocument>",
    ] {
        assert!(
            runtime_host.contains(required),
            "builtin v2 host runtime should use heap-resident file cache marker `{required}`"
        );
    }
    for forbidden in [
        "UiV2AssetLoader",
        "UiV2DocumentCompiler",
        "v2_prototype_store",
    ] {
        assert!(
            !runtime_host.contains(forbidden),
            "builtin v2 host runtime should not keep per-registration deserialize/compile marker `{forbidden}`"
        );
    }

    let view_projection = source("src/ui/layouts/views/view_projection.rs");
    for required in [
        "LegacyAssetPath",
        "UiV2PrototypeStoreFileCache",
        "UiV2SurfaceBuilder",
    ] {
        assert!(
            view_projection.contains(required),
            "editor view projection should expose v2 hard-cut marker `{required}`"
        );
    }
    for forbidden in [
        "UiTemplateSurfaceBuilder",
        "UiPrototypeStoreFileCache",
        "UiDocumentCompiler",
        "build_view_template_nodes_from_prototype_store",
    ] {
        assert!(
            !view_projection.contains(forbidden),
            "editor view projection should not keep old schema fallback marker `{forbidden}`"
        );
    }

    let asset_browser = source("src/ui/layouts/views/asset_browser.rs");
    assert!(asset_browser.contains("asset_browser.v2.ui.toml"));
    assert!(!asset_browser.contains("\"/assets/ui/editor/asset_browser.ui.toml\""));

    for (relative, required, forbidden) in [
        (
            "src/ui/layouts/views/console.rs",
            "console.v2.ui.toml",
            "\"/assets/ui/editor/console.ui.toml\"",
        ),
        (
            "src/ui/layouts/views/hierarchy.rs",
            "hierarchy.v2.ui.toml",
            "\"/assets/ui/editor/hierarchy.ui.toml\"",
        ),
        (
            "src/ui/layouts/views/inspector.rs",
            "inspector.v2.ui.toml",
            "\"/assets/ui/editor/inspector.ui.toml\"",
        ),
        (
            "src/ui/layouts/views/assets_activity.rs",
            "assets_activity.v2.ui.toml",
            "\"/assets/ui/editor/assets_activity.ui.toml\"",
        ),
        (
            "src/ui/layouts/views/animation_editor.rs",
            "animation_editor.v2.ui.toml",
            "\"/assets/ui/editor/animation_editor.ui.toml\"",
        ),
        (
            "src/ui/layouts/views/welcome.rs",
            "welcome.v2.ui.toml",
            "\"/assets/ui/editor/welcome.ui.toml\"",
        ),
    ] {
        let source = source(relative);
        assert!(
            source.contains(required),
            "{relative} should route projection through v2 asset `{required}`"
        );
        assert!(
            !source.contains(forbidden),
            "{relative} should not route projection through old asset `{forbidden}`"
        );
    }

    let ui_asset_editor_projection = source("src/ui/asset_editor/node_projection.rs");
    assert!(
        !Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("assets/ui/editor/ui_asset_editor.ui.toml")
            .exists(),
        "UI Asset Editor bootstrap must stay on the v2 authoring asset"
    );
    for required in [
        "ui_asset_editor.v2.ui.toml",
        "UiV2PrototypeStoreFileCache",
        "node_projection_v2_store_file_cache()",
        "UiV2SurfaceBuilder::build_surface_from_compiled_document",
        "extract_ui_render_tree",
    ] {
        assert!(
            ui_asset_editor_projection.contains(required),
            "UI Asset Editor node projection should route through v2 marker `{required}`"
        );
    }
    for forbidden in [
        "\"/assets/ui/editor/ui_asset_editor.ui.toml\"",
        "EditorTemplateRuntimeService",
        "UiCompiledDocument",
        "load_document_file",
        "compile_document_with_import_maps",
    ] {
        assert!(
            !ui_asset_editor_projection.contains(forbidden),
            "UI Asset Editor node projection should not keep old recursive projection marker `{forbidden}`"
        );
    }
}

#[test]
fn welcome_startup_demo_routes_to_component_showcase_window() {
    let welcome_asset = source("assets/ui/editor/welcome.v2.ui.toml");
    assert!(welcome_asset.contains("text = \"Component Showcase\""));
    assert!(welcome_asset.contains("route = \"Welcome.OpenStartupDemo\""));

    let welcome_session = source("src/ui/retained_host/app/welcome_session.rs");
    assert!(welcome_session.contains("OpenStartupDemo"));
    assert!(welcome_session.contains("editor.ui_component_showcase"));
    assert!(!welcome_session.contains("\"editor.material_demo_window\""));
}

#[test]
fn component_showcase_imported_zui_components_are_single_component_assets() {
    for (relative, component) in [
        (
            "assets/ui/editor/components/showcase_command_toolbar.zui",
            "ShowcaseCommandToolbar",
        ),
        (
            "assets/ui/editor/components/showcase_bottom_log.zui",
            "ShowcaseBottomLog",
        ),
        (
            "assets/ui/editor/components/showcase_category_nav.zui",
            "ShowcaseCategoryNav",
        ),
        (
            "assets/ui/editor/components/showcase_state_panel.zui",
            "ShowcaseStatePanel",
        ),
        (
            "assets/ui/editor/components/showcase_visual_section.zui",
            "ShowcaseVisualSection",
        ),
        (
            "assets/ui/editor/components/showcase_input_section.zui",
            "ShowcaseInputSection",
        ),
        (
            "assets/ui/editor/components/showcase_selection_section.zui",
            "ShowcaseSelectionSection",
        ),
        (
            "assets/ui/editor/components/showcase_collections_section.zui",
            "ShowcaseCollectionsSection",
        ),
    ] {
        let document = UiZuiAssetLoader::load_zui_str(&source(relative))
            .unwrap_or_else(|error| panic!("{relative} should load as .zui: {error}"));
        assert!(
            document.components.contains_key(component),
            "{relative} should declare `{component}`"
        );
        assert_eq!(
            document.components.len(),
            1,
            "{relative} should stay a single component prototype"
        );
    }

    let nav = UiZuiAssetLoader::load_zui_str(&source(
        "assets/ui/editor/components/showcase_category_nav.zui",
    ))
    .expect("showcase category nav should load as .zui");
    for control_id in [
        "ComponentCategoryNav",
        "ShowAllCategory",
        "ShowVisualCategory",
        "ShowFeedbackCategory",
        "ShowInputCategory",
        "ShowNumericCategory",
        "ShowSelectionCategory",
        "ShowReferenceCategory",
        "ShowDataCategory",
    ] {
        assert!(
            nav.nodes
                .values()
                .any(|node| node.control_id.as_deref() == Some(control_id)),
            "showcase category nav .zui should preserve `{control_id}`"
        );
    }
    assert!(
        nav.nodes.values().any(|node| {
            node.events
                .iter()
                .any(|event| event.id == "UiComponentShowcase/ShowAllCategory")
        }),
        "showcase category nav .zui should preserve category click bindings"
    );

    let state_panel = UiZuiAssetLoader::load_zui_str(&source(
        "assets/ui/editor/components/showcase_state_panel.zui",
    ))
    .expect("showcase state panel should load as .zui");
    for control_id in [
        "ComponentShowcaseStatePanel",
        "ComponentShowcaseStateTitle",
        "ComponentShowcaseSelectedCategory",
        "ComponentShowcaseLastControl",
        "ComponentShowcaseLastAction",
        "ComponentShowcaseCurrentValue",
        "ComponentShowcaseValidation",
        "ComponentShowcaseDragPayload",
        "ComponentShowcaseEventLog",
    ] {
        assert!(
            state_panel
                .nodes
                .values()
                .any(|node| node.control_id.as_deref() == Some(control_id)),
            "showcase state panel .zui should preserve `{control_id}`"
        );
    }

    let visual_section = UiZuiAssetLoader::load_zui_str(&source(
        "assets/ui/editor/components/showcase_visual_section.zui",
    ))
    .expect("showcase visual section should load as .zui");
    for control_id in [
        "ComponentShowcaseVisualSectionTitle",
        "LabelDemo",
        "RichLabelDemo",
        "ImageDemo",
        "IconDemo",
        "SvgIconDemo",
        "SeparatorDemo",
        "ProgressBarDemo",
        "SpinnerDemo",
        "BadgeDemo",
        "HelpRowDemo",
    ] {
        assert!(
            visual_section
                .nodes
                .values()
                .any(|node| node.control_id.as_deref() == Some(control_id)),
            "showcase visual section .zui should preserve `{control_id}`"
        );
    }

    let input_section = UiZuiAssetLoader::load_zui_str(&source(
        "assets/ui/editor/components/showcase_input_section.zui",
    ))
    .expect("showcase input section should load as .zui");
    for control_id in [
        "ComponentShowcaseInputSectionTitle",
        "ButtonDemo",
        "IconButtonDemo",
        "ToggleButtonDemo",
        "CheckboxDemo",
        "RadioDemo",
        "SegmentedControlDemo",
        "InputFieldDemo",
        "TextFieldDemo",
        "NumberFieldDemo",
        "RangeFieldDemo",
        "ColorFieldDemo",
        "Vector2FieldDemo",
        "Vector3FieldDemo",
        "Vector4FieldDemo",
    ] {
        assert!(
            input_section
                .nodes
                .values()
                .any(|node| node.control_id.as_deref() == Some(control_id)),
            "showcase input section .zui should preserve `{control_id}`"
        );
    }
    for event_id in [
        "UiComponentShowcase/ButtonCommit",
        "UiComponentShowcase/InputFieldCommitted",
        "UiComponentShowcase/NumberFieldDragUpdate",
        "UiComponentShowcase/RangeFieldChanged",
        "UiComponentShowcase/Vector4FieldChanged",
    ] {
        assert!(
            input_section
                .nodes
                .values()
                .any(|node| node.events.iter().any(|event| event.id == event_id)),
            "showcase input section .zui should preserve `{event_id}`"
        );
    }

    let selection_section = UiZuiAssetLoader::load_zui_str(&source(
        "assets/ui/editor/components/showcase_selection_section.zui",
    ))
    .expect("showcase selection section should load as .zui");
    for control_id in [
        "ComponentShowcaseSelectionSectionTitle",
        "DropdownDemo",
        "ComboBoxDemo",
        "EnumFieldDemo",
        "FlagsFieldDemo",
        "SearchSelectDemo",
        "AssetFieldDemo",
        "InstanceFieldDemo",
        "ObjectFieldDemo",
    ] {
        assert!(
            selection_section
                .nodes
                .values()
                .any(|node| node.control_id.as_deref() == Some(control_id)),
            "showcase selection section .zui should preserve `{control_id}`"
        );
    }
    for event_id in [
        "UiComponentShowcase/DropdownChanged",
        "UiComponentShowcase/SearchSelectQueryChanged",
        "UiComponentShowcase/AssetFieldDropped",
        "UiComponentShowcase/AssetFieldClear",
        "UiComponentShowcase/ObjectFieldClear",
    ] {
        assert!(
            selection_section
                .nodes
                .values()
                .any(|node| node.events.iter().any(|event| event.id == event_id)),
            "showcase selection section .zui should preserve `{event_id}`"
        );
    }

    let collections_section = UiZuiAssetLoader::load_zui_str(&source(
        "assets/ui/editor/components/showcase_collections_section.zui",
    ))
    .expect("showcase collections section should load as .zui");
    for control_id in [
        "ComponentShowcaseCollectionsSectionTitle",
        "GroupDemo",
        "FoldoutDemo",
        "PropertyRowDemo",
        "InspectorSectionDemo",
        "ArrayFieldDemo",
        "MapFieldDemo",
        "ListRowDemo",
        "TableRowDemo",
        "VirtualListDemo",
        "PagedListDemo",
        "WorldSpaceSurfaceDemo",
        "TreeRowDemo",
        "ContextActionMenuDemo",
    ] {
        assert!(
            collections_section
                .nodes
                .values()
                .any(|node| node.control_id.as_deref() == Some(control_id)),
            "showcase collections section .zui should preserve `{control_id}`"
        );
    }
    for event_id in [
        "UiComponentShowcase/GroupToggled",
        "UiComponentShowcase/FoldoutToggled",
        "UiComponentShowcase/InspectorSectionToggled",
        "UiComponentShowcase/ArrayFieldAddElement",
        "UiComponentShowcase/MapFieldSetEntry",
        "UiComponentShowcase/ListRowClicked",
        "UiComponentShowcase/VirtualListScrolled",
        "UiComponentShowcase/PagedListNextPage",
        "UiComponentShowcase/WorldSpaceSurfaceMoved",
        "UiComponentShowcase/TreeRowToggled",
        "UiComponentShowcase/ContextActionMenuOpenAt",
    ] {
        assert!(
            collections_section
                .nodes
                .values()
                .any(|node| node.events.iter().any(|event| event.id == event_id)),
            "showcase collections section .zui should preserve `{event_id}`"
        );
    }
}

#[test]
fn editor_v2_projection_collectors_use_explicit_stacks() {
    let projection = source("src/ui/template_runtime/runtime/projection.rs");

    for required in [
        "let mut stack = vec![V2ProjectionFrame::Enter(root)]",
        "struct HostProjectionFrame",
        "let mut stack = vec![HostProjectionFrame",
        "let mut stack = vec![node_id]",
        "node_bindings_from_ids(",
    ] {
        assert!(
            projection.contains(required),
            "projection.rs missing explicit-stack marker `{required}`"
        );
    }
    for forbidden in [
        "collect_host_nodes(child,",
        "collect_surface_host_nodes(tree, *child_id",
    ] {
        assert!(
            !projection.contains(forbidden),
            "projection.rs should not recurse through host collectors via `{forbidden}`"
        );
    }
}

#[test]
fn pointer_handlers_do_not_force_slow_path_rebuilds() {
    for relative in [
        "src/ui/retained_host/app/asset_content_pointer.rs",
        "src/ui/retained_host/app/asset_reference_pointer.rs",
        "src/ui/retained_host/app/asset_tree_pointer.rs",
        "src/ui/retained_host/app/detail_scroll_pointer.rs",
        "src/ui/retained_host/app/hierarchy_pointer.rs",
        "src/ui/retained_host/app/menu_pointer.rs",
        "src/ui/retained_host/app/viewport.rs",
        "src/ui/retained_host/app/welcome_recent_pointer.rs",
        "src/ui/retained_host/app/workbench_pointer.rs",
        "src/ui/retained_host/app/workspace_docking.rs",
    ] {
        let source = source(relative);
        assert!(
            source.contains("use_committed_pointer_layout("),
            "{relative} should route pointer events against the last committed layout"
        );
        assert!(
            !source.contains("recompute_if_dirty("),
            "{relative} should not rebuild the editor UI inside pointer callbacks"
        );
        assert!(
            !source.contains("chrome_snapshot("),
            "{relative} should use committed retained-host caches instead of rebuilding chrome snapshots"
        );
    }

    let lifecycle = source("src/ui/retained_host/app/host_lifecycle.rs");
    assert!(
        lifecycle.contains("fn use_committed_pointer_layout(&self)"),
        "host lifecycle should expose the cached pointer-layout entrypoint"
    );
}

#[test]
fn host_dirty_flags_route_through_invalidation_root_outside_lifecycle_owner() {
    let lifecycle = source("src/ui/retained_host/app/host_lifecycle.rs");
    for required in [
        "fn mark_presentation_dirty(&mut self)",
        "HostInvalidationMask::PRESENTATION_DATA",
    ] {
        assert!(
            lifecycle.contains(required),
            "host lifecycle missing invalidation entrypoint marker `{required}`"
        );
    }

    let app_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/ui/retained_host/app");
    for path in collect_rust_files(&app_root) {
        if path.file_name().and_then(|value| value.to_str()) == Some("host_lifecycle.rs") {
            continue;
        }
        let text =
            fs::read_to_string(&path).unwrap_or_else(|error| panic!("read {:?}: {error}", path));
        for forbidden in [
            "self.presentation_dirty = true",
            "self.layout_dirty = true",
            "self.window_metrics_dirty = true",
            "self.render_dirty = true",
        ] {
            assert!(
                !text.contains(forbidden),
                "{:?} should route host dirty state through HostInvalidationRoot, not `{forbidden}`",
                path.file_name().expect("file name")
            );
        }
    }

    let ui_asset_editor = source("src/ui/retained_host/app/ui_asset_editor.rs");
    assert!(
        ui_asset_editor.contains("mark_presentation_dirty()"),
        "UI Asset Editor actions should mark presentation changes through dirty-domain invalidation"
    );
}

#[test]
fn retained_event_effects_route_dirty_domains_through_invalidation_mask() {
    let event_bridge = source("src/ui/retained_host/event_bridge.rs");
    for required in [
        "pub dirty_domains: HostInvalidationMask",
        "fn request_presentation(&mut self)",
        "fn request_layout(&mut self)",
        "fn request_render(&mut self)",
        "fn request_render_and_presentation(&mut self)",
        "fn request_paint_only(&mut self)",
        "fn dirty_domains(&self) -> HostInvalidationMask",
        "fn merge_dirty_domains(&mut self, dirty_domains: HostInvalidationMask)",
    ] {
        assert!(
            event_bridge.contains(required),
            "retained event bridge missing dirty-domain marker `{required}`"
        );
    }

    let lifecycle = source("src/ui/retained_host/app/host_lifecycle.rs");
    assert!(
        lifecycle.contains("self.invalidate_host(effects.dirty_domains())"),
        "host lifecycle should consume event effects through HostInvalidationMask"
    );

    let common_effects = source("src/ui/retained_host/callback_dispatch/common/effects.rs");
    assert!(
        common_effects.contains("target.merge_dirty_domains(source.dirty_domains())"),
        "callback effect merging should preserve dirty-domain provenance"
    );

    let drawer_resize = source("src/ui/retained_host/drawer_resize.rs");
    assert!(
        drawer_resize.contains("combined.merge_dirty_domains(effects.dirty_domains())"),
        "drawer resize dispatch should merge dirty domains instead of OR-ing legacy flags"
    );

    let retained_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/ui/retained_host");
    for path in collect_rust_files(&retained_root) {
        let file_name = path.file_name().and_then(|value| value.to_str());
        if matches!(file_name, Some("event_bridge.rs" | "host_lifecycle.rs")) {
            continue;
        }
        let text =
            fs::read_to_string(&path).unwrap_or_else(|error| panic!("read {:?}: {error}", path));
        for forbidden in [
            ".presentation_dirty |=",
            ".layout_dirty |=",
            ".render_dirty |=",
            ".presentation_dirty = true",
            ".layout_dirty = true",
            ".render_dirty = true",
        ] {
            assert!(
                !text.contains(forbidden),
                "{:?} should merge event dirty state through UiHostEventEffects dirty domains, not `{forbidden}`",
                path.file_name().expect("file name")
            );
        }
    }
}

#[test]
fn runtime_ui_golden_is_hard_cut_to_v2_fixtures() {
    let runtime_golden = source("src/tests/ui/boundary/runtime_ui_golden.rs");
    for required in [
        "UiV2PrototypeStoreFileCache",
        "UiV2SurfaceBuilder",
        "hud_overlay.v2.ui.toml",
        "pause_menu.v2.ui.toml",
        "settings_dialog.v2.ui.toml",
        "inventory_list.v2.ui.toml",
        "quest_log_dialog.v2.ui.toml",
    ] {
        assert!(
            runtime_golden.contains(required),
            "runtime UI golden should cover v2 fixture marker `{required}`"
        );
    }

    for forbidden in [
        "UiAssetLoader",
        "UiDocumentCompiler",
        "UiTemplateSurfaceBuilder",
        "build_legacy_surface",
        "runtime_hud.ui.toml",
        "pause_dialog.ui.toml",
        "settings_dialog.ui.toml",
        "inventory_dialog.ui.toml",
        "quest_log_dialog.ui.toml",
    ] {
        assert!(
            !runtime_golden.contains(forbidden),
            "runtime UI golden should not keep old runtime schema fallback `{forbidden}`"
        );
    }
}

#[test]
fn runtime_fixture_host_tests_are_hard_cut_to_v2_paths() {
    let pane_body_documents = source("src/tests/host/template_runtime/pane_body_documents.rs");
    let material_surface_assets = source("src/tests/ui/boundary/global_material_surface_assets.rs");
    let ui_asset_editor_preview = source("src/tests/ui/ui_asset_editor/runtime_previews.rs");
    let ui_asset_editor_support = source("src/tests/ui/ui_asset_editor/support.rs");

    for required in [
        "runtime_v2_fixture_path",
        "register_document_file",
        "runtime_v2_fixture_buttons_project_interactive_metadata",
        "UiV2AssetLoader",
        "runtime_v2_fixture_assets_parse_from_runtime_crate_assets",
        "from_v2_source",
        "open_v2_preview_session",
    ] {
        assert!(
            pane_body_documents.contains(required)
                || material_surface_assets.contains(required)
                || ui_asset_editor_preview.contains(required)
                || ui_asset_editor_support.contains(required),
            "runtime fixture host/material tests should keep v2 marker `{required}`"
        );
    }

    for forbidden in [
        "runtime_hud.ui.toml",
        "pause_dialog.ui.toml",
        "settings_dialog.ui.toml",
        "inventory_dialog.ui.toml",
        "quest_log_dialog.ui.toml",
    ] {
        assert!(
            !pane_body_documents.contains(forbidden),
            "host/template runtime tests should not keep old runtime schema asset `{forbidden}`"
        );
        assert!(
            !material_surface_assets.contains(forbidden),
            "global material surface tests should not keep old runtime schema asset `{forbidden}`"
        );
        assert!(
            !ui_asset_editor_preview.contains(forbidden),
            "ui asset editor runtime preview tests should not keep old runtime schema asset `{forbidden}`"
        );
        assert!(
            !ui_asset_editor_support.contains(forbidden),
            "ui asset editor support should not include old runtime schema asset `{forbidden}`"
        );
    }
}

#[test]
fn component_showcase_is_hard_cut_to_v2_catalog_components() {
    let searchable_assets = [
        "assets/ui/editor/component_showcase.v2.ui.toml",
        "assets/ui/editor/components/showcase_visual_section.zui",
        "assets/ui/editor/components/showcase_input_section.zui",
        "assets/ui/editor/components/showcase_selection_section.zui",
        "assets/ui/editor/components/showcase_collections_section.zui",
    ]
    .into_iter()
    .map(source)
    .collect::<Vec<_>>()
    .join("\n");

    for forbidden in [
        "material_meta_components.ui.toml",
        "component_widgets.ui.toml#ShowcaseSection",
        "component_ref",
        "kind = \"reference\"",
    ] {
        assert!(
            !searchable_assets.contains(forbidden),
            "component showcase v2 asset should not depend on old recursive `{forbidden}`"
        );
    }

    for required in [
        "component = \"ProgressBar\"",
        "component = \"Spinner\"",
        "component = \"Button\"",
        "component = \"IconButton\"",
        "component = \"ToggleButton\"",
        "component = \"Checkbox\"",
        "component = \"InputField\"",
        "component = \"TextField\"",
        "component = \"NumberField\"",
        "component = \"RangeField\"",
        "component = \"Dropdown\"",
        "component = \"ComboBox\"",
        "component = \"EnumField\"",
        "component = \"FlagsField\"",
        "component = \"SearchSelect\"",
        "component = \"AssetField\"",
        "component = \"InstanceField\"",
        "component = \"ObjectField\"",
        "component = \"Group\"",
        "component = \"Foldout\"",
        "component = \"PropertyRow\"",
        "component = \"InspectorSection\"",
        "component = \"ArrayField\"",
        "component = \"MapField\"",
        "component = \"ListRow\"",
        "component = \"TableRow\"",
        "component = \"VirtualList\"",
        "component = \"PagedList\"",
        "component = \"WorldSpaceSurface\"",
        "component = \"TreeRow\"",
        "component = \"ContextActionMenu\"",
        "res://ui/theme/editor_material.v2.ui.toml",
        "res://ui/editor/components/showcase_command_toolbar.zui#ShowcaseCommandToolbar",
        "res://ui/editor/components/showcase_bottom_log.zui#ShowcaseBottomLog",
        "res://ui/editor/components/showcase_category_nav.zui#ShowcaseCategoryNav",
        "res://ui/editor/components/showcase_state_panel.zui#ShowcaseStatePanel",
        "res://ui/editor/components/showcase_visual_section.zui#ShowcaseVisualSection",
        "res://ui/editor/components/showcase_input_section.zui#ShowcaseInputSection",
        "res://ui/editor/components/showcase_selection_section.zui#ShowcaseSelectionSection",
        "res://ui/editor/components/showcase_collections_section.zui#ShowcaseCollectionsSection",
        "component = \"ShowcaseCommandToolbar\"",
        "component = \"ShowcaseBottomLog\"",
        "component = \"ShowcaseCategoryNav\"",
        "component = \"ShowcaseStatePanel\"",
        "component = \"ShowcaseVisualSection\"",
        "component = \"ShowcaseInputSection\"",
        "component = \"ShowcaseSelectionSection\"",
        "component = \"ShowcaseCollectionsSection\"",
    ] {
        assert!(
            searchable_assets.contains(required),
            "component showcase v2 asset missing `{required}`"
        );
    }
}

#[test]
fn material_meta_components_cover_retained_material_exports() {
    let asset = source("src/tests/fixtures/ui_legacy/editor/material_meta_components.ui.toml");
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
            "material_meta_components.ui.toml missing Retained Material export `{component}`"
        );
    }
}

#[test]
fn rust_owned_template_node_contract_keeps_retained_widget_state() {
    let template_nodes = source("src/ui/retained_host/host_contract/data/template_nodes.rs");

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
