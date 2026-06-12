use std::sync::Arc;

use super::super::GpuMaterialUniformResource;
use super::super::MaterialRuntime;

pub(in crate::graphics::scene::resources) struct PreparedMaterial {
    pub(in crate::graphics::scene::resources) revision: Option<u64>,
    pub(in crate::graphics::scene::resources) runtime: MaterialRuntime,
    pub(in crate::graphics::scene::resources) uniform: Arc<GpuMaterialUniformResource>,
    pub(in crate::graphics::scene::resources) standard_uniform: Arc<GpuMaterialUniformResource>,
}
