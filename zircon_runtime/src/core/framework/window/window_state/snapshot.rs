use super::{
    WindowCreateGeneration, WindowCreateSpec, WindowEffectiveGeneration, WindowEffectiveState,
    WindowRequestedGeneration, WindowRequestedState,
};
use crate::core::framework::window::{WindowId, WindowObservedGeneration};

/// Immutable create-time window intent with its independent publication
/// generation.
#[derive(Clone, Debug, PartialEq)]
pub struct WindowCreateSnapshot {
    generation: WindowCreateGeneration,
    spec: WindowCreateSpec,
}

impl WindowCreateSnapshot {
    pub(crate) fn new(generation: WindowCreateGeneration, spec: WindowCreateSpec) -> Self {
        Self { generation, spec }
    }

    pub const fn generation(&self) -> WindowCreateGeneration {
        self.generation
    }

    pub const fn spec(&self) -> &WindowCreateSpec {
        &self.spec
    }
}

/// The latest runtime request accepted for a window generation.
#[derive(Clone, Debug, PartialEq)]
pub struct WindowRequestedSnapshot {
    generation: WindowRequestedGeneration,
    state: WindowRequestedState,
}

impl WindowRequestedSnapshot {
    pub(crate) fn new(generation: WindowRequestedGeneration, state: WindowRequestedState) -> Self {
        Self { generation, state }
    }

    pub const fn generation(&self) -> WindowRequestedGeneration {
        self.generation
    }

    pub const fn state(&self) -> &WindowRequestedState {
        &self.state
    }
}

/// The latest OS-owned window observation for a window generation.
#[derive(Clone, Debug, PartialEq)]
pub struct WindowObservedSnapshot {
    generation: WindowObservedGeneration,
    state: super::WindowObservedState,
}

impl WindowObservedSnapshot {
    pub(crate) fn new(
        generation: WindowObservedGeneration,
        state: super::WindowObservedState,
    ) -> Self {
        Self { generation, state }
    }

    pub const fn generation(&self) -> WindowObservedGeneration {
        self.generation
    }

    pub const fn state(&self) -> &super::WindowObservedState {
        &self.state
    }
}

/// The host-accepted configuration reflected in a terminal command receipt.
#[derive(Clone, Debug, PartialEq)]
pub struct WindowEffectiveSnapshot {
    generation: WindowEffectiveGeneration,
    requested_generation: WindowRequestedGeneration,
    state: WindowEffectiveState,
}

impl WindowEffectiveSnapshot {
    pub(crate) fn new(
        generation: WindowEffectiveGeneration,
        requested_generation: WindowRequestedGeneration,
        state: WindowEffectiveState,
    ) -> Self {
        Self {
            generation,
            requested_generation,
            state,
        }
    }

    pub const fn generation(&self) -> WindowEffectiveGeneration {
        self.generation
    }

    /// Identifies the accepted desired-state generation that produced this
    /// host-observed effective configuration. It may be older than the latest
    /// requested state while a newer native command remains outstanding.
    pub const fn requested_generation(&self) -> WindowRequestedGeneration {
        self.requested_generation
    }

    pub const fn state(&self) -> &WindowEffectiveState {
        &self.state
    }
}

/// One read-only projection of every state authority associated with a live
/// engine window generation.
#[derive(Clone, Debug, PartialEq)]
pub struct WindowStateSnapshot {
    window: WindowId,
    create: WindowCreateSnapshot,
    requested: WindowRequestedSnapshot,
    observed: WindowObservedSnapshot,
    effective: WindowEffectiveSnapshot,
}

impl WindowStateSnapshot {
    pub(crate) fn new(
        window: WindowId,
        create: WindowCreateSnapshot,
        requested: WindowRequestedSnapshot,
        observed: WindowObservedSnapshot,
        effective: WindowEffectiveSnapshot,
    ) -> Self {
        Self {
            window,
            create,
            requested,
            observed,
            effective,
        }
    }

    pub const fn window(&self) -> WindowId {
        self.window
    }

    pub const fn create(&self) -> &WindowCreateSnapshot {
        &self.create
    }

    pub const fn requested(&self) -> &WindowRequestedSnapshot {
        &self.requested
    }

    pub const fn observed(&self) -> &WindowObservedSnapshot {
        &self.observed
    }

    pub const fn effective(&self) -> &WindowEffectiveSnapshot {
        &self.effective
    }
}
