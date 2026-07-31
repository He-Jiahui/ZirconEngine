#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum FontCoverage {
    Known(Vec<(u32, u32)>),
    Unknown,
}

impl FontCoverage {
    pub(super) fn from_codepoint_values(mut codepoints: Vec<u32>) -> Self {
        codepoints.sort_unstable();
        codepoints.dedup();
        Self::from_sorted_unique_codepoints(codepoints)
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

#[cfg(test)]
mod tests {
    use super::FontCoverage;

    #[test]
    fn coverage_contains_uses_ordered_ranges_without_gap_false_positives() {
        let coverage = FontCoverage::Known(vec![(0x0020, 0x007E), (0x0400, 0x04FF)]);

        assert!(coverage.contains('A'));
        assert!(coverage.contains('Ж'));
        assert!(!coverage.contains('中'));
    }
}
