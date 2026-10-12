use super::super::support::*;
use crate::core::editor_event::{EditorEventSource, EditorEventUndoPolicy};
use crate::ui::binding::{EditorUiBinding, EditorUiBindingPayload};
use crate::ui::retained_host::host_contract::{
    componentized_workbench_regions::{
        assert_control_source_paint_and_hit, assert_module_tools_menu_paint_once,
        assert_retired_source_has_no_paint, module_tools_popups,
    },
    data::{FrameRect, HostWindowPresentationData, TemplatePaneNodeData},
    hit_test_workbench_window_template_node_with_index, menu_item_row_frame, HostWorkbenchHitIndex,
};
use std::sync::Arc;
use zircon_runtime_interface::ui::binding::UiEventKind;

use super::cases::{MenuCase, WorkspaceCase, BLEND_SPACE_IMPORT_SOURCES, MENUS, WORKSPACES};

pub(super) fn harness() -> ChildWindowHostHarness {
    let harness = ChildWindowHostHarness::new("zircon_module_tools_native_sources");
    harness.activate_workbench_page();
    resize(&harness, 1672, 941);
    harness
}

pub(super) fn resize(harness: &ChildWindowHostHarness, width: u32, height: u32) {
    harness
        .root_ui
        .window()
        .set_size(PhysicalSize::new(width, height));
    let mut host = harness.host.borrow_mut();
    host.sync_shell_size();
    host.refresh_ui();
    host.recompute_if_dirty();
}

pub(super) fn node<'a>(
    presentation: &'a HostWindowPresentationData,
    control: &str,
) -> &'a TemplatePaneNodeData {
    let mut matching = presentation
        .workbench_window_nodes
        .iter()
        .filter(|node| node.control_id.as_str() == control);
    let node = matching
        .next()
        .expect("a real published control is present");
    assert!(
        matching.next().is_none(),
        "the selected surface has one control owner"
    );
    node
}

pub(super) fn frame(node: &TemplatePaneNodeData) -> FrameRect {
    FrameRect {
        x: node.frame.x,
        y: node.frame.y,
        width: node.frame.width,
        height: node.frame.height,
    }
}

pub(super) fn intersect(a: &FrameRect, b: &FrameRect) -> Option<FrameRect> {
    let x = a.x.max(b.x);
    let y = a.y.max(b.y);
    let right = (a.x + a.width).min(b.x + b.width);
    let bottom = (a.y + a.height).min(b.y + b.height);
    (right > x && bottom > y).then_some(FrameRect {
        x,
        y,
        width: right - x,
        height: bottom - y,
    })
}

pub(super) fn window_bounds(presentation: &HostWindowPresentationData) -> FrameRect {
    frame(node(presentation, "WorkbenchWindowRoot"))
}

fn trace_window_menu_native_click(
    harness: &ChildWindowHostHarness,
    stage: &str,
    x: f32,
    y: f32,
) {
    let presentation = harness.root_ui.get_host_presentation();
    let current = harness
        .host
        .borrow()
        .workbench_window_bridge
        .surface()
        .surface_frame();
    eprintln!(
        "window-menu-native: stage={stage} point=({x},{y}) current_source={:p} legacy_top_bar_height={} legacy_open={}",
        Arc::as_ptr(&current),
        presentation.host_scene_data.menu_chrome.top_bar_height_px,
        harness.root_ui.get_menu_state().open_menu_index,
    );
    for control in [
        "WorkbenchWindowRoot",
        "WorkbenchMenuBarRoot",
        "MenuSlot2",
        "UnrealEditorMenuPopups",
        "UnrealWindowMenuPopup",
    ] {
        if let Some(actual) = presentation
            .workbench_window_nodes
            .iter()
            .find(|node| node.control_id.as_str() == control)
        {
            eprintln!(
                "window-menu-node: stage={stage} control={control} parent={} frame=({},{},{},{}) open={} focused={} action={} source={:?} source_path={} source_node={} instance_path={}",
                actual.parent_node_id.as_str(),
                actual.frame.x, actual.frame.y, actual.frame.width, actual.frame.height,
                actual.popup_open, actual.focused, actual.action_id.as_str(),
                actual.source_surface_frame.as_ref().map(Arc::as_ptr),
                actual.source_path.as_str(), actual.source_node_id.as_str(), actual.instance_path.as_str(),
            );
        } else {
            eprintln!("window-menu-node: stage={stage} control={control} absent");
        }
    }
    let index = HostWorkbenchHitIndex::from_presentation(&presentation);
    let hit = hit_test_workbench_window_template_node_with_index(&presentation, &index, x, y);
    eprintln!(
        "window-menu-hit: stage={stage} control={:?} action={:?}",
        hit.as_ref().map(|hit| hit.control_id.as_str()),
        hit.as_ref().map(|hit| hit.action_id.as_str()),
    );
}

