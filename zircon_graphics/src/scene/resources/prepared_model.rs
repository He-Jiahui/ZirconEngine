use std::sync::Arc;

use super::gpu_model_resource::GpuModelResource;

pub(super) struct PreparedModel {
    pub(super) revision: u64,
    pub(super) resource: Arc<GpuModelResource>,
}
