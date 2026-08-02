use serde::{Deserialize, Serialize};

#[cfg(feature = "text")]
use crate::asset::assets::CompositeFontDescriptor;
use crate::asset::{
    FontAsset, FontAssetCmapCoverage, FontAssetCodepointRange, FontAssetFaceMetrics,
    FontAssetFaceStyle, FontAssetFamilyMember, FontAssetLineMetrics, FontAssetMetadata,
    FontAssetParsedFace, FontAssetRenderStrategy, FontAssetSourceFormat, FontAssetVariableInstance,
    FontAssetVariationAxis, FontAssetVariationCoord,
};
use zircon_runtime_interface::ui::surface::UiTextRenderMode;

/// Bincode-safe font cache payload. Authoring serde attributes must not decide
/// which fields exist in the sequential artifact cache wire format.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub(in crate::asset::artifact) struct ArtifactCacheFontAsset {
    source: String,
    family: Option<String>,
    render_mode: Option<UiTextRenderMode>,
    face_index: u32,
    family_members: Vec<ArtifactCacheFontAssetFamilyMember>,
    variable_instances: Vec<ArtifactCacheFontAssetVariableInstance>,
    fallback_families: Vec<String>,
    #[cfg(feature = "text")]
    composite_font: Option<CompositeFontDescriptor>,
    render_strategy: ArtifactCacheFontAssetRenderStrategy,
    metadata: Option<ArtifactCacheFontAssetMetadata>,
}

impl From<&FontAsset> for ArtifactCacheFontAsset {
    fn from(asset: &FontAsset) -> Self {
        Self {
            source: asset.source.clone(),
            family: asset.family.clone(),
            render_mode: asset.render_mode,
            face_index: asset.face_index,
            family_members: asset
                .family_members
                .iter()
                .map(ArtifactCacheFontAssetFamilyMember::from)
                .collect(),
            variable_instances: asset
                .variable_instances
                .iter()
                .map(ArtifactCacheFontAssetVariableInstance::from)
                .collect(),
            fallback_families: asset.fallback_families.clone(),
            #[cfg(feature = "text")]
            composite_font: asset.composite_font.clone(),
            render_strategy: ArtifactCacheFontAssetRenderStrategy::from(&asset.render_strategy),
            metadata: asset
                .metadata
                .as_ref()
                .map(ArtifactCacheFontAssetMetadata::from),
        }
    }
}

