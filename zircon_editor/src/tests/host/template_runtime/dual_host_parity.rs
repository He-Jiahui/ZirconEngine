use std::collections::BTreeSet;

use super::support::*;
use crate::ui::control::EditorUiControlService;
use crate::ui::template_runtime::{
    EditorUiHostRuntime, SlintUiHostValue, UiComponentShowcaseDemoEventInput,
};
use zircon_runtime_interface::ui::component::{UiComponentAdapterResult, UiValue};

const COMPONENT_SHOWCASE_DOCUMENT_ID: &str = "editor.window.ui_component_showcase";
const UI_ASSET_EDITOR_SHELL_DOCUMENT_ID: &str = "editor.ui_asset_editor";
const UI_ASSET_EDITOR_SHELL_TOML: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/assets/ui/editor/ui_asset_editor.ui.toml"
));
const WORKBENCH_WINDOW_DOCUMENT_ID: &str = "editor.window.workbench";

#[test]
fn dual_host_parity_preserves_layout_attributes_and_routes_for_representative_documents() {
    let _guard = crate::tests::support::env_lock()
        .lock()
        .unwrap_or_else(|poison| poison.into_inner());
    let mut runtime = EditorUiHostRuntime::default();
    runtime.load_builtin_host_templates().unwrap();
    runtime
        .register_document_source(
            UI_ASSET_EDITOR_SHELL_DOCUMENT_ID,
            UI_ASSET_EDITOR_SHELL_TOML,
        )
        .unwrap();

    for document_id in [
        UI_HOST_WINDOW_DOCUMENT_ID,
        UI_ASSET_EDITOR_SHELL_DOCUMENT_ID,
        COMPONENT_SHOWCASE_DOCUMENT_ID,
        WORKBENCH_WINDOW_DOCUMENT_ID,
    ] {
        let mut projection = runtime.project_document(document_id).unwrap();
        let mut service = EditorUiControlService::default();
        runtime
            .register_projection_routes(&mut service, &mut projection)
            .unwrap();

        let mut surface = runtime.build_shared_surface(document_id).unwrap();
        surface.compute_layout(UiSize::new(1280.0, 720.0)).unwrap();
        let host_model = runtime
            .build_host_model_with_surface(&projection, &surface)
            .unwrap();
        let slint_projection = runtime
            .build_slint_host_projection_with_surface(&projection, &surface)
            .unwrap();

        let surface_snapshot =
            EditorUiCompatibilityHarness::capture_shared_surface_snapshot(&surface);
        let host_snapshot = EditorUiCompatibilityHarness::capture_host_model_snapshot(&host_model);
        let slint_snapshot =
            EditorUiCompatibilityHarness::capture_slint_host_projection_snapshot(&slint_projection);
        let surface_frames = sorted(surface_snapshot.frame_entries);
        let surface_attributes =
            stable_attribute_entries(document_id, surface_snapshot.attribute_entries);
        let surface_style_tokens = sorted(surface_snapshot.style_token_entries);
        let surface_bindings = sorted(surface_snapshot.binding_ids);
        let host_frames = sorted(host_snapshot.frame_entries);
        let host_attributes =
            stable_attribute_entries(document_id, host_snapshot.attribute_entries);
        let host_style_tokens = sorted(host_snapshot.style_token_entries);
        let host_bindings = sorted(host_snapshot.binding_ids);
        let host_route_bindings = sorted(host_snapshot.route_bindings);
        let slint_frames = sorted(slint_snapshot.frame_entries);
        let slint_route_bindings = sorted(slint_snapshot.route_bindings);

        assert_sets_equal(
            &surface_frames,
            &host_frames,
            "{document_id} host model must consume the same arranged frames as UiSurfaceFrame",
        );
        assert_sets_equal(
            &surface_attributes,
            &host_attributes,
            "{document_id} host model must preserve runtime template attributes",
        );
        assert_sets_equal(
            &surface_style_tokens,
            &host_style_tokens,
            "{document_id} host model must preserve runtime style tokens",
        );
        assert_sets_equal(
            &surface_bindings,
            &host_bindings,
            "{document_id} host model route bindings must mirror runtime bindings",
        );
        assert_sets_equal(
            &host_frames,
            &slint_frames,
            "{document_id} Slint/native projection must retain host model frames",
        );
        assert_sets_equal(
            &host_route_bindings,
            &slint_route_bindings,
            "{document_id} Slint/native projection must retain registered host routes",
        );
    }
}

