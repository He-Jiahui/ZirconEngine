use zircon_runtime_interface::ui::surface::{UiTextCaretAffinity, UiTextDirection, UiTextRange};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum VisualBoundaryBias {
    LeadingCurrent,
    TrailingPrevious,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct VisualSourceCluster {
    pub(super) source_range: UiTextRange,
    pub(super) direction: UiTextDirection,
}

pub(super) fn source_caret_for_visual_boundary(
    clusters: &[VisualSourceCluster],
    visual_boundary: usize,
    bias: VisualBoundaryBias,
    fallback: usize,
) -> (usize, UiTextCaretAffinity) {
    match bias {
        VisualBoundaryBias::LeadingCurrent => clusters
            .get(visual_boundary)
            .map(|cluster| {
                (
                    leading_source_offset(*cluster),
                    UiTextCaretAffinity::Downstream,
                )
            })
            .unwrap_or((fallback, UiTextCaretAffinity::Downstream)),
        VisualBoundaryBias::TrailingPrevious => visual_boundary
            .checked_sub(1)
            .and_then(|index| clusters.get(index))
            .map(|cluster| {
                (
                    trailing_source_offset(*cluster),
                    UiTextCaretAffinity::Upstream,
                )
            })
            .unwrap_or((fallback, UiTextCaretAffinity::Upstream)),
    }
}

fn leading_source_offset(cluster: VisualSourceCluster) -> usize {
    if matches!(cluster.direction, UiTextDirection::RightToLeft) {
        cluster.source_range.end
    } else {
        cluster.source_range.start
    }
}

fn trailing_source_offset(cluster: VisualSourceCluster) -> usize {
    if matches!(cluster.direction, UiTextDirection::RightToLeft) {
        cluster.source_range.start
    } else {
        cluster.source_range.end
    }
}

#[cfg(test)]
mod tests {
    use super::{source_caret_for_visual_boundary, VisualBoundaryBias, VisualSourceCluster};
    use zircon_runtime_interface::ui::surface::{
        UiTextCaretAffinity, UiTextDirection, UiTextRange,
    };

    fn cluster(start: usize, end: usize, direction: UiTextDirection) -> VisualSourceCluster {
        VisualSourceCluster {
            source_range: UiTextRange { start, end },
            direction,
        }
    }

    #[test]
    fn mixed_bidi_leading_rtl_cluster_maps_to_logical_end() {
        let clusters = [
            cluster(0, 1, UiTextDirection::LeftToRight),
            cluster(4, 6, UiTextDirection::RightToLeft),
        ];

        assert_eq!(
            source_caret_for_visual_boundary(&clusters, 1, VisualBoundaryBias::LeadingCurrent, 0,),
            (6, UiTextCaretAffinity::Downstream)
        );
    }

    #[test]
    fn mixed_bidi_trailing_rtl_cluster_maps_to_logical_start() {
        let clusters = [
            cluster(0, 1, UiTextDirection::LeftToRight),
            cluster(6, 8, UiTextDirection::RightToLeft),
            cluster(4, 6, UiTextDirection::RightToLeft),
        ];

        assert_eq!(
            source_caret_for_visual_boundary(&clusters, 2, VisualBoundaryBias::TrailingPrevious, 0,),
            (6, UiTextCaretAffinity::Upstream)
        );
    }
}
