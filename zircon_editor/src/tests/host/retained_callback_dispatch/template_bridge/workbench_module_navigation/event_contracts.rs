use std::collections::BTreeSet;

use super::super::super::support::{
    env_lock, BuiltinWorkbenchWindowTemplateSurfaceBridge, EditorUiBindingPayload, UiEventKind,
    UiSize,
};
use crate::ui::retained_host::workbench_preview_actions::is_workbench_preview_action;

const DECLARED_WORKBENCH_MODULE_EVENT_COUNT: usize = 189;

const WORKBENCH_MODULE_EVENT_SOURCES: &[(&str, &str)] = &[
    (
        "workbench/shell/workbench_top_toolbar.zui",
        include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/assets/ui/editor/components/workbench/shell/workbench_top_toolbar.zui"
        )),
    ),
    (
        "workbench/modules/core/gameplay/workbench_effect_workspace.zui",
        include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/assets/ui/editor/components/workbench/modules/core/gameplay/workbench_effect_workspace.zui"
        )),
    ),
    (
        "workbench/modules/core/rendering/workbench_material_workspace.zui",
        include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/assets/ui/editor/components/workbench/modules/core/rendering/workbench_material_workspace.zui"
        )),
    ),
    (
        "workbench/modules/core/ai/workbench_behavior_workspace.zui",
        include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/assets/ui/editor/components/workbench/modules/core/ai/workbench_behavior_workspace.zui"
        )),
    ),
    (
        "workbench/modules/core/assets/workbench_assets_workspace.zui",
        include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/assets/ui/editor/components/workbench/modules/core/assets/workbench_assets_workspace.zui"
        )),
    ),
    (
        "workbench/modules/core/rendering/workbench_vfx_workspace.zui",
        include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/assets/ui/editor/components/workbench/modules/core/rendering/workbench_vfx_workspace.zui"
        )),
    ),
    (
        "workbench/modules/core/gameplay/workbench_ability_workspace.zui",
        include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/assets/ui/editor/components/workbench/modules/core/gameplay/workbench_ability_workspace.zui"
        )),
    ),
    (
        "workbench/modules/core/gameplay/workbench_tags_workspace.zui",
        include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/assets/ui/editor/components/workbench/modules/core/gameplay/workbench_tags_workspace.zui"
        )),
    ),
    (
        "workbench/modules/core/ai/workbench_perception_workspace.zui",
        include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/assets/ui/editor/components/workbench/modules/core/ai/workbench_perception_workspace.zui"
        )),
    ),
    (
        "workbench/modules/core/rendering/workbench_render_workspace.zui",
        include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/assets/ui/editor/components/workbench/modules/core/rendering/workbench_render_workspace.zui"
        )),
    ),
    (
        "workbench/modules/core/ui/workbench_hud_workspace.zui",
        include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/assets/ui/editor/components/workbench/modules/core/ui/workbench_hud_workspace.zui"
        )),
    ),
];

#[derive(Debug)]
struct WorkbenchModuleEventCase {
    source: &'static str,
    control_id: String,
    binding_id: String,
    event_kind: UiEventKind,
}

fn declared_workbench_module_events() -> Vec<WorkbenchModuleEventCase> {
    let mut cases = Vec::new();
    for &(source, document) in WORKBENCH_MODULE_EVENT_SOURCES {
        let parsed = toml::from_str::<toml::Value>(document).unwrap_or_else(|error| {
            panic!("failed to parse {source} as TOML for module event coverage: {error}")
        });
        let nodes = parsed
            .get("nodes")
            .and_then(toml::Value::as_table)
            .unwrap_or_else(|| panic!("{source} should contain a [nodes] table"));
        for (node_name, node) in nodes {
            let control_id = node
                .get("control_id")
                .and_then(toml::Value::as_str)
                .map(str::to_string);
            let events = node
                .get("events")
                .and_then(toml::Value::as_array)
                .into_iter()
                .flatten();
            for event in events {
                let Some(binding_id) = event.get("id").and_then(toml::Value::as_str) else {
                    continue;
                };
                if !binding_id.starts_with("WorkbenchModule/") {
                    continue;
                }
                let control_id = control_id.clone().unwrap_or_else(|| {
                    panic!("{source}:{node_name} declares {binding_id} without a control_id")
                });
                cases.push(WorkbenchModuleEventCase {
                    source,
                    control_id,
                    binding_id: binding_id.to_string(),
                    event_kind: module_event_kind(source, node_name, binding_id, event),
                });
            }
        }
    }
    cases
}

fn module_event_kind(
    source: &str,
    node_name: &str,
    binding_id: &str,
    event: &toml::Value,
) -> UiEventKind {
    match event.get("event").and_then(toml::Value::as_str) {
        Some("Click") => UiEventKind::Click,
        Some("Change") => UiEventKind::Change,
        Some("Submit") => UiEventKind::Submit,
        Some(other) => {
            panic!("{source}:{node_name} {binding_id} uses unsupported module event kind {other}")
        }
        None => panic!("{source}:{node_name} {binding_id} is missing an event kind"),
    }
}

#[test]
fn declared_workbench_module_events_dispatch_preview_actions() {
    let _guard = env_lock().lock().unwrap_or_else(|error| error.into_inner());

    let mut bridge =
        BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(1672.0, 941.0)).unwrap();
    let events = declared_workbench_module_events();
    assert_eq!(
        events.len(),
        DECLARED_WORKBENCH_MODULE_EVENT_COUNT,
        "the ZUI module surfaces should keep every declared WorkbenchModule/* event under coverage"
    );

    let mut seen_binding_ids = BTreeSet::new();
    for event in events {
        assert!(
            seen_binding_ids.insert(event.binding_id.clone()),
            "{} is declared more than once",
            event.binding_id
        );
        assert!(
            bridge.has_control(&event.control_id),
            "{} declares {} for missing control {}",
            event.source,
            event.binding_id,
            event.control_id
        );

        let binding = bridge
            .dispatch_binding_state_for_control(&event.control_id, &event.binding_id)
            .unwrap_or_else(|error| {
                panic!(
                    "{} {} on {} failed to dispatch: {error}",
                    event.source, event.binding_id, event.control_id
                )
            })
            .unwrap_or_else(|| {
                panic!(
                    "{} {} on {} did not resolve to a template binding",
                    event.source, event.binding_id, event.control_id
                )
            });
        assert_eq!(binding.path().view_id, "WorkbenchModule");
        assert_eq!(binding.path().event_kind, event.event_kind);
        assert_eq!(
            event.binding_id.strip_prefix("WorkbenchModule/"),
            Some(binding.path().control_id.as_str())
        );
        let EditorUiBindingPayload::MenuAction { action_id } = binding.payload() else {
            panic!(
                "{} {} should dispatch as a Workbench preview menu action",
                event.source, event.binding_id
            );
        };
        assert!(
            is_workbench_preview_action(action_id),
            "{} {} resolves to unregistered preview action {}",
            event.source,
            event.binding_id,
            action_id
        );
    }
}
