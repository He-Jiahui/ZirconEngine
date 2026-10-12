use std::collections::BTreeSet;
use std::path::PathBuf;

use zircon_runtime::ui::v2::UiV2PrototypeStoreFileCache;
use zircon_runtime_interface::ui::{binding::UiEventKind, layout::UiSize};

use crate::core::editor_event::{EditorEvent, EditorEventSource, EditorEventTransient};
use crate::tests::editor_event::support::EventRuntimeHarness;
use crate::tests::support::env_lock;
use crate::ui::binding::{EditorUiBinding, EditorUiBindingPayload, EditorUiEventKind};
use crate::ui::host::EditorHostEventController;
use crate::ui::retained_host::callback_dispatch::workbench::{
    dispatch_componentized_workbench_control, dispatch_componentized_workbench_menu_item_selected,
};

use super::BuiltinWorkbenchWindowTemplateSurfaceBridge;

const DOCUMENT: &str = "res://ui/editor/components/workbench/modules/extensions/rendering/workbench_extension_particle_library_workspace.zui";
const CONTROL: &str = "WorkbenchExtensionParticleLibraryWorkspace";
const ACTION: &str = "workbench.extension.particle_library.open";

// These are the fifteen existing authored events, with their existing typed payloads.
const EVENTS: &[(&str, &str, UiEventKind, &str)] = &[
    (
        "WorkbenchExtensionParticleLibrarySparksRow",
        "ParticleLibrarySparksRow",
        UiEventKind::Click,
        "workbench.extension.particle_library.sparks_row.select",
    ),
    (
        "WorkbenchExtensionParticleLibraryDustRow",
        "ParticleLibraryDustRow",
        UiEventKind::Click,
        "workbench.extension.particle_library.dust_row.select",
    ),
    (
        "WorkbenchExtensionParticleLibraryImpactRow",
        "ParticleLibraryImpactRow",
        UiEventKind::Click,
        "workbench.extension.particle_library.impact_row.select",
    ),
    (
        "WorkbenchExtensionParticleLibraryGpuSparkRow",
        "ParticleLibraryGpuSparkRow",
        UiEventKind::Click,
        "workbench.extension.particle_library.gpu_spark_row.select",
    ),
    (
        "WorkbenchExtensionParticleLibraryCpuDustRow",
        "ParticleLibraryCpuDustRow",
        UiEventKind::Click,
        "workbench.extension.particle_library.cpu_dust_row.select",
    ),
    (
        "WorkbenchExtensionParticleLibraryImpactGpuRow",
        "ParticleLibraryImpactGpuRow",
        UiEventKind::Click,
        "workbench.extension.particle_library.impact_gpu_row.select",
    ),
    (
        "WorkbenchExtensionParticleLibraryArchivedRow",
        "ParticleLibraryArchivedRow",
        UiEventKind::Click,
        "workbench.extension.particle_library.archived_row.select",
    ),
    (
        "WorkbenchExtensionParticleLibrarySimulateButton",
        "ParticleLibrarySimulate",
        UiEventKind::Click,
        "workbench.extension.particle_library.simulate.invoke",
    ),
    (
        "WorkbenchExtensionParticleLibraryCompileButton",
        "ParticleLibraryCompile",
        UiEventKind::Click,
        "workbench.extension.particle_library.compile.invoke",
    ),
    (
        "WorkbenchExtensionParticleLibraryEmitterDropdown",
        "ParticleLibraryEmitterEdit",
        UiEventKind::Change,
        "workbench.extension.particle_library.emitter.edit",
    ),
    (
        "WorkbenchExtensionParticleLibraryEmitterDropdown",
        "ParticleLibraryEmitterCommit",
        UiEventKind::Submit,
        "workbench.extension.particle_library.emitter.commit",
    ),
    (
        "WorkbenchExtensionParticleLibraryDurationField",
        "ParticleLibraryDurationEdit",
        UiEventKind::Change,
        "workbench.extension.particle_library.duration.edit",
    ),
    (
        "WorkbenchExtensionParticleLibraryDurationField",
        "ParticleLibraryDurationCommit",
        UiEventKind::Submit,
        "workbench.extension.particle_library.duration.commit",
    ),
    (
        "WorkbenchExtensionParticleLibraryBoundsDropdown",
        "ParticleLibraryBoundsEdit",
        UiEventKind::Change,
        "workbench.extension.particle_library.bounds.edit",
    ),
    (
        "WorkbenchExtensionParticleLibraryBoundsDropdown",
        "ParticleLibraryBoundsCommit",
        UiEventKind::Submit,
        "workbench.extension.particle_library.bounds.commit",
    ),
];

