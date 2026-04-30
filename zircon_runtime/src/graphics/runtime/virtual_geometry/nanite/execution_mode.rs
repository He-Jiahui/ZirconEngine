#![allow(dead_code)]

use crate::core::framework::render::RenderCapabilitySummary;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum VirtualGeometryExecutionMode {
    #[default]
    CpuDebug,
    BaselineGpu,
    FlagshipGpu,
}

impl VirtualGeometryExecutionMode {
    pub(crate) fn from_capabilities(capabilities: &RenderCapabilitySummary) -> Self {
        if capabilities.virtual_geometry_supported {
            return Self::FlagshipGpu;
        }

        if capabilities.supports_offscreen || capabilities.supports_surface {
            return Self::BaselineGpu;
        }

        Self::CpuDebug
    }
}