pub(super) fn click(harness: &ChildWindowHostHarness, x: f32, y: f32) {
    trace_window_menu_native_click(harness, "before-press", x, y);
    let press = harness.root_ui.dispatch_native_primary_press_for_test(x, y);
    trace_window_menu_native_click(harness, "after-press", x, y);
    let release = harness
        .root_ui
        .dispatch_native_primary_release_for_test(x, y);
    trace_window_menu_native_click(harness, "after-release", x, y);
    assert!(
        press.request_redraw() || release.request_redraw(),
        "native input reached its real owner"
    );
    harness.host.borrow_mut().recompute_if_dirty();
    trace_window_menu_native_click(harness, "after-recompute", x, y);
}

pub(super) fn click_control(harness: &ChildWindowHostHarness, control: &str) {
    let presentation = harness.root_ui.get_host_presentation();
    let actual = node(&presentation, control);
    assert_control_source_paint_and_hit(&presentation, control);
    click(
        harness,
        actual.frame.x + actual.frame.width * 0.5,
        actual.frame.y + actual.frame.height * 0.5,
    );
}

pub(super) fn row_point(
    presentation: &HostWindowPresentationData,
    control: &str,
    action: &str,
) -> (f32, f32) {
    let menu = node(presentation, control);
    let row = (0..menu.structured_menu_items.row_count())
        .find(|row| {
            menu.structured_menu_items
                .get(*row)
                .unwrap()
                .action_id
                .as_str()
                == action
        })
        .unwrap();
    let row_frame = menu_item_row_frame(
        menu,
        &frame(menu),
        menu.structured_menu_items.row_count(),
        row,
    )
    .unwrap();
    let visible = intersect(&row_frame, &window_bounds(presentation))
        .expect("the actual row has visible pixels");
    let point = (
        visible.x + visible.width * 0.5,
        visible.y + visible.height * 0.5,
    );
    let index = HostWorkbenchHitIndex::from_presentation(presentation);
    let hit =
        hit_test_workbench_window_template_node_with_index(presentation, &index, point.0, point.1)
            .unwrap();
    assert_eq!(hit.control_id.as_str(), control);
    assert_eq!(hit.action_id.as_str(), action);
    point
}

pub(super) fn select_module(harness: &ChildWindowHostHarness, item: &str) {
    click_control(harness, "MenuSlot2");
    let presentation = harness.root_ui.get_host_presentation();
    let point = row_point(&presentation, "UnrealWindowMenuPopup", item);
    click(harness, point.0, point.1);
}

fn assert_mounted_source_closure(harness: &ChildWindowHostHarness, root: &TemplatePaneNodeData) {
    let receipts = harness
        .host
        .borrow()
        .workbench_window_bridge
        .mounted_module_source_receipts_for_test();
    let selected = receipts
        .iter()
        .find(|receipt| receipt.source_path.as_deref() == Some(root.source_path.as_str()))
        .expect("the actual selected root has its mounted physical source receipt");
    let mut cache = zircon_runtime::ui::v2::UiV2PrototypeStoreFileCache::new();
    assert!(cache.persistent_store().is_none());
    let asset_root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets");
    let theme_path = zircon_runtime::asset::runtime_asset_path_with_dev_asset_root(
        "/assets/ui/theme/editor_workbench_strict.zui",
        &asset_root,
    );
    let loaded = cache
        .load_store([selected.physical_path.clone(), theme_path])
        .expect("validate the actual selected physical source and the production strict theme");
    assert!(loaded.unresolved_imports.is_empty());
    assert_eq!(loaded.root_asset_id, selected.asset_id);
    assert_eq!(
        loaded.source_receipts, receipts,
        "the mounted source is the complete ordered selected-source and strict-theme closure"
    );
}