fn select(bridge: &mut BuiltinWorkbenchWindowTemplateSurfaceBridge, item: &str) {
    assert!(bridge
        .dispatch_control_state("MenuSlot2", UiEventKind::Click)
        .unwrap()
        .is_some());
    assert!(bridge.control_bool("UnrealWindowMenuPopup", "popup_open"));
    assert!(bridge
        .control_string_array("UnrealWindowMenuPopup", "menu_items")
        .iter()
        .any(|value| value.contains(&format!("action={item}"))));
    assert!(bridge
        .dispatch_workbench_module_overflow_menu_item_state("UnrealWindowMenuPopup", item)
        .unwrap()
        .is_some());
}

fn open(bridge: &mut BuiltinWorkbenchWindowTemplateSurfaceBridge) {
    select(bridge, "menu.item.v_f_x");
    let binding = bridge
        .dispatch_control_state("WorkbenchVfxParticleLibraryButton", UiEventKind::Click)
        .unwrap()
        .expect("the actual VFX gateway must dispatch its owned event");
    assert_eq!(
        binding,
        EditorUiBinding::new(
            "WorkbenchExtension",
            "ParticleLibraryOpen",
            EditorUiEventKind::Click,
            EditorUiBindingPayload::menu_action(ACTION)
        )
    );
}

fn assert_preview_callback(
    runtime: &EditorHostEventController,
    bridge: &mut BuiltinWorkbenchWindowTemplateSurfaceBridge,
    control_id: &str,
    id: &str,
    kind: UiEventKind,
    action_id: &str,
) {
    let event_kind = match kind {
        UiEventKind::Click => EditorUiEventKind::Click,
        UiEventKind::Change => EditorUiEventKind::Change,
        UiEventKind::Submit => EditorUiEventKind::Submit,
        _ => panic!("unexpected authored event kind"),
    };
    let expected = EditorUiBinding::new(
        "WorkbenchExtension",
        id,
        event_kind,
        EditorUiBindingPayload::menu_action(action_id),
    );
    assert_eq!(
        bridge.binding_for_control(control_id, kind),
        Some(&expected)
    );
    let previous_sequence = runtime
        .journal()
        .records()
        .last()
        .map(|record| record.sequence.0);
    let effects = dispatch_componentized_workbench_control(runtime, bridge, control_id, kind)
        .expect("authored control must enter the public production callback")
        .expect("existing preview action must be admitted by the real host controller");
    let journal = runtime.journal();
    let record = journal.records().last().unwrap();
    if let Some(previous_sequence) = previous_sequence {
        assert_eq!(record.sequence.0, previous_sequence + 1);
    }
    assert_eq!(record.source, EditorEventSource::RetainedHost);
    assert_eq!(
        record.event,
        EditorEvent::Transient(EditorEventTransient::PressNode {
            node_path: format!("WorkbenchExtension/{id}"),
            pressed: false,
        })
    );
    let expected_prefix = expected.path().native_prefix();
    assert_eq!(
        record.binding_path.as_deref(),
        Some(expected_prefix.as_str())
    );
    assert_eq!(
        record.operation_group.as_deref(),
        Some("ComponentLabPreview")
    );
    assert!(record.operation_id.is_none());
    assert_eq!(
        record.operation_arguments.as_ref().unwrap()["control_id"].as_str(),
        Some(id)
    );
    assert_eq!(
        record.operation_arguments.as_ref().unwrap()["action_id"].as_str(),
        Some(action_id)
    );
    assert!(record.effects.is_empty());
    assert!(!effects.render_dirty);
    assert!(!effects.presentation_dirty);
}

