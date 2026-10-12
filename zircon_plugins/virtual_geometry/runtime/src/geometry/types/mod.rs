//! 定义准备帧、簇选择和 GPU 遍历工作项在插件内部流转的数据结构。
#[path = "../../virtual_geometry_cluster_raster.rs"]
mod virtual_geometry_cluster_raster_draw;
#[path = "../../virtual_geometry_cluster.rs"]
mod virtual_geometry_cluster_selection;
#[path = "../../and_cluster_cull_child_work_item.rs"]
mod virtual_geometry_node_and_cluster_cull_child_work_item;
#[path = "../../virtual_geometry_node_and_cluster.rs"]
mod virtual_geometry_node_and_cluster_cull_cluster_work_item;
#[path = "../../and_cluster_cull_traversal_record.rs"]
mod virtual_geometry_node_and_cluster_cull_traversal_record;
#[path = "prepare/mod.rs"]
mod virtual_geometry_prepare;

pub(crate) use virtual_geometry_cluster_raster_draw::VirtualGeometryClusterRasterDraw;
pub(crate) use virtual_geometry_cluster_selection::{
    build_cluster_selections, cluster_raster_draws_from_selections, VirtualGeometryClusterSelection,
};
pub(crate) use virtual_geometry_node_and_cluster_cull_child_work_item::VirtualGeometryNodeAndClusterCullChildWorkItem;
pub(crate) use virtual_geometry_node_and_cluster_cull_cluster_work_item::VirtualGeometryNodeAndClusterCullClusterWorkItem;
pub(crate) use virtual_geometry_node_and_cluster_cull_traversal_record::{
    VirtualGeometryNodeAndClusterCullTraversalChildSource,
    VirtualGeometryNodeAndClusterCullTraversalOp, VirtualGeometryNodeAndClusterCullTraversalRecord,
};
pub(crate) use virtual_geometry_prepare::{
    VirtualGeometryPrepareCluster, VirtualGeometryPrepareClusterState,
    VirtualGeometryPrepareDrawSegment, VirtualGeometryPrepareFrame,
    VirtualGeometryPrepareIndirectDraw, VirtualGeometryPreparePage, VirtualGeometryPrepareRequest,
};
