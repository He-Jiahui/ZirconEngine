use rustybuzz::ttf_parser::Tag;

use super::{
    explicit_script, projected_vertical_features, vertical_feature_set, vertical_features_disabled,
    vertical_substitution_clusters,
};
use crate::text::{Iso15924Tag, OpenTypeFeature, TextVerticalGlyphFeatureSet};

#[test]
fn vertical_features_enable_defaults_and_respect_explicit_overrides() {
    let defaults = projected_vertical_features(&[], true);
    assert!(defaults
        .iter()
        .any(|feature| feature.tag == Tag::from_bytes(b"vert") && feature.value == 1));
    assert!(defaults
        .iter()
        .any(|feature| feature.tag == Tag::from_bytes(b"vrt2") && feature.value == 1));
    assert_eq!(
        vertical_feature_set(&defaults),
        TextVerticalGlyphFeatureSet::VertAndVrt2
    );

    let requested = [OpenTypeFeature::new(*b"vert", 0)];
    let overridden = projected_vertical_features(&requested, false);
    assert_eq!(
        overridden
            .iter()
            .filter(|feature| feature.tag == Tag::from_bytes(b"vert"))
            .map(|feature| feature.value)
            .collect::<Vec<_>>(),
        vec![0]
    );
    assert!(overridden
        .iter()
        .any(|feature| feature.tag == Tag::from_bytes(b"vrt2") && feature.value == 1));
    assert!(overridden
        .iter()
        .any(|feature| feature.tag == Tag::from_bytes(b"kern") && feature.value == 0));
    assert!(overridden
        .iter()
        .any(|feature| feature.tag == Tag::from_bytes(b"vkrn") && feature.value == 0));
    assert_eq!(
        vertical_feature_set(&overridden),
        TextVerticalGlyphFeatureSet::Vrt2
    );

    let disabled = vertical_features_disabled(&defaults);
    assert_eq!(
        disabled
            .iter()
            .filter(|feature| {
                feature.tag == Tag::from_bytes(b"vert") || feature.tag == Tag::from_bytes(b"vrt2")
            })
            .map(|feature| feature.value)
            .collect::<Vec<_>>(),
        vec![0, 0]
    );
    assert_eq!(
        vertical_feature_set(&disabled),
        TextVerticalGlyphFeatureSet::None
    );
}

#[test]
fn vertical_backend_sets_resolved_non_common_script() {
    assert!(explicit_script(script_tag("Hani")).is_some());
    assert_eq!(explicit_script(script_tag("Zyyy")), None);
}

fn script_tag(value: &str) -> Iso15924Tag {
    Iso15924Tag::parse(value).expect("test script tag must be canonical")
}

#[test]
fn vertical_substitution_provenance_is_derived_from_cluster_output_differences() {
    let substituted = vertical_substitution_clusters(
        [(0, 11), (0, 12), (4, 20), (8, 30)].into_iter(),
        [(0, 11), (0, 12), (4, 21), (8, 30)].into_iter(),
    );

    assert_eq!(substituted.into_iter().collect::<Vec<_>>(), vec![4]);
}

#[test]
fn vertical_backend_limits_the_comparison_shape_to_provenance_requests() {
    let source = include_str!("../backend.rs");
    let shape_call = ["rustybuzz::", "shape("].concat();

    assert_eq!(source.matches(&shape_call).count(), 2);
    assert!(source.contains("detect_vertical_substitution\n        .then"));
    assert!(source.contains("record_vertical_substitution_comparison"));
}