#[test]
fn particle_library_public_callbacks_admit_open_and_all_fifteen_local_actions() {
    let _guard = env_lock().lock().unwrap_or_else(|error| error.into_inner());
    let harness = EventRuntimeHarness::new("zircon_particle_library_public_callback_admission");
    let mut bridge =
        BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(1672.0, 941.0)).unwrap();
    dispatch_componentized_workbench_control(
        &harness.runtime,
        &mut bridge,
        "MenuSlot2",
        UiEventKind::Click,
    )
    .expect("actual Window control must enter the production callback")
    .unwrap();
    assert!(bridge.control_bool("UnrealWindowMenuPopup", "popup_open"));
    dispatch_componentized_workbench_menu_item_selected(
        &harness.runtime,
        &mut bridge,
        "UnrealWindowMenuPopup",
        "menu.item.v_f_x",
    )
    .expect("actual Window VFX item must enter its production consumer")
    .unwrap();
    assert!(bridge.has_control("WorkbenchVfxParticleLibraryButton"));
    assert_preview_callback(
        &harness.runtime,
        &mut bridge,
        "WorkbenchVfxParticleLibraryButton",
        "ParticleLibraryOpen",
        UiEventKind::Click,
        ACTION,
    );
    assert!(bridge.has_control(CONTROL));
    assert!(
        bridge.has_pending_host_projection_commit() || bridge.has_pending_surface_state_change()
    );
    for (control, id, kind, action) in EVENTS {
        assert_preview_callback(&harness.runtime, &mut bridge, control, id, *kind, action);
    }
    assert_preview_callback(
        &harness.runtime,
        &mut bridge,
        "WorkbenchExtensionParticleLibraryCompileButton",
        "ParticleLibraryCompile",
        UiEventKind::Click,
        "workbench.extension.particle_library.compile.invoke",
    );
    assert_eq!(
        bridge
            .control_string("WorkbenchExtensionParticleLibraryOutputRow", "value_text")
            .as_deref(),
        Some("Compile queued   4 emitters   bounds fixed")
    );
}

#[test]
fn particle_library_public_vfx_route_loads_only_its_actual_source_and_all_authored_events() {
    let _guard = env_lock().lock().unwrap_or_else(|error| error.into_inner());
    let mut bridge =
        BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(1672.0, 941.0)).unwrap();
    assert!(!bridge.has_control(CONTROL));
    open(&mut bridge);
    assert!(bridge.has_control(CONTROL));
    assert!(!bridge.has_control("WorkbenchModuleVfxWorkspace"));
    assert!(!bridge.has_control("WorkbenchExtensionModuleWorkspacesHost"));
    assert!(bridge.control_frame(CONTROL).is_some());
    for control in super::super::extension_module_navigation::EXTENSION_MODULE_WORKSPACE_CONTROLS {
        assert_eq!(
            bridge.has_control(control),
            *control == CONTROL,
            "{control}"
        );
    }
    let mounted = bridge.module_workspace.as_ref().unwrap();
    assert_eq!(mounted.action_id, ACTION);
    assert_eq!(mounted.workspace_control_id, CONTROL);
    assert!(bridge.surface().tree.node(mounted.root_id).is_some());

    // A separate fresh real cache proves the same selected physical source/hash closure.
    let asset_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets");
    let physical = asset_root.join("ui/editor/components/workbench/modules/extensions/rendering/workbench_extension_particle_library_workspace.zui");
    let theme = asset_root.join("ui/theme/editor_workbench_strict.zui");
    let mut cache = UiV2PrototypeStoreFileCache::new();
    assert!(cache.persistent_store().is_none());
    let loaded = cache.load_store([physical.clone(), theme]).unwrap();
    assert_eq!(loaded.root_asset_id, DOCUMENT);
    assert!(loaded.unresolved_imports.is_empty());
    assert!(
        loaded.root_document.root.is_none(),
        "source remains a Component document"
    );
    assert_eq!(mounted._source_receipts, loaded.source_receipts);
    let receipt = mounted
        ._source_receipts
        .iter()
        .find(|receipt| receipt.asset_id == DOCUMENT)
        .unwrap();
    assert_eq!(receipt.physical_path, physical.canonicalize().unwrap());
    assert!(mounted
        ._source_receipts
        .iter()
        .filter(|receipt| receipt.asset_id.contains("/modules/extensions/"))
        .all(|receipt| receipt.asset_id == DOCUMENT));

    let authored: toml::Value =
        toml::from_str(&std::fs::read_to_string(physical).unwrap()).unwrap();
    assert_eq!(authored["asset"]["kind"].as_str(), Some("component"));
    let authored_ids = authored["nodes"]
        .as_table()
        .unwrap()
        .values()
        .flat_map(|node| {
            node.get("events")
                .and_then(toml::Value::as_array)
                .into_iter()
                .flatten()
        })
        .map(|event| event["id"].as_str().unwrap().to_string())
        .collect::<BTreeSet<_>>();
    let expected_ids = EVENTS
        .iter()
        .map(|(_, id, _, _)| format!("WorkbenchExtension/{id}"))
        .collect::<BTreeSet<_>>();
    assert_eq!(authored_ids, expected_ids);
    for (control, id, kind, action) in EVENTS {
        assert!(bridge.has_control(control), "{control}");
        let binding = bridge
            .dispatch_binding_state_for_control(control, &format!("WorkbenchExtension/{id}"))
            .unwrap()
            .expect("an authored event must actually dispatch from its mounted owner");
        let event_kind = match kind {
            UiEventKind::Click => EditorUiEventKind::Click,
            UiEventKind::Change => EditorUiEventKind::Change,
            UiEventKind::Submit => EditorUiEventKind::Submit,
            _ => panic!("unexpected authored event kind"),
        };
        assert_eq!(
            binding,
            EditorUiBinding::new(
                "WorkbenchExtension",
                *id,
                event_kind,
                EditorUiBindingPayload::menu_action(*action)
            )
        );
    }
    bridge
        .dispatch_control_state(
            "WorkbenchExtensionParticleLibraryDustRow",
            UiEventKind::Click,
        )
        .unwrap()
        .expect("row selection must dispatch");
    assert!(bridge.control_bool("WorkbenchExtensionParticleLibraryDustRow", "selected"));
    assert!(!bridge.control_bool("WorkbenchExtensionParticleLibrarySparksRow", "selected"));
    bridge
        .dispatch_control_state(
            "WorkbenchExtensionParticleLibraryCompileButton",
            UiEventKind::Click,
        )
        .unwrap()
        .expect("the existing local compile feedback must dispatch");
    assert_eq!(
        bridge
            .control_string("WorkbenchExtensionParticleLibraryOutputRow", "value_text")
            .as_deref(),
        Some("Compile queued   4 emitters   bounds fixed")
    );
}