impl ArtifactCacheFontAsset {
    pub(super) fn into_asset(self) -> FontAsset {
        FontAsset {
            source: self.source,
            family: self.family,
            render_mode: self.render_mode,
            face_index: self.face_index,
            family_members: self
                .family_members
                .into_iter()
                .map(ArtifactCacheFontAssetFamilyMember::into_asset)
                .collect(),
            variable_instances: self
                .variable_instances
                .into_iter()
                .map(ArtifactCacheFontAssetVariableInstance::into_asset)
                .collect(),
            fallback_families: self.fallback_families,
            #[cfg(feature = "text")]
            composite_font: self.composite_font,
            render_strategy: self.render_strategy.into_asset(),
            metadata: self
                .metadata
                .map(ArtifactCacheFontAssetMetadata::into_asset),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
struct ArtifactCacheFontAssetRenderStrategy {
    default_mode: Option<UiTextRenderMode>,
    allow_native: Option<bool>,
    allow_sdf: Option<bool>,
}

impl From<&FontAssetRenderStrategy> for ArtifactCacheFontAssetRenderStrategy {
    fn from(strategy: &FontAssetRenderStrategy) -> Self {
        Self {
            default_mode: strategy.default_mode,
            allow_native: strategy.allow_native,
            allow_sdf: strategy.allow_sdf,
        }
    }
}

impl ArtifactCacheFontAssetRenderStrategy {
    fn into_asset(self) -> FontAssetRenderStrategy {
        FontAssetRenderStrategy {
            default_mode: self.default_mode,
            allow_native: self.allow_native,
            allow_sdf: self.allow_sdf,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
struct ArtifactCacheFontAssetFamilyMember {
    family: String,
    face_index: u32,
    weight: Option<u16>,
    width_class: Option<u16>,
    style: Option<FontAssetFaceStyle>,
    variations: Vec<ArtifactCacheFontAssetVariationCoord>,
}

impl From<&FontAssetFamilyMember> for ArtifactCacheFontAssetFamilyMember {
    fn from(member: &FontAssetFamilyMember) -> Self {
        Self {
            family: member.family.clone(),
            face_index: member.face_index,
            weight: member.weight,
            width_class: member.width_class,
            style: member.style,
            variations: member
                .variations
                .iter()
                .map(ArtifactCacheFontAssetVariationCoord::from)
                .collect(),
        }
    }
}

impl ArtifactCacheFontAssetFamilyMember {
    fn into_asset(self) -> FontAssetFamilyMember {
        FontAssetFamilyMember {
            family: self.family,
            face_index: self.face_index,
            weight: self.weight,
            width_class: self.width_class,
            style: self.style,
            variations: self
                .variations
                .into_iter()
                .map(ArtifactCacheFontAssetVariationCoord::into_asset)
                .collect(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
struct ArtifactCacheFontAssetVariationAxis {
    tag: String,
    min: f32,
    default: f32,
    max: f32,
    name: Option<String>,
    hidden: bool,
}

impl From<&FontAssetVariationAxis> for ArtifactCacheFontAssetVariationAxis {
    fn from(axis: &FontAssetVariationAxis) -> Self {
        Self {
            tag: axis.tag.clone(),
            min: axis.min,
            default: axis.default,
            max: axis.max,
            name: axis.name.clone(),
            hidden: axis.hidden,
        }
    }
}

impl ArtifactCacheFontAssetVariationAxis {
    fn into_asset(self) -> FontAssetVariationAxis {
        FontAssetVariationAxis {
            tag: self.tag,
            min: self.min,
            default: self.default,
            max: self.max,
            name: self.name,
            hidden: self.hidden,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
struct ArtifactCacheFontAssetVariationCoord {
    tag: String,
    value: f32,
}

impl From<&FontAssetVariationCoord> for ArtifactCacheFontAssetVariationCoord {
    fn from(coord: &FontAssetVariationCoord) -> Self {
        Self {
            tag: coord.tag.clone(),
            value: coord.value,
        }
    }
}

impl ArtifactCacheFontAssetVariationCoord {
    fn into_asset(self) -> FontAssetVariationCoord {
        FontAssetVariationCoord {
            tag: self.tag,
            value: self.value,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
struct ArtifactCacheFontAssetVariableInstance {
    name: Option<String>,
    post_script_name: Option<String>,
    coordinates: Vec<ArtifactCacheFontAssetVariationCoord>,
}

impl From<&FontAssetVariableInstance> for ArtifactCacheFontAssetVariableInstance {
    fn from(instance: &FontAssetVariableInstance) -> Self {
        Self {
            name: instance.name.clone(),
            post_script_name: instance.post_script_name.clone(),
            coordinates: instance
                .coordinates
                .iter()
                .map(ArtifactCacheFontAssetVariationCoord::from)
                .collect(),
        }
    }
}

impl ArtifactCacheFontAssetVariableInstance {
    fn into_asset(self) -> FontAssetVariableInstance {
        FontAssetVariableInstance {
            name: self.name,
            post_script_name: self.post_script_name,
            coordinates: self
                .coordinates
                .into_iter()
                .map(ArtifactCacheFontAssetVariationCoord::into_asset)
                .collect(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
struct ArtifactCacheFontAssetMetadata {
    source_format: FontAssetSourceFormat,
    face_count: u32,
    faces: Vec<ArtifactCacheFontAssetParsedFace>,
}

impl From<&FontAssetMetadata> for ArtifactCacheFontAssetMetadata {
    fn from(metadata: &FontAssetMetadata) -> Self {
        Self {
            source_format: metadata.source_format,
            face_count: metadata.face_count,
            faces: metadata
                .faces
                .iter()
                .map(ArtifactCacheFontAssetParsedFace::from)
                .collect(),
        }
    }
}

impl ArtifactCacheFontAssetMetadata {
    fn into_asset(self) -> FontAssetMetadata {
        FontAssetMetadata {
            source_format: self.source_format,
            face_count: self.face_count,
            faces: self
                .faces
                .into_iter()
                .map(ArtifactCacheFontAssetParsedFace::into_asset)
                .collect(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
struct ArtifactCacheFontAssetParsedFace {
    face_index: u32,
    family: Option<String>,
    subfamily: Option<String>,
    full_name: Option<String>,
    post_script_name: Option<String>,
    weight: u16,
    width_class: u16,
    style: FontAssetFaceStyle,
    metrics: ArtifactCacheFontAssetFaceMetrics,
    variation_axes: Vec<ArtifactCacheFontAssetVariationAxis>,
    named_instances: Vec<ArtifactCacheFontAssetVariableInstance>,
    cmap: ArtifactCacheFontAssetCmapCoverage,
}

impl From<&FontAssetParsedFace> for ArtifactCacheFontAssetParsedFace {
    fn from(face: &FontAssetParsedFace) -> Self {
        Self {
            face_index: face.face_index,
            family: face.family.clone(),
            subfamily: face.subfamily.clone(),
            full_name: face.full_name.clone(),
            post_script_name: face.post_script_name.clone(),
            weight: face.weight,
            width_class: face.width_class,
            style: face.style,
            metrics: ArtifactCacheFontAssetFaceMetrics::from(face.metrics),
            variation_axes: face
                .variation_axes
                .iter()
                .map(ArtifactCacheFontAssetVariationAxis::from)
                .collect(),
            named_instances: face
                .named_instances
                .iter()
                .map(ArtifactCacheFontAssetVariableInstance::from)
                .collect(),
            cmap: ArtifactCacheFontAssetCmapCoverage::from(&face.cmap),
        }
    }
}

impl ArtifactCacheFontAssetParsedFace {
    fn into_asset(self) -> FontAssetParsedFace {
        FontAssetParsedFace {
            face_index: self.face_index,
            family: self.family,
            subfamily: self.subfamily,
            full_name: self.full_name,
            post_script_name: self.post_script_name,
            weight: self.weight,
            width_class: self.width_class,
            style: self.style,
            metrics: self.metrics.into_asset(),
            variation_axes: self
                .variation_axes
                .into_iter()
                .map(ArtifactCacheFontAssetVariationAxis::into_asset)
                .collect(),
            named_instances: self
                .named_instances
                .into_iter()
                .map(ArtifactCacheFontAssetVariableInstance::into_asset)
                .collect(),
            cmap: self.cmap.into_asset(),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
struct ArtifactCacheFontAssetFaceMetrics {
    units_per_em: u16,
    ascender: i16,
    descender: i16,
    line_gap: i16,
    uses_typographic_metrics: bool,
    windows_ascender: i16,
    windows_descender: i16,
    underline: Option<ArtifactCacheFontAssetLineMetrics>,
    strikeout: Option<ArtifactCacheFontAssetLineMetrics>,
}

impl From<FontAssetFaceMetrics> for ArtifactCacheFontAssetFaceMetrics {
    fn from(metrics: FontAssetFaceMetrics) -> Self {
        Self {
            units_per_em: metrics.units_per_em,
            ascender: metrics.ascender,
            descender: metrics.descender,
            line_gap: metrics.line_gap,
            uses_typographic_metrics: metrics.uses_typographic_metrics,
            windows_ascender: metrics.windows_ascender,
            windows_descender: metrics.windows_descender,
            underline: metrics
                .underline
                .map(ArtifactCacheFontAssetLineMetrics::from),
            strikeout: metrics
                .strikeout
                .map(ArtifactCacheFontAssetLineMetrics::from),
        }
    }
}

impl ArtifactCacheFontAssetFaceMetrics {
    fn into_asset(self) -> FontAssetFaceMetrics {
        FontAssetFaceMetrics {
            units_per_em: self.units_per_em,
            ascender: self.ascender,
            descender: self.descender,
            line_gap: self.line_gap,
            uses_typographic_metrics: self.uses_typographic_metrics,
            windows_ascender: self.windows_ascender,
            windows_descender: self.windows_descender,
            underline: self
                .underline
                .map(ArtifactCacheFontAssetLineMetrics::into_asset),
            strikeout: self
                .strikeout
                .map(ArtifactCacheFontAssetLineMetrics::into_asset),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
struct ArtifactCacheFontAssetLineMetrics {
    position: i16,
    thickness: i16,
}

impl From<FontAssetLineMetrics> for ArtifactCacheFontAssetLineMetrics {
    fn from(metrics: FontAssetLineMetrics) -> Self {
        Self {
            position: metrics.position,
            thickness: metrics.thickness,
        }
    }
}

impl ArtifactCacheFontAssetLineMetrics {
    fn into_asset(self) -> FontAssetLineMetrics {
        FontAssetLineMetrics {
            position: self.position,
            thickness: self.thickness,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
struct ArtifactCacheFontAssetCmapCoverage {
    codepoint_count: u32,
    ranges: Vec<ArtifactCacheFontAssetCodepointRange>,
}

impl From<&FontAssetCmapCoverage> for ArtifactCacheFontAssetCmapCoverage {
    fn from(coverage: &FontAssetCmapCoverage) -> Self {
        Self {
            codepoint_count: coverage.codepoint_count,
            ranges: coverage
                .ranges
                .iter()
                .map(ArtifactCacheFontAssetCodepointRange::from)
                .collect(),
        }
    }
}

impl ArtifactCacheFontAssetCmapCoverage {
    fn into_asset(self) -> FontAssetCmapCoverage {
        FontAssetCmapCoverage {
            codepoint_count: self.codepoint_count,
            ranges: self
                .ranges
                .into_iter()
                .map(ArtifactCacheFontAssetCodepointRange::into_asset)
                .collect(),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
struct ArtifactCacheFontAssetCodepointRange {
    start: u32,
    end: u32,
}

impl From<&FontAssetCodepointRange> for ArtifactCacheFontAssetCodepointRange {
    fn from(range: &FontAssetCodepointRange) -> Self {
        Self {
            start: range.start,
            end: range.end,
        }
    }
}

impl ArtifactCacheFontAssetCodepointRange {
    fn into_asset(self) -> FontAssetCodepointRange {
        FontAssetCodepointRange {
            start: self.start,
            end: self.end,
        }
    }
}
