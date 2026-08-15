use std::time::Instant;

use zircon_runtime_interface::ui::layout::{UiFrame, UiLayoutEngineSelectionReport, UiSize};
use zircon_runtime_interface::ui::tree::{UiTree, UiTreeError};

use crate::ui::text::UiTextMeasureCache;

use super::arrange::arrange_node;
use super::engine::UiLayoutPassEngineContext;
use super::measure::measure_node;
use super::pipeline::{assert_layout_pass_stage, UiLayoutPassStage};
use super::responsive_mui::apply_mui_responsive_layout;
use super::slot::UiLayoutSlotIndex;

pub fn compute_layout_tree(
    tree: &mut UiTree,
    root_size: UiSize,
) -> Result<UiLayoutEngineSelectionReport, UiTreeError> {
    compute_layout_tree_with_text_measure_cache(tree, root_size, None)
}

pub(crate) fn compute_layout_tree_with_text_measure_cache(
    tree: &mut UiTree,
    root_size: UiSize,
    text_measure_cache: Option<&mut UiTextMeasureCache>,
) -> Result<UiLayoutEngineSelectionReport, UiTreeError> {
    let slot_index = UiLayoutSlotIndex::default();
    compute_layout_tree_with_text_measure_cache_and_slot_index(
        tree,
        root_size,
        text_measure_cache,
        &slot_index,
    )
}

pub(crate) fn compute_layout_tree_with_text_measure_cache_and_slot_index(
    tree: &mut UiTree,
    root_size: UiSize,
    mut text_measure_cache: Option<&mut UiTextMeasureCache>,
    slot_index: &UiLayoutSlotIndex,
) -> Result<UiLayoutEngineSelectionReport, UiTreeError> {
    let profile_layout = std::env::var_os("ZR_UI_LAYOUT_PROFILE").is_some();
    let profile_started = Instant::now();
    assert_layout_pass_stage(UiLayoutPassStage::ResponsiveStyleResolution, 0);
    apply_mui_responsive_layout(tree, root_size)?;
    emit_layout_profile(profile_layout, profile_started, "responsive-style", None);
    slot_index.refresh_for_tree(tree);

    let roots = tree.roots.clone();
    assert_layout_pass_stage(UiLayoutPassStage::Measurement, 1);
    for root_id in &roots {
        emit_layout_profile(
            profile_layout,
            profile_started,
            "measure-start",
            Some(*root_id),
        );
        let _ = measure_node(
            tree,
            *root_id,
            text_measure_cache.as_deref_mut(),
            slot_index,
        )?;
        emit_layout_profile(
            profile_layout,
            profile_started,
            "measure-complete",
            Some(*root_id),
        );
    }

    assert_layout_pass_stage(UiLayoutPassStage::BackendSelection, 2);
    let mut engine_context = UiLayoutPassEngineContext::default();
    assert_layout_pass_stage(UiLayoutPassStage::TaffyBridgeArrangement, 3);
    assert_layout_pass_stage(UiLayoutPassStage::ZirconFallbackArrangement, 4);
    assert_layout_pass_stage(UiLayoutPassStage::ClipAndVirtualWindowPropagation, 5);
    for root_id in roots {
        emit_layout_profile(
            profile_layout,
            profile_started,
            "arrange-start",
            Some(root_id),
        );
        arrange_node(
            tree,
            root_id,
            UiFrame::new(
                0.0,
                0.0,
                root_size.width.max(0.0),
                root_size.height.max(0.0),
            ),
            None,
            slot_index,
            &mut engine_context,
        )?;
        emit_layout_profile(
            profile_layout,
            profile_started,
            "arrange-complete",
            Some(root_id),
        );
    }

    assert_layout_pass_stage(UiLayoutPassStage::SelectionReport, 6);
    emit_layout_profile(profile_layout, profile_started, "selection-report", None);
    Ok(engine_context.finish())
}

fn emit_layout_profile(
    enabled: bool,
    started: Instant,
    stage: &str,
    root_id: Option<zircon_runtime_interface::ui::event_ui::UiNodeId>,
) {
    if !enabled {
        return;
    }
    eprintln!(
        "ui-layout-profile stage={stage} elapsed_ms={} root_id={root_id:?}",
        started.elapsed().as_millis(),
    );
}
