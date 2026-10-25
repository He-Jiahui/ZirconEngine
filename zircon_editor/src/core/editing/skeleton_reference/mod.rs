//! Read-only bone identities and hierarchy derived from an installed skeleton payload.
//!
//! The resource owner supplies the active version. This module does not load assets, mutate an
//! authoring document, admit preview frames, or replace the shared collection and selection owners.

#[path = "details.rs"]
mod details;
#[path = "hierarchy.rs"]
mod hierarchy;
#[path = "reference.rs"]
mod reference;
#[path = "reveal.rs"]
mod reveal;
mod skin;
#[path = "space.rs"]
mod space;

pub(crate) use details::{SkeletonBoneReferenceDetails, SkeletonBoneReferenceParent};
pub(crate) use hierarchy::SkeletonReferenceHierarchyError;
pub(crate) use reference::{
    SkeletonBoneKey, SkeletonBoneRow, SkeletonReference, SkeletonReferenceError,
};
pub(crate) use reveal::SkeletonBoneReveal;
pub(crate) use skin::{
    GltfSkinDataError, GltfSkinJoint, GltfSkinReference, GltfSkinReferenceError,
};
pub(crate) use space::{SkeletonReferenceSpace, SkeletonReferenceTransformError};

#[cfg(test)]
#[path = "tests/cases.rs"]
mod tests;
