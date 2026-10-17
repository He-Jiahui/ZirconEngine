//! 在测量、安排和线性兄弟分配之间转换单轴尺寸语义。
//! 这里只读取节点和槽位，结果留在调用方工作区；几何写回由安排阶段完成。

use crate::ui::layout::constraints::solve_axis_constraints_into;
use zircon_runtime_interface::ui::{
    event_ui::UiNodeId,
    layout::{
        AxisConstraint, LayoutBoundary, StretchMode, UiAxis, UiFrame, UiLinearSlotSizeRule,
        UiLinearSlotSizing, UiSize,
    },
    tree::{UiTree, UiTreeError},
};

use super::slot::{slot_for_container_child, slot_padding, UiLayoutSlotIndex};
use super::workspace::UiLinearArrangeScratch;

/// 按给定子项顺序求解线性容器的主轴外部长度，供后续安排逐项取用。
///
/// # 参数
/// - `tree`：本次计算只读的节点和槽位来源，不在工作区保存对它的借用。
/// - `parent_id`、`slot_index`：用于查找父容器所拥有的线性槽位。
/// - `children`：调用方提供的子项顺序；Collapsed 仍占一个零长结果位置，Hidden 正常占位。
/// - `axis`：选择宽度或高度及对应的槽位内边距。
/// - `available_extent`、`gap`：主轴长度和项间距；先扣除实际占位项之间的非负间距。
/// - `scratch`：调用方独占并可跨次复用的约束、结果与求解临时列表。
///
/// # 返回与错误
/// 成功时结果数量与 `children` 相同，包含槽位内边距，供 `arrange_linear_children` 按索引消费。
/// 缺失节点返回 [UiTreeError::MissingNode]；此前收集的约束前缀可能已改变，其他求解列表尚未重算。
/// 错误后的旧结果不能作为本次成功输出使用；此函数不回滚工作区。
///
/// # 约束
/// 逐项核对标识存在性；父子关系和顺序由调用方提供，不在这里验证。
pub(crate) fn resolve_linear_child_main_extents(
    tree: &UiTree,
    parent_id: UiNodeId,
    children: &[UiNodeId],
    axis: UiAxis,
    available_extent: f32,
    gap: f32,
    slot_index: &UiLayoutSlotIndex,
    scratch: &mut UiLinearArrangeScratch,
) -> Result<(), UiTreeError> {
    let mut layout_child_count = 0usize;
    scratch.constraints.clear();

    for child_id in children {
        let node = tree
            .node(*child_id)
            .ok_or(UiTreeError::MissingNode(*child_id))?;
        if !node.effective_visibility().occupies_layout() {
            scratch.constraints.push(collapsed_axis_constraint());
            continue;
        }
        layout_child_count += 1;
        let slot = slot_for_container_child(
            tree,
            slot_index,
            parent_id,
            *child_id,
            linear_container(axis),
        );
        let padding = slot_padding(slot);
        let padding_extent = match axis {
            UiAxis::Horizontal => padding.horizontal(),
            UiAxis::Vertical => padding.vertical(),
        };
        let desired_extent = size_axis_extent(
            UiSize::new(
                node.layout_cache.desired_size.width,
                node.layout_cache.desired_size.height,
            ),
            axis,
        );
        let preserve_stretch = match axis {
            UiAxis::Horizontal => node.layout_stretch_width,
            UiAxis::Vertical => node.layout_stretch_height,
        };
        scratch.constraints.push(linear_main_axis_constraint(
            match axis {
                UiAxis::Horizontal => node.constraints.width,
                UiAxis::Vertical => node.constraints.height,
            },
            desired_extent,
            padding_extent,
            preserve_stretch,
            slot.and_then(|slot| slot.linear_sizing),
        ));
    }

    let gap_total = gap.max(0.0) * layout_child_count.saturating_sub(1) as f32;
    let available_extent = (available_extent - gap_total).max(0.0);

    let UiLinearArrangeScratch {
        constraints,
        resolved,
        priorities,
        active_indices,
    } = scratch;
    solve_axis_constraints_into(
        available_extent,
        constraints,
        resolved,
        priorities,
        active_indices,
    );
    Ok(())
}

pub(crate) fn size_axis_extent(size: UiSize, axis: UiAxis) -> f32 {
    match axis {
        UiAxis::Horizontal => size.width,
        UiAxis::Vertical => size.height,
    }
}

pub(crate) fn frame_axis_extent(frame: UiFrame, axis: UiAxis) -> f32 {
    match axis {
        UiAxis::Horizontal => frame.width,
        UiAxis::Vertical => frame.height,
    }
}

/// 测量阶段的期望轴长：内容驱动边界取内容与偏好值中的较大者，另两种边界只用偏好值。
/// 返回值再受规整后的上下界限制；这里不进行兄弟之间的空间分配。
pub(crate) fn desired_axis(
    layout_boundary: LayoutBoundary,
    constraint: AxisConstraint,
    content_value: f32,
) -> f32 {
    let resolved = constraint.resolved();
    let preferred = match layout_boundary {
        LayoutBoundary::ContentDriven => content_value.max(resolved.preferred),
        LayoutBoundary::ParentDirected | LayoutBoundary::Fixed => resolved.preferred,
    };
    clamp_axis(preferred, resolved.min, resolved.max)
}

