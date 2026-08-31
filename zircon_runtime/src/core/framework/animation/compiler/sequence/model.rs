//! Immutable sequence compiler products.

use crate::core::framework::animation::{AnimationChannelValueAsset, AnimationInterpolationAsset};
use crate::core::framework::scene::{ComponentPropertyPath, EntityPath};
use crate::core::math::Real;

use super::super::AnimationCompileDiagnostic;

/// Value domain carried by one validated sequence track.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AnimationCompiledSequenceValueKind {
    Bool,
    Integer,
    Scalar,
    Vec2,
    Vec3,
    Vec4,
    Quaternion,
}

/// A validated key, kept in canonical increasing time order.
#[derive(Clone, Debug, PartialEq)]
pub struct AnimationCompiledSequenceKey {
    time_seconds: Real,
    value: AnimationChannelValueAsset,
    in_tangent: Option<AnimationChannelValueAsset>,
    out_tangent: Option<AnimationChannelValueAsset>,
}

impl AnimationCompiledSequenceKey {
    pub(super) fn new(
        time_seconds: Real,
        value: AnimationChannelValueAsset,
        in_tangent: Option<AnimationChannelValueAsset>,
        out_tangent: Option<AnimationChannelValueAsset>,
    ) -> Self {
        Self {
            time_seconds,
            value,
            in_tangent,
            out_tangent,
        }
    }

    pub fn time_seconds(&self) -> Real {
        self.time_seconds
    }

    pub fn value(&self) -> &AnimationChannelValueAsset {
        &self.value
    }

    pub fn in_tangent(&self) -> Option<&AnimationChannelValueAsset> {
        self.in_tangent.as_ref()
    }

    pub fn out_tangent(&self) -> Option<&AnimationChannelValueAsset> {
        self.out_tangent.as_ref()
    }
}

/// A property track whose keys have one stable value domain.
#[derive(Clone, Debug, PartialEq)]
pub struct AnimationCompiledSequenceTrack {
    property_path: ComponentPropertyPath,
    interpolation: AnimationInterpolationAsset,
    value_kind: AnimationCompiledSequenceValueKind,
    keys: Vec<AnimationCompiledSequenceKey>,
}

impl AnimationCompiledSequenceTrack {
    pub(super) fn new(
        property_path: ComponentPropertyPath,
        interpolation: AnimationInterpolationAsset,
        value_kind: AnimationCompiledSequenceValueKind,
        keys: Vec<AnimationCompiledSequenceKey>,
    ) -> Self {
        Self {
            property_path,
            interpolation,
            value_kind,
            keys,
        }
    }

    pub fn property_path(&self) -> &ComponentPropertyPath {
        &self.property_path
    }

    pub fn interpolation(&self) -> AnimationInterpolationAsset {
        self.interpolation
    }

    pub fn value_kind(&self) -> AnimationCompiledSequenceValueKind {
        self.value_kind
    }

    pub fn keys(&self) -> &[AnimationCompiledSequenceKey] {
        &self.keys
    }
}

/// A source binding with tracks that can later be resolved against a concrete world.
#[derive(Clone, Debug, PartialEq)]
pub struct AnimationCompiledSequenceBinding {
    entity_path: EntityPath,
    target_id: Option<String>,
    tracks: Vec<AnimationCompiledSequenceTrack>,
}

impl AnimationCompiledSequenceBinding {
    pub(super) fn new(
        entity_path: EntityPath,
        target_id: Option<String>,
        tracks: Vec<AnimationCompiledSequenceTrack>,
    ) -> Self {
        Self {
            entity_path,
            target_id,
            tracks,
        }
    }

    pub fn entity_path(&self) -> &EntityPath {
        &self.entity_path
    }

    pub fn target_id(&self) -> Option<&str> {
        self.target_id.as_deref()
    }

    pub fn tracks(&self) -> &[AnimationCompiledSequenceTrack] {
        &self.tracks
    }
}

/// A validated source-only sequence IR.
#[derive(Clone, Debug, PartialEq)]
pub struct AnimationCompiledSequence {
    duration_seconds: Real,
    frames_per_second: Real,
    bindings: Vec<AnimationCompiledSequenceBinding>,
}

impl AnimationCompiledSequence {
    pub(super) fn new(
        duration_seconds: Real,
        frames_per_second: Real,
        bindings: Vec<AnimationCompiledSequenceBinding>,
    ) -> Self {
        Self {
            duration_seconds,
            frames_per_second,
            bindings,
        }
    }

    pub fn duration_seconds(&self) -> Real {
        self.duration_seconds
    }

    pub fn frames_per_second(&self) -> Real {
        self.frames_per_second
    }

    pub fn bindings(&self) -> &[AnimationCompiledSequenceBinding] {
        &self.bindings
    }
}

/// Result of compiling one sequence asset without resolving a world binding.
#[derive(Clone, Debug, PartialEq)]
pub struct AnimationSequenceCompilation {
    artifact: Option<AnimationCompiledSequence>,
    diagnostics: Vec<AnimationCompileDiagnostic>,
}

impl AnimationSequenceCompilation {
    pub(crate) fn new(
        artifact: Option<AnimationCompiledSequence>,
        diagnostics: Vec<AnimationCompileDiagnostic>,
    ) -> Self {
        Self {
            artifact,
            diagnostics,
        }
    }

    pub fn artifact(&self) -> Option<&AnimationCompiledSequence> {
        self.artifact.as_ref()
    }

    pub fn diagnostics(&self) -> &[AnimationCompileDiagnostic] {
        &self.diagnostics
    }
}
