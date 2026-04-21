use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::core::editor_event::{EditorAssetEvent, EditorEvent, EditorEventSource};
use crate::tests::editor_event::support::{env_lock, EventRuntimeHarness};
use crate::ui::host::module::EDITOR_MANAGER_NAME;
use crate::ui::host::EditorManager;
use crate::ui::workbench::view::ViewDescriptorId;
use zircon_runtime::asset::assets::{
    AnimationChannelAsset, AnimationChannelKeyAsset, AnimationChannelValueAsset,
    AnimationInterpolationAsset, AnimationSequenceAsset, AnimationSequenceBindingAsset,
    AnimationSequenceTrackAsset,
};
use zircon_runtime::core::framework::scene::{ComponentPropertyPath, EntityPath};
use zircon_runtime::ui::{
    binding::UiBindingValue, event_ui::UiControlRequest, event_ui::UiControlResponse,
    event_ui::UiNodePath,
};

fn unique_temp_path(prefix: &str, extension: &str) -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!("{prefix}_{unique}{extension}"))
}

fn scalar_channel(value: f32) -> AnimationChannelAsset {
    AnimationChannelAsset {
        interpolation: AnimationInterpolationAsset::Step,
        keys: vec![AnimationChannelKeyAsset {
            time_seconds: 0.0,
            value: AnimationChannelValueAsset::Scalar(value),
            in_tangent: None,
            out_tangent: None,
        }],
    }
}

fn write_sequence_asset(path: &Path) {
    let asset = AnimationSequenceAsset {
        name: Some("Reflection Sequence".to_string()),
        duration_seconds: 2.0,
        frames_per_second: 30.0,
        bindings: vec![AnimationSequenceBindingAsset {
            entity_path: EntityPath::parse("Root/Hero").unwrap(),
            tracks: vec![AnimationSequenceTrackAsset {
                property_path: ComponentPropertyPath::parse("Transform.translation").unwrap(),
                channel: scalar_channel(1.0),
            }],
        }],
    };
    fs::write(path, asset.to_bytes().unwrap()).unwrap();
}

#[test]
fn workbench_reflection_call_action_dispatches_docking_inspector_and_viewport_actions() {
    let _guard = env_lock().lock().unwrap();
    let runtime = EventRuntimeHarness::new("zircon_workbench_reflection_runtime");

    let inspector = runtime
        .runtime
        .handle_control_request(UiControlRequest::CallAction {
            node_path: UiNodePath::new("editor/workbench/drawers/right_top/editor.inspector#1"),
            action_id: "apply_batch".to_string(),
            arguments: vec![
                UiBindingValue::string("entity://selected"),
                UiBindingValue::array(vec![
                    UiBindingValue::array(vec![
                        UiBindingValue::string("name"),
                        UiBindingValue::string("Bound Cube"),
                    ]),
                    UiBindingValue::array(vec![
                        UiBindingValue::string("transform.translation.x"),
                        UiBindingValue::Float(4.0),
                    ]),
                ]),
            ],
        });
    assert!(matches!(
        inspector,
        UiControlResponse::Invocation(result)
            if result.error.is_none() && result.value.is_some()
    ));
    let editor_snapshot = runtime.runtime.editor_snapshot();
    assert_eq!(
        editor_snapshot
            .inspector
            .as_ref()
            .map(|inspector| inspector.name.as_str()),
        Some("Bound Cube")
    );
    assert_eq!(
        editor_snapshot
            .inspector
            .as_ref()
            .map(|inspector| inspector.translation[0].as_str()),
        Some("4.00")
    );

    let viewport = runtime
        .runtime
        .handle_control_request(UiControlRequest::CallAction {
            node_path: UiNodePath::new("editor/workbench/pages/workbench/editor.scene#1"),
            action_id: "resize".to_string(),
            arguments: vec![
                UiBindingValue::Unsigned(1024),
                UiBindingValue::Unsigned(768),
            ],
        });
    assert!(matches!(
        viewport,
        UiControlResponse::Invocation(result)
            if result.error.is_none() && result.value.is_some()
    ));
    assert_eq!(
        runtime.runtime.editor_snapshot().viewport_size,
        zircon_runtime::core::math::UVec2::new(1024, 768)
    );

    let docking = runtime
        .runtime
        .handle_control_request(UiControlRequest::CallAction {
            node_path: UiNodePath::new("editor/workbench/pages/workbench/editor.scene#1"),
            action_id: "detach_to_window".to_string(),
            arguments: Vec::new(),
        });
    assert!(matches!(
        docking,
        UiControlResponse::Invocation(result)
            if result.error.is_none() && result.value.is_some()
    ));
    assert_eq!(runtime.runtime.current_layout().floating_windows.len(), 1);
}

