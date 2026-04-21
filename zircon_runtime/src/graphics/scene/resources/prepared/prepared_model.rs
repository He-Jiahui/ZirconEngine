use std::sync::Arc;

use crate::asset::ModelAsset;

use super::super::GpuModelResource;

pub(in crate::graphics::scene::resources) struct PreparedModel {
    pub(in crate::graphics::scene::resources) revision: u64,
    pub(in crate::graphics::scene::resources) asset: Arc<ModelAsset>,
    pub(in crate::graphics::scene::resources) resource: Arc<GpuModelResource>,
}
