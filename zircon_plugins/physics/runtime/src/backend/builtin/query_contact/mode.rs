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
mod tests {
    use std::cell::Cell;

    use super::*;

    #[test]
    fn query_modes_preserve_first_closest_and_sorted_all_contracts() {
        let values = [5_u32, 2, 4, 1, 3];

        assert_eq!(
            collect_query_mode(values.into_iter(), PhysicsQueryMode::First, u32::cmp),
            vec![5]
        );
        assert_eq!(
            collect_query_mode(values.into_iter(), PhysicsQueryMode::Closest, u32::cmp),
            vec![1]
        );
        assert_eq!(
            collect_query_mode(values.into_iter(), PhysicsQueryMode::All, u32::cmp),
            vec![1, 2, 3, 4, 5]
        );
    }

    #[test]
    fn first_mode_stops_after_the_first_candidate() {
        let visited = Cell::new(0_usize);
        let values = (0..100_000).inspect(|_| visited.set(visited.get() + 1));

        let result = collect_query_mode(values, PhysicsQueryMode::First, usize::cmp);

        assert_eq!(result, vec![0]);
        assert_eq!(visited.get(), 1);
    }

    #[test]
    fn append_mode_retains_the_best_existing_result() {
        let mut out = vec![2_u32];

        append_query_mode(
            &mut out,
            [5_u32, 3, 4].into_iter(),
            PhysicsQueryMode::Closest,
            u32::cmp,
        );

        assert_eq!(out, vec![2]);
    }

    #[test]
    fn first_mode_truncates_prefilled_output_to_its_first_result() {
        let mut out = vec![7_u32, 3, 5];

        append_query_mode(
            &mut out,
            [1_u32, 2].into_iter(),
            PhysicsQueryMode::First,
            u32::cmp,
        );

        assert_eq!(out, vec![7]);
    }
}
