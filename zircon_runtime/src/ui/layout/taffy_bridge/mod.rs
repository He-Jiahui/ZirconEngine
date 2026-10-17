//! 将受支持的布局契约投影为 Taffy 样式，并保留每个父节点的布局产品。
//! 样式转换、调用前的契约准入和实际求解分属不同步骤；样式转换成功本身不代表整棵子树可由 Taffy 处理。
//! 父级安排代码决定 Full/Exact 输入范围，并在桥接错误时记录后端回退。
#[path = "compute.rs"]
mod compute;
#[path = "product_cache.rs"]
mod product_cache;

use taffy::style::{Display, Style};
use zircon_runtime_interface::ui::layout::{
    BoxConstraints, UiContainerKind, UiLayoutEngineFamily, UiLayoutEngineRequest,
};

pub(crate) use compute::{
    compute_taffy_child_frames, taffy_main_axis, taffy_supports_axis_constraint_priority,
    taffy_supports_child_layout_values, taffy_supports_parent_layout_values,
    taffy_supports_slot_alignment, taffy_supports_slot_layout_values, taffy_supports_slot_padding,
    TaffyChildLayoutInput, TaffyLayoutBridgeScratch,
};
pub(crate) use product_cache::{TaffyParentProductCache, TaffyParentProductUpdate};

use super::style_mapping::{
    taffy_display_for_family as mapped_taffy_display_for_family, taffy_style_from_ui_layout_style,
    ui_layout_style_from_container,
};

/// 父产品更新时允许提供的子契约范围。
///
/// `Full` 提供完整有序子项并允许协调拓扑；`Exact` 只提供已保留产品中的变更子集，
/// 要求顺序修订匹配且子项仍在产品中。实际子帧发布范围还取决于计算、父框架和继承裁剪是否复用。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum UiTaffyChildContractScope {
    Full,
    Exact,
}

/// Converts the subset of Zircon layout contracts that can be solved by Taffy.
/// Overlay, Canvas, Popup, Scroll, and VirtualList stay outside this bridge by design.
///
/// 将容器和盒约束转换成后端样式，供桥接建立父节点样式。
///
/// # 参数与返回
/// `container` 决定布局族；`constraints` 提供轴约束。族不归 Taffy 所有或任一转换失败时返回 `None`。
/// 成功值只是拥有自身数据的样式，不包含树节点借用，也不执行求解。
///
/// # 参考
/// 调用方仍需检查父子数值、槽位和对齐契约；布局族显示类型映射见 `taffy_display_for_family`。
pub fn taffy_style_for_container(
    container: UiContainerKind,
    constraints: BoxConstraints,
) -> Option<Style> {
    let request = UiLayoutEngineRequest::from_container_kind(container);
    request.family.is_taffy_owned().then_some(())?;
    let style = ui_layout_style_from_container(container, constraints).ok()?;
    taffy_style_from_ui_layout_style(&style).ok()
}

/// 查询布局族对应的 Taffy 显示类型，供样式投影使用。
///
/// # 参数与返回
/// `family` 是引擎契约中的布局族；返回静态映射的显示类型或 `None`。
/// 本查询不读取树、不选择执行后端；布局族定义见 `UiLayoutEngineFamily`。
pub fn taffy_display_for_family(family: UiLayoutEngineFamily) -> Option<Display> {
    mapped_taffy_display_for_family(family)
}
