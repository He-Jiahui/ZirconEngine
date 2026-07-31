mod codec;
mod material;
mod model;
mod scene;

pub(in crate::asset) use codec::ProjectDocumentArtifact;
pub(in crate::asset) use material::deserialize_material_artifact;
pub(in crate::asset::assets) use material::{deserialize_material, serialize_material};
pub(in crate::asset) use model::deserialize_model_artifact;
pub(in crate::asset::assets) use model::{deserialize_model, serialize_model};
pub(in crate::asset) use scene::deserialize_scene_artifact;
pub(in crate::asset::assets) use scene::{deserialize_scene, serialize_scene};
