use std::collections::BTreeSet;

use crate::ui::layouts::common::model_rc;
use crate::ui::retained_host::host_contract::{FrameRect, TemplatePaneNodeData};
use crate::ui::workbench::layout::MainPageId;
use crate::ui::workbench::snapshot::ViewContentKind;

use super::{
    scoped_patch_covers_all_presenters, shell_content_requires_component_runtime,
    shell_content_requires_hierarchy_filter, workbench_projection_damage,
};

#[test]
fn early_shell_content_prepares_only_target_specific_support_data() {
    assert!(shell_content_requires_hierarchy_filter(
        ViewContentKind::Hierarchy
    ));
    assert!(!shell_content_requires_hierarchy_filter(
        ViewContentKind::Inspector
    ));

    assert!(shell_content_requires_component_runtime(
        ViewContentKind::UiComponentShowcase
    ));
    assert!(!shell_content_requires_component_runtime(
        ViewContentKind::Hierarchy
    ));
}

#[test]
fn workbench_projection_damage_covers_old_and_new_row_frames() {
    let mut previous_node = TemplatePaneNodeData::default();
    previous_node.frame.x = 12.0;
    previous_node.frame.y = 20.0;
    previous_node.frame.width = 100.0;
    previous_node.frame.height = 24.0;
    let mut next_node = previous_node.clone();
    next_node.frame.x = 32.0;
    let previous = model_rc(vec![previous_node]);
    let next = model_rc(vec![next_node]);

    assert_eq!(
        workbench_projection_damage(&previous, &next, &[0]),
        vec![
            FrameRect {
                x: 12.0,
                y: 20.0,
                width: 100.0,
                height: 24.0,
            },
            FrameRect {
                x: 32.0,
                y: 20.0,
                width: 100.0,
                height: 24.0,
            },
        ]
    );
}

#[test]
fn scoped_patch_falls_back_when_a_root_declared_native_presenter_is_not_patched() {
    let root_damage = [FrameRect {
        x: 0.0,
        y: 0.0,
        width: 320.0,
        height: 180.0,
    }];

    let expected = BTreeSet::from([MainPageId::new("window:target")]);
    let missing = BTreeSet::new();

    assert!(!scoped_patch_covers_all_presenters(
        &root_damage,
        &expected,
        &missing
    ));
    assert!(scoped_patch_covers_all_presenters(
        &root_damage,
        &expected,
        &expected
    ));
    assert!(!scoped_patch_covers_all_presenters(
        &root_damage,
        &expected,
        &BTreeSet::from([MainPageId::new("window:other")])
    ));
}

#[test]
fn coverage_fallback_records_damage_before_returning() {
    let source = include_str!("../presentation.rs");
    let scoped_patch = source
        .split_once("let patched_native_presenter_ids")
        .and_then(|(_, tail)| tail.split_once("damage.extend(root_patch.damage)"))
        .map(|(_, tail)| tail)
        .expect("scoped patch aggregation should remain in the recompute fast path");
    let damage_count = scoped_patch
        .find("work.damage_region_count +=")
        .expect("damage should be counted");
    let coverage_fallback = scoped_patch
        .find("if !scoped_patch_covers_all_presenters")
        .expect("coverage should be checked");

    assert!(
        damage_count < coverage_fallback,
        "damage produced before a coverage fallback must remain observable"
    );
}

#[test]
fn missing_root_projection_records_probe_work_before_returning() {
    let source = include_str!("../presentation.rs");
    let scoped_patch = source
        .split_once("let root_patch = patch_ui_asset_presentation")
        .and_then(|(_, tail)| tail.split_once("let patched_native_presenter_ids"))
        .map(|(_, tail)| tail)
        .expect("root patch work should remain in the scoped recompute path");
    let root_rows = scoped_patch
        .find("work.floating_window_rows_visited += root_patch")
        .expect("root probe work should be counted");
    let projection_missing = scoped_patch
        .find("if !root_patch.matched_presentation")
        .expect("root projection failure should fall back");

    assert!(
        root_rows < projection_missing,
        "a root projection fallback must retain the probe work already performed"
    );
}