#[test]
fn workbench_reflection_call_action_dispatches_typed_draft_actions() {
    let _guard = env_lock().lock().unwrap();
    let runtime = EventRuntimeHarness::new("zircon_workbench_reflection_draft_runtime");

    let inspector = runtime
        .runtime
        .handle_control_request(UiControlRequest::CallAction {
            node_path: UiNodePath::new("editor/workbench/drawers/right_top/editor.inspector#1"),
            action_id: "edit_field".to_string(),
            arguments: vec![
                UiBindingValue::string("entity://selected"),
                UiBindingValue::string("name"),
                UiBindingValue::string("Drafted Cube"),
            ],
        });
    assert!(matches!(
        inspector,
        UiControlResponse::Invocation(result)
            if result.error.is_none()
                && result.value.is_some()
                && result
                    .binding
                    .as_ref()
                    .map(|binding| binding.path.control_id.as_str())
                    == Some("NameField")
    ));
    assert_eq!(
        runtime
            .runtime
            .editor_snapshot()
            .inspector
            .as_ref()
            .map(|inspector| inspector.name.as_str()),
        Some("Drafted Cube")
    );

    let mesh_import = runtime
        .runtime
        .handle_control_request(UiControlRequest::CallAction {
            node_path: UiNodePath::new("editor/workbench/drawers/left_top/editor.assets#1"),
            action_id: "set_mesh_import_path".to_string(),
            arguments: vec![UiBindingValue::string("E:/Models/cube.glb")],
        });
    assert!(matches!(
        mesh_import,
        UiControlResponse::Invocation(result)
            if result.error.is_none() && result.value.is_some()
    ));
    assert_eq!(
        runtime.runtime.editor_snapshot().mesh_import_path,
        "E:/Models/cube.glb"
    );
}

#[test]
fn workbench_reflection_call_action_dispatches_asset_import_action() {
    let _guard = env_lock().lock().unwrap();
    let runtime = EventRuntimeHarness::new("zircon_workbench_reflection_asset_import_runtime");

    let import_model = runtime
        .runtime
        .handle_control_request(UiControlRequest::CallAction {
            node_path: UiNodePath::new("editor/workbench/drawers/left_top/editor.assets#1"),
            action_id: "import_model".to_string(),
            arguments: Vec::new(),
        });
    assert!(matches!(
        import_model,
        UiControlResponse::Invocation(result)
            if result.error.is_none() && result.value.is_some()
    ));
    assert_eq!(
        runtime
            .runtime
            .journal()
            .records()
            .last()
            .map(|record| &record.event),
        Some(&EditorEvent::Asset(EditorAssetEvent::ImportModel))
    );
}

#[test]
fn workbench_reflection_call_action_dispatches_animation_track_creation_from_inspector() {
    let _guard = env_lock().lock().unwrap();
    let runtime = EventRuntimeHarness::new("zircon_workbench_reflection_animation_track_runtime");
    let asset_path = unique_temp_path(
        "zircon_workbench_reflection_animation_track",
        ".sequence.zranim",
    );
    write_sequence_asset(&asset_path);

    runtime
        .runtime
        .dispatch_event(
            EditorEventSource::Headless,
            EditorEvent::Asset(EditorAssetEvent::OpenAsset {
                asset_path: asset_path.to_string_lossy().into_owned(),
            }),
        )
        .expect("open animation sequence asset");

    let response = runtime
        .runtime
        .handle_control_request(UiControlRequest::CallAction {
            node_path: UiNodePath::new("editor/workbench/drawers/right_top/editor.inspector#1"),
            action_id: "create_animation_track".to_string(),
            arguments: vec![UiBindingValue::string("Root/Hero:AnimationPlayer.weight")],
        });
    assert!(matches!(
        response,
        UiControlResponse::Invocation(result)
            if result.error.is_none() && result.value.is_some()
    ));

    let manager = runtime
        .core
        .resolve_manager::<EditorManager>(EDITOR_MANAGER_NAME)
        .unwrap();
    let instance = runtime
        .runtime
        .current_view_instances()
        .into_iter()
        .find(|instance| {
            instance.descriptor_id == ViewDescriptorId::new("editor.animation_sequence")
        })
        .expect("animation sequence view should remain open");
    let pane = manager
        .animation_editor_pane_presentation(&instance.instance_id)
        .expect("sequence session should be queryable after inspector action");

    assert!(pane
        .track_items
        .contains(&"Root/Hero:AnimationPlayer.weight".to_string()));
    assert_eq!(
        runtime.runtime.editor_snapshot().status_line,
        "Created animation track Root/Hero:AnimationPlayer.weight"
    );

    let _ = fs::remove_file(asset_path);
}
