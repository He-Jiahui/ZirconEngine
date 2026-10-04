//! 回退候选的 cmap 覆盖预筛：已知覆盖压缩为有序区间，未知覆盖不在此阶段误拒绝。
use std::collections::HashSet;

const HASH_DEDUP_CODEPOINT_THRESHOLD: usize = 128;

/// Unknown 表示尚不能证明缺字，可保留候选；最终字形能力仍由后续 shaping/raster 阶段决定。
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum FontCoverage {
    Known(Vec<(u32, u32)>),
    Unknown,
}

impl FontCoverage {
    pub(super) fn from_codepoint_values(codepoints: Vec<u32>) -> Self {
        Self::from_sorted_unique_codepoints(normalize_codepoint_values(codepoints))
    }

    /// Compacts a canonical codepoint stream without copying or re-sorting it.
    pub(super) fn from_sorted_unique_codepoints(codepoints: impl IntoIterator<Item = u32>) -> Self {
        let mut codepoints = codepoints.into_iter();
        let Some(mut start) = codepoints.next() else {
            return Self::Unknown;
        };

        let mut end = start;
        let mut ranges = Vec::new();
        for codepoint in codepoints {
            if codepoint <= end {
                continue;
            }
            if codepoint == end.saturating_add(1) {
                end = codepoint;
                continue;
            }
            ranges.push((start, end));
            start = codepoint;
            end = codepoint;
        }
        ranges.push((start, end));
        Self::Known(ranges)
    }

    pub(super) fn contains(&self, codepoint: char) -> bool {
        match self {
            Self::Known(ranges) => {
                let codepoint = codepoint as u32;
                ranges
                    .binary_search_by(|(start, end)| {
                        if *end < codepoint {
                            std::cmp::Ordering::Less
                        } else if *start > codepoint {
                            std::cmp::Ordering::Greater
                        } else {
                            std::cmp::Ordering::Equal
                        }
                    })
                    .is_ok()
            }
            Self::Unknown => true,
        }
    }

    #[cfg(test)]
    pub(super) fn from_codepoints(codepoints: &[char]) -> Self {
        let codepoints = codepoints
            .iter()
            .map(|codepoint| *codepoint as u32)
            .collect::<Vec<_>>();
        Self::from_codepoint_values(codepoints)
    }
}

fn normalize_codepoint_values(mut codepoints: Vec<u32>) -> Vec<u32> {
    if codepoints.len() < HASH_DEDUP_CODEPOINT_THRESHOLD {
        codepoints.sort_unstable();
        codepoints.dedup();
        return codepoints;
    }

    let mut unique = HashSet::with_capacity(codepoints.len());
    for codepoint in codepoints {
        unique.insert(codepoint);
    }
    let mut codepoints = unique.into_iter().collect::<Vec<_>>();
    codepoints.sort_unstable();
    codepoints
}

#[cfg(test)]
#[path = "tests/coverage.rs"]
mod tests;

#[cfg(test)]
#[path = "coverage/tests/hash_dedup_tests.rs"]
mod hash_dedup_tests;