pub(super) fn open(harness: &ChildWindowHostHarness, case: &MenuCase) {
    select_module(harness, case.module_item);
    let presentation = harness.root_ui.get_host_presentation();
    let trigger = node(&presentation, case.trigger);
    let root = node(&presentation, case.root);
    let spec = crate::ui::retained_host::host_contract::componentized_workbench_regions::module_tools_menu_spec(case.menu).unwrap();
    type Step = zircon_runtime_interface::ui::v2::UiTemplateNodeInstancePathStep;
    let root_instance_path: Vec<Step> = serde_json::from_str(root.instance_path.as_str()).unwrap();
    let trigger_instance_path: Vec<Step> = serde_json::from_str(trigger.instance_path.as_str()).unwrap();
    assert_eq!(trigger.source_node_id.as_str(), "root", "the original primitive leaf is preserved");
    let root_source_path = root.source_path.as_str().replace('\\', "/");
    assert!(root_source_path.ends_with(case.source_suffix));
    let source_prefix = root_source_path.strip_suffix(spec.source_suffix).unwrap();
    assert_eq!(
        trigger.source_path.as_str().replace('\\', "/"),
        format!("{source_prefix}/components/workbench/primitives/inputs/workbench_button.zui"),
        "the actual primitive source is preserved",
    );
    let (caller, prefix) = trigger_instance_path.split_last().unwrap();
    assert_eq!(prefix, root_instance_path.as_slice());
    assert_eq!(caller.source_path, root.source_path.as_str());
    assert_eq!(caller.source_node_id, case.trigger_source);
    assert!(spec.trigger_source_is_owned(
        root.source_path.as_str(), Some(&root_instance_path),
        trigger.source_path.as_str(), trigger.source_node_id.as_str(), Some(&trigger_instance_path),
    ));
    assert!(Arc::ptr_eq(
        trigger.source_surface_frame.as_ref().unwrap(),
        node(&presentation, case.root)
            .source_surface_frame
            .as_ref()
            .unwrap()
    ));
    click_control(harness, case.trigger);
    let presentation = harness.root_ui.get_host_presentation();
    let menu = node(&presentation, case.menu);
    assert!(menu.popup_open);
    assert_eq!(menu.source_node_id.as_str(), case.popup_source);
    assert!(menu
        .source_path
        .as_str()
        .replace('\\', "/")
        .ends_with(case.source_suffix));
    assert_mounted_source_closure(harness, node(&presentation, case.root));
    assert_eq!(menu.structured_menu_items.row_count(), case.rows.len());
    for (row, action) in case.rows.iter().enumerate() {
        let item = menu.structured_menu_items.get(row).unwrap();
        assert_eq!(item.action_id.as_str(), *action);
        assert!(!item.disabled && !item.separator && !item.label.is_empty());
    }
}

pub(super) fn assert_current_module_tools_sources(
    harness: &ChildWindowHostHarness,
    case: &MenuCase,
    open: bool,
) {
    let host = harness.host.borrow();
    let bridge = &host.workbench_window_bridge;
    let current = bridge.surface().surface_frame();
    let projection = bridge.host_projection();
    assert!(Arc::ptr_eq(
        projection.source_surface_frame.as_ref().unwrap(),
        &current
    ));
    for control in [
        case.root,
        case.trigger,
        case.menu,
        "UnrealSceneViewViewport",
        "WorkbenchWindowRoot",
    ] {
        let model = projection.node_by_control_id(control).unwrap();
        assert!(Arc::ptr_eq(
            model.source_surface_frame.as_ref().unwrap(),
            &current
        ));
        let actual = current
            .arranged_tree
            .get(model.surface_node_id.unwrap())
            .unwrap();
        assert_eq!(actual.node_path.0, model.node_id);
        assert_eq!(actual.control_id.as_deref(), Some(control));
    }
    let popup = projection.node_by_control_id(case.menu).unwrap();
    assert_eq!(popup.popup_open, open);
    if !open {
        assert_eq!(
            current
                .arranged_tree
                .get(popup.surface_node_id.unwrap())
                .unwrap()
                .visibility,
            zircon_runtime_interface::ui::tree::UiVisibility::Collapsed
        );
    }
    drop(host);
    let presentation = harness.root_ui.get_host_presentation();
    for control in [
        case.root,
        case.trigger,
        "UnrealSceneViewViewport",
        "WorkbenchWindowRoot",
    ] {
        let projected = node(&presentation, control);
        assert!(Arc::ptr_eq(
            projected.source_surface_frame.as_ref().unwrap(),
            &current
        ));
        let reference = projected.surface_render_command_ref.unwrap();
        assert_eq!(
            reference,
            zircon_runtime_interface::ui::surface::UiRenderFrameCommandRef::new(
                projected.surface_node_id.unwrap(),
                0
            )
        );
        assert!(current.render_extract.command_by_ref(reference).is_some());
    }
    if open {
        let projected = node(&presentation, case.menu);
        assert!(Arc::ptr_eq(
            projected.source_surface_frame.as_ref().unwrap(),
            &current
        ));
        let reference = projected.surface_render_command_ref.unwrap();
        assert!(current.render_extract.command_by_ref(reference).is_some());
        assert_module_tools_menu_paint_once(&presentation, case.menu);
    } else {
        assert!(!presentation
            .workbench_window_nodes
            .iter()
            .any(|node| node.control_id.as_str() == case.menu));
    }
}

