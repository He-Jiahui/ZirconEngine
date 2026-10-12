use super::support::*;
use crate::ui::binding::{EditorUiBinding, EditorUiBindingPayload, EditorUiEventKind};
use crate::ui::retained_host::callback_dispatch::template_bridge::BuiltinWorkbenchWindowTemplateSurfaceBridge;
use crate::ui::retained_host::host_contract::{
    componentized_workbench_regions::{
        assert_module_tools_menu_paint_once, assert_retired_source_has_no_paint,
        module_tools_popups,
    },
    hit_test_workbench_window_template_node_with_index, menu_item_row_frame,
    HostWorkbenchHitIndex,
};
use zircon_runtime_interface::ui::component::UiValue;
#[path = "module_tools_menus/binding_owner.rs"]
mod binding_owner;
#[path = "module_tools_menus/cases.rs"]
mod cases;
#[path = "module_tools_menus/support.rs"]
mod support;
use cases::{MENUS, WORKSPACES};
use support::*;

#[test]
fn module_tools_menu_pointer_open_paints_and_hits_all_six_sources() {
    let _guard = lock_env();
    let harness = harness();
    for case in MENUS {
        open(&harness, case);
        let presentation = harness.root_ui.get_host_presentation();
        let popup = assert_module_tools_menu_paint_once(&presentation, case.menu);
        assert!(intersect(&popup.frame, &popup.clip).is_some());
        for action in case.rows {
            let _ = row_point(&presentation, case.menu, action);
        }
        assert!(harness
            .root_ui
            .dispatch_native_popup_arrow_down_for_test()
            .request_redraw());
        assert!(harness
            .root_ui
            .dispatch_native_popup_escape_for_test()
            .request_redraw());
        harness.host.borrow_mut().recompute_if_dirty();
        assert!(!harness
            .host
            .borrow()
            .workbench_window_bridge
            .module_tools_menu_is_open(case.menu));
        exercise_sparse_closed_menu_source_restore(&harness, case);
    }
}

#[test]
fn module_tools_menu_rows_open_all_forty_three_selected_workspaces() {
    let _guard = lock_env();
    let consumed = crate::ui::retained_host::UiHostWindow::exercise_module_tools_sources_for_test();
    assert!(!consumed.is_empty());
}

impl crate::ui::retained_host::UiHostWindow {
    /// The caller holds the existing editor test environment lock.
    /// Only receipts from real mounted sources are returned, after their native witnesses retire.
    pub(crate) fn exercise_module_tools_sources_for_test(
    ) -> Vec<zircon_runtime::ui::v2::UiV2SourceFileReceipt> {
        let harness = harness();
        let frames = harness.host.borrow().workbench_window_bridge.frames();
        let mut consumed_sources = Vec::new();
        for case in WORKSPACES {
            let menu = &MENUS[case.menu_index];
            open(&harness, menu);
            let presentation = harness.root_ui.get_host_presentation();
            let old = node(&presentation, menu.menu);
            let old_source = old.source_surface_frame.as_ref().unwrap().clone();
            let old_ref = old.surface_render_command_ref.unwrap();
            assert_module_tools_menu_paint_once(&presentation, menu.menu);
            let point = row_point(&presentation, menu.menu, case.row_action);
            let parent_receipts = harness
                .host
                .borrow()
                .workbench_window_bridge
                .mounted_module_source_receipts_for_test();
            assert!(!parent_receipts.is_empty());
            let binding = EditorUiBinding::new(
                menu.menu,
                case.row_action,
                EditorUiEventKind::Click,
                EditorUiBindingPayload::menu_action(case.extension_action),
            );
            let before = sequence(&harness);
            click(&harness, point.0, point.1);
            assert_preview(&harness, before, &binding, case.extension_action);
            assert_workspace(&harness, case);
            assert_eq!(
                harness.host.borrow().workbench_window_bridge.frames(),
                frames
            );
            assert!(!harness
                .host
                .borrow()
                .workbench_window_bridge
                .has_control(menu.menu));
            assert_retired_source_has_no_paint(
                &harness.root_ui.get_host_presentation(),
                &old_source,
                old_ref,
            );
            let control = exercise_live_workspace_control(&harness, case);
            let consumed = harness
                .host
                .borrow()
                .workbench_window_bridge
                .mounted_module_source_receipts_for_test();
            assert!(!consumed.is_empty());
            let published = harness.root_ui.get_host_presentation();
            let live = node(&published, &control);
            let live_source = live.source_surface_frame.as_ref().unwrap().clone();
            let live_ref = live.surface_render_command_ref.unwrap();
            let retired_index = HostWorkbenchHitIndex::from_presentation(&published);
            select_module(&harness, "menu.item.scene");
            let current = harness.root_ui.get_host_presentation();
            assert!(!harness
                .host
                .borrow()
                .workbench_window_bridge
                .has_control(case.root));
            assert!(!harness
                .host
                .borrow()
                .workbench_window_bridge
                .has_control(&control));
            assert_retired_source_has_no_paint(&current, &live_source, live_ref);
            assert!(
                hit_test_workbench_window_template_node_with_index(
                    &current,
                    &retired_index,
                    live.frame.x + live.frame.width * 0.5,
                    live.frame.y + live.frame.height * 0.5,
                )
                .is_none(),
                "retired page source cannot be used by a stale native hit index"
            );
            consumed_sources.extend(parent_receipts);
            consumed_sources.extend(consumed);
        }
        select_module(&harness, "menu.item.scene");
        let host = harness.host.borrow();
        assert!(!host
            .workbench_window_bridge
            .has_control("WorkbenchExtensionModuleWorkspacesHost"));
        for case in WORKSPACES {
            assert!(!host.workbench_window_bridge.has_control(case.root));
        }
        drop(host);
        consumed_sources
    }
}