#[test]
fn dual_host_parity_routes_material_events_to_runtime_component_state() {
    let _guard = crate::tests::support::env_lock()
        .lock()
        .unwrap_or_else(|poison| poison.into_inner());
    let mut runtime = EditorUiHostRuntime::default();
    runtime.load_builtin_host_templates().unwrap();

    let mut projection = runtime
        .project_document(COMPONENT_SHOWCASE_DOCUMENT_ID)
        .unwrap();
    let mut service = EditorUiControlService::default();
    runtime
        .register_projection_routes(&mut service, &mut projection)
        .unwrap();
    let mut surface = runtime
        .build_shared_surface(COMPONENT_SHOWCASE_DOCUMENT_ID)
        .unwrap();
    surface.compute_layout(UiSize::new(1280.0, 720.0)).unwrap();
    let host_projection = runtime
        .build_slint_host_projection_with_surface(&projection, &surface)
        .unwrap();

    let input = host_projection
        .node_by_control_id("InputFieldDemo")
        .expect("component showcase should project InputFieldDemo");
    assert!(input.routes.iter().any(|route| {
        route.binding_id == "UiComponentShowcase/InputFieldChanged"
            && route.event_kind == UiEventKind::Change
            && route.route_id.is_some()
    }));
    assert!(input.routes.iter().any(|route| {
        route.binding_id == "UiComponentShowcase/InputFieldCommitted"
            && route.event_kind == UiEventKind::Submit
            && route.route_id.is_some()
    }));

    let button = host_projection
        .node_by_control_id("ButtonDemo")
        .expect("component showcase should project ButtonDemo");
    assert!(button.routes.iter().any(|route| {
        route.binding_id == "UiComponentShowcase/ButtonCommit"
            && route.event_kind == UiEventKind::Click
            && route.route_id.is_some()
    }));

    let array_field = host_projection
        .node_by_control_id("ArrayFieldDemo")
        .expect("component showcase should project ArrayFieldDemo");
    assert!(
        array_field
            .collection_items
            .iter()
            .any(|item| item.contains("UiComponentRef = NumberField")),
        "ArrayField collection slot rows should be generated for editor host projection"
    );
    let map_field = host_projection
        .node_by_control_id("MapFieldDemo")
        .expect("component showcase should project MapFieldDemo");
    assert!(
        map_field
            .collection_items
            .iter()
            .any(|item| item.contains("speed: String -> UiValue")),
        "MapField key/value slot rows should be generated for editor host projection"
    );
    let virtual_list = host_projection
        .node_by_control_id("VirtualListDemo")
        .expect("component showcase should project VirtualListDemo");
    assert_eq!(
        virtual_list.properties.get("viewport_count"),
        Some(&SlintUiHostValue::Integer(25))
    );
    assert_eq!(
        virtual_list.properties.get("item_extent"),
        Some(&SlintUiHostValue::Float(28.0))
    );
    let world_surface = host_projection
        .node_by_control_id("WorldSpaceSurfaceDemo")
        .expect("component showcase should project WorldSpaceSurfaceDemo");
    assert_eq!(
        world_surface.properties.get("pixels_per_meter"),
        Some(&SlintUiHostValue::Float(256.0))
    );
    assert_eq!(
        world_surface.properties.get("camera_target"),
        Some(&SlintUiHostValue::String("viewport-main".to_string()))
    );

    assert_changed(
        apply_showcase_binding(
            &mut runtime,
            "UiComponentShowcase/InputFieldChanged",
            UiComponentShowcaseDemoEventInput::Value(UiValue::String("live parity".to_string())),
        ),
        true,
    );
    assert_eq!(
        runtime
            .showcase_demo_state()
            .value_text("InputFieldDemo", "value")
            .as_deref(),
        Some("live parity")
    );

    assert_changed(
        apply_showcase_binding(
            &mut runtime,
            "UiComponentShowcase/InputFieldCommitted",
            UiComponentShowcaseDemoEventInput::Value(UiValue::String(
                "committed parity".to_string(),
            )),
        ),
        true,
    );
    assert_eq!(
        runtime
            .showcase_demo_state()
            .value_text("InputFieldDemo", "value")
            .as_deref(),
        Some("committed parity")
    );

    assert_changed(
        apply_showcase_binding(
            &mut runtime,
            "UiComponentShowcase/CheckboxChanged",
            UiComponentShowcaseDemoEventInput::Toggle(false),
        ),
        true,
    );
    assert_eq!(
        runtime
            .showcase_demo_state()
            .value_text("CheckboxDemo", "value")
            .as_deref(),
        Some("false")
    );

    let button_result = apply_showcase_binding(
        &mut runtime,
        "UiComponentShowcase/ButtonCommit",
        UiComponentShowcaseDemoEventInput::None,
    );
    assert_changed(button_result, true);
    assert_eq!(
        runtime
            .showcase_demo_state()
            .value_text("ButtonDemo", "value")
            .as_deref(),
        Some(""),
        "ButtonCommit should dispatch through the same runtime component state reducer"
    );
}