#[test]
fn particle_library_return_to_core_and_scene_retires_focus_routes_and_child_storage() {
    let _guard = env_lock().lock().unwrap_or_else(|error| error.into_inner());
    let mut bridge =
        BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(1672.0, 941.0)).unwrap();
    let base_ids = bridge
        .surface()
        .tree
        .nodes
        .keys()
        .copied()
        .collect::<BTreeSet<_>>();
    let base_slots = bridge.surface().tree.layout_slots().len();
    let mut extension_nodes = None;
    for _ in 0..3 {
        for destination in ["menu.item.v_f_x", "menu.item.scene"] {
            open(&mut bridge);
            let count = bridge.surface().tree.nodes.len();
            assert_eq!(*extension_nodes.get_or_insert(count), count);
            let field = bridge
                .control_node_id("WorkbenchExtensionParticleLibraryDurationField")
                .unwrap();
            bridge
                .template_surface
                .surface
                .focus_node(field)
                .expect("actual extension field must accept focus");
            assert_eq!(bridge.surface().focus.focused, Some(field));
            select(&mut bridge, destination);
            assert!(!bridge.has_control(CONTROL));
            assert!(!bridge.has_control("WorkbenchExtensionParticleLibraryDurationField"));
            assert!(bridge.surface().focus.focused.is_none());
            for (_, id, _, _) in EVENTS {
                assert!(
                    bridge
                        .binding_by_id(&format!("WorkbenchExtension/{id}"))
                        .is_none(),
                    "{id}"
                );
            }
            if destination == "menu.item.scene" {
                assert!(bridge.module_workspace.is_none());
                assert_eq!(
                    bridge
                        .surface()
                        .tree
                        .nodes
                        .keys()
                        .copied()
                        .collect::<BTreeSet<_>>(),
                    base_ids
                );
                assert_eq!(bridge.surface().tree.layout_slots().len(), base_slots);
                assert!(bridge.control_frame("WorkbenchViewportSurface").is_some());
            } else {
                assert!(bridge.has_control("WorkbenchModuleVfxWorkspace"));
                assert!(bridge
                    .dispatch_control_state("WorkbenchVfxSimulateButton", UiEventKind::Click)
                    .unwrap()
                    .is_some());
                assert!(bridge
                    .dispatch_control_state(
                        "WorkbenchExtensionParticleLibrarySimulateButton",
                        UiEventKind::Click
                    )
                    .unwrap()
                    .is_none());
            }
        }
    }
}
