//! Real Details imports must retain density widths and the existing editability contract.

use std::path::PathBuf;

use zircon_runtime::ui::{
    dispatch::{UiNavigationDispatcher, UiPointerDispatcher},
    v2::{UiV2PrototypeStoreFileCache, UiV2SurfaceBuilder},
};
use zircon_runtime_interface::ui::{
    binding::UiEventKind,
    dispatch::{UiDispatchDisposition, UiInputEvent, UiTextInputEvent},
    event_ui::UiTreeId,
    layout::UiSize,
    v2::UiV2AssetDocument,
};

use zircon_runtime_interface::math::{EulerRot, Quat};

use crate::tests::support::env_lock;
use crate::ui::{
    retained_host::callback_dispatch::template_bridge::BuiltinWorkbenchWindowTemplateSurfaceBridge,
    v2_design_tokens::prepare_editor_v2_document,
    workbench::snapshot::{InspectorSnapshot, SceneEntries},
};

#[test]
fn script_binding_controls_load_from_single_component_assets() {
    let _guard = env_lock().lock().unwrap();
    let assets = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets");
    let mut cache = UiV2PrototypeStoreFileCache::new();
    let loaded = cache
        .load_store([assets.join("ui/editor/unreal/level_editor_window.zui")])
        .expect("the actual Details imports must compile with single-component assets");
    let prepared = prepare_editor_v2_document(loaded.root_document.as_ref());
    let surface = UiV2SurfaceBuilder::build_surface_from_compiled_document(
        UiTreeId::new("details.script_binding.regression"),
        &prepared,
        loaded.compiled.as_ref(),
    )
    .expect("the imported script binding controls must mount in the retained surface");

    for (asset_id, component) in [
        (
            "res://ui/editor/unreal/components/script_binding_fields.zui",
            "InspectorScriptBindingFields",
        ),
        (
            "res://ui/editor/unreal/components/script_bindings.zui",
            "InspectorScriptsSection",
        ),
    ] {
        let physical_path = assets
            .join(asset_id.strip_prefix("res://").expect("resource asset"))
            .canonicalize()
            .expect("the actual script binding asset must exist");
        assert!(loaded.source_receipts.iter().any(|receipt| {
            receipt.asset_id == asset_id && receipt.physical_path == physical_path
        }));
        let document = loaded.store.get(asset_id).expect("imported script asset");
        assert_eq!(document.components.len(), 1);
        assert!(document.components.contains_key(component));

        for authored in document.nodes.values() {
            let control_id = authored.control_id.as_deref().expect("script control id");
            let mut mounted = surface.tree.nodes.values().filter_map(|node| {
                node.template_metadata
                    .as_ref()
                    .filter(|metadata| metadata.control_id.as_deref() == Some(control_id))
            });
            let metadata = mounted.next().unwrap_or_else(|| {
                panic!("{control_id} must mount through the actual Details imports")
            });
            assert!(mounted.next().is_none(), "duplicate control {control_id}");
            for event in &authored.events {
                assert!(
                    metadata.bindings.contains(event),
                    "{control_id} must retain its actual {} binding",
                    event.id
                );
            }
        }
    }
}

