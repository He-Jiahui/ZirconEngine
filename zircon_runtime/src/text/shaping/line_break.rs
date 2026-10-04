use unicode_linebreak::{linebreaks, BreakOpportunity};

use crate::text::{
    compiled_unicode_data_snapshot_id, LineBreakTailoringProfile, ShapedGlyphLineBreakOpportunity,
    ShapedGlyphLineBreakReceipt, UnicodeDataSnapshotId,
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
#[path = "tests/line_break.rs"]
mod tests;