/// 将单项安排长度限制到规整区间：Fixed 取测量结果，Stretch 取可用长度。
/// 用于自由子项及线性容器交叉轴等单项场景，不替代主轴兄弟求解。
pub(crate) fn arranged_axis_extent(
    constraint: AxisConstraint,
    desired: f32,
    available: f32,
) -> f32 {
    let resolved = constraint.resolved();
    let base = match constraint.stretch_mode {
        StretchMode::Fixed => desired,
        StretchMode::Stretch => available,
    };
    clamp_axis(base, resolved.min, resolved.max)
}

// 先将槽位内边距并入外部约束，再应用显式线性尺寸规则。
// 无显式规则时，只有原约束完全等于默认值且不要求保留伸展，才把正的含边距期望值固化。
fn linear_main_axis_constraint(
    mut constraint: AxisConstraint,
    desired_extent: f32,
    padding_extent: f32,
    preserve_stretch: bool,
    slot_sizing: Option<UiLinearSlotSizing>,
) -> AxisConstraint {
    let was_default = constraint == AxisConstraint::default();
    let padding_extent = padding_extent.max(0.0);
    constraint = inflate_axis_constraint(constraint, padding_extent);

    let desired_extent = desired_extent.max(0.0) + padding_extent;
    if let Some(slot_sizing) = slot_sizing {
        return apply_linear_slot_sizing(constraint, desired_extent, slot_sizing);
    }
    if desired_extent > 0.0 && was_default && !preserve_stretch {
        constraint.preferred = desired_extent;
        constraint.stretch_mode = StretchMode::Fixed;
    }
    constraint
}

// 槽位下界与节点下界取较大者，上界取交集后抬到下界，避免倒置区间。
// Auto 把缩减值写入统一权重；两种 Stretch 规则都把 value 写入该权重，实际缩减也使用它。
fn apply_linear_slot_sizing(
    mut constraint: AxisConstraint,
    desired_extent: f32,
    sizing: UiLinearSlotSizing,
) -> AxisConstraint {
    let slot_max = (sizing.max >= 0.0).then_some(sizing.max.max(0.0));
    constraint.min = constraint.min.max(sizing.min.max(0.0));
    constraint.max = merge_max_constraint(constraint.max, slot_max);
    if constraint.max >= 0.0 && constraint.max < constraint.min {
        constraint.max = constraint.min;
    }
    match sizing.rule {
        UiLinearSlotSizeRule::Auto => {
            constraint.preferred = desired_extent.max(constraint.preferred).max(constraint.min);
            constraint.weight = sizing.shrink_value.max(0.0);
            constraint.stretch_mode = StretchMode::Fixed;
        }
        UiLinearSlotSizeRule::Stretch => {
            constraint.preferred = constraint.min;
            constraint.weight = sizing.value.max(0.0);
            constraint.stretch_mode = StretchMode::Stretch;
        }
        UiLinearSlotSizeRule::StretchContent => {
            constraint.preferred = desired_extent.max(constraint.preferred).max(constraint.min);
            constraint.weight = sizing.value.max(0.0);
            constraint.stretch_mode = StretchMode::Stretch;
        }
    }
    constraint
}

fn merge_max_constraint(current: f32, slot_max: Option<f32>) -> f32 {
    match (current >= 0.0, slot_max) {
        (true, Some(slot_max)) => current.min(slot_max),
        (true, None) => current,
        (false, Some(slot_max)) => slot_max,
        (false, None) => current,
    }
}

// 仅为非负上界增加边距；负数无上界哨兵保持原值，后续由接口解析为可选上界。
fn inflate_axis_constraint(mut constraint: AxisConstraint, padding_extent: f32) -> AxisConstraint {
    if padding_extent <= 0.0 {
        return constraint;
    }
    constraint.min += padding_extent;
    if constraint.max >= 0.0 {
        constraint.max += padding_extent;
    }
    constraint.preferred += padding_extent;
    constraint
}

fn linear_container(axis: UiAxis) -> zircon_runtime_interface::ui::layout::UiContainerKind {
    match axis {
        UiAxis::Horizontal => {
            zircon_runtime_interface::ui::layout::UiContainerKind::HorizontalBox(Default::default())
        }
        UiAxis::Vertical => {
            zircon_runtime_interface::ui::layout::UiContainerKind::VerticalBox(Default::default())
        }
    }
}

// 为折叠子项保留结果索引，固定零区间不会消耗长度或参与增长。
fn collapsed_axis_constraint() -> AxisConstraint {
    AxisConstraint {
        min: 0.0,
        max: 0.0,
        preferred: 0.0,
        priority: 0,
        weight: 1.0,
        stretch_mode: StretchMode::Fixed,
    }
}

/// 沿滚动堆叠主轴保留期望长度，仅裁剪上下界，不因 Stretch 把每项撑满视口。
pub(crate) fn stacked_axis_extent(constraint: AxisConstraint, desired: f32) -> f32 {
    let resolved = constraint.resolved();
    clamp_axis(desired, resolved.min, resolved.max)
}

fn clamp_axis(value: f32, min: f32, max: Option<f32>) -> f32 {
    max.map(|max| value.clamp(min, max))
        .unwrap_or_else(|| value.max(min))
}

#[cfg(test)]
#[path = "axis/tests/cases.rs"]
mod tests;
