use serde::{Deserialize, Serialize};

use super::{TextFontFaceHandle, TextGlyphRotation};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum TextVerticalGlyphOrientation {
    Upright,
    Sideways,
    TransformOrRotate,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum TextVerticalGlyphFeatureSet {
    #[default]
    None,
    Vert,
    Vrt2,
    VertAndVrt2,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum TextVerticalGlyphSubstitution {
    #[default]
    NotChecked,
    NotObserved,
    Observed,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum TextVerticalGlyphFallbackReason {
    #[default]
    None,
    ForcedSideways,
    UnicodeSideways,
    NoVerticalSubstitution,
    BackendProvenanceUnavailable,
    NonRenderingControl,
}

/// Compact cluster-head provenance retained beside the glyph's existing font and rotation fields.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TextVerticalGlyphDecisionBasis {
    pub orientation: TextVerticalGlyphOrientation,
    pub features: TextVerticalGlyphFeatureSet,
    pub substitution: TextVerticalGlyphSubstitution,
    pub fallback_reason: TextVerticalGlyphFallbackReason,
}

/// Complete renderer-neutral view of a cluster's vertical shaping decision.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TextVerticalGlyphDecision {
    pub basis: TextVerticalGlyphDecisionBasis,
    pub rotation: TextGlyphRotation,
    pub font_face: Option<TextFontFaceHandle>,
    pub font_instance: Option<TextFontFaceHandle>,
}