/// A real disabled trigger changes only the current sparse publication; no resize/full refresh.
pub(super) fn exercise_sparse_closed_menu_source_restore(
    harness: &ChildWindowHostHarness,
    case: &MenuCase,
) {
    let (frames, full_rebuilds) = {
        let host = harness.host.borrow();
        (
            host.workbench_window_bridge.frames(),
            host.workbench_window_bridge
                .host_projection_full_rebuild_count_for_test(),
        )
    };
    for disabled in [true, false] {
        {
            let mut host = harness.host.borrow_mut();
            let previous = host.workbench_window_bridge.surface().surface_frame();
            host.workbench_window_bridge
                .mutate_control_property_for_test(
                    case.trigger,
                    "disabled",
                    zircon_runtime_interface::ui::component::UiValue::Bool(disabled),
                )
                .unwrap();
            host.workbench_window_bridge
                .refresh_prepared_state_change()
                .unwrap();
            let current = host.workbench_window_bridge.surface().surface_frame();
            assert!(!Arc::ptr_eq(&previous, &current));
            let pending = host
                .workbench_window_bridge
                .pending_host_projection_patch_nodes()
                .expect("source-only closure preserves the existing sparse projection commit");
            assert!(
                !pending
                    .iter()
                    .any(|node| node.control_id.as_deref() == Some(case.menu)),
                "closed source-only popup is compiled evidence, not a new UI row"
            );
            assert!(
                pending
                    .iter()
                    .any(|node| node.control_id.as_deref() == Some(case.trigger)),
                "the actual original semantic workset is retained"
            );
            assert_eq!(host.workbench_window_bridge.frames(), frames);
            assert_eq!(
                host.workbench_window_bridge
                    .host_projection_full_rebuild_count_for_test(),
                full_rebuilds,
            );
            host.apply_dispatch_effects(UiHostEventEffects::default());
            host.recompute_if_dirty();
        }
        assert_current_module_tools_sources(harness, case, false);
    }
    click_control(harness, case.trigger);
    assert_current_module_tools_sources(harness, case, true);
    let presentation = harness.root_ui.get_host_presentation();
    let _ = row_point(&presentation, case.menu, case.rows[0]);
    assert!(harness
        .root_ui
        .dispatch_native_popup_escape_for_test()
        .request_redraw());
    harness.host.borrow_mut().recompute_if_dirty();
    assert_current_module_tools_sources(harness, case, false);
}

pub(super) fn sequence(harness: &ChildWindowHostHarness) -> u64 {
    let host = harness.host.borrow();
    let journal = host.runtime.journal();
    journal
        .records()
        .last()
        .map_or(0, |record| record.sequence.0)
}

pub(super) fn assert_preview(
    harness: &ChildWindowHostHarness,
    before: u64,
    binding: &EditorUiBinding,
    action: &str,
) {
    let host = harness.host.borrow();
    let journal = host.runtime.journal();
    let record = journal
        .records()
        .last()
        .expect("the actual callback journals its observation");
    assert_eq!(record.sequence.0, before + 1);
    assert_eq!(record.source, EditorEventSource::RetainedHost);
    assert_eq!(
        record.binding_path.as_deref(),
        Some(binding.path().native_prefix().as_str())
    );
    let expected_path = format!("{}/{}", binding.path().view_id, binding.path().control_id);
    assert_eq!(
        record.event,
        EditorEvent::Transient(EditorEventTransient::PressNode {
            node_path: expected_path.clone(),
            pressed: false,
        })
    );
    assert_eq!(
        record.operation_group.as_deref(),
        Some("ComponentLabPreview")
    );
    let arguments = record.operation_arguments.as_ref().unwrap();
    assert_eq!(
        arguments["control_id"].as_str(),
        Some(binding.path().control_id.as_str())
    );
    assert_eq!(arguments["action_id"].as_str(), Some(action));
    assert_eq!(
        arguments["node_path"].as_str(),
        Some(expected_path.as_str())
    );
    assert_eq!(record.undo_policy, EditorEventUndoPolicy::NonUndoable);
    assert!(record.effects.is_empty());
    assert!(
        record.operation_id.is_none()
            && record.transaction_id.is_none()
            && record.save_generation.is_none()
    );
    assert_eq!(record.before_revision, record.after_revision);
}

