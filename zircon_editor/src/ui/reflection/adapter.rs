use serde_json::json;
use zircon_runtime_interface::ui::binding::UiEventKind;
use zircon_runtime_interface::ui::event_ui::{
    UiActionDescriptor, UiPropertyDescriptor, UiReflectionSnapshot, UiStateFlags, UiValueType,
};

use super::activity::activity_node;
use super::builder::SnapshotBuilder;
use super::model::EditorWorkbenchReflectionModel;
use super::state_flags::visible_enabled_flags;
use crate::ui::binding::EditorUiBinding;

const MENU_ITEM_PROPERTY_CAPACITY: usize = 4;

#[derive(Default)]
pub struct EditorUiReflectionAdapter;

impl EditorUiReflectionAdapter {
    pub fn build_snapshot(model: &EditorWorkbenchReflectionModel) -> UiReflectionSnapshot {
        let mut builder = SnapshotBuilder::new(model.tree_id.clone());
        let root = builder.push_node(
            "editor/workbench",
            "EditorWorkbench",
            "Workbench",
            UiStateFlags {
                visible: true,
                enabled: true,
                clickable: false,
                hoverable: false,
                focusable: false,
                pressed: false,
                checked: false,
                dirty: false,
            },
            vec![UiPropertyDescriptor::new(
                "status_line",
                UiValueType::String,
                json!(model.status_line),
            )],
            Vec::new(),
        );
        let menu_root = builder.push_node(
            "editor/workbench/menu",
            "MenuBar",
            "Menu",
            visible_enabled_flags(true, true),
            Vec::new(),
            Vec::new(),
        );
        builder.add_child(root, menu_root);
        for item in &model.menu_items {
            let (event_kind, symbol, native_binding) = menu_binding_projection(&item.binding);
            let mut properties = Vec::with_capacity(MENU_ITEM_PROPERTY_CAPACITY);
            properties.extend([
                UiPropertyDescriptor::new("menu_id", UiValueType::String, json!(item.menu_id)),
                UiPropertyDescriptor::new(
                    "native_binding",
                    UiValueType::String,
                    json!(native_binding),
                ),
            ]);
            if let Some(operation_path) = &item.operation_path {
                properties.push(UiPropertyDescriptor::new(
                    "operation_path",
                    UiValueType::String,
                    json!(operation_path),
                ));
            }
            if let Some(shortcut) = &item.shortcut {
                properties.push(UiPropertyDescriptor::new(
                    "shortcut",
                    UiValueType::String,
                    json!(shortcut),
                ));
            }
            let node = builder.push_node(
                format!("editor/workbench/menu/{}/{}", item.menu_id, item.control_id),
                "MenuItem",
                item.label.clone(),
                visible_enabled_flags(true, item.enabled),
                properties,
                vec![match item.route_id {
                    Some(route_id) => {
                        UiActionDescriptor::new("workbench.menu.item.click", event_kind, symbol)
                            .with_callable_from_remote(true)
                            .with_route_id(route_id)
                    }
                    None => {
                        UiActionDescriptor::new("workbench.menu.item.click", event_kind, symbol)
                    }
                }],
            );
            builder.add_child(menu_root, node);
        }

        let pages_root = builder.push_node(
            "editor/workbench/pages",
            "PageCollection",
            "Pages",
            visible_enabled_flags(true, true),
            Vec::new(),
            Vec::new(),
        );
        builder.add_child(root, pages_root);
        for page in &model.pages {
            let page_node = builder.push_node(
                format!("editor/workbench/pages/{}", page.page_id),
                if page.exclusive {
                    "ExclusivePage"
                } else {
                    "WorkbenchPage"
                },
                page.title.clone(),
                UiStateFlags {
                    visible: true,
                    enabled: true,
                    clickable: false,
                    hoverable: false,
                    focusable: true,
                    pressed: false,
                    checked: page.active,
                    dirty: false,
                },
                vec![UiPropertyDescriptor::new(
                    "exclusive",
                    UiValueType::Bool,
                    json!(page.exclusive),
                )],
                Vec::new(),
            );
            builder.add_child(pages_root, page_node);
            for activity in &page.activities {
                let node = activity_node(
                    &mut builder,
                    activity,
                    format!(
                        "editor/workbench/pages/{}/{}",
                        page.page_id, activity.instance_id
                    ),
                );
                builder.add_child(page_node, node);
            }
        }

        let drawers_root = builder.push_node(
            "editor/workbench/drawers",
            "DrawerCollection",
            "Drawers",
            visible_enabled_flags(true, true),
            Vec::new(),
            Vec::new(),
        );
        builder.add_child(root, drawers_root);
        for drawer in &model.drawers {
            let drawer_node = builder.push_node(
                format!("editor/workbench/drawers/{}", drawer.drawer_id),
                "ActivityDrawer",
                drawer.title.clone(),
                visible_enabled_flags(drawer.visible, true),
                Vec::new(),
                Vec::new(),
            );
            builder.add_child(drawers_root, drawer_node);
            for activity in &drawer.activities {
                let node = activity_node(
                    &mut builder,
                    activity,
                    format!(
                        "editor/workbench/drawers/{}/{}",
                        drawer.drawer_id, activity.instance_id
                    ),
                );
                builder.add_child(drawer_node, node);
            }
        }

        let floating_root = builder.push_node(
            "editor/workbench/floating",
            "FloatingWindows",
            "Floating Windows",
            visible_enabled_flags(true, true),
            Vec::new(),
            Vec::new(),
        );
        builder.add_child(root, floating_root);
        for window in &model.floating_windows {
            let window_node = builder.push_node(
                format!("editor/workbench/floating/{}", window.window_id),
                "FloatingWindow",
                window.title.clone(),
                visible_enabled_flags(true, true),
                Vec::new(),
                Vec::new(),
            );
            builder.add_child(floating_root, window_node);
            for activity in &window.activities {
                let node = activity_node(
                    &mut builder,
                    activity,
                    format!(
                        "editor/workbench/floating/{}/{}",
                        window.window_id, activity.instance_id
                    ),
                );
                builder.add_child(window_node, node);
            }
        }

        builder.finish(root)
    }
}