#[test]
fn module_tools_menu_edge_clip_matches_visible_native_hits() {
    let _guard = lock_env();
    let harness = harness();
    let case = &MENUS[0];
    let mut visible_rows = 0;
    let mut clipped_rows = 0;
    for (width, height) in [(1672, 180), (720, 220)] {
        resize(&harness, 1672, 941);
        open(&harness, case);
        resize(&harness, width, height);
        let presentation = harness.root_ui.get_host_presentation();
        let bounds = window_bounds(&presentation);
        let index = HostWorkbenchHitIndex::from_presentation(&presentation);
        let popups = module_tools_popups(&presentation, &bounds);
        let menu = node(&presentation, case.menu);
        if let Some(popup) = popups.into_iter().find(|popup| {
            popup.row
                == presentation
                    .workbench_window_nodes
                    .iter()
                    .position(|node| node.control_id.as_str() == case.menu)
                    .unwrap()
        }) {
            assert_module_tools_menu_paint_once(&presentation, case.menu);
            let mut keyboard_rows = Vec::new();
            for row in 0..menu.structured_menu_items.row_count() {
                let item = menu.structured_menu_items.get(row).unwrap();
                let row_frame = menu_item_row_frame(
                    menu,
                    &popup.frame,
                    menu.structured_menu_items.row_count(),
                    row,
                )
                .unwrap();
                if let Some(visible) = intersect(&row_frame, &popup.clip) {
                    visible_rows += 1;
                    keyboard_rows.push((item.action_id.clone(), visible.clone()));
                    let hit = hit_test_workbench_window_template_node_with_index(
                        &presentation,
                        &index,
                        visible.x + visible.width * 0.5,
                        visible.y + visible.height * 0.5,
                    )
                    .unwrap();
                    assert_eq!(hit.control_id.as_str(), case.menu);
                    assert_eq!(hit.action_id, item.action_id);
                } else {
                    clipped_rows += 1;
                    let hit = hit_test_workbench_window_template_node_with_index(
                        &presentation,
                        &index,
                        row_frame.x + row_frame.width * 0.5,
                        row_frame.y + row_frame.height * 0.5,
                    );
                    assert!(!hit.is_some_and(|hit| hit.control_id.as_str() == case.menu
                        && hit.action_id == item.action_id));
                }
            }
            if let Some((last_action, last_frame)) = keyboard_rows.last() {
                assert!(harness
                    .root_ui
                    .dispatch_native_popup_end_for_test()
                    .request_redraw());
                let interaction = harness.root_ui.get_pane_interaction_state();
                assert_eq!(interaction.hovered_template_control_id.as_str(), case.menu);
                assert_eq!(&interaction.hovered_template_action_id, last_action);
                assert_eq!(&interaction.hovered_template_frame, last_frame);
                for _ in 0..=keyboard_rows.len() {
                    assert!(harness
                        .root_ui
                        .dispatch_native_popup_arrow_down_for_test()
                        .request_redraw());
                    let interaction = harness.root_ui.get_pane_interaction_state();
                    let (_, visible) = keyboard_rows
                        .iter()
                        .find(|(action, _)| action == &interaction.hovered_template_action_id)
                        .expect("native keyboard skips every nonpainted row");
                    assert_eq!(interaction.hovered_template_control_id.as_str(), case.menu);
                    assert_eq!(&interaction.hovered_template_frame, visible);
                    let hit = hit_test_workbench_window_template_node_with_index(
                        &presentation,
                        &index,
                        visible.x + visible.width * 0.5,
                        visible.y + visible.height * 0.5,
                    )
                    .unwrap();
                    assert_eq!(hit.control_id.as_str(), case.menu);
                    assert_eq!(hit.action_id, interaction.hovered_template_action_id);
                }
            } else {
                let before_sequence = sequence(&harness);
                let _ = harness.root_ui.dispatch_native_popup_end_for_test();
                let _ = harness.root_ui.dispatch_native_popup_enter_for_test();
                assert_eq!(sequence(&harness), before_sequence);
            }
            for (x, y) in [
                (bounds.x - 1.0, popup.frame.y + 1.0),
                (bounds.x + bounds.width + 1.0, popup.frame.y + 1.0),
                (popup.frame.x + 1.0, bounds.y + bounds.height + 1.0),
            ] {
                assert!(!hit_test_workbench_window_template_node_with_index(
                    &presentation,
                    &index,
                    x,
                    y
                )
                .is_some_and(|hit| hit.control_id.as_str() == case.menu));
            }
        } else {
            for row in 0..menu.structured_menu_items.row_count() {
                let item = menu.structured_menu_items.get(row).unwrap();
                let row = menu_item_row_frame(
                    menu,
                    &frame(menu),
                    menu.structured_menu_items.row_count(),
                    row,
                )
                .unwrap();
                assert!(!hit_test_workbench_window_template_node_with_index(
                    &presentation,
                    &index,
                    row.x + row.width * 0.5,
                    row.y + row.height * 0.5
                )
                .is_some_and(
                    |hit| hit.control_id.as_str() == case.menu && hit.action_id == item.action_id
                ));
            }
        }
    }
    resize(&harness, 1672, 941);
    open(&harness, case);
    let proven = harness.root_ui.get_host_presentation();
    let old_node = node(&proven, case.menu);
    let old_source = old_node.source_surface_frame.as_ref().unwrap().clone();
    let old_ref = old_node.surface_render_command_ref.unwrap();
    let old_point = row_point(&proven, case.menu, case.rows[0]);
    let mut rows = proven
        .workbench_window_nodes
        .iter()
        .cloned()
        .collect::<Vec<_>>();
    rows.iter_mut()
        .find(|node| node.control_id.as_str() == case.menu)
        .unwrap()
        .source_node_id = "unproven_module_popup".into();
    let mut unproven = proven.clone();
    unproven.workbench_window_nodes = crate::ui::retained_host::primitives::ModelRc::from(
        std::rc::Rc::new(crate::ui::retained_host::primitives::VecModel::from(rows)),
    );
    assert!(module_tools_popups(&unproven, &window_bounds(&unproven)).is_empty());
    let index = HostWorkbenchHitIndex::from_presentation(&unproven);
    assert!(!hit_test_workbench_window_template_node_with_index(
        &unproven,
        &index,
        old_point.0,
        old_point.1
    )
    .is_some_and(|hit| hit.control_id.as_str() == case.menu));
    assert_retired_source_has_no_paint(&unproven, &old_source, old_ref);
    assert!(
        visible_rows > 0,
        "edge placement keeps visible popup rows hittable"
    );
    assert!(
        clipped_rows > 0,
        "the real short window exercises at least one nonpainted nonhit row"
    );
}

