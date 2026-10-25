//! Source joint mapping from the existing imported glTF SkinN Data contract.

mod binding;
mod metadata;

pub(crate) use binding::{GltfSkinJoint, GltfSkinReference, GltfSkinReferenceError};
pub(crate) use metadata::GltfSkinDataError;
