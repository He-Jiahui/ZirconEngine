mod support;

use zircon_runtime::render_graph::RenderGraphComputeDispatchExtent;

#[test]
fn virtual_geometry_support_descriptor_matches_the_plugin_compute_workload() {
    let descriptor = support::virtual_geometry_render_feature_descriptor();
    let pass = descriptor
        .stage_passes
        .iter()
        .find(|pass| pass.pass_name == "virtual-geometry-node-cluster-cull")
        .expect("virtual geometry support should declare the node-cluster cull pass");
    let workload = pass
        .compute_workload
        .as_ref()
        .expect("virtual geometry support cull pass should declare a compute workload");

    assert_eq!(
        workload.pipeline_label,
        "zircon-virtual-geometry-node-cluster-cull"
    );
    assert_eq!(workload.workgroup_size, [64, 1, 1]);
    assert_eq!(
        workload.dispatch_extent,
        RenderGraphComputeDispatchExtent::Fixed([1, 1, 1])
    );
}