#[test]
fn module_tools_menu_toggle_escape_and_clickaway_retire_popup_targets() {
    let _guard = lock_env();
    let harness = harness();
    let case = &MENUS[0];
    for dismissal in 0..3 {
        open(&harness, case);
        let before = harness.root_ui.get_host_presentation();
        let popup_node = node(&before, case.menu);
        let old_source = popup_node.source_surface_frame.as_ref().unwrap().clone();
        let old_ref = popup_node.surface_render_command_ref.unwrap();
        let old_point = row_point(&before, case.menu, case.rows[0]);
        match dismissal {
            0 => click_control(&harness, case.trigger),
            1 => {
                assert!(harness
                    .root_ui
                    .dispatch_native_popup_escape_for_test()
                    .request_redraw());
            }
            _ => {
                let bounds = window_bounds(&before);
                click(
                    &harness,
                    bounds.x + bounds.width - 1.0,
                    bounds.y + bounds.height - 1.0,
                );
            }
        }
        harness.host.borrow_mut().recompute_if_dirty();
        assert!(!harness
            .host
            .borrow()
            .workbench_window_bridge
            .module_tools_menu_is_open(case.menu));
        let after = harness.root_ui.get_host_presentation();
        let index = HostWorkbenchHitIndex::from_presentation(&after);
        assert!(!hit_test_workbench_window_template_node_with_index(
            &after,
            &index,
            old_point.0,
            old_point.1
        )
        .is_some_and(|hit| hit.control_id.as_str() == case.menu));
        assert_retired_source_has_no_paint(&after, &old_source, old_ref);
        let before_sequence = sequence(&harness);
        let _ = harness.root_ui.dispatch_native_popup_enter_for_test();
        assert_eq!(
            sequence(&harness),
            before_sequence,
            "closed popup keyboard target cannot fire"
        );
    }
    open(&harness, case);
    click_control(&harness, MENUS[1].trigger);
    let presentation = harness.root_ui.get_host_presentation();
    assert!(!harness
        .host
        .borrow()
        .workbench_window_bridge
        .module_tools_menu_is_open(case.menu));
    assert!(harness
        .host
        .borrow()
        .workbench_window_bridge
        .module_tools_menu_is_open(MENUS[1].menu));
    assert_module_tools_menu_paint_once(&presentation, MENUS[1].menu);
    assert!(harness
        .root_ui
        .dispatch_native_popup_escape_for_test()
        .request_redraw());
    harness.host.borrow_mut().recompute_if_dirty();
}

