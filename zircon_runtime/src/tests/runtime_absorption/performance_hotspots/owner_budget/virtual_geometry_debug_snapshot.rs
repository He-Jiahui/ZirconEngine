#[test]
fn runtime_07_virtual_geometry_debug_snapshot_owner_split_keeps_contracts_folder_backed() {
    let root = include_str!("../../../../core/framework/render/virtual_geometry_debug_snapshot.rs");
    let bvh = include_str!(
        "../../../../core/framework/render/virtual_geometry_debug_snapshot/bvh_visualization.rs"
    );
    let cpu_reference = include_str!(
        "../../../../core/framework/render/virtual_geometry_debug_snapshot/cpu_reference.rs"
    );
    let cull_input = include_str!(
        "../../../../core/framework/render/virtual_geometry_debug_snapshot/cull_input.rs"
    );
    let execution = include_str!(
        "../../../../core/framework/render/virtual_geometry_debug_snapshot/execution.rs"
    );
    let node_and_cluster = include_str!(
        "../../../../core/framework/render/virtual_geometry_debug_snapshot/node_and_cluster_cull.rs"
    );
    let page_payload = include_str!(
        "../../../../core/framework/render/virtual_geometry_debug_snapshot/page_payload.rs"
    );
    let snapshot = include_str!(
        "../../../../core/framework/render/virtual_geometry_debug_snapshot/snapshot.rs"
    );
    let sources = include_str!(
        "../../../../core/framework/render/virtual_geometry_debug_snapshot/sources.rs"
    );
    let module_doc = include_str!(
        "../../../../../../docs/zircon_runtime/core/framework/render/virtual_geometry_debug_snapshot.md"
    );
    let hotspot_doc =
        include_str!("../../../../../../docs/zircon_runtime/performance/hotspot_inventory.md");

    for root_anchor in [
        "mod bvh_visualization;",
        "mod cpu_reference;",
        "mod cull_input;",
        "mod execution;",
        "mod node_and_cluster_cull;",
        "mod page_payload;",
        "mod snapshot;",
        "mod sources;",
        "pub use snapshot::RenderVirtualGeometryDebugSnapshot;",
    ] {
        assert!(
            root.contains(root_anchor),
            "virtual geometry debug snapshot root should stay structural with `{root_anchor}`"
        );
    }

    for root_forbidden in [
        "pub struct RenderVirtualGeometryDebugSnapshot {",
        "pub struct RenderVirtualGeometryCullInputSnapshot {",
        "pub struct RenderVirtualGeometryNodeAndClusterCullLaunchWorklistSnapshot {",
        "pub struct RenderVirtualGeometryCpuReferenceInstance {",
    ] {
        assert!(
            !root.contains(root_forbidden),
            "virtual geometry debug snapshot root should not regain owner declaration `{root_forbidden}`"
        );
    }

    for (source_name, source, owner_anchor) in [
        (
            "bvh_visualization.rs",
            bvh,
            "pub struct RenderVirtualGeometryBvhVisualizationInstance",
        ),
        (
            "cpu_reference.rs",
            cpu_reference,
            "pub struct RenderVirtualGeometryCpuReferenceInstance",
        ),
        (
            "cull_input.rs",
            cull_input,
            "pub struct RenderVirtualGeometryCullInputSnapshot",
        ),
        (
            "execution.rs",
            execution,
            "pub struct RenderVirtualGeometryVisBuffer64Entry",
        ),
        (
            "node_and_cluster_cull.rs",
            node_and_cluster,
            "pub struct RenderVirtualGeometryNodeAndClusterCullLaunchWorklistSnapshot",
        ),
        (
            "page_payload.rs",
            page_payload,
            "pub struct RenderVirtualGeometryPagePayload",
        ),
        (
            "snapshot.rs",
            snapshot,
            "pub struct RenderVirtualGeometryDebugSnapshot",
        ),
        (
            "sources.rs",
            sources,
            "pub enum RenderVirtualGeometryClusterSelectionInputSource",
        ),
    ] {
        assert!(
            source.contains(owner_anchor),
            "{source_name} should own `{owner_anchor}` after the folder split"
        );
    }

    assert!(
        node_and_cluster.contains("RenderVirtualGeometryCullInputSnapshot::GPU_WORD_COUNT"),
        "NodeAndClusterCull layout should consume the cull-input owner instead of duplicating cull words"
    );
    assert!(
        snapshot.contains("use super::node_and_cluster_cull::{"),
        "top-level debug snapshot should compose the NodeAndClusterCull owner through the folder boundary"
    );
    assert!(
        module_doc.contains(
            "virtual_geometry_debug_snapshot/{cull_input,node_and_cluster_cull,page_payload,snapshot}.rs"
        ) && hotspot_doc
            .contains("virtual_geometry_debug_snapshot_owner_split_static_passed_cargo_deferred"),
        "Runtime 07 docs should record the virtual geometry debug snapshot owner split"
    );
}
