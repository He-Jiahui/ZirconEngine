use super::skybox::IblBakeKey;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RealtimeIblFailureKind {
    Recording,
    Submission,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RealtimeIblReadiness {
    Fallback,
    Baking,
    Ready,
    RefreshingLastGood,
    FailedFallback,
    FailedLastGood,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RealtimeIblFailureOperation {
    CaptureSky,
    GenerateSourceMip,
    Prefilter,
    ProjectDiffuseSh9,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RealtimeIblFailureReport {
    pub bake_key: IblBakeKey,
    pub generation: u64,
    pub logical_state: u8,
    pub substep: u8,
    pub operation: RealtimeIblFailureOperation,
    pub failure_kind: RealtimeIblFailureKind,
    pub failed_attempt_count: u8,
    pub retry_not_before_frame: Option<u64>,
    pub terminal: bool,
    pub last_good_available: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RealtimeIblStatusReport {
    pub readiness: RealtimeIblReadiness,
    pub current_frame_number: u64,
    pub published_key: Option<IblBakeKey>,
    pub pending_key: Option<IblBakeKey>,
    pub queued_key: Option<IblBakeKey>,
    pub published_generation_frame_number: Option<u64>,
    pub last_good_age_frame_count: Option<u64>,
    pub active_generation_start_frame_number: Option<u64>,
    pub active_generation_elapsed_frame_count: Option<u64>,
    pub active_generation_coalesced_source_change_count: u64,
    pub failure: Option<RealtimeIblFailureReport>,
}
