use std::cmp::Ordering;

use zircon_runtime::core::framework::physics::PhysicsQueryMode;

pub(crate) fn collect_query_mode<T>(
    candidates: impl Iterator<Item = T>,
    mode: PhysicsQueryMode,
    compare: impl Fn(&T, &T) -> Ordering,
) -> Vec<T> {
    let mut out = Vec::new();
    append_query_mode(&mut out, candidates, mode, compare);
    out
}

pub(crate) fn append_query_mode<T>(
    out: &mut Vec<T>,
    mut candidates: impl Iterator<Item = T>,
    mode: PhysicsQueryMode,
    compare: impl Fn(&T, &T) -> Ordering,
) {
    match mode {
        PhysicsQueryMode::First => {
            if out.is_empty() {
                out.extend(candidates.next());
            } else {
                out.truncate(1);
            }
        }
        PhysicsQueryMode::Closest => {
            out.extend(candidates.min_by(|left, right| compare(left, right)));
            if let Some(index) = out
                .iter()
                .enumerate()
                .min_by(|(_, left), (_, right)| compare(left, right))
                .map(|(index, _)| index)
            {
                out.swap(0, index);
                out.truncate(1);
            }
        }
        PhysicsQueryMode::All => {
            out.extend(candidates);
            out.sort_by(|left, right| compare(left, right));
        }
    }
}

#[cfg(test)]
#[path = "tests/mode.rs"]
mod tests;