fn menu_binding_projection(binding: &EditorUiBinding) -> (UiEventKind, String, String) {
    let action = binding.as_ui_binding();
    let native_binding = action.native_binding();
    let event_kind = action.path.event_kind;
    let symbol = action
        .action
        .map(|call| call.symbol)
        .unwrap_or_else(|| "Action".to_string());
    (event_kind, symbol, native_binding)
}

#[cfg(test)]
mod optimization_batch_df_editor343_tests {
    use std::hint::black_box;
    use std::time::Instant;

    use crate::ui::binding::{EditorUiBinding, EditorUiBindingPayload, EditorUiEventKind};
    use zircon_runtime_interface::ui::binding::{UiBindingCall, UiBindingValue};

    use super::menu_binding_projection;

    const SAMPLE_PAIRS: usize = 17;
    const PROJECTIONS_PER_SAMPLE: usize = 8_192;

    #[test]
    fn optimization_batch_df_editor343_single_binding_projection_matches_legacy() {
        let binding = benchmark_binding();

        assert_eq!(
            menu_binding_projection(&binding),
            legacy_projection(&binding)
        );
    }

    #[test]
    fn optimization_batch_df_editor343_menu_projection_converts_binding_once() {
        let source = include_str!("adapter.rs");
        let production = source.split("#[cfg(test)]").next().unwrap();

        assert!(production.contains("const MENU_ITEM_PROPERTY_CAPACITY: usize = 4"));
        assert!(production.contains("Vec::with_capacity(MENU_ITEM_PROPERTY_CAPACITY)"));
        assert!(production.contains("fn menu_binding_projection"));
        assert!(!production.contains("item.binding.native_binding()"));
        assert!(!production.contains("call.symbol.clone()"));
    }

    #[test]
    #[ignore = "managed Windows release performance evidence"]
    fn optimization_batch_df_editor343_single_binding_projection_p95() {
        let binding = benchmark_binding();
        let mut legacy = Vec::with_capacity(SAMPLE_PAIRS);
        let mut optimized = Vec::with_capacity(SAMPLE_PAIRS);
        for pair in 0..SAMPLE_PAIRS {
            if pair % 2 == 0 {
                legacy.push(measure(&binding, false));
                optimized.push(measure(&binding, true));
            } else {
                optimized.push(measure(&binding, true));
                legacy.push(measure(&binding, false));
            }
        }
        let legacy_p95_ns = percentile(&legacy, 95);
        let optimized_p95_ns = percentile(&optimized, 95);
        println!(
            "EDITOR343_MENU_BINDING_SINGLE_PROJECTION_BENCH_V1 sample_pairs={SAMPLE_PAIRS} projections_per_sample={PROJECTIONS_PER_SAMPLE} legacy_binding_conversions_per_projection=2 optimized_binding_conversions_per_projection=1 legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} legacy_raw_ns={} optimized_raw_ns={}",
            csv(&legacy),
            csv(&optimized)
        );
        assert!(
            optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70),
            "single menu binding projection must reduce P95 by at least 30%: legacy={legacy_p95_ns}ns optimized={optimized_p95_ns}ns"
        );
    }

    fn benchmark_binding() -> EditorUiBinding {
        let call = (0..16).fold(
            UiBindingCall::new(format!("PluginMenuAction{}", "x".repeat(96))),
            |call, index| {
                call.with_argument(UiBindingValue::string(format!(
                    "plugin.menu.argument.{index:02}.{}",
                    "y".repeat(96)
                )))
            },
        );
        EditorUiBinding::new(
            "WorkbenchMenuBarWithStableReflectionIdentity",
            "OpenDeepPluginConfigurationWithStableControlIdentity",
            EditorUiEventKind::Click,
            EditorUiBindingPayload::Custom(call),
        )
    }

    fn legacy_projection(
        binding: &EditorUiBinding,
    ) -> (
        zircon_runtime_interface::ui::binding::UiEventKind,
        String,
        String,
    ) {
        let action = binding.as_ui_binding();
        let symbol = action
            .action
            .as_ref()
            .map(|call| call.symbol.clone())
            .unwrap_or_else(|| "Action".to_string());
        (action.path.event_kind, symbol, binding.native_binding())
    }

    fn measure(binding: &EditorUiBinding, optimized: bool) -> u128 {
        let started = Instant::now();
        for _ in 0..PROJECTIONS_PER_SAMPLE {
            let projection = if optimized {
                menu_binding_projection(black_box(binding))
            } else {
                legacy_projection(black_box(binding))
            };
            black_box(projection);
        }
        started.elapsed().as_nanos()
    }

    fn percentile(samples: &[u128], percentile: usize) -> u128 {
        let mut ordered = samples.to_vec();
        ordered.sort_unstable();
        ordered[(ordered.len() - 1) * percentile / 100]
    }

    fn csv(samples: &[u128]) -> String {
        samples
            .iter()
            .map(u128::to_string)
            .collect::<Vec<_>>()
            .join(",")
    }
}
