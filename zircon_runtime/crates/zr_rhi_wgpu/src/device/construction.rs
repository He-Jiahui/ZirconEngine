use std::sync::{Arc, Mutex};

use zr_rhi::{
    DeviceGeneration, DeviceId, GpuMemoryBudget, RenderBackendCaps, RenderResourceHandleAllocator,
    RenderSurfaceHandleAllocator, SubmissionHistory, SubmissionLimits, SurfaceFrameTerminalHistory,
};

use super::contract_caps::deterministic_contract_caps;
use super::{DeterministicRhiContractDevice, DeterministicRhiContractDeviceState};

impl DeterministicRhiContractDevice {
    pub(crate) fn new_headless() -> Self {
        Self::new_headless_with_config(
            deterministic_contract_caps(),
            GpuMemoryBudget::default(),
            SubmissionLimits::default(),
        )
    }

    pub(crate) fn new_headless_with_caps(caps: RenderBackendCaps) -> Self {
        Self::new_headless_with_config(
            caps,
            GpuMemoryBudget::default(),
            SubmissionLimits::default(),
        )
    }

    pub(crate) fn new_headless_with_limits(
        memory_budget: GpuMemoryBudget,
        submission_limits: SubmissionLimits,
    ) -> Self {
        Self::new_headless_with_config(
            deterministic_contract_caps(),
            memory_budget,
            submission_limits,
        )
    }

    pub(crate) fn new_headless_with_config(
        caps: RenderBackendCaps,
        memory_budget: GpuMemoryBudget,
        submission_limits: SubmissionLimits,
    ) -> Self {
        Self::new_headless_with_identity_and_config(
            DeviceId::new(1),
            DeviceGeneration::initial(),
            caps,
            memory_budget,
            submission_limits,
        )
    }

    pub(crate) fn new_headless_with_identity(
        device_id: DeviceId,
        generation: DeviceGeneration,
    ) -> Self {
        Self::new_headless_with_identity_and_config(
            device_id,
            generation,
            deterministic_contract_caps(),
            GpuMemoryBudget::default(),
            SubmissionLimits::default(),
        )
    }

    fn new_headless_with_identity_and_config(
        device_id: DeviceId,
        generation: DeviceGeneration,
        caps: RenderBackendCaps,
        memory_budget: GpuMemoryBudget,
        submission_limits: SubmissionLimits,
    ) -> Self {
        Self {
            caps,
            device_id,
            generation,
            memory_budget,
            handle_allocator: RenderResourceHandleAllocator::new(device_id, generation),
            surface_handle_allocator: RenderSurfaceHandleAllocator::new(device_id, generation),
            state: Arc::new(Mutex::new(DeterministicRhiContractDeviceState {
                next_submission_sequence: 1,
                submission_history: SubmissionHistory::new(submission_limits),
                terminal_surface_frames: SurfaceFrameTerminalHistory::new(
                    submission_limits.max_terminal_statuses(),
                ),
                ..DeterministicRhiContractDeviceState::default()
            })),
        }
    }
}
