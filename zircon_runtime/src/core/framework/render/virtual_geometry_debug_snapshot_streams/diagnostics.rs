use super::{
    RenderVirtualGeometryDebugSnapshotReadbackStreamDecodeError,
    RenderVirtualGeometryDebugSnapshotReadbackStreamFootprint,
    RenderVirtualGeometryDebugSnapshotReadbackStreamReport,
    RenderVirtualGeometryNodeAndClusterCullWordStreamDecodeError,
    RenderVirtualGeometryRenderPathWordStreamDecodeError,
    RenderVirtualGeometryVisBuffer64ReadbackStreamDecodeError,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RenderVirtualGeometryDebugSnapshotReadbackStreamSection {
    NodeAndClusterCull,
    RenderPath,
    VisBuffer64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RenderVirtualGeometryDebugSnapshotReadbackStreamDecodeDiagnostic {
    pub section: RenderVirtualGeometryDebugSnapshotReadbackStreamSection,
    pub section_u32_word_count: usize,
    pub section_byte_count: usize,
    pub section_payload_u32_word_count: usize,
    pub section_payload_byte_count: usize,
    pub malformed_u32_word_count: Option<usize>,
}

impl RenderVirtualGeometryDebugSnapshotReadbackStreamDecodeError {
    pub fn section(&self) -> RenderVirtualGeometryDebugSnapshotReadbackStreamSection {
        match self {
            Self::NodeAndClusterCull(_) => {
                RenderVirtualGeometryDebugSnapshotReadbackStreamSection::NodeAndClusterCull
            }
            Self::RenderPath(_) => {
                RenderVirtualGeometryDebugSnapshotReadbackStreamSection::RenderPath
            }
            Self::VisBuffer64(_) => {
                RenderVirtualGeometryDebugSnapshotReadbackStreamSection::VisBuffer64
            }
        }
    }

    pub fn malformed_u32_word_count(&self) -> Option<usize> {
        match self {
            Self::NodeAndClusterCull(error) => Some(error.malformed_u32_word_count()),
            Self::RenderPath(error) => Some(error.malformed_u32_word_count()),
            Self::VisBuffer64(error) => error.malformed_u32_word_count(),
        }
    }
}

impl RenderVirtualGeometryDebugSnapshotReadbackStreamDecodeDiagnostic {
    fn from_decode_error(
        error: RenderVirtualGeometryDebugSnapshotReadbackStreamDecodeError,
        footprint: &RenderVirtualGeometryDebugSnapshotReadbackStreamFootprint,
    ) -> Self {
        let section = error.section();

        Self {
            section,
            section_u32_word_count: footprint.section_u32_word_count(section),
            section_byte_count: footprint.section_byte_count(section),
            section_payload_u32_word_count: footprint.section_payload_u32_word_count(section),
            section_payload_byte_count: footprint.section_payload_byte_count(section),
            malformed_u32_word_count: error.malformed_u32_word_count(),
        }
    }
}

impl RenderVirtualGeometryNodeAndClusterCullWordStreamDecodeError {
    pub fn malformed_u32_word_count(&self) -> usize {
        match self {
            Self::GlobalState { word_count }
            | Self::DispatchSetup { word_count }
            | Self::LaunchWorklist { word_count }
            | Self::InstanceSeeds { word_count }
            | Self::InstanceWorkItems { word_count }
            | Self::ClusterWorkItems { word_count }
            | Self::ChildWorkItems { word_count }
            | Self::TraversalRecords { word_count } => *word_count,
        }
    }
}

impl RenderVirtualGeometryRenderPathWordStreamDecodeError {
    pub fn malformed_u32_word_count(&self) -> usize {
        match self {
            Self::SelectedClusters { word_count }
            | Self::HardwareRasterizationRecords { word_count } => *word_count,
        }
    }
}

impl RenderVirtualGeometryVisBuffer64ReadbackStreamDecodeError {
    pub fn malformed_u32_word_count(&self) -> Option<usize> {
        None
    }
}

impl RenderVirtualGeometryDebugSnapshotReadbackStreamReport {
    pub fn decode_error_section(
        &self,
    ) -> Option<RenderVirtualGeometryDebugSnapshotReadbackStreamSection> {
        self.decode_error.map(|error| error.section())
    }

    pub fn decode_diagnostic(
        &self,
    ) -> Option<RenderVirtualGeometryDebugSnapshotReadbackStreamDecodeDiagnostic> {
        self.decode_error.map(|error| {
            RenderVirtualGeometryDebugSnapshotReadbackStreamDecodeDiagnostic::from_decode_error(
                error,
                &self.footprint,
            )
        })
    }
}