#[test]
fn module_tools_menu_disabled_unknown_and_failed_admission_preserve_live_state() {
    let _guard = lock_env();
    let harness = harness();
    let case = &MENUS[0];
    open(&harness, case);
    {
        let mut host = harness.host.borrow_mut();
        let mut items = host
            .workbench_window_bridge
            .host_projection()
            .node_by_control_id(case.menu)
            .unwrap()
            .menu_items
            .clone();
        items[0].push_str(",disabled");
        host.workbench_window_bridge
            .mutate_control_property_for_test(
                case.menu,
                "menu_items",
                UiValue::Array(items.into_iter().map(UiValue::String).collect()),
            )
            .unwrap();
        host.workbench_window_bridge
            .refresh_prepared_state_change()
            .unwrap();
        host.apply_dispatch_effects(UiHostEventEffects::default());
        host.recompute_if_dirty();
    }
    let before_sequence = sequence(&harness);
    let before_surface = harness
        .host
        .borrow()
        .workbench_window_bridge
        .surface()
        .clone();
    let before_projection = harness
        .host
        .borrow()
        .workbench_window_bridge
        .host_projection()
        .clone();
    let before = harness.root_ui.get_host_presentation();
    let menu = node(&before, case.menu);
    assert!(menu.structured_menu_items.get(0).unwrap().disabled);
    assert_module_tools_menu_paint_once(&before, case.menu);
    for action in [case.rows[0], "menu.item.assets.unknown"] {
        let mut host = harness.host.borrow_mut();
        let effects = host
            .dispatch_componentized_workbench_surface_control(case.menu, action)
            .expect("the actual compiled popup owns this callback")
            .unwrap();
        assert!(!effects.render_dirty && !effects.presentation_dirty);
        assert_eq!(host.workbench_window_bridge.surface(), &before_surface);
        assert_eq!(
            host.workbench_window_bridge.host_projection(),
            &before_projection
        );
        assert!(host
            .workbench_window_bridge
            .module_tools_menu_is_open(case.menu));
    }
    assert_eq!(sequence(&harness), before_sequence);
    let row = menu_item_row_frame(
        menu,
        &frame(menu),
        menu.structured_menu_items.row_count(),
        0,
    )
    .unwrap();
    let index = HostWorkbenchHitIndex::from_presentation(&before);
    assert!(!hit_test_workbench_window_template_node_with_index(
        &before,
        &index,
        row.x + row.width * 0.5,
        row.y + row.height * 0.5
    )
    .is_some_and(
        |hit| hit.control_id.as_str() == case.menu && hit.action_id.as_str() == case.rows[0]
    ));
    let _ = harness
        .root_ui
        .dispatch_native_primary_press_for_test(row.x + row.width * 0.5, row.y + row.height * 0.5);
    let _ = harness.root_ui.dispatch_native_primary_release_for_test(
        row.x + row.width * 0.5,
        row.y + row.height * 0.5,
    );
    harness.host.borrow_mut().recompute_if_dirty();
    assert_eq!(sequence(&harness), before_sequence);
    let host = harness.host.borrow();
    assert!(host
        .workbench_window_bridge
        .module_tools_menu_is_open(case.menu));
    assert_eq!(host.workbench_window_bridge.surface(), &before_surface);
    assert_eq!(
        host.workbench_window_bridge.host_projection(),
        &before_projection
    );
    drop(host);
    assert_module_tools_menu_paint_once(&harness.root_ui.get_host_presentation(), case.menu);
    // Keep the compiled popup open while disabling its real owner, then its actual parent.
    // A direct public row callback obeys the same availability as pointer/keyboard admission.
    for disabled_control in [
        case.menu,
        "WorkbenchModuleAssetsWorkspace",
        "UnrealSceneViewViewport",
    ] {
        {
            let mut host = harness.host.borrow_mut();
            host.workbench_window_bridge
                .mutate_control_property_for_test(disabled_control, "disabled", UiValue::Bool(true))
                .unwrap();
            host.workbench_window_bridge
                .refresh_prepared_state_change()
                .unwrap();
            host.apply_dispatch_effects(UiHostEventEffects::default());
            host.recompute_if_dirty();
        }
        let before_sequence = sequence(&harness);
        let mut host = harness.host.borrow_mut();
        let before_surface = host.workbench_window_bridge.surface().clone();
        let before_projection = host.workbench_window_bridge.host_projection().clone();
        let popup = before_projection.node_by_control_id(case.menu).unwrap();
        assert!(
            popup.popup_open,
            "disabled admission must not close the authored popup"
        );
        let before_source = popup.source_surface_frame.as_ref().unwrap().clone();
        let before_frame = host.workbench_window_bridge.control_frame(case.menu);
        let effects = host
            .dispatch_componentized_workbench_surface_control(case.menu, case.rows[1])
            .expect("the disabled actual popup still owns its public row callback")
            .unwrap();
        assert!(!effects.render_dirty && !effects.presentation_dirty);
        assert_eq!(host.workbench_window_bridge.surface(), &before_surface);
        assert_eq!(
            host.workbench_window_bridge.surface().tree,
            before_surface.tree
        );
        assert_eq!(
            host.workbench_window_bridge.surface().navigation,
            before_surface.navigation
        );
        assert_eq!(
            host.workbench_window_bridge.surface().focus,
            before_surface.focus
        );
        assert_eq!(
            host.workbench_window_bridge.surface().input,
            before_surface.input
        );
        assert_eq!(
            host.workbench_window_bridge.host_projection(),
            &before_projection
        );
        let current_popup = host
            .workbench_window_bridge
            .host_projection()
            .node_by_control_id(case.menu)
            .unwrap();
        assert!(current_popup.popup_open);
        assert!(std::sync::Arc::ptr_eq(
            current_popup.source_surface_frame.as_ref().unwrap(),
            &before_source
        ));
        assert_eq!(
            host.workbench_window_bridge.control_frame(case.menu),
            before_frame
        );
        assert!(!host
            .workbench_window_bridge
            .has_control("WorkbenchExtensionFoliageEditorWorkspace"));
        drop(host);
        assert_eq!(sequence(&harness), before_sequence);
        let disabled = harness.root_ui.get_host_presentation();
        let index = HostWorkbenchHitIndex::from_presentation(&disabled);
        let row = menu_item_row_frame(
            menu,
            &frame(menu),
            menu.structured_menu_items.row_count(),
            1,
        )
        .unwrap();
        assert!(!hit_test_workbench_window_template_node_with_index(
            &disabled,
            &index,
            row.x + row.width * 0.5,
            row.y + row.height * 0.5,
        )
        .is_some_and(|hit| hit.control_id.as_str() == case.menu));
        assert!(!module_tools_popups(&disabled, &window_bounds(&disabled))
            .into_iter()
            .any(|popup| {
                disabled
                    .workbench_window_nodes
                    .get(popup.row)
                    .unwrap()
                    .control_id
                    .as_str()
                    == case.menu
            }));
        assert!(!harness
            .root_ui
            .dispatch_native_popup_enter_for_test()
            .request_redraw());
        assert_eq!(sequence(&harness), before_sequence);
        {
            let mut host = harness.host.borrow_mut();
            host.workbench_window_bridge
                .mutate_control_property_for_test(
                    disabled_control,
                    "disabled",
                    UiValue::Bool(false),
                )
                .unwrap();
            host.workbench_window_bridge
                .refresh_prepared_state_change()
                .unwrap();
            host.apply_dispatch_effects(UiHostEventEffects::default());
            host.recompute_if_dirty();
        }
        assert!(harness
            .host
            .borrow()
            .workbench_window_bridge
            .module_tools_menu_is_open(case.menu));
        assert_current_module_tools_sources(&harness, case, true);
        let restored = harness.root_ui.get_host_presentation();
        let _ = row_point(&restored, case.menu, case.rows[1]);
        assert!(harness
            .root_ui
            .dispatch_native_popup_escape_for_test()
            .request_redraw());
        harness.host.borrow_mut().recompute_if_dirty();
        assert_current_module_tools_sources(&harness, case, false);
        click_control(&harness, case.trigger);
        assert_current_module_tools_sources(&harness, case, true);
        let reopened = harness.root_ui.get_host_presentation();
        let _ = row_point(&reopened, case.menu, case.rows[1]);
    }
    BuiltinWorkbenchWindowTemplateSurfaceBridge::assert_actual_stale_child_rejection_preserves_module_state();
}