fn apply_showcase_binding(
    runtime: &mut EditorUiHostRuntime,
    binding_id: &str,
    input: UiComponentShowcaseDemoEventInput,
) -> UiComponentAdapterResult {
    let binding = runtime
        .project_document(COMPONENT_SHOWCASE_DOCUMENT_ID)
        .unwrap()
        .bindings
        .into_iter()
        .find(|binding| binding.binding_id == binding_id)
        .unwrap_or_else(|| panic!("missing showcase binding `{binding_id}`"))
        .binding;
    runtime
        .apply_showcase_demo_binding(&binding, input)
        .unwrap()
}

fn assert_changed(result: UiComponentAdapterResult, expected_refresh: bool) {
    assert!(result.changed);
    assert_eq!(result.refresh_projection, expected_refresh);
}

fn sorted(entries: Vec<String>) -> BTreeSet<String> {
    entries.into_iter().collect()
}

fn stable_attribute_entries(document_id: &str, entries: Vec<String>) -> BTreeSet<String> {
    entries
        .into_iter()
        .filter(|entry| {
            document_id != COMPONENT_SHOWCASE_DOCUMENT_ID || !is_showcase_state_overlay_key(entry)
        })
        .collect()
}

fn is_showcase_state_overlay_key(entry: &str) -> bool {
    let Some((path_and_key, _value)) = entry.split_once('=') else {
        return false;
    };
    let Some((_path, key)) = path_and_key.rsplit_once('.') else {
        return false;
    };
    matches!(
        key,
        "selected"
            | "selection_state"
            | "value_text"
            | "validation_level"
            | "validation_message"
            | "collection_items"
    )
}

fn assert_sets_equal(left: &BTreeSet<String>, right: &BTreeSet<String>, context: &str) {
    if left == right {
        return;
    }
    let missing = left.difference(right).take(12).cloned().collect::<Vec<_>>();
    let extra = right.difference(left).take(12).cloned().collect::<Vec<_>>();
    panic!(
        "{context}; left_count={}, right_count={}, missing_from_right={missing:?}, extra_in_right={extra:?}",
        left.len(),
        right.len()
    );
}
