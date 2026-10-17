//! 一维尺寸分配内核：先规整约束，再按优先级分组增长或缩减。
//! 本模块不访问界面树、不写入几何；调用方自行处理间隙、槽位和结果发布。

use zircon_runtime_interface::ui::layout::{AxisConstraint, ResolvedAxisConstraint, StretchMode};

// 以输入长度单位判断余量、饱和和进展；这是绝对容差，不按容器大小缩放。
const EPSILON: f32 = 0.001;

/// 为按顺序给出的一组同轴约束分配可用长度。
///
/// # 参数
/// - `available`：与约束使用同一单位的长度；负值按零处理，不自动扣除间隙或内边距。
/// - `constraints`：只在本次调用中借用的约束序列，先经 [AxisConstraint::resolved] 规整。
///
/// # 返回
/// 独立拥有的结果向量，与输入顺序和数量一一对应；分配结果写在每项的 `resolved` 中。
/// 增长先处理高优先级，缩减先处理低优先级，同级按规整后的权重分摊。
/// `Fixed` 不参与增长，但仍可缩减；各最小值之和超过可用长度时不会突破最小值来强行装下。
///
/// # 约束与生命周期
/// 不保证填满全部空间，也不另作浮点有限性准入；数值规整边界沿用接口解析方法。
/// 返回值不借用输入；需要跨布局复用临时容量的内部调用使用另一个工作区入口。
pub fn solve_axis_constraints(
    available: f32,
    constraints: &[AxisConstraint],
) -> Vec<ResolvedAxisConstraint> {
    let mut resolved = Vec::with_capacity(constraints.len());
    let mut priorities = Vec::new();
    let mut active_indices = Vec::new();
    solve_axis_constraints_into(
        available,
        constraints,
        &mut resolved,
        &mut priorities,
        &mut active_indices,
    );
    resolved
}

/// 将分配结果写入调用方工作区，供重复布局保留向量容量。
///
/// # 参数
/// - `available`、`constraints`：与 [solve_axis_constraints] 相同的长度和有序输入。
/// - `resolved`：本次结果，开始时清空并以规整后的约束重建。
/// - `priorities`、`active_indices`：本次计算独占的临时列表，其最终内容不是额外输出契约。
///
/// # 约束与生命周期
/// 三个可变借用仅持续本次计算，清空长度不主动释放容量；新规模仍可能需要扩容。
/// 调用结束后不保存任何输入借用，也不修改原约束序列。
pub(crate) fn solve_axis_constraints_into(
    available: f32,
    constraints: &[AxisConstraint],
    resolved: &mut Vec<ResolvedAxisConstraint>,
    priorities: &mut Vec<i32>,
    active_indices: &mut Vec<usize>,
) {
    let available = available.max(0.0);
    resolved.clear();
    priorities.clear();
    active_indices.clear();
    resolved.extend(constraints.iter().copied().map(AxisConstraint::resolved));
    let mut total: f32 = resolved.iter().map(|axis| axis.resolved).sum();

    let needs_final_clamp = if total + EPSILON < available {
        priorities_descending(
            resolved,
            |axis| {
                axis.stretch_mode == StretchMode::Stretch
                    && axis.max.is_none_or(|max| axis.resolved + EPSILON < max)
            },
            priorities,
        );
        let mut remaining = available - total;
        for priority_index in 0..priorities.len() {
            if remaining <= EPSILON {
                break;
            }
            remaining = distribute_growth(
                resolved,
                priorities[priority_index],
                remaining,
                active_indices,
            );
        }
        true
    } else if total > available + EPSILON {
        priorities_ascending(
            resolved,
            |axis| axis.resolved > axis.min + EPSILON,
            priorities,
        );
        let mut deficit = total - available;
        for priority_index in 0..priorities.len() {
            if deficit <= EPSILON {
                break;
            }
            deficit = distribute_shrink(
                resolved,
                priorities[priority_index],
                deficit,
                active_indices,
            );
        }
        true
    } else {
        false
    };

    if needs_final_clamp {
        total = resolved.iter().map(|axis| axis.resolved).sum();
    }
    // 分组分配后若仍超出容差，按原输入顺序继续消减可缩空间；最小值不可突破。
    if needs_final_clamp && total > available + EPSILON {
        let mut deficit = total - available;
        for axis in resolved.iter_mut() {
            if deficit <= EPSILON {
                break;
            }
            let shrink = (axis.resolved - axis.min).max(0.0).min(deficit);
            axis.resolved -= shrink;
            deficit -= shrink;
        }
    }
}