#[test]
fn module_tools_menu_module_switch_retires_owner_focus_and_stale_routes() {
    let _guard = lock_env();
    let harness = harness();
    let case = &MENUS[0];
    open(&harness, case);
    assert!(harness
        .root_ui
        .dispatch_native_popup_arrow_down_for_test()
        .request_redraw());
    let interaction = harness.root_ui.get_pane_interaction_state();
    assert_eq!(interaction.hovered_template_control_id.as_str(), case.menu);
    assert_eq!(
        interaction.hovered_template_action_id.as_str(),
        case.rows[0]
    );
    let before = harness.root_ui.get_host_presentation();
    let source = node(&before, case.menu)
        .source_surface_frame
        .as_ref()
        .unwrap()
        .clone();
    let reference = node(&before, case.menu).surface_render_command_ref.unwrap();
    let stale_index = HostWorkbenchHitIndex::from_presentation(&before);
    let old_point = row_point(&before, case.menu, case.rows[0]);
    select_module(&harness, "menu.item.scene");
    let after = harness.root_ui.get_host_presentation();
    assert!(!harness
        .host
        .borrow()
        .workbench_window_bridge
        .has_control(case.menu));
    assert!(!harness
        .host
        .borrow()
        .workbench_window_bridge
        .has_control(case.trigger));
    assert!(!harness
        .host
        .borrow()
        .workbench_window_bridge
        .has_control(case.root));
    assert_ne!(
        harness
            .root_ui
            .get_pane_interaction_state()
            .focused_template_control_id
            .as_str(),
        case.menu
    );
    assert_ne!(
        harness
            .root_ui
            .get_pane_interaction_state()
            .hovered_template_control_id
            .as_str(),
        case.menu
    );
    assert!(
        hit_test_workbench_window_template_node_with_index(
            &after,
            &stale_index,
            old_point.0,
            old_point.1
        )
        .is_none(),
        "an old source/index cannot route against the new publication"
    );
    assert_retired_source_has_no_paint(&after, &source, reference);
    let before_sequence = sequence(&harness);
    assert!(harness
        .host
        .borrow_mut()
        .dispatch_componentized_workbench_surface_control(case.menu, case.rows[0])
        .is_none());
    let _ = harness.root_ui.dispatch_native_popup_enter_for_test();
    assert_eq!(sequence(&harness), before_sequence);

    open(&harness, case);
    let reopened = harness.root_ui.get_host_presentation();
    let current_source = node(&reopened, case.menu)
        .source_surface_frame
        .as_ref()
        .unwrap();
    assert!(
        !std::sync::Arc::ptr_eq(&source, current_source),
        "reopening publishes the newly selected source incarnation"
    );
    assert_module_tools_menu_paint_once(&reopened, case.menu);
    let point = row_point(&reopened, case.menu, case.rows[0]);
    assert!(
        hit_test_workbench_window_template_node_with_index(
            &reopened,
            &stale_index,
            point.0,
            point.1
        )
        .is_none(),
        "retired source indexes stay invalid when the same control ids return"
    );
    let fresh_index = HostWorkbenchHitIndex::from_presentation(&reopened);
    let hit = hit_test_workbench_window_template_node_with_index(
        &reopened,
        &fresh_index,
        point.0,
        point.1,
    )
    .unwrap();
    assert_eq!(hit.control_id.as_str(), case.menu);
    assert_eq!(hit.action_id.as_str(), case.rows[0]);
    assert!(harness
        .root_ui
        .dispatch_native_popup_escape_for_test()
        .request_redraw());
    harness.host.borrow_mut().recompute_if_dirty();
    assert!(!harness
        .host
        .borrow()
        .workbench_window_bridge
        .module_tools_menu_is_open(case.menu));
}

