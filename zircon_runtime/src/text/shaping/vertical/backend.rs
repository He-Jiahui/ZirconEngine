use std::collections::{HashMap, HashSet};
use std::str::FromStr;

use rustybuzz::{
    script, ttf_parser::Tag, Direction, Feature, Language, Script, UnicodeBuffer, Variation,
};

use crate::text::font::FontDatabase;
use crate::text::{FontFaceId, InstancedFaceId, OpenTypeFeature};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum VerticalBackendDirection {
    TopToBottom,
    BottomToTop,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct VerticalBackendGlyph {
    pub(super) glyph_id: u32,
    pub(super) source_offset: usize,
    pub(super) y_advance: f32,
    pub(super) x_offset: f32,
    pub(super) y_offset: f32,
    pub(super) vertical_substituted: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub(super) struct VerticalBackendRun {
    pub(super) glyphs: Vec<VerticalBackendGlyph>,
}

#[allow(clippy::too_many_arguments)]
pub(super) fn shape_vertical_run(
    database: &FontDatabase,
    face: FontFaceId,
    instance: Option<InstancedFaceId>,
    text: &str,
    direction: VerticalBackendDirection,
    script_tag: &str,
    language: Option<&str>,
    features: &[OpenTypeFeature],
    include_kerning: bool,
    detect_vertical_substitution: bool,
    font_weight: u16,
    font_size: f32,
) -> Option<VerticalBackendRun> {
    if text.is_empty() {
        return Some(VerticalBackendRun { glyphs: Vec::new() });
    }

    let face_id = face;
    let variations = database
        .effective_instance_variations_shared(face_id, instance, font_weight)
        .ok()?;
    let bytes = database.face_bytes(face_id).ok()?;
    let face_index = database.face_index(face_id).ok()?;
    let mut face = rustybuzz::Face::from_slice(bytes.as_ref(), face_index)?;
    let variations = variations
        .0
        .iter()
        .map(|(tag, value)| Variation {
            tag: Tag::from_bytes(&tag.to_be_bytes()),
            value: *value,
        })
        .collect::<Vec<_>>();
    face.set_variations(&variations);
    let scale = font_size.max(1.0) / face.units_per_em().max(1) as f32;
    let projected_features = projected_vertical_features(features, include_kerning);
    #[cfg(any(feature = "profiling", feature = "profiling-tracy"))]
    super::super::cosmic::record_direct_backend_shape_call();
    let shaped = rustybuzz::shape(
        &face,
        &projected_features,
        vertical_buffer(text, direction, script_tag, language),
    );
    // RustyBuzz does not expose lookup execution traces. Compare against the
    // same request with vert/vrt2 disabled, retaining every non-vertical
    // feature and the same buffer context, to establish actual provenance.
    let substituted_clusters = detect_vertical_substitution
        .then(|| {
            #[cfg(any(feature = "profiling", feature = "profiling-tracy"))]
            super::super::cosmic::record_direct_backend_shape_call();
            let without_vertical = rustybuzz::shape(
                &face,
                &vertical_features_disabled(&projected_features),
                vertical_buffer(text, direction, script_tag, language),
            );
            vertical_substitution_clusters(
                shaped
                    .glyph_infos()
                    .iter()
                    .map(|info| (info.cluster, info.glyph_id)),
                without_vertical
                    .glyph_infos()
                    .iter()
                    .map(|info| (info.cluster, info.glyph_id)),
            )
        })
        .unwrap_or_default();
    let glyphs = shaped
        .glyph_infos()
        .iter()
        .zip(shaped.glyph_positions())
        .map(|(info, position)| VerticalBackendGlyph {
            glyph_id: info.glyph_id,
            source_offset: info.cluster as usize,
            y_advance: position.y_advance as f32 * scale,
            x_offset: position.x_offset as f32 * scale,
            y_offset: position.y_offset as f32 * scale,
            vertical_substituted: substituted_clusters.contains(&info.cluster),
        })
        .collect::<Vec<_>>();
    (!glyphs.is_empty()).then_some(VerticalBackendRun { glyphs })
}

fn vertical_buffer(
    text: &str,
    direction: VerticalBackendDirection,
    script_tag: &str,
    language: Option<&str>,
) -> UnicodeBuffer {
    let mut buffer = UnicodeBuffer::new();
    buffer.push_str(text);
    buffer.set_direction(match direction {
        VerticalBackendDirection::TopToBottom => Direction::TopToBottom,
        VerticalBackendDirection::BottomToTop => Direction::BottomToTop,
    });
    if let Some(script) = explicit_script(script_tag) {
        buffer.set_script(script);
    }
    if let Some(language) = language.and_then(|value| Language::from_str(value).ok()) {
        buffer.set_language(language);
    }
    buffer.guess_segment_properties();
    buffer
}

pub(super) fn vertical_substitution_clusters(
    with_vertical: impl Iterator<Item = (u32, u32)>,
    without_vertical: impl Iterator<Item = (u32, u32)>,
) -> HashSet<u32> {
    let with_vertical = glyphs_by_cluster(with_vertical);
    let without_vertical = glyphs_by_cluster(without_vertical);
    with_vertical
        .into_iter()
        .filter_map(|(cluster, glyphs)| {
            (without_vertical.get(&cluster) != Some(&glyphs)).then_some(cluster)
        })
        .collect()
}

fn glyphs_by_cluster(glyphs: impl Iterator<Item = (u32, u32)>) -> HashMap<u32, Vec<u32>> {
    let mut by_cluster = HashMap::new();
    for (cluster, glyph) in glyphs {
        by_cluster
            .entry(cluster)
            .or_insert_with(Vec::new)
            .push(glyph);
    }
    by_cluster
}

fn explicit_script(script_tag: &str) -> Option<Script> {
    let script = Script::from_str(script_tag).ok()?;
    (!matches!(script, script::COMMON | script::INHERITED | script::UNKNOWN)).then_some(script)
}

fn projected_vertical_features(
    features: &[OpenTypeFeature],
    include_kerning: bool,
) -> Vec<Feature> {
    let mut projected = features
        .iter()
        .map(|feature| Feature::new(Tag::from_bytes(&feature.tag), feature.value, ..))
        .collect::<Vec<_>>();
    for tag in [*b"vert", *b"vrt2"] {
        if !features.iter().any(|feature| feature.tag == tag) {
            projected.push(Feature::new(Tag::from_bytes(&tag), 1, ..));
        }
    }
    if !include_kerning {
        projected.push(Feature::new(Tag::from_bytes(b"kern"), 0, ..));
        projected.push(Feature::new(Tag::from_bytes(b"vkrn"), 0, ..));
    }
    projected
}

fn vertical_features_disabled(features: &[Feature]) -> Vec<Feature> {
    let mut disabled = features
        .iter()
        .copied()
        .filter(|feature| {
            feature.tag != Tag::from_bytes(b"vert") && feature.tag != Tag::from_bytes(b"vrt2")
        })
        .collect::<Vec<_>>();
    disabled.push(Feature::new(Tag::from_bytes(b"vert"), 0, ..));
    disabled.push(Feature::new(Tag::from_bytes(b"vrt2"), 0, ..));
    disabled
}

#[cfg(test)]
mod tests {
    use rustybuzz::ttf_parser::Tag;

    use super::{
        explicit_script, projected_vertical_features, vertical_features_disabled,
        vertical_substitution_clusters,
    };
    use crate::text::OpenTypeFeature;

    #[test]
    fn vertical_features_enable_defaults_and_respect_explicit_overrides() {
        let defaults = projected_vertical_features(&[], true);
        assert!(defaults
            .iter()
            .any(|feature| feature.tag == Tag::from_bytes(b"vert") && feature.value == 1));
        assert!(defaults
            .iter()
            .any(|feature| feature.tag == Tag::from_bytes(b"vrt2") && feature.value == 1));

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

        let disabled = vertical_features_disabled(&defaults);
        assert_eq!(
            disabled
                .iter()
                .filter(|feature| {
                    feature.tag == Tag::from_bytes(b"vert")
                        || feature.tag == Tag::from_bytes(b"vrt2")
                })
                .map(|feature| feature.value)
                .collect::<Vec<_>>(),
            vec![0, 0]
        );
    }

    #[test]
    fn vertical_backend_sets_resolved_non_common_script() {
        assert!(explicit_script("Hani").is_some());
        assert_eq!(explicit_script("Zyyy"), None);
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
        let source = include_str!("backend.rs");
        let shape_call = ["rustybuzz::", "shape("].concat();

        assert_eq!(source.matches(&shape_call).count(), 2);
        assert!(source.contains("detect_vertical_substitution\n        .then"));
    }
}