#[cfg(test)]
#[path = "constraints/tests/final_sum_tests.rs"]
mod final_sum_tests;

fn priorities_descending(
    resolved: &[ResolvedAxisConstraint],
    filter: impl Fn(&ResolvedAxisConstraint) -> bool,
    priorities: &mut Vec<i32>,
) {
    priorities.clear();
    priorities.reserve(resolved.len());
    priorities.extend(
        resolved
            .iter()
            .filter(|axis| filter(axis))
            .map(|axis| axis.priority),
    );
    priorities.sort_unstable();
    priorities.dedup();
    priorities.reverse();
}

fn priorities_ascending(
    resolved: &[ResolvedAxisConstraint],
    filter: impl Fn(&ResolvedAxisConstraint) -> bool,
    priorities: &mut Vec<i32>,
) {
    priorities.clear();
    priorities.reserve(resolved.len());
    priorities.extend(
        resolved
            .iter()
            .filter(|axis| filter(axis))
            .map(|axis| axis.priority),
    );
    priorities.sort_unstable();
    priorities.dedup();
}

fn distribute_growth(
    resolved: &mut [ResolvedAxisConstraint],
    priority: i32,
    remaining: f32,
    active_indices: &mut Vec<usize>,
) -> f32 {
    let mut remaining = remaining;
    active_indices.reserve(resolved.len());
    loop {
        active_indices.clear();
        active_indices.extend(
            resolved
                .iter()
                .enumerate()
                .filter(|(_, axis)| {
                    axis.priority == priority
                        && axis.stretch_mode == StretchMode::Stretch
                        && axis.max.is_none_or(|max| axis.resolved + EPSILON < max)
                })
                .map(|(index, _)| index),
        );
        if active_indices.is_empty() || remaining <= EPSILON {
            return remaining;
        }
        let weight_sum: f32 = active_indices
            .iter()
            .map(|index| resolved[*index].weight)
            .sum();
        let active_count = active_indices.len() as f32;
        let mut consumed = 0.0;
        // 同一轮的各项使用同一份余量，饱和项退出后再把未消耗部分交给下一轮。
        for index in active_indices.iter().copied() {
            let share = if weight_sum <= EPSILON {
                remaining / active_count
            } else {
                remaining * (resolved[index].weight / weight_sum)
            };
            let capacity = resolved[index]
                .max
                .map(|max| (max - resolved[index].resolved).max(0.0))
                .unwrap_or(share);
            let delta = share.min(capacity);
            resolved[index].resolved += delta;
            consumed += delta;
        }
        if consumed <= EPSILON {
            return remaining;
        }
        remaining -= consumed;
    }
}

// 缩减候选包含 Fixed 项；增长与缩减共用同一个已规整权重，不引入独立的缩减系数。
fn distribute_shrink(
    resolved: &mut [ResolvedAxisConstraint],
    priority: i32,
    deficit: f32,
    active_indices: &mut Vec<usize>,
) -> f32 {
    let mut deficit = deficit;
    active_indices.reserve(resolved.len());
    loop {
        active_indices.clear();
        active_indices.extend(
            resolved
                .iter()
                .enumerate()
                .filter(|(_, axis)| axis.priority == priority && axis.resolved > axis.min + EPSILON)
                .map(|(index, _)| index),
        );
        if active_indices.is_empty() || deficit <= EPSILON {
            return deficit;
        }
        let weight_sum: f32 = active_indices
            .iter()
            .map(|index| resolved[*index].weight)
            .sum();
        let active_count = active_indices.len() as f32;
        let mut consumed = 0.0;
        for index in active_indices.iter().copied() {
            let share = if weight_sum <= EPSILON {
                deficit / active_count
            } else {
                deficit * (resolved[index].weight / weight_sum)
            };
            let capacity = (resolved[index].resolved - resolved[index].min).max(0.0);
            let delta = share.min(capacity);
            resolved[index].resolved -= delta;
            consumed += delta;
        }
        if consumed <= EPSILON {
            return deficit;
        }
        deficit -= consumed;
    }
}

#[cfg(test)]
#[path = "tests/constraints.rs"]
mod tests;
