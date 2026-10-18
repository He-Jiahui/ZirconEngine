//! Real visible retained controls submit the registered operation into document history.
use super::*;
use crate::core::editing::engine::HistoryContextId;
use crate::core::editor_event::{EditorEvent, EditorOperationEvent};
use crate::core::editor_message::DocumentId;
use crate::ui::retained_host::callback_dispatch::{
    dispatch_componentized_workbench_option_selected,
    dispatch_componentized_workbench_surface_control_edited,
};
use serde_json::{json, Value};
use zircon_runtime::core::framework::scene::ComponentTypeDescriptor;
use zircon_runtime::scene::{NodeId, Scene};

const VALUES: &str = "WorkbenchComponentAddValues";
const PANEL: &str = "WorkbenchComponentAddPanel";

fn type_path(index: usize) -> String {
    format!("test.Component.Add{index:02}")
}
fn history() -> HistoryContextId {
    HistoryContextId::Document(DocumentId::new(7))
}

fn fixture(
    count: usize,
) -> (
    EventRuntimeHarness,
    BuiltinWorkbenchWindowTemplateSurfaceBridge,
    NodeId,
) {
    let harness = EventRuntimeHarness::new("zircon_inspector_component_add");
    let target = {
        let mut shell = harness.runtime.shell().lock();
        shell.state.bind_scene_document(DocumentId::new(7));
        let target = shell
            .state
            .viewport_controller
            .selection()
            .active_primary()
            .unwrap();
        shell.state.world.expect_with_world_mut(|scene| {
            for index in 0..count {
                scene
                    .register_component_type(
                        ComponentTypeDescriptor::new(
                            type_path(index),
                            "test",
                            format!("A deliberately long component display name {index}"),
                        )
                        .with_property("coverage", "Scalar", true)
                        .with_property("label", "String", false),
                    )
                    .unwrap();
            }
        });
        target
    };
    let mut bridge =
        BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(1672.0, 941.0)).unwrap();
    bridge
        .sync_from_chrome(&harness.runtime.chrome_snapshot().unwrap())
        .unwrap();
    assert_eq!(
        control_visibility(&bridge, "WorkbenchAddComponent"),
        Some(UiVisibility::Visible)
    );
    (harness, bridge, target)
}

fn click(
    harness: &EventRuntimeHarness,
    bridge: &mut BuiltinWorkbenchWindowTemplateSurfaceBridge,
    control: &str,
) {
    dispatch_componentized_workbench_control(&harness.runtime, bridge, control, UiEventKind::Click)
        .expect("the actual authored control must have a registered binding")
        .unwrap();
}

fn edit(bridge: &mut BuiltinWorkbenchWindowTemplateSurfaceBridge, value: &str) {
    dispatch_componentized_workbench_surface_control_edited(
        bridge,
        VALUES,
        "Inspector/ComponentAddValuesEdit",
        value,
    )
    .expect("the actual JSON edit callback must be consumed")
    .unwrap();
}

fn value(harness: &EventRuntimeHarness, target: NodeId, index: usize) -> Option<Value> {
    harness
        .runtime
        .shell()
        .lock()
        .state
        .world
        .expect_with_world(|scene| scene.dynamic_component(target, &type_path(index)).cloned())
}

fn mutate(harness: &EventRuntimeHarness, write: impl FnOnce(&mut Scene)) {
    harness
        .runtime
        .shell()
        .lock()
        .state
        .world
        .expect_with_world_mut(write);
}

#[test]
fn inspector_component_add_visible_controls_commit_registered_operation_and_shared_undo_redo() {
    let _guard = env_lock().lock().unwrap();
    let (harness, mut bridge, target) = fixture(1);
    click(&harness, &mut bridge, "WorkbenchAddComponent");
    assert_eq!(
        control_visibility(&bridge, PANEL),
        Some(UiVisibility::Visible)
    );
    assert_eq!(
        control_string(&bridge, "WorkbenchComponentAddType", "value").as_deref(),
        Some(type_path(0).as_str())
    );
    assert_eq!(
        control_string(&bridge, VALUES, "query").as_deref(),
        Some("{}")
    );
    let expected = json!({"coverage":0.25,"label":"from visible Inspector"});
    edit(&mut bridge, &expected.to_string());
    click(&harness, &mut bridge, "WorkbenchComponentAddConfirm");
    assert_eq!(value(&harness, target, 0), Some(expected.clone()));
    assert_eq!(
        control_visibility(&bridge, PANEL),
        Some(UiVisibility::Collapsed)
    );
    let records = harness.runtime.journal();
    assert!(matches!(&records.records().last().unwrap().event,
        EditorEvent::Operation(EditorOperationEvent::CommandExecuted { operation_id, .. })
        if operation_id == "inspector.component.add"));
    let transactions = harness.runtime.context().transactions();
    assert_eq!(transactions.history_status(history()).unwrap().len, 1);
    assert!(transactions.undo(history()).unwrap());
    assert_eq!(value(&harness, target, 0), None);
    assert!(transactions.redo(history()).unwrap());
    assert_eq!(value(&harness, target, 0), Some(expected));
}

