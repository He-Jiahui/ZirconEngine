use super::{ClusterLineBreakFlags, LineBreakKind, LineBreakOpportunity, LineBreakOpportunityMap};
use crate::text::{
    compiled_unicode_data_snapshot_id, LineBreakTailoringProfile, ShapedGlyphLineBreakOpportunity,
    ShapedGlyphLineBreakReceipt,
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
            receipt: unicode_default_receipt(ShapedGlyphLineBreakOpportunity::ProviderMandatory,),
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
    let source = include_str!("../line_break.rs");
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
