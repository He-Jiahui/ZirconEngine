use std::collections::BTreeSet;

use super::super::super::support::{
    env_lock, BuiltinWorkbenchWindowTemplateSurfaceBridge, EditorUiBindingPayload, UiEventKind,
    UiSize,
};
use crate::ui::retained_host::workbench_preview_actions::is_workbench_preview_action;

const DECLARED_WORKBENCH_MODULE_EVENT_COUNT: usize = 139;

const UNREAL_TOOLBAR_SOURCE: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/assets/ui/editor/unreal/components/toolbar.zui"
));

const WORKBENCH_MODULE_EVENT_SOURCES: &[(&str, &str)] = &[
    (
        "workbench/modules/core/gameplay/workbench_effect_workspace.zui",
        include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/assets/ui/editor/modules/core/gameplay/effect_workspace.zui"
        )),
    ),
    (
        "workbench/modules/core/rendering/workbench_material_workspace.zui",
        include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/assets/ui/editor/modules/core/rendering/material_workspace.zui"
        )),
    ),
    (
        "workbench/modules/core/ai/workbench_behavior_workspace.zui",
        include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/assets/ui/editor/workbench/modules/core/ai/behavior_workspace.zui"
        )),
    ),
    (
        "workbench/modules/core/assets/workbench_assets_workspace.zui",
        include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/assets/ui/editor/modules/core/assets/assets_workspace.zui"
        )),
    ),
    (
        "workbench/modules/core/rendering/workbench_vfx_workspace.zui",
        include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/assets/ui/editor/modules/core/rendering/vfx_workspace.zui"
        )),
    ),
    (
        "workbench/modules/core/gameplay/workbench_ability_workspace.zui",
        include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/assets/ui/editor/modules/core/gameplay/ability_workspace.zui"
        )),
    ),
    (
        "workbench/modules/core/gameplay/workbench_tags_workspace.zui",
        include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/assets/ui/editor/modules/core/gameplay/tags_workspace.zui"
        )),
    ),
    (
        "workbench/modules/core/ai/workbench_perception_workspace.zui",
        include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/assets/ui/editor/modules/core/ai/perception_workspace.zui"
        )),
    ),
    (
        "workbench/modules/core/rendering/workbench_render_workspace.zui",
        include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/assets/ui/editor/modules/core/rendering/render_workspace.zui"
        )),
    ),
    (
        "workbench/modules/core/ui/workbench_hud_workspace.zui",
        include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/assets/ui/editor/workbench/modules/core/ui/hud_workspace.zui"
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

#[test]
fn unreal_toolbar_actions_remain_explicit_after_module_shell_retirement() {
    let parsed = toml::from_str::<toml::Value>(UNREAL_TOOLBAR_SOURCE)
        .expect("Unreal toolbar must remain valid TOML");
    let nodes = parsed
        .get("nodes")
        .and_then(toml::Value::as_table)
        .expect("Unreal toolbar should expose its node table");
    for (node_name, control_id, binding_id, expected) in [
        (
            "save",
            "WorkbenchToolbarSave",
            "MenuAction/SaveProject",
            "file.project.save",
        ),
        (
            "play",
            "WorkbenchRunPlay",
            "Run/Play",
            "runtime.play_mode.enter",
        ),
        (
            "stop",
            "WorkbenchRunStop",
            "Run/Stop",
            "runtime.play_mode.exit",
        ),
        (
            "play_options",
            "WorkbenchRunMode",
            "Run/OpenModeMenu",
            "workbench.run.open_mode_menu",
        ),
    ] {
        let node = nodes
            .get(node_name)
            .unwrap_or_else(|| panic!("Unreal toolbar must declare node {node_name}"));
        assert_eq!(
            node.get("control_id").and_then(toml::Value::as_str),
            Some(control_id)
        );
        let events = node
            .get("events")
            .and_then(toml::Value::as_array)
            .unwrap_or_else(|| panic!("{control_id} must declare events"));
        let event = events
            .iter()
            .find(|event| event.get("id").and_then(toml::Value::as_str) == Some(binding_id))
            .unwrap_or_else(|| panic!("{control_id} must retain {binding_id}"));
        let action = event
            .get("action")
            .and_then(|value| value.get("action"))
            .and_then(toml::Value::as_str)
            .or_else(|| event.get("route").and_then(toml::Value::as_str));
        assert_eq!(
            action,
            Some(expected),
            "{control_id} must retain its Unreal action route"
        );
    }
}

#[test]
fn every_unreal_toolbar_event_resolves_through_the_live_template_bridge() {
    let _guard = env_lock().lock().unwrap_or_else(|error| error.into_inner());
    let parsed = toml::from_str::<toml::Value>(UNREAL_TOOLBAR_SOURCE).unwrap();
    let nodes = parsed.get("nodes").and_then(toml::Value::as_table).unwrap();
    let mut bridge =
        BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(1280.0, 800.0)).unwrap();
    let mut seen = BTreeSet::new();
    for (name, node) in nodes {
        for event in node
            .get("events")
            .and_then(toml::Value::as_array)
            .into_iter()
            .flatten()
        {
            let control = node
                .get("control_id")
                .and_then(toml::Value::as_str)
                .unwrap();
            let id = event.get("id").and_then(toml::Value::as_str).unwrap();
            assert!(
                seen.insert(id.to_owned()),
                "duplicate Unreal toolbar binding {id}"
            );
            assert!(
                bridge.has_control(control),
                "missing live Unreal toolbar control {control}"
            );
            let binding = bridge
                .dispatch_binding_state_for_control(control, id)
                .unwrap_or_else(|error| panic!("{name}:{id} failed: {error}"))
                .unwrap_or_else(|| panic!("{name}:{id} did not resolve to a live binding"));
            let (view, binding_control) = id.split_once('/').unwrap();
            assert_eq!(binding.path().view_id, view);
            assert_eq!(binding.path().control_id, binding_control);
            assert_eq!(
                binding.path().event_kind,
                module_event_kind("Unreal toolbar", name, id, event)
            );
        }
    }
    assert_eq!(
        seen.len(),
        10,
        "every shipped Unreal toolbar event must have live routing coverage"
    );
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
    let mut mounted_source = None;
    for event in events {
        if mounted_source != Some(event.source) {
            let menu_action = match event.source.rsplit('/').next().unwrap() {
                "workbench_effect_workspace.zui" => "menu.item.effect",
                "workbench_material_workspace.zui" => "menu.item.material",
                "workbench_behavior_workspace.zui" => "menu.item.behavior",
                "workbench_assets_workspace.zui" => "menu.item.assets",
                "workbench_vfx_workspace.zui" => "menu.item.v_f_x",
                "workbench_ability_workspace.zui" => "menu.item.ability",
                "workbench_tags_workspace.zui" => "menu.item.tags",
                "workbench_perception_workspace.zui" => "menu.item.perception",
                "workbench_render_workspace.zui" => "menu.item.render",
                "workbench_hud_workspace.zui" => "menu.item.h_u_d",
                source => panic!("module source {source} has no approved Window entry"),
            };
            assert!(bridge
                .dispatch_control_state("MenuSlot2", UiEventKind::Click)
                .unwrap()
                .is_some());
            assert!(bridge.control_bool_for_test("UnrealWindowMenuPopup", "popup_open"));
            assert!(bridge
                .dispatch_workbench_module_overflow_menu_item_state(
                    "UnrealWindowMenuPopup",
                    menu_action,
                )
                .unwrap()
                .is_some());
            mounted_source = Some(event.source);
        }
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