#[test]
fn inspector_component_add_preserves_invalid_or_stale_draft_without_command_history_and_cancel_restores_focus(
) {
    let _guard = env_lock().lock().unwrap();
    let (harness, mut bridge, target) = fixture(1);
    click(&harness, &mut bridge, "WorkbenchAddComponent");
    for draft in [
        "{",
        "{\"coverage\":0.25}",
        "{\"coverage\":\"bad\",\"label\":\"thin\"}",
    ] {
        edit(&mut bridge, draft);
        click(&harness, &mut bridge, "WorkbenchComponentAddConfirm");
        assert_eq!(
            control_visibility(&bridge, PANEL),
            Some(UiVisibility::Visible)
        );
        assert_eq!(
            control_string(&bridge, VALUES, "query").as_deref(),
            Some(draft)
        );
        assert_eq!(value(&harness, target, 0), None);
        assert_eq!(
            harness
                .runtime
                .context()
                .transactions()
                .history_status(history())
                .unwrap()
                .len,
            0
        );
    }
    let valid = json!({"coverage":0.25,"label":"stale"}).to_string();
    edit(&mut bridge, &valid);
    mutate(&harness, |scene| {
        scene
            .spawn((zircon_runtime::scene::components::Name("unrelated".into()),))
            .unwrap();
    });
    click(&harness, &mut bridge, "WorkbenchComponentAddConfirm");
    assert_eq!(
        control_string(&bridge, VALUES, "query").as_deref(),
        Some(valid.as_str())
    );
    assert_eq!(value(&harness, target, 0), None);
    assert_eq!(
        harness
            .runtime
            .context()
            .transactions()
            .history_status(history())
            .unwrap()
            .len,
        0
    );
    click(&harness, &mut bridge, "WorkbenchComponentAddCancel");
    assert_eq!(
        control_visibility(&bridge, PANEL),
        Some(UiVisibility::Collapsed)
    );
    assert_eq!(
        control_string(&bridge, VALUES, "query").as_deref(),
        Some("")
    );
    assert!(control_component_focused(&bridge, "WorkbenchAddComponent"));
}

#[test]
fn inspector_component_add_pages_real_catalog_and_disables_existing_types_without_copying_all_options(
) {
    let _guard = env_lock().lock().unwrap();
    {
        let (harness, mut bridge, _) = fixture(0);
        assert_eq!(
            harness
                .runtime
                .inspector_component_add_catalog(0, None)
                .unwrap()
                .total,
            0
        );
        click(&harness, &mut bridge, "WorkbenchAddComponent");
        assert!(control_bool(
            &bridge,
            "WorkbenchComponentAddConfirm",
            "disabled"
        ));
        assert!(control_bool(
            &bridge,
            "WorkbenchComponentAddType",
            "disabled"
        ));
        click(&harness, &mut bridge, "WorkbenchComponentAddCancel");
    }
    let (harness, mut bridge, target) = fixture(35);
    bridge.recompute_layout(UiSize::new(680.0, 720.0)).unwrap();
    click(&harness, &mut bridge, "WorkbenchAddComponent");
    let options = control_attribute(&bridge, "WorkbenchComponentAddType", "options")
        .unwrap()
        .as_array()
        .unwrap();
    assert_eq!(options.len(), 32);
    assert_eq!(
        options.first().and_then(toml::Value::as_str),
        Some(type_path(0).as_str())
    );
    assert_eq!(
        options.last().and_then(toml::Value::as_str),
        Some(type_path(31).as_str())
    );
    assert!(!control_bool(
        &bridge,
        "WorkbenchComponentAddNext",
        "disabled"
    ));
    click(&harness, &mut bridge, "WorkbenchComponentAddNext");
    let options = control_attribute(&bridge, "WorkbenchComponentAddType", "options")
        .unwrap()
        .as_array()
        .unwrap();
    assert_eq!(
        options
            .iter()
            .filter_map(toml::Value::as_str)
            .collect::<Vec<_>>(),
        (32..35).map(type_path).collect::<Vec<_>>()
    );
    dispatch_componentized_workbench_option_selected(
        &harness.runtime,
        &mut bridge,
        "WorkbenchComponentAddType",
        &type_path(34),
    )
    .unwrap();
    assert_eq!(
        control_string(&bridge, "WorkbenchComponentAddType", "value").as_deref(),
        Some(type_path(34).as_str())
    );
    assert!(control_bool(
        &bridge,
        "WorkbenchComponentAddNext",
        "disabled"
    ));
    click(&harness, &mut bridge, "WorkbenchComponentAddPrevious");
    assert_eq!(
        control_string(&bridge, "WorkbenchComponentAddPage", "text").as_deref(),
        Some("1/2")
    );
    click(&harness, &mut bridge, "WorkbenchComponentAddCancel");
    mutate(&harness, |scene| {
        scene
            .set_dynamic_component(
                target,
                &type_path(0),
                json!({"coverage":0.5,"label":"existing"}),
            )
            .unwrap();
    });
    click(&harness, &mut bridge, "WorkbenchAddComponent");
    let disabled = control_attribute(&bridge, "WorkbenchComponentAddType", "disabled_options")
        .unwrap()
        .as_array()
        .unwrap();
    assert!(disabled
        .iter()
        .any(|item| item.as_str() == Some(type_path(0).as_str())));
    let selected = control_string(&bridge, "WorkbenchComponentAddType", "value").unwrap();
    dispatch_componentized_workbench_option_selected(
        &harness.runtime,
        &mut bridge,
        "WorkbenchComponentAddType",
        &type_path(0),
    )
    .unwrap();
    assert_eq!(
        control_string(&bridge, "WorkbenchComponentAddType", "value").as_deref(),
        Some(selected.as_str())
    );
    assert_eq!(
        value(&harness, target, 0),
        Some(json!({"coverage":0.5,"label":"existing"}))
    );
    assert_eq!(
        harness
            .runtime
            .context()
            .transactions()
            .history_status(history())
            .unwrap()
            .len,
        0
    );
}
