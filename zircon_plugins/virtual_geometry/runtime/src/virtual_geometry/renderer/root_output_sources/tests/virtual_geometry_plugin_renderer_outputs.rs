use super::*;

#[test]
fn runtime_prepare_keeps_frame_sideband_as_the_feedback_owner() {
    let source = include_str!("../virtual_geometry_plugin_renderer_outputs.rs");

    assert!(!source.contains(concat!(
        "prepared_virtual_geometry_readback_outputs().",
        "clone()"
    )));
    assert!(source.contains(concat!(
        "register_prepared_virtual_geometry_feedback_buffer(context);",
        "\n    RenderPluginRendererOutputs::default()"
    )));
}

#[test]
fn plugin_renderer_outputs_package_node_cluster_cull_readback_under_virtual_geometry() {
    let outputs = plugin_renderer_outputs_from_node_cluster_cull_readback(
        RenderVirtualGeometryNodeClusterCullReadbackOutputs {
            page_request_ids: vec![300, 301],
            ..RenderVirtualGeometryNodeClusterCullReadbackOutputs::default()
        },
    );

    assert_eq!(
        outputs.virtual_geometry.node_cluster_cull.page_request_ids,
        vec![300, 301]
    );
    assert!(outputs.hybrid_gi.is_empty());
    assert!(outputs.particles.is_empty());
    assert!(!outputs.is_empty());
}

#[test]
fn runtime_prepare_renderer_outputs_do_not_fabricate_virtual_geometry_readbacks() {
    let outputs = plugin_renderer_outputs_from_virtual_geometry_readback(
        RenderVirtualGeometryReadbackOutputs::default(),
    );

    assert!(outputs.is_empty());
    assert!(outputs.virtual_geometry.is_empty());
}

#[test]
fn runtime_prepare_renderer_outputs_package_prepared_virtual_geometry_sideband() {
    let outputs = plugin_renderer_outputs_from_virtual_geometry_readback(
        RenderVirtualGeometryReadbackOutputs {
            node_cluster_cull: RenderVirtualGeometryNodeClusterCullReadbackOutputs {
                page_request_ids: vec![401, 402],
                ..RenderVirtualGeometryNodeClusterCullReadbackOutputs::default()
            },
            ..RenderVirtualGeometryReadbackOutputs::default()
        },
    );

    assert_eq!(
        outputs.virtual_geometry.node_cluster_cull.page_request_ids,
        vec![401, 402]
    );
    assert!(outputs.hybrid_gi.is_empty());
    assert!(outputs.particles.is_empty());
}

#[test]
fn virtual_geometry_feedback_binding_names_stay_stable() {
    assert_eq!(
        VIRTUAL_GEOMETRY_FEEDBACK_EXTERNAL_BUFFER,
        "virtual-geometry-feedback"
    );
    assert_eq!(
        VIRTUAL_GEOMETRY_FEEDBACK_BACKING,
        "virtual-geometry-feedback:runtime-prepare-page-requests"
    );
}

#[test]
fn runtime_prepare_feedback_binding_uses_the_real_wgpu_buffer_contract() {
    let source = include_str!("../virtual_geometry_plugin_renderer_outputs.rs");

    assert!(source.contains("buffer.size()"));
    assert!(source.contains("register_external_buffer_binding_with_backing_and_physical_desc("));
    assert!(source.contains("BufferUsage::COPY_SRC | BufferUsage::COPY_DST | BufferUsage::STORAGE"));
}
