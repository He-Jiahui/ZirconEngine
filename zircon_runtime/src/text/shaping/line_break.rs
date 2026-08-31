use unicode_linebreak::{BreakOpportunity, linebreaks};

use crate::text::{
    LineBreakTailoringProfile, ShapedGlyphLineBreakOpportunity, ShapedGlyphLineBreakReceipt,
    UnicodeDataSnapshotId, compiled_unicode_data_snapshot_id,
};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct ClusterLineBreakFlags {
    pub soft_break: bool,
    pub mandatory_break: bool,
    pub receipt: ShapedGlyphLineBreakReceipt,
}

impl ClusterLineBreakFlags {
    pub(crate) fn receipt_for_cluster(
        self,
        cluster_start: bool,
        mandatory_control: bool,
    ) -> ShapedGlyphLineBreakReceipt {
        if !cluster_start {
            return ShapedGlyphLineBreakReceipt::default();
        }
        if mandatory_control
            && matches!(
                self.receipt.opportunity,
                ShapedGlyphLineBreakOpportunity::None
            )
        {
            return ShapedGlyphLineBreakReceipt::mandatory_control();
        }
        self.receipt
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum LineBreakKind {
    Soft,
    Mandatory,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct LineBreakOpportunity {
    byte_index: usize,
    kind: LineBreakKind,
}

#[derive(Clone, Debug)]
pub(crate) struct LineBreakOpportunityMap {
    opportunities: Vec<LineBreakOpportunity>,
    unicode_data_snapshot: UnicodeDataSnapshotId,
    tailoring_profile: LineBreakTailoringProfile,
}

impl LineBreakOpportunityMap {
    pub(crate) fn new(text: &str) -> Self {
        Self::for_snapshot(text, compiled_unicode_data_snapshot_id())
    }

    pub(crate) fn for_snapshot(text: &str, unicode_data_snapshot: UnicodeDataSnapshotId) -> Self {
        #[cfg(any(test, feature = "profiling", feature = "profiling-tracy"))]
        let profile_started = super::analysis_profile::start_build();
        let opportunities = linebreaks(text)
            .filter_map(|(byte_index, opportunity)| match opportunity {
                BreakOpportunity::Allowed => Some(LineBreakOpportunity {
                    byte_index,
                    kind: LineBreakKind::Soft,
                }),
                BreakOpportunity::Mandatory if is_content_mandatory_break(text, byte_index) => {
                    Some(LineBreakOpportunity {
                        byte_index,
                        kind: LineBreakKind::Mandatory,
                    })
                }
                BreakOpportunity::Mandatory => None,
            })
            .collect();

        let map = Self {
            opportunities,
            unicode_data_snapshot,
            tailoring_profile: LineBreakTailoringProfile::UnicodeDefault,
        };
        #[cfg(any(test, feature = "profiling", feature = "profiling-tracy"))]
        super::analysis_profile::record_line_break_build(text.len(), profile_started);
        map
    }

    pub(crate) const fn unicode_data_snapshot(&self) -> UnicodeDataSnapshotId {
        self.unicode_data_snapshot
    }

    pub(crate) fn flags_for_cluster(
        &self,
        visual_start: usize,
        visual_end: usize,
    ) -> ClusterLineBreakFlags {
        if visual_start > visual_end {
            return ClusterLineBreakFlags::default();
        }
        let first = self
            .opportunities
            .partition_point(|opportunity| opportunity.byte_index < visual_start);
        let end = self
            .opportunities
            .partition_point(|opportunity| opportunity.byte_index <= visual_end);
        let mut flags = ClusterLineBreakFlags {
            receipt: ShapedGlyphLineBreakReceipt {
                profile: self.tailoring_profile,
                opportunity: ShapedGlyphLineBreakOpportunity::None,
            },
            ..ClusterLineBreakFlags::default()
        };
        for opportunity in &self.opportunities[first..end] {
            match opportunity.kind {
                LineBreakKind::Soft if opportunity.byte_index == visual_end => {
                    flags.soft_break = true;
                    flags.receipt.opportunity = ShapedGlyphLineBreakOpportunity::ProviderAllowed;
                }
                LineBreakKind::Mandatory if opportunity.byte_index > visual_start => {
                    flags.mandatory_break = true;
                    flags.receipt.opportunity = ShapedGlyphLineBreakOpportunity::ProviderMandatory;
                }
                _ => {}
            }
        }
        flags
    }
}

impl Default for LineBreakOpportunityMap {
    fn default() -> Self {
        Self::new("")
    }
}

fn is_content_mandatory_break(text: &str, byte_index: usize) -> bool {
    let preceding_text = text.get(..byte_index.min(text.len())).unwrap_or_default();
    preceding_text
        .chars()
        .next_back()
        .is_some_and(is_mandatory_break_control)
}

pub(super) fn contains_mandatory_break_control(text: &str) -> bool {
    text.chars().any(is_mandatory_break_control)
}

const fn is_mandatory_break_control(ch: char) -> bool {
    matches!(
        ch,
        '\n' | '\r' | '\u{000b}' | '\u{000c}' | '\u{0085}' | '\u{2028}' | '\u{2029}'
    )
}

#[cfg(test)]
mod tests {
    use super::{
        ClusterLineBreakFlags, LineBreakKind, LineBreakOpportunity, LineBreakOpportunityMap,
    };
    use crate::text::{
        LineBreakTailoringProfile, ShapedGlyphLineBreakOpportunity, ShapedGlyphLineBreakReceipt,
        compiled_unicode_data_snapshot_id,
    };

    const fn unicode_default_receipt(
        opportunity: ShapedGlyphLineBreakOpportunity,
    ) -> ShapedGlyphLineBreakReceipt {
        ShapedGlyphLineBreakReceipt {
            profile: LineBreakTailoringProfile::UnicodeDefault,
            opportunity,
        }
    }

    #[test]
    fn cluster_flags_only_visit_the_cluster_opportunity_window() {
        let map = LineBreakOpportunityMap {
            opportunities: vec![
                LineBreakOpportunity {
                    byte_index: 2,
                    kind: LineBreakKind::Soft,
                },
                LineBreakOpportunity {
                    byte_index: 4,
                    kind: LineBreakKind::Mandatory,
                },
                LineBreakOpportunity {
                    byte_index: 8,
                    kind: LineBreakKind::Soft,
                },
            ],
            unicode_data_snapshot: compiled_unicode_data_snapshot_id(),
            tailoring_profile: LineBreakTailoringProfile::UnicodeDefault,
        };

        assert_eq!(
            map.flags_for_cluster(0, 2),
            ClusterLineBreakFlags {
                soft_break: true,
                mandatory_break: false,
                receipt: unicode_default_receipt(ShapedGlyphLineBreakOpportunity::ProviderAllowed),
            }
        );
        assert_eq!(
            map.flags_for_cluster(2, 4),
            ClusterLineBreakFlags {
                soft_break: false,
                mandatory_break: true,
                receipt: unicode_default_receipt(
                    ShapedGlyphLineBreakOpportunity::ProviderMandatory,
                ),
            }
        );
        assert_eq!(
            map.flags_for_cluster(5, 8),
            ClusterLineBreakFlags {
                soft_break: true,
                mandatory_break: false,
                receipt: unicode_default_receipt(ShapedGlyphLineBreakOpportunity::ProviderAllowed),
            }
        );
    }

    #[test]
    fn cluster_without_an_opportunity_still_records_the_analysis_profile() {
        let map = LineBreakOpportunityMap::new("alpha beta");

        assert_eq!(
            map.flags_for_cluster(0, 1).receipt,
            unicode_default_receipt(ShapedGlyphLineBreakOpportunity::None)
        );
    }

    #[test]
    fn cluster_flags_do_not_restore_a_full_opportunity_fold() {
        let source = include_str!("line_break.rs");
        let compact = source
            .chars()
            .filter(|character| !character.is_whitespace())
            .collect::<String>();

        assert!(!compact.contains(concat!("self.opportunities.iter()", ".fold")));
    }

    #[test]
    fn line_break_analysis_retains_request_unicode_snapshot() {
        let current = compiled_unicode_data_snapshot_id();
        let next = current.with_generation_for_test(current.generation() + 1);
        let map = LineBreakOpportunityMap::for_snapshot("alpha beta", next);

        assert_eq!(map.unicode_data_snapshot(), next);
    }
}
