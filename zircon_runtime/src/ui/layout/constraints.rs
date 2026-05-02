use zircon_runtime_interface::ui::layout::{AxisConstraint, ResolvedAxisConstraint, StretchMode};

const EPSILON: f32 = 0.001;

pub fn solve_axis_constraints(
    available: f32,
    constraints: &[AxisConstraint],
) -> Vec<ResolvedAxisConstraint> {
    let available = available.max(0.0);
    let mut resolved: Vec<_> = constraints
        .iter()
        .copied()
        .map(AxisConstraint::resolved)
        .collect();
    let mut total: f32 = resolved.iter().map(|axis| axis.resolved).sum();

    if total + EPSILON < available {
        let priorities = priorities_descending(&resolved, |axis| {
            axis.stretch_mode == StretchMode::Stretch
                && axis.max.is_none_or(|max| axis.resolved + EPSILON < max)
        });
        let mut remaining = available - total;
        for priority in priorities {
            if remaining <= EPSILON {
                break;
            }
            remaining = distribute_growth(&mut resolved, priority, remaining);
        }
    } else if total > available + EPSILON {
        let priorities = priorities_ascending(&resolved, |axis| axis.resolved > axis.min + EPSILON);
        let mut deficit = total - available;
        for priority in priorities {
            if deficit <= EPSILON {
                break;
            }
            deficit = distribute_shrink(&mut resolved, priority, deficit);
        }
    }

    total = resolved.iter().map(|axis| axis.resolved).sum();
    if total > available + EPSILON {
        let mut deficit = total - available;
        for axis in &mut resolved {
            if deficit <= EPSILON {
                break;
            }
            let shrink = (axis.resolved - axis.min).max(0.0).min(deficit);
            axis.resolved -= shrink;
            deficit -= shrink;
        }
    }

    resolved
}

fn priorities_descending(
    resolved: &[ResolvedAxisConstraint],
    filter: impl Fn(&ResolvedAxisConstraint) -> bool,
) -> Vec<i32> {
    let mut priorities: Vec<_> = resolved
        .iter()
        .filter(|axis| filter(axis))
        .map(|axis| axis.priority)
        .collect();
    priorities.sort_unstable();
    priorities.dedup();
    priorities.reverse();
    priorities
}

fn priorities_ascending(
    resolved: &[ResolvedAxisConstraint],
    filter: impl Fn(&ResolvedAxisConstraint) -> bool,
) -> Vec<i32> {
    let mut priorities: Vec<_> = resolved
        .iter()
        .filter(|axis| filter(axis))
        .map(|axis| axis.priority)
        .collect();
    priorities.sort_unstable();
    priorities.dedup();
    priorities
}

fn distribute_growth(
    resolved: &mut [ResolvedAxisConstraint],
    priority: i32,
    remaining: f32,
) -> f32 {
    let mut remaining = remaining;
    loop {
        let indices: Vec<_> = resolved
            .iter()
            .enumerate()
            .filter(|(_, axis)| {
                axis.priority == priority
                    && axis.stretch_mode == StretchMode::Stretch
                    && axis.max.is_none_or(|max| axis.resolved + EPSILON < max)
            })
            .map(|(index, _)| index)
            .collect();
        if indices.is_empty() || remaining <= EPSILON {
            return remaining;
        }
        let weight_sum: f32 = indices.iter().map(|index| resolved[*index].weight).sum();
        let active_count = indices.len() as f32;
        let mut consumed = 0.0;
        for index in indices {
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

fn distribute_shrink(resolved: &mut [ResolvedAxisConstraint], priority: i32, deficit: f32) -> f32 {
    let mut deficit = deficit;
    loop {
        let indices: Vec<_> = resolved
            .iter()
            .enumerate()
            .filter(|(_, axis)| axis.priority == priority && axis.resolved > axis.min + EPSILON)
            .map(|(index, _)| index)
            .collect();
        if indices.is_empty() || deficit <= EPSILON {
            return deficit;
        }
        let weight_sum: f32 = indices.iter().map(|index| resolved[*index].weight).sum();
        let active_count = indices.len() as f32;
        let mut consumed = 0.0;
        for index in indices {
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

fn clamp_axis_value(value: f32, min: f32, max: Option<f32>) -> f32 {
    max.map(|max| value.clamp(min, max))
        .unwrap_or_else(|| value.max(min))
}
