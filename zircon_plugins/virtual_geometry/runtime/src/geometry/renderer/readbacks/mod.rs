// 旧渲染器状态投影辅助模块；目前没有生产调用方，测试专用读取器也未连入旧的大型测试源。
#[path = "../../../read_gpu_readback_hardware.rs"]
mod read_gpu_readback_hardware_rasterization_record_count;
#[path = "../../../read_gpu_readback.rs"]
mod read_gpu_readback_hardware_rasterization_source;
#[path = "../../../read_gpu_readback_selected.rs"]
mod read_gpu_readback_selected_cluster_count;
#[path = "../../../readback_selected_cluster_source.rs"]
mod read_gpu_readback_selected_cluster_source;
#[path = "../../../gpu_readback_selected_clusters.rs"]
mod read_gpu_readback_selected_clusters;
#[path = "../../../read_gpu_readback_visbuffer64.rs"]
mod read_gpu_readback_visbuffer64;
#[path = "../../../readback_visbuffer64_entry_count.rs"]
mod read_gpu_readback_visbuffer64_entry_count;
#[path = "../../../gpu_readback_visbuffer64_source.rs"]
mod read_gpu_readback_visbuffer64_source;
#[path = "../../../read_indirect_authority_records.rs"]
mod read_indirect_authority_records;
#[path = "../../../read_indirect_execution_indices.rs"]
mod read_indirect_execution_indices;
#[path = "../../../read_indirect_execution_records.rs"]
mod read_indirect_execution_records;
#[path = "../../../read_indirect_execution_segments.rs"]
mod read_indirect_execution_segments;
#[path = "../../read_indirect_segments.rs"]
mod read_indirect_segments;
#[path = "../../../read_node_and_cluster_cull_child.rs"]
mod read_node_and_cluster_cull_child_work_items;
#[path = "../../../read_node_and_cluster_cull.rs"]
mod read_node_and_cluster_cull_cluster_work_items;
#[path = "../../../read_node_and_cluster.rs"]
mod read_node_and_cluster_cull_dispatch_setup_snapshot;
#[path = "../../../read_node_and_cluster_cull_global.rs"]
mod read_node_and_cluster_cull_global_state_snapshot;
#[path = "../../../cluster_cull_hierarchy_child_ids.rs"]
mod read_node_and_cluster_cull_hierarchy_child_ids;
#[path = "../../../and_cluster_cull_instance_seeds.rs"]
mod read_node_and_cluster_cull_instance_seeds;
#[path = "../../../readbacks_read_node_and_cluster.rs"]
mod read_node_and_cluster_cull_instance_work_items;
#[path = "../../../read_node_and_cluster_cull_launch.rs"]
mod read_node_and_cluster_cull_launch_worklist_snapshot;
#[path = "../../../read_node_and_cluster_cull_source.rs"]
mod read_node_and_cluster_cull_source;
#[path = "../../../renderer_read_node_and_cluster.rs"]
mod read_node_and_cluster_cull_traversal_records;
#[path = "../../take_gpu_complete_parts.rs"]
mod take_gpu_completion_parts;
#[path = "../../take_gpu_readback.rs"]
mod take_gpu_readback;
