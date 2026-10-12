use std::ops::Range;

/// Splits a depth-sorted transmission command list into at most `step_count`
/// non-overlapping ranges, distributing the remainder toward the back-most
/// ranges that render first.
pub(crate) fn transmission_step_range(
    command_count: usize,
    step_count: usize,
    step_index: usize,
) -> Option<Range<usize>> {
    if command_count == 0 || step_count == 0 {
        return None;
    }
    let effective_step_count = step_count.min(command_count);
    if step_index >= effective_step_count {
        return None;
    }

    let commands_per_step = command_count / effective_step_count;
    let remainder = command_count % effective_step_count;
    let start = step_index
        .saturating_mul(commands_per_step)
        .saturating_add(step_index.min(remainder));
    let step_len = commands_per_step + usize::from(step_index < remainder);
    Some(start..start.saturating_add(step_len).min(command_count))
}

#[cfg(test)]
#[path = "tests/steps.rs"]
mod tests;
