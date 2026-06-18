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
