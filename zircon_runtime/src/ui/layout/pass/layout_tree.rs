use web_time::Instant;

use zircon_runtime_interface::ui::layout::{UiFrame, UiLayoutEngineSelectionReport, UiSize};
use zircon_runtime_interface::ui::tree::{UiTree, UiTreeError};

use crate::ui::text::UiTextMeasureCache;

use super::arrange::arrange_node;
use super::engine::UiLayoutPassEngineContext;
use super::inline_widgets::arrange_inline_widget_children;
use super::measure::measure_node;
use super::pipeline::{assert_layout_pass_stage, UiLayoutPassStage};
use super::responsive_mui::apply_mui_responsive_layout;
use super::slot::UiLayoutSlotIndex;

/// Runs a standalone layout pass with one process-default compatibility text cache.
/// Retained product surfaces call the owner-aware layout pass with their existing cache.
///
/// 对可独立计算的树执行响应式解析、全树测量和根安排，并返回本次后端选择报告。
///
/// # 参数
/// - `tree`：原地更新的树；根、父子边及节点引用应满足 `UiTree` 的结构契约。
/// - `root_size`：每个根的可用布局尺寸；安排根时将负宽高钳到零。
///
/// # 返回与失败
/// 成功返回后端选择诊断，几何结果写入 `tree`。错误可能发生在部分节点已经更新之后，
/// 调用方不能用旧报告推断树仍是旧几何。
///
/// # 生命周期与参考
/// 文本缓存和槽位索引只服务本次调用；普通 `Result` 返回前结束文本缓存帧。
/// 长期持有的表面使用 `compute_layout_tree_with_text_measure_cache_and_slot_index` 复用所属缓存。
pub fn compute_layout_tree(
    tree: &mut UiTree,
    root_size: UiSize,
) -> Result<UiLayoutEngineSelectionReport, UiTreeError> {
    let mut text_measure_cache = UiTextMeasureCache::default();
    text_measure_cache.begin_frame();
    let result =
        compute_layout_tree_with_text_measure_cache(tree, root_size, &mut text_measure_cache);
    text_measure_cache.finish_frame();
    result
}

fn compute_layout_tree_with_text_measure_cache(
    tree: &mut UiTree,
    root_size: UiSize,
    text_measure_cache: &mut UiTextMeasureCache,
) -> Result<UiLayoutEngineSelectionReport, UiTreeError> {
    let slot_index = UiLayoutSlotIndex::default();
    compute_layout_tree_with_text_measure_cache_and_slot_index(
        tree,
        root_size,
        text_measure_cache,
        &slot_index,
    )
}

/// 使用表面持有的文本缓存和索引执行完整布局，供完整重建路径调用。
///
/// # 参数与顺序
/// `tree` 和 `root_size` 是待发布几何的树与根尺寸；`text_measure_cache`、`slot_index`
/// 由同一布局所有者跨调用保留。先解析响应式属性并重建索引，再测量所有根、安排根和内联组件。
///
/// # 返回与生命周期
/// 返回本次选择报告或结构错误；错误不回滚已经写入的树状态。
/// 本入口不开始或结束文本缓存帧，帧边界由表面调用方管理。
///
/// # 参考
/// 独立调用使用 `compute_layout_tree`；增量重建使用 `super::compute_incremental_layout_tree_with_text_measure_cache`。
pub(crate) fn compute_layout_tree_with_text_measure_cache_and_slot_index(
    tree: &mut UiTree,
    root_size: UiSize,
    text_measure_cache: &mut UiTextMeasureCache,
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
        let _ = measure_node(tree, *root_id, &mut *text_measure_cache, slot_index)?;
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
    for root_id in roots.iter().copied() {
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

    arrange_inline_widget_children(
        tree,
        &roots,
        text_measure_cache,
        slot_index,
        &mut engine_context,
    )?;

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
