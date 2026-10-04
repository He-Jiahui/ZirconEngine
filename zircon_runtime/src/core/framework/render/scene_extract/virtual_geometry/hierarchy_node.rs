/// 扁平层级中的节点范围；裁剪器按 child 与 cluster 区间遍历同一提取包。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct RenderVirtualGeometryHierarchyNode {
    pub instance_index: u32,
    pub node_id: u32,
    pub child_base: u32,
    pub child_count: u32,
    pub cluster_start: u32,
    pub cluster_count: u32,
}
