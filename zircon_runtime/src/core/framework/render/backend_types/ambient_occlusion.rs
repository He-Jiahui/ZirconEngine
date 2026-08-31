#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum RenderAmbientOcclusionExecutionStatus {
    #[default]
    Disabled,
    Ready,
    Unsupported,
    UsingLastGood,
    Recovering,
    Failed,
}

impl RenderAmbientOcclusionExecutionStatus {
    pub const fn code(self) -> u8 {
        match self {
            Self::Disabled => 0,
            Self::Ready => 1,
            Self::Unsupported => 2,
            Self::UsingLastGood => 3,
            Self::Recovering => 4,
            Self::Failed => 5,
        }
    }

    pub const fn label(self) -> &'static str {
        match self {
            Self::Disabled => "disabled",
            Self::Ready => "ready",
            Self::Unsupported => "unsupported",
            Self::UsingLastGood => "using_last_good",
            Self::Recovering => "recovering",
            Self::Failed => "failed",
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct RenderAmbientOcclusionExecutionFailureFlags(u32);

impl RenderAmbientOcclusionExecutionFailureFlags {
    pub const NONE: Self = Self(0);
    pub const MISSING_COMPILED_CONTRACT: Self = Self(1 << 0);
    pub const GENERATION_MISMATCH: Self = Self(1 << 1);
    pub const OUTPUT_PRODUCER_MISMATCH: Self = Self(1 << 2);
    pub const UNEXPECTED_DISABLED_WORK: Self = Self(1 << 3);
    pub const EVALUATE_PASS: Self = Self(1 << 4);
    pub const EVALUATE_DISPATCH: Self = Self(1 << 5);
    pub const EVALUATE_RAW_WRITE: Self = Self(1 << 6);
    pub const SPATIAL_PASS: Self = Self(1 << 7);
    pub const SPATIAL_DISPATCH: Self = Self(1 << 8);
    pub const SPATIAL_RAW_READ: Self = Self(1 << 9);
    pub const SPATIAL_FINAL_WRITE: Self = Self(1 << 10);
    pub const LIGHTING_PASS: Self = Self(1 << 11);
    pub const LIGHTING_FINAL_READ: Self = Self(1 << 12);
    pub const EVALUATE_PIPELINE_RESOLUTION: Self = Self(1 << 13);
    pub const SPATIAL_PIPELINE_RESOLUTION: Self = Self(1 << 14);
    pub const PIPELINE_DEVICE_EPOCH_MISMATCH: Self = Self(1 << 15);
    pub const UPSAMPLE_PASS: Self = Self(1 << 16);
    pub const UPSAMPLE_DISPATCH: Self = Self(1 << 17);
    pub const SPATIAL_INTERMEDIATE_WRITE: Self = Self(1 << 18);
    pub const UPSAMPLE_SPATIAL_READ: Self = Self(1 << 19);
    pub const UPSAMPLE_FINAL_WRITE: Self = Self(1 << 20);
    pub const UPSAMPLE_PIPELINE_RESOLUTION: Self = Self(1 << 21);

    pub const fn bits(self) -> u32 {
        self.0
    }

    pub const fn is_empty(self) -> bool {
        self.0 == 0
    }

    pub const fn contains(self, flag: Self) -> bool {
        self.0 & flag.0 == flag.0
    }

    pub(crate) fn insert(&mut self, flag: Self) {
        self.0 |= flag.0;
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct RenderAmbientOcclusionExecutionReport {
    pub status: RenderAmbientOcclusionExecutionStatus,
    pub failure_flags: RenderAmbientOcclusionExecutionFailureFlags,
    pub frame_generation: u64,
    pub profile_artifact_version: u32,
    pub profile_compiler_version: u32,
    pub shader_interface_version: u32,
    pub pipeline_generation: u64,
    pub output_generation: u64,
    pub device_id: Option<u64>,
    pub device_generation: Option<u64>,
    pub evaluate_candidate_artifact_fingerprint: u64,
    pub evaluate_resolved_artifact_fingerprint: u64,
    pub spatial_candidate_artifact_fingerprint: u64,
    pub spatial_resolved_artifact_fingerprint: u64,
    pub upsample_candidate_artifact_fingerprint: u64,
    pub upsample_resolved_artifact_fingerprint: u64,
    pub last_good_dispatch_count: usize,
    pub expected_pass_count: usize,
    pub recorded_pass_count: usize,
    pub evaluate_pass_count: usize,
    pub spatial_pass_count: usize,
    pub upsample_pass_count: usize,
    pub lighting_pass_count: usize,
    pub evaluate_dispatch_count: usize,
    pub spatial_dispatch_count: usize,
    pub upsample_dispatch_count: usize,
    pub evaluate_dispatch_group_count: usize,
    pub spatial_dispatch_group_count: usize,
    pub upsample_dispatch_group_count: usize,
    pub evaluate_raw_write_count: usize,
    pub spatial_raw_read_count: usize,
    pub spatial_final_write_count: usize,
    pub spatial_intermediate_write_count: usize,
    pub upsample_spatial_read_count: usize,
    pub upsample_final_write_count: usize,
    pub lighting_final_read_count: usize,
}

#[cfg(test)]
mod tests {
    use super::RenderAmbientOcclusionExecutionFailureFlags as FailureFlags;

    #[test]
    fn ambient_occlusion_execution_failure_flags_are_a_dense_22_bit_contract() {
        let flags = [
            FailureFlags::MISSING_COMPILED_CONTRACT,
            FailureFlags::GENERATION_MISMATCH,
            FailureFlags::OUTPUT_PRODUCER_MISMATCH,
            FailureFlags::UNEXPECTED_DISABLED_WORK,
            FailureFlags::EVALUATE_PASS,
            FailureFlags::EVALUATE_DISPATCH,
            FailureFlags::EVALUATE_RAW_WRITE,
            FailureFlags::SPATIAL_PASS,
            FailureFlags::SPATIAL_DISPATCH,
            FailureFlags::SPATIAL_RAW_READ,
            FailureFlags::SPATIAL_FINAL_WRITE,
            FailureFlags::LIGHTING_PASS,
            FailureFlags::LIGHTING_FINAL_READ,
            FailureFlags::EVALUATE_PIPELINE_RESOLUTION,
            FailureFlags::SPATIAL_PIPELINE_RESOLUTION,
            FailureFlags::PIPELINE_DEVICE_EPOCH_MISMATCH,
            FailureFlags::UPSAMPLE_PASS,
            FailureFlags::UPSAMPLE_DISPATCH,
            FailureFlags::SPATIAL_INTERMEDIATE_WRITE,
            FailureFlags::UPSAMPLE_SPATIAL_READ,
            FailureFlags::UPSAMPLE_FINAL_WRITE,
            FailureFlags::UPSAMPLE_PIPELINE_RESOLUTION,
        ];

        assert_eq!(flags.len(), 22);
        assert_eq!(
            flags
                .into_iter()
                .fold(0_u32, |bits, flag| bits | flag.bits()),
            (1_u32 << 22) - 1
        );
    }
}
