pub(crate) const HALF_RES_TRANSPARENCY_DEPTH_DOWNSAMPLE_PASS_NAME: &str =
    "halfres-transparency-depth-downsample";
pub(crate) const HALF_RES_TRANSPARENCY_DEPTH_DOWNSAMPLE_EXECUTOR_ID: &str =
    "transparency.halfres-depth-downsample";
pub(crate) const HALF_RES_TRANSPARENCY_MESH_PASS_NAME: &str = "halfres-transparent-mesh";
pub(crate) const HALF_RES_TRANSPARENCY_MESH_EXECUTOR_ID: &str = "mesh.halfres-transparent";
pub(crate) const HALF_RES_TRANSPARENCY_PARTICLE_EXECUTOR_ID: &str = "particle.halfres-transparent";
pub(crate) const HALF_RES_TRANSPARENCY_COMPOSITE_PASS_NAME: &str = "halfres-transparency-composite";
pub(crate) const HALF_RES_TRANSPARENCY_COMPOSITE_EXECUTOR_ID: &str =
    "transparency.halfres-composite";

/// 半分辨率透明仅在单采样图上启用；管线装配须在插入对应 pass 前检查。
pub(crate) const fn half_resolution_transparency_supported(graph_msaa_sample_count: u32) -> bool {
    graph_msaa_sample_count == 1
}

#[cfg(test)]
#[path = "tests/half_res.rs"]
mod tests;
