use zircon_runtime_interface::ui::layout::{AxisConstraint, ResolvedAxisConstraint, StretchMode};

const EPSILON: f32 = 0.001;

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
