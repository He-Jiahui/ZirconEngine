/// 标记数据来自真实渲染路径还是回退快照；不能仅凭条目非空推断 GPU 路径成功。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RenderVirtualGeometryVisBuffer64Source {
    #[default]
    Unavailable,
    RenderPathClearOnly,
    RenderPathExecutionSelections,
    SnapshotFallback,
    GpuReadbackFallback,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RenderVirtualGeometryHardwareRasterizationSource {
    #[default]
    Unavailable,
    RenderPathClearOnly,
    RenderPathExecutionSelections,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RenderVirtualGeometryNodeAndClusterCullSource {
    #[default]
    Unavailable,
    RenderPathClearOnly,
    RenderPathCullInput,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RenderVirtualGeometrySelectedClusterSource {
    #[default]
    Unavailable,
    RenderPathClearOnly,
    RenderPathExecutionSelections,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RenderVirtualGeometryClusterSelectionInputSource {
    #[default]
    Unavailable,
    ExplicitFrameOwned,
    PrepareDerivedFrameOwned,
    PrepareOnDemand,
}
