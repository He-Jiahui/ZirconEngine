use std::sync::Arc;

use super::super::GpuModelResource;

pub(in crate::scene::resources) struct PreparedModel {
    pub(in crate::scene::resources) revision: u64,
    pub(in crate::scene::resources) resource: Arc<GpuModelResource>,
}