pub(super) fn assert_workspace(harness: &ChildWindowHostHarness, case: &WorkspaceCase) {
    let presentation = harness.root_ui.get_host_presentation();
    let root = node(&presentation, case.root);
    assert_eq!(root.source_node_id.as_str(), case.source_root);
    assert!(root
        .source_path
        .as_str()
        .replace('\\', "/")
        .ends_with(case.source_suffix));
    let source = root.source_surface_frame.as_ref().unwrap();
    let viewport = node(&presentation, "UnrealSceneViewViewport");
    assert_eq!(root.parent_node_id, viewport.node_id);
    assert!(Arc::ptr_eq(
        source,
        viewport.source_surface_frame.as_ref().unwrap()
    ));
    let actual = source
        .arranged_tree
        .get(root.surface_node_id.unwrap())
        .unwrap();
    assert_eq!(actual.node_path.0, root.node_id.as_str());
    assert_eq!(actual.control_id.as_deref(), Some(case.root));
    assert_eq!(actual.parent, viewport.surface_node_id);
    assert!(actual.enabled && actual.is_render_visible());
    assert!(intersect(&frame(root), &window_bounds(&presentation)).is_some());
    assert_mounted_source_closure(harness, root);
    let host = harness.host.borrow();
    assert!(!host
        .workbench_window_bridge
        .has_control("WorkbenchExtensionModuleWorkspacesHost"));
    let consumed = host
        .workbench_window_bridge
        .mounted_module_source_receipts_for_test();
    assert!(
        consumed.iter().any(|receipt| {
            receipt
                .physical_path
                .to_string_lossy()
                .replace('\\', "/")
                .ends_with(case.source_suffix)
                && receipt
                    .source_path
                    .as_deref()
                    .is_some_and(|path| path.replace('\\', "/").ends_with(case.source_suffix))
        }),
        "the selected live root consumes its actual physical canonical source"
    );
    for other in WORKSPACES {
        assert_eq!(
            host.workbench_window_bridge.has_control(other.root),
            other.root == case.root,
            "only the selected production URI is retained"
        );
    }
    assert!(!host
        .workbench_window_bridge
        .has_control(MENUS[case.menu_index].root));
    if case.extension_action == "workbench.extension.blend_space.open" {
        let source = root.source_surface_frame.as_ref().unwrap();
        for suffix in BLEND_SPACE_IMPORT_SOURCES {
            let actual = host
                .workbench_window_bridge
                .host_projection()
                .nodes
                .iter()
                .filter(|actual| {
                    actual
                        .surface_node_id
                        .is_some_and(|id| source.arranged_tree.get(id).is_some())
                })
                .find(|actual| {
                    actual
                        .source_path
                        .as_deref()
                        .is_some_and(|path| path.replace('\\', "/").ends_with(suffix))
                        || actual.instance_path.as_ref().is_some_and(|ancestry| {
                            ancestry
                                .iter()
                                .any(|site| site.source_path.replace('\\', "/").ends_with(suffix))
                        })
                })
                .expect("a real recursively imported component supplies compiled retained nodes");
            assert!(
                actual
                    .source_surface_frame
                    .as_ref()
                    .is_some_and(|actual| Arc::ptr_eq(source, actual)),
                "recursive components belong to the same selected live surface publication"
            );
        }
    }
}

pub(super) fn workspace_descendant(
    presentation: &HostWindowPresentationData,
    actual: &TemplatePaneNodeData,
    root_control: &str,
) -> bool {
    let mut current = Some(actual);
    for _ in 0..presentation.workbench_window_nodes.row_count() {
        let Some(node) = current else {
            return false;
        };
        if node.control_id.as_str() == root_control {
            return true;
        }
        current = presentation
            .workbench_window_nodes
            .iter()
            .find(|candidate| candidate.node_id == node.parent_node_id);
    }
    false
}