#[test]
fn module_tools_menu_native_cancel_restores_actual_anchor_focus_and_current_source() {
    let _guard = lock_env();
    let harness = harness();
    for case in MENUS {
        for escape in [true, false] {
            open(&harness, case);
            assert_current_module_tools_sources(&harness, case, true);
            let (popup_id, trigger_id, original_source) = {
                let host = harness.host.borrow();
                let bridge = &host.workbench_window_bridge;
                let popup = bridge.host_projection().node_by_control_id(case.menu).unwrap();
                let trigger = bridge.host_projection().node_by_control_id(case.trigger).unwrap();
                let popup_id = popup.surface_node_id.unwrap();
                let trigger_id = trigger.surface_node_id.unwrap();
                assert_eq!(bridge.surface().focus.focused, Some(popup_id),
                    "the actual open popup must own focus: {}", case.menu);
                let restore = bridge.surface().focus.modal_restore_stack.iter()
                    .find(|entry| entry.modal == popup_id).expect("the real popup focus scope");
                assert_eq!(restore.restore, Some(trigger_id),
                    "the authored anchor must own focus restoration: {}", case.menu);
                let (prepared, _) = bridge.prepared_viewport_toolbar_publication()
                    .expect("the current toolbar publication remains valid while a module popup is open");
                let source = bridge.surface().surface_frame();
                assert!(std::sync::Arc::ptr_eq(&prepared, &source));
                (popup_id, trigger_id, source)
            };
            let before = harness.root_ui.get_host_presentation();
            let journal_before = sequence(&harness);
            if escape {
                assert!(harness.root_ui.dispatch_native_popup_escape_for_test().request_redraw());
            } else {
                let bounds = window_bounds(&before);
                click(&harness, bounds.x + bounds.width - 1.0, bounds.y + bounds.height - 1.0);
            }
            harness.host.borrow_mut().recompute_if_dirty();
            assert_current_module_tools_sources(&harness, case, false);
            {
                let host = harness.host.borrow();
                let bridge = &host.workbench_window_bridge;
                assert_eq!(bridge.surface().focus.focused, Some(trigger_id),
                    "Native cancel must return focus to the actual anchor: {}", case.menu);
                assert!(!bridge.surface().focus.modal_restore_stack.iter().any(|entry| entry.modal == popup_id));
                assert!(!bridge.surface().input.popup_stack.iter().any(|popup| popup.popup_node == Some(popup_id)));
                let current = bridge.surface().surface_frame();
                assert!(!std::sync::Arc::ptr_eq(&original_source, &current));
                let (prepared, _) = bridge.prepared_viewport_toolbar_publication()
                    .expect("focus restoration must publish the current toolbar source");
                assert!(std::sync::Arc::ptr_eq(&prepared, &current));
            }
            assert_eq!(sequence(&harness), journal_before,
                "cancellation cannot execute a module business row");
            let _ = harness.root_ui.dispatch_native_popup_enter_for_test();
            assert_eq!(sequence(&harness), journal_before,
                "a cancelled popup cannot retain a keyboard activation target");
        }
    }
}
