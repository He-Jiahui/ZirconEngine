//! 为 Runtime UI 提供完整与增量布局入口，并导出供诊断读取的阶段顺序。
//! 独立布局会创建临时槽位索引；保留表面可沿用调用方持有的索引，具体阶段由子模块实现。
#[path = "arrange.rs"]
mod arrange;
#[path = "axis.rs"]
mod axis;
#[path = "child_frame.rs"]
mod child_frame;
#[path = "clip.rs"]
mod clip;
#[path = "engine.rs"]
mod engine;
#[path = "incremental.rs"]
mod incremental;
#[path = "inline_widgets.rs"]
mod inline_widgets;
#[path = "layout_tree.rs"]
mod layout_tree;
#[path = "material.rs"]
mod material;
#[path = "measure.rs"]
mod measure;
#[path = "pipeline.rs"]
mod pipeline;
#[path = "responsive_mui.rs"]
mod responsive_mui;
#[path = "slot.rs"]
mod slot;
#[path = "taffy_arrange.rs"]
mod taffy_arrange;
#[path = "virtual_list_layout.rs"]
mod virtual_list_layout;
#[path = "workspace.rs"]
mod workspace;

pub(crate) use incremental::compute_incremental_layout_tree_with_text_measure_cache;
pub use layout_tree::compute_layout_tree;
pub(crate) use layout_tree::compute_layout_tree_with_text_measure_cache_and_slot_index;
pub use pipeline::{ui_layout_pass_stage_names, UiLayoutPassStage, UI_LAYOUT_PASS_ORDER};
pub(crate) use slot::UiLayoutSlotIndex;