pub(super) fn exercise_live_workspace_control(
    harness: &ChildWindowHostHarness,
    case: &WorkspaceCase,
) -> String {
    let presentation = harness.root_ui.get_host_presentation();
    let index = HostWorkbenchHitIndex::from_presentation(&presentation);
    let prefix = format!("{}.", case.extension_action.trim_end_matches(".open"));
    let targets = {
        let host = harness.host.borrow();
        presentation
            .workbench_window_nodes
            .iter()
            .filter_map(|actual| {
                if !workspace_descendant(&presentation, actual, case.root) {
                    return None;
                }
                let binding = host
                    .workbench_window_bridge
                    .binding_for_control(actual.control_id.as_str(), UiEventKind::Click)?;
                let EditorUiBindingPayload::MenuAction { action_id } = binding.payload() else {
                    return None;
                };
                if !action_id.starts_with(&prefix) || action_id.ends_with(".open") {
                    return None;
                }
                let x = actual.frame.x + actual.frame.width * 0.5;
                let y = actual.frame.y + actual.frame.height * 0.5;
                let hit = hit_test_workbench_window_template_node_with_index(
                    &presentation,
                    &index,
                    x,
                    y,
                )?;
                (hit.control_id == actual.control_id).then(|| {
                    (
                        actual.control_id.to_string(),
                        action_id.clone(),
                        binding.clone(),
                        x,
                        y,
                    )
                })
            })
            .take(2)
            .collect::<Vec<_>>()
    };
    assert_eq!(
        targets.len(),
        2,
        "each real page exposes two independently hittable authored preview controls"
    );
    let target = &targets[0];
    let stable = &targets[1];
    let stable_before = node(&presentation, &stable.0);
    let stable_source = stable_before.source_surface_frame.as_ref().unwrap().clone();
    let stable_reference = stable_before.surface_render_command_ref.unwrap();
    assert_control_source_paint_and_hit(&presentation, &target.0);
    assert_control_source_paint_and_hit(&presentation, &stable.0);
    let before = sequence(harness);
    click(harness, target.3, target.4);
    assert_preview(harness, before, &target.2, &target.1);
    assert_workspace(harness, case);
    let published = harness.root_ui.get_host_presentation();
    assert_control_source_paint_and_hit(&published, &target.0);
    let untouched = node(&published, &stable.0);
    assert!(
        Arc::ptr_eq(
            untouched.source_surface_frame.as_ref().unwrap(),
            &stable_source,
        ),
        "an unpatched leaf keeps its original frame and command reference together"
    );
    assert_eq!(untouched.surface_render_command_ref, Some(stable_reference));
    assert!(stable_source
        .render_extract
        .command_by_ref(stable_reference)
        .is_some());
    let current = harness
        .host
        .borrow()
        .workbench_window_bridge
        .surface()
        .surface_frame();
    assert!(
        !Arc::ptr_eq(&stable_source, &current),
        "the actual sparse input publishes a new frame while retaining the unpatched leaf"
    );
    assert_control_source_paint_and_hit(&published, &stable.0);
    let before = sequence(harness);
    click(
        harness,
        untouched.frame.x + untouched.frame.width * 0.5,
        untouched.frame.y + untouched.frame.height * 0.5,
    );
    assert_preview(harness, before, &stable.2, &stable.1);
    assert_workspace(harness, case);
    let updated = harness.root_ui.get_host_presentation();
    assert_control_source_paint_and_hit(&updated, &stable.0);
    let actual = node(&updated, &stable.0);
    let current = harness
        .host
        .borrow()
        .workbench_window_bridge
        .surface()
        .surface_frame();
    assert!(
        Arc::ptr_eq(actual.source_surface_frame.as_ref().unwrap(), &current),
        "the original typed-input workset resolves the leaf against its new current publication"
    );
    let reference = actual.surface_render_command_ref.unwrap();
    assert_eq!(
        reference,
        zircon_runtime_interface::ui::surface::UiRenderFrameCommandRef::new(
            actual.surface_node_id.unwrap(),
            0,
        )
    );
    assert!(current.render_extract.command_by_ref(reference).is_some());
    stable.0.clone()
}