#[test]
fn unreal_details_axes_inherit_resolved_width_tokens_and_editability_contract() {
    let _guard = env_lock().lock().unwrap();
    let assets = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets");
    let mut cache = UiV2PrototypeStoreFileCache::new();
    let loaded = cache
        .load_store([assets.join("ui/editor/unreal/level_editor_window.zui")])
        .expect("the actual level-editor view and its imports must compile");
    assert_eq!(
        loaded.root_asset_id,
        "res://ui/editor/unreal/level_editor_window.zui"
    );

    let details_id = "res://ui/editor/unreal/components/details.zui";
    let axis_id = "res://ui/editor/workbench/primitives/chrome/axis_value_field.zui";
    for asset_id in [details_id, axis_id] {
        let physical_path = assets
            .join(asset_id.strip_prefix("res://").expect("resource asset"))
            .canonicalize()
            .expect("the actual imported asset must exist");
        assert!(
            loaded.source_receipts.iter().any(|receipt| {
                receipt.asset_id == asset_id && receipt.physical_path == physical_path
            }),
            "{asset_id} must be loaded from its actual production file"
        );
    }

    let details = loaded
        .store
        .get(details_id)
        .expect("actual Details document");
    let prepared = prepare_editor_v2_document(loaded.root_document.as_ref());
    let surface = UiV2SurfaceBuilder::build_surface_from_compiled_document(
        UiTreeId::new("details.inheritance.regression"),
        &prepared,
        loaded.compiled.as_ref(),
    )
    .expect("the actual compiled view must build a retained surface");

    let inspector = InspectorSnapshot {
        id: 2,
        name: "Available rotation".to_string(),
        parent: "World".to_string(),
        translation: ["0.0".to_string(), "0.0".to_string(), "0.0".to_string()],
        rotation_degrees: InspectorSnapshot::rotation_degrees_from_quaternion(Quat::from_euler(
            EulerRot::XYZ,
            30.0_f32.to_radians(),
            10.0_f32.to_radians(),
            (-20.0_f32).to_radians(),
        )),
        scale: ["1.0".to_string(), "1.0".to_string(), "1.0".to_string()],
        render_layer_mask: 1,
        native_fields: Vec::new(),
        script_bindings: Ok(Vec::new()),
        plugin_components: Vec::new(),
    };
    let degrees = inspector.rotation_degrees.as_ref().expect("valid rotation");
    let mut bridge = BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(1672.0, 941.0))
        .expect("actual builtin workbench bridge");
    bridge
        .sync_scene_and_inspector(&SceneEntries::default(), Some(&inspector))
        .expect("available rotation must synchronize through the actual bridge");
    // The bridge exposes its actual retained surface by shared reference.
    // Own that synchronized state to exercise the public runtime input consumer.
    let mut readonly_surface = bridge.surface().clone();
    let editable = readonly_surface
        .tree
        .nodes
        .values()
        .find(|node| {
            node.template_metadata
                .as_ref()
                .and_then(|metadata| metadata.control_id.as_deref())
                == details.nodes["position_x"].control_id.as_deref()
        })
        .expect("available editable position axis");
    for event_kind in [UiEventKind::Change, UiEventKind::Submit] {
        let binding = details.nodes["position_x"]
            .events
            .iter()
            .find(|binding| binding.event == event_kind)
            .expect("actual position binding");
        assert!(
            editable
                .template_metadata
                .as_ref()
                .expect("position metadata")
                .bindings
                .contains(binding),
            "the positive control must retain its full actual {event_kind:?} binding"
        );
    }
    let editable_id = editable.node_id;
    readonly_surface
        .focus_node(editable_id)
        .expect("the same real surface must accept an editable axis");
    assert_eq!(readonly_surface.focus.focused, Some(editable_id));
    readonly_surface.clear_focus();
    assert_eq!(readonly_surface.focus.focused, None);
    // Public focus release queues its legitimate Submit/IME lifecycle.
    // Consume it through the real input entry before checking rotation rejection.
    let focus_release = readonly_surface
        .dispatch_input_event(
            &UiPointerDispatcher::default(),
            &UiNavigationDispatcher::default(),
            UiInputEvent::Text(UiTextInputEvent {
                metadata: Default::default(),
                text: String::new(),
            }),
        )
        .expect("the positive control's focus lifecycle must dispatch");
    assert_eq!(
        focus_release.reply.disposition,
        UiDispatchDisposition::Unhandled
    );
    assert_eq!(focus_release.diagnostics.route_target, None);
    assert!(
        focus_release
            .component_events
            .iter()
            .all(|event| event.target == editable_id),
        "focus release must publish only the actual positive control's lifecycle"
    );
    assert_eq!(readonly_surface.focus.focused, None);

    for axis in [
        "position_x",
        "position_y",
        "position_z",
        "rotation_x",
        "rotation_y",
        "rotation_z",
        "scale_x",
        "scale_y",
        "scale_z",
    ] {
        let authored = details
            .nodes
            .get(axis)
            .unwrap_or_else(|| panic!("missing actual Details axis {axis}"));
        assert_eq!(authored.component, "WorkbenchAxisValueField");
        let control_id = authored
            .control_id
            .as_deref()
            .expect("axis control identity");
        let mut compiled_matches = loaded
            .compiled
            .arena
            .nodes
            .iter()
            .filter(|node| node.control_id.as_deref() == Some(control_id));
        let compiled = compiled_matches
            .next()
            .unwrap_or_else(|| panic!("missing compiled axis {control_id}"));
        assert!(
            compiled_matches.next().is_none(),
            "duplicate axis {control_id}"
        );
        assert_eq!(compiled.component, "InputField");
        let width = compiled
            .layout
            .as_ref()
            .and_then(|layout| layout.get("width"))
            .and_then(toml::Value::as_table)
            .expect("the compiled axis must inherit its primitive width");
        for field in ["min", "preferred", "max"] {
            let token = format!("$editor.density.axis_value_field.{field}_width");
            assert_eq!(
                width.get(field).and_then(toml::Value::as_str),
                Some(token.as_str()),
                "{control_id} must inherit the primitive's {field} token"
            );
        }

        let mut retained_matches = surface.tree.nodes.values().filter(|node| {
            node.template_metadata
                .as_ref()
                .and_then(|metadata| metadata.control_id.as_deref())
                == Some(control_id)
        });
        let retained = retained_matches
            .next()
            .unwrap_or_else(|| panic!("missing retained axis {control_id}"));
        assert!(
            retained_matches.next().is_none(),
            "duplicate retained axis {control_id}"
        );
        let metadata = retained.template_metadata.as_ref().expect("axis metadata");
        let width = retained.constraints.width;
        for (field, actual) in [
            ("min", width.min),
            ("preferred", width.preferred),
            ("max", width.max),
        ] {
            let token = format!("editor.density.axis_value_field.{field}_width");
            assert_eq!(
                actual,
                prepared_token_number(&prepared, &token),
                "{control_id} must consume the prepared {field} token in its real width constraint"
            );
        }

        if axis.starts_with("rotation_") {
            assert!(
                authored.events.is_empty(),
                "{control_id} must remain read-only"
            );
            assert!(
                compiled.events.is_empty(),
                "{control_id} gained a compiled route"
            );
            assert!(
                metadata.bindings.is_empty(),
                "{control_id} gained a retained route"
            );

            let axis_index = ["rotation_x", "rotation_y", "rotation_z"]
                .iter()
                .position(|rotation_axis| *rotation_axis == axis)
                .expect("rotation axis index");
            let expected_value = format!("{} deg", degrees[axis_index]);
            let projected = bridge
                .host_projection()
                .node_by_control_id(control_id)
                .expect("available rotation must remain projected");
            assert_eq!(
                projected.value_text.as_deref(),
                Some(expected_value.as_str())
            );
            assert!(
                projected.routes.is_empty(),
                "{control_id} must have no business route"
            );
            for event_kind in [UiEventKind::Change, UiEventKind::Submit] {
                assert!(
                    bridge
                        .dispatch_control_state(control_id, event_kind)
                        .expect("read-only dispatch must return a result")
                        .is_none(),
                    "{control_id} must reject actual bridge {event_kind:?} dispatch"
                );
            }
            let live = readonly_surface
                .tree
                .nodes
                .values()
                .find(|node| {
                    node.template_metadata
                        .as_ref()
                        .and_then(|metadata| metadata.control_id.as_deref())
                        == Some(control_id)
                })
                .expect("available rotation must remain in the actual retained tree");
            let live_metadata = live
                .template_metadata
                .as_ref()
                .expect("available axis metadata");
            assert!(
                live_metadata.bindings.is_empty(),
                "{control_id} must retain no route"
            );
            assert_eq!(
                live_metadata
                    .attributes
                    .get("value")
                    .and_then(toml::Value::as_str),
                Some(expected_value.as_str()),
                "{control_id} must consume the available rotation projection"
            );

            for (property, expected) in [
                ("disabled", true),
                ("editable_text", false),
                ("input_interactive", false),
                ("input_clickable", false),
                ("input_hoverable", false),
                ("input_focusable", false),
            ] {
                assert_eq!(
                    authored.props.get(property).and_then(toml::Value::as_bool),
                    Some(expected),
                    "{control_id} must retain its authored {property} contract"
                );
                assert_eq!(
                    compiled.props.get(property).and_then(toml::Value::as_bool),
                    Some(expected),
                    "{control_id} must retain its compiled {property} contract"
                );
                assert_eq!(
                    live_metadata
                        .attributes
                        .get(property)
                        .and_then(toml::Value::as_bool),
                    Some(expected),
                    "{control_id} must consume {property} in the retained surface"
                );
            }
            assert!(!live.is_focus_candidate(), "{control_id} must reject focus");
            assert!(
                !live.supports_pointer(),
                "{control_id} must reject pointer interaction"
            );
            let retained_id = live.node_id;
            let value_before = live_metadata.attributes.get("value").cloned();
            assert!(
                value_before.is_some(),
                "{control_id} must retain its display value"
            );
            assert_eq!(readonly_surface.focus.focused, None);
            assert!(
                readonly_surface.focus_node(retained_id).is_err(),
                "the real surface must reject focus on {control_id}"
            );
            assert_eq!(readonly_surface.focus.focused, None);
            let result = readonly_surface
                .dispatch_input_event(
                    &UiPointerDispatcher::default(),
                    &UiNavigationDispatcher::default(),
                    UiInputEvent::Text(UiTextInputEvent {
                        metadata: Default::default(),
                        text: "123".to_string(),
                    }),
                )
                .expect("a rejected text edit must return a dispatch result");
            assert_eq!(result.reply.disposition, UiDispatchDisposition::Unhandled);
            assert_eq!(result.diagnostics.route_target, None);
            assert!(result.applied_effects.is_empty());
            assert!(result.host_requests.is_empty());
            assert!(result.component_events.is_empty());
            assert!(result.widget_events.is_empty());
            assert!(result.binding_reports.is_empty());
            let metadata_after = readonly_surface
                .tree
                .node(retained_id)
                .expect("read-only axis must remain mounted")
                .template_metadata
                .as_ref()
                .expect("read-only axis metadata");
            assert_eq!(
                metadata_after.attributes.get("value"),
                value_before.as_ref()
            );
            assert!(metadata_after.bindings.is_empty());
            continue;
        }

        for event_kind in [UiEventKind::Change, UiEventKind::Submit] {
            let binding = authored
                .events
                .iter()
                .find(|binding| binding.event == event_kind)
                .unwrap_or_else(|| panic!("{control_id} lost its {event_kind:?} binding"));
            assert!(
                binding
                    .route
                    .as_deref()
                    .is_some_and(|route| !route.is_empty()),
                "{control_id} must retain its actual business route"
            );
            assert!(
                compiled.events.contains(binding),
                "{control_id} must preserve its actual {event_kind:?} binding in the compiled arena"
            );
            assert!(
                metadata.bindings.contains(binding),
                "{control_id} must preserve its actual {event_kind:?} binding in the retained surface"
            );
        }
    }
}

fn prepared_token_number(document: &UiV2AssetDocument, token: &str) -> f32 {
    match document
        .tokens
        .get(token)
        .unwrap_or_else(|| panic!("missing prepared editor token {token}"))
    {
        toml::Value::Float(value) => *value as f32,
        toml::Value::Integer(value) => *value as f32,
        value => panic!("{token} must resolve to a numeric density value, got {value:?}"),
    }
}
