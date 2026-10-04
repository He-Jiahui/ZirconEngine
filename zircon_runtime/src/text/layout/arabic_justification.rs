use crate::text::{text_glyph_clusters, FontFaceId, InstancedFaceId, ShapedGlyph, TextRange};

use super::MeasuredTextLine;

const ARABIC_TATWEEL: &str = "\u{0640}";
const TATWEEL_ADVANCE_EPSILON: f32 = 0.01;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct ArabicTatweelCandidateReceipt {
    width: f32,
    insertion_count: usize,
}

impl ArabicTatweelCandidateReceipt {
    pub(crate) const fn width(self) -> f32 {
        self.width
    }

    pub(crate) const fn insertion_count(self) -> usize {
        self.insertion_count
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ArabicTatweelCandidateRejection {
    MissingInsertion,
    InvalidSourceIdentity,
    InvalidInsertionRange,
    MalformedClusterOrder,
    MissingInsertionCluster,
    MixedSourceCluster,
    MissingJoiningContext,
    InvalidGlyph,
    MissingFace,
    MixedClusterFace,
    JoiningFaceMismatch,
    NonRtlJoiningContext,
    NonExpandingCandidate,
}

impl ArabicTatweelCandidateRejection {
    /// Stable low-cardinality code for aggregate profiling receipts. Zero means no rejection.
    pub(crate) const fn profile_code(self) -> usize {
        match self {
            Self::MissingInsertion => 1,
            Self::InvalidSourceIdentity => 2,
            Self::InvalidInsertionRange => 3,
            Self::MalformedClusterOrder => 4,
            Self::MissingInsertionCluster => 5,
            Self::MixedSourceCluster => 6,
            Self::MissingJoiningContext => 7,
            Self::InvalidGlyph => 8,
            Self::MissingFace => 9,
            Self::MixedClusterFace => 10,
            Self::JoiningFaceMismatch => 11,
            Self::NonRtlJoiningContext => 12,
            Self::NonExpandingCandidate => 13,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct ClusterFaceIdentity {
    face: FontFaceId,
    instance: Option<InstancedFaceId>,
}

#[derive(Clone, Copy)]
struct PreviousCluster {
    face: ClusterFaceIdentity,
    right_to_left: Option<bool>,
}

/// Validates a fully shaped Tatweel candidate before UI materializes zero-width source anchors.
///
/// Unicode Joining_Type only admits a candidate. This receipt additionally proves that the
/// selected backend produced an independent, visible Tatweel cluster in the same face/instance
/// and RTL context as both neighbors. The later logical-virtual projection can therefore map the
/// cluster to a zero-width source anchor without mixing generated and source-owned content.
pub(crate) fn validate_arabic_tatweel_candidate(
    measured: &MeasuredTextLine,
    candidate: &str,
    insertion_offsets: &[usize],
    natural_width: f32,
) -> Result<ArabicTatweelCandidateReceipt, ArabicTatweelCandidateRejection> {
    // 只有插入簇及两侧 RTL 邻接簇属于同一字体身份且确实增加宽度，调用层才接受虚拟锚点投影。
    if insertion_offsets.is_empty() {
        return Err(ArabicTatweelCandidateRejection::MissingInsertion);
    }
    let shaped = measured.shaped.as_ref();
    let expected_range = TextRange {
        start: 0,
        end: candidate.len(),
    };
    let line = shaped.lines.first().filter(|_| shaped.lines.len() == 1);
    if shaped.source_text.as_ref() != candidate
        || shaped.source_range != expected_range
        || line.is_none_or(|line| line.source_range != expected_range)
    {
        return Err(ArabicTatweelCandidateRejection::InvalidSourceIdentity);
    }
    if !natural_width.is_finite()
        || !measured.metrics.width.is_finite()
        || measured.metrics.width <= natural_width + TATWEEL_ADVANCE_EPSILON
    {
        return Err(ArabicTatweelCandidateRejection::NonExpandingCandidate);
    }

    let insertion_ranges = validate_insertion_ranges(candidate, insertion_offsets)?;
    let line = line.expect("single shaped line was checked above");
    let mut insertion_index = 0_usize;
    let mut previous: Option<PreviousCluster> = None;
    let mut pending_right_context = None;
    let mut previous_source_end = 0_usize;

    for cluster in text_glyph_clusters(&line.glyphs) {
        if cluster.source_range.start < previous_source_end
            || cluster.source_range.start >= cluster.source_range.end
            || cluster.source_range.end > candidate.len()
        {
            return Err(ArabicTatweelCandidateRejection::MalformedClusterOrder);
        }
        let face = cluster_face_identity(&line.glyphs, cluster.glyph_start, cluster.glyph_end)?;
        if let Some(expected_face) = pending_right_context.take() {
            if face != expected_face {
                return Err(ArabicTatweelCandidateRejection::JoiningFaceMismatch);
            }
            if cluster.right_to_left != Some(true) {
                return Err(ArabicTatweelCandidateRejection::NonRtlJoiningContext);
            }
        }

        let insertion = insertion_ranges.get(insertion_index).copied();
        if insertion.is_some_and(|range| range.start < cluster.source_range.start) {
            return Err(ArabicTatweelCandidateRejection::MissingInsertionCluster);
        }
        if insertion.is_some_and(|range| ranges_overlap(range, cluster.source_range)) {
            let insertion = insertion.expect("overlap requires an insertion range");
            if cluster.source_range != insertion {
                return Err(ArabicTatweelCandidateRejection::MixedSourceCluster);
            }
            let Some(left) = previous else {
                return Err(ArabicTatweelCandidateRejection::MissingJoiningContext);
            };
            if left.face != face {
                return Err(ArabicTatweelCandidateRejection::JoiningFaceMismatch);
            }
            if left.right_to_left != Some(true) || cluster.right_to_left != Some(true) {
                return Err(ArabicTatweelCandidateRejection::NonRtlJoiningContext);
            }
            if !cluster.advance.is_finite() || cluster.advance <= TATWEEL_ADVANCE_EPSILON {
                return Err(ArabicTatweelCandidateRejection::NonExpandingCandidate);
            }
            pending_right_context = Some(face);
            insertion_index = insertion_index.saturating_add(1);
        }

        previous = Some(PreviousCluster {
            face,
            right_to_left: cluster.right_to_left,
        });
        previous_source_end = cluster.source_range.end;
    }

    if insertion_index != insertion_ranges.len() {
        return Err(ArabicTatweelCandidateRejection::MissingInsertionCluster);
    }
    if pending_right_context.is_some() {
        return Err(ArabicTatweelCandidateRejection::MissingJoiningContext);
    }
    Ok(ArabicTatweelCandidateReceipt {
        width: measured.metrics.width,
        insertion_count: insertion_ranges.len(),
    })
}

fn validate_insertion_ranges(
    candidate: &str,
    insertion_offsets: &[usize],
) -> Result<Vec<TextRange>, ArabicTatweelCandidateRejection> {
    let mut ranges = Vec::with_capacity(insertion_offsets.len());
    let mut previous_end = 0_usize;
    for offset in insertion_offsets.iter().copied() {
        let end = offset
            .checked_add(ARABIC_TATWEEL.len())
            .ok_or(ArabicTatweelCandidateRejection::InvalidInsertionRange)?;
        if offset < previous_end || candidate.get(offset..end) != Some(ARABIC_TATWEEL) {
            return Err(ArabicTatweelCandidateRejection::InvalidInsertionRange);
        }
        ranges.push(TextRange { start: offset, end });
        previous_end = end;
    }
    Ok(ranges)
}

fn cluster_face_identity(
    glyphs: &[ShapedGlyph],
    start: usize,
    end: usize,
) -> Result<ClusterFaceIdentity, ArabicTatweelCandidateRejection> {
    let cluster = glyphs
        .get(start..end)
        .filter(|cluster| !cluster.is_empty())
        .ok_or(ArabicTatweelCandidateRejection::InvalidGlyph)?;
    let first = cluster
        .first()
        .expect("non-empty cluster was checked above");
    if first.glyph_id == 0 {
        return Err(ArabicTatweelCandidateRejection::InvalidGlyph);
    }
    let face = ClusterFaceIdentity {
        face: first
            .font_id
            .ok_or(ArabicTatweelCandidateRejection::MissingFace)?,
        instance: first.font_instance_id,
    };
    for glyph in cluster.iter().skip(1) {
        if glyph.glyph_id == 0 {
            return Err(ArabicTatweelCandidateRejection::InvalidGlyph);
        }
        if glyph.font_id != Some(face.face) || glyph.font_instance_id != face.instance {
            return Err(ArabicTatweelCandidateRejection::MixedClusterFace);
        }
    }
    Ok(face)
}

const fn ranges_overlap(left: TextRange, right: TextRange) -> bool {
    left.start < right.end && right.start < left.end
}

#[cfg(test)]
#[path = "tests/arabic_justification.rs"]
mod tests;
