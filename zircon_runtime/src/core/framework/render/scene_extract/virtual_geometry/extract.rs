use super::{
    RenderVirtualGeometryCluster, RenderVirtualGeometryDebugState,
    RenderVirtualGeometryHierarchyNode, RenderVirtualGeometryInstance, RenderVirtualGeometryPage,
    RenderVirtualGeometryPageDependency,
};

#[derive(Clone, Debug, PartialEq)]
pub struct RenderVirtualGeometryExtract {
    pub cluster_budget: u32,
    pub page_budget: u32,
    pub clusters: Vec<RenderVirtualGeometryCluster>,
    pub hierarchy_nodes: Vec<RenderVirtualGeometryHierarchyNode>,
    pub hierarchy_child_ids: Vec<u32>,
    pub pages: Vec<RenderVirtualGeometryPage>,
    pub page_dependencies: Vec<RenderVirtualGeometryPageDependency>,
    pub instances: Vec<RenderVirtualGeometryInstance>,
    pub debug: RenderVirtualGeometryDebugState,
}

impl Default for RenderVirtualGeometryExtract {
    fn default() -> Self {
        Self {
            cluster_budget: 0,
            page_budget: 0,
            clusters: Vec::new(),
            hierarchy_nodes: Vec::new(),
            hierarchy_child_ids: Vec::new(),
            pages: Vec::new(),
            page_dependencies: Vec::new(),
            instances: Vec::new(),
            debug: RenderVirtualGeometryDebugState::default(),
        }
    }
}
