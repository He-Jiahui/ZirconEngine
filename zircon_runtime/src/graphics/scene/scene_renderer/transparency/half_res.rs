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

pub(crate) const fn half_resolution_transparency_supported(graph_msaa_sample_count: u32) -> bool {
    graph_msaa_sample_count == 1
}

#[cfg(test)]
mod tests {
    use super::half_resolution_transparency_supported;

    #[test]
    fn half_resolution_transparency_requires_a_single_sample_graph() {
        assert!(half_resolution_transparency_supported(1));
        assert!(!half_resolution_transparency_supported(2));
        assert!(!half_resolution_transparency_supported(4));
    }
}
