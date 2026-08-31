use std::error::Error;
use std::fmt;

use zircon_runtime_interface::ZrRuntimeViewportHandle;

use super::SurfaceLease;
use crate::core::framework::window::{DisplayId, DisplayTopologyGeneration, WindowId};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SurfaceLeaseError {
    InvalidViewport {
        viewport: ZrRuntimeViewportHandle,
    },
    TopologyGenerationMismatch {
        requested: DisplayTopologyGeneration,
        observed: DisplayTopologyGeneration,
    },
    OutputUnavailable {
        output: DisplayId,
        topology_generation: DisplayTopologyGeneration,
    },
    ReplacementInFlight {
        window: WindowId,
        viewport: ZrRuntimeViewportHandle,
    },
    ViewportAlreadyBound {
        window: WindowId,
        viewport: ZrRuntimeViewportHandle,
    },
    InconsistentViewportBinding {
        window: WindowId,
        viewport: ZrRuntimeViewportHandle,
    },
    LeaseRetiring {
        lease: SurfaceLease,
    },
    WindowHasPreparedLease {
        window: WindowId,
    },
    StaleLease {
        lease: SurfaceLease,
    },
    LeaseNotRetiring {
        lease: SurfaceLease,
    },
    GenerationExhausted,
    CapacityExhausted,
}

impl fmt::Display for SurfaceLeaseError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidViewport { viewport } => write!(
                formatter,
                "surface lease requires a valid runtime viewport handle, received {}",
                viewport.raw()
            ),
            Self::TopologyGenerationMismatch {
                requested,
                observed,
            } => write!(
                formatter,
                "surface lease requests display topology generation {} but host observed {}",
                requested.get(),
                observed.get()
            ),
            Self::OutputUnavailable {
                output,
                topology_generation,
            } => write!(
                formatter,
                "surface lease output {output} is unavailable in display topology generation {}",
                topology_generation.get()
            ),
            Self::ReplacementInFlight { window, viewport } => write!(
                formatter,
                "window registry {} slot {} generation {} viewport {} already has a prepared surface replacement",
                window.registry().raw(),
                window.slot(),
                window.generation(),
                viewport.raw()
            ),
            Self::ViewportAlreadyBound { window, viewport } => write!(
                formatter,
                "viewport {} is already bound to window registry {} slot {} generation {}",
                viewport.raw(),
                window.registry().raw(),
                window.slot(),
                window.generation()
            ),
            Self::InconsistentViewportBinding { window, viewport } => write!(
                formatter,
                "viewport {} has an inconsistent surface lease binding for window registry {} slot {} generation {}",
                viewport.raw(),
                window.registry().raw(),
                window.slot(),
                window.generation()
            ),
            Self::LeaseRetiring { lease } => write!(
                formatter,
                "surface lease window registry {} slot {} generation {} viewport {} surface generation {} is retiring",
                lease.window().registry().raw(),
                lease.window().slot(),
                lease.window().generation(),
                lease.viewport().raw(),
                lease.generation().get()
            ),
            Self::WindowHasPreparedLease { window } => write!(
                formatter,
                "window registry {} slot {} generation {} retains a prepared surface lease",
                window.registry().raw(),
                window.slot(),
                window.generation()
            ),
            Self::StaleLease { lease } => write!(
                formatter,
                "surface lease window registry {} slot {} generation {} viewport {} surface generation {} is stale",
                lease.window().registry().raw(),
                lease.window().slot(),
                lease.window().generation(),
                lease.viewport().raw(),
                lease.generation().get()
            ),
            Self::LeaseNotRetiring { lease } => write!(
                formatter,
                "surface lease window registry {} slot {} generation {} viewport {} surface generation {} has not entered retirement",
                lease.window().registry().raw(),
                lease.window().slot(),
                lease.window().generation(),
                lease.viewport().raw(),
                lease.generation().get()
            ),
            Self::GenerationExhausted => {
                formatter.write_str("surface lease generation space is exhausted")
            }
            Self::CapacityExhausted => {
                formatter.write_str("surface lease registry cannot reserve additional state")
            }
        }
    }
}

impl Error for SurfaceLeaseError {}
