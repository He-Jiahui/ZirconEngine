use crate::core::framework::render::{
    IblBakeKey, RealtimeIblFailureKind, RealtimeIblFailureOperation, RealtimeIblFailureReport,
    RealtimeIblReadiness,
};

const CUBE_FACE_COUNT: u8 = 6;
const REALTIME_IBL_MAX_CONSECUTIVE_FAILURES: u8 = 3;
const REALTIME_IBL_INITIAL_RETRY_BACKOFF_FRAMES: u64 = 1;
const REALTIME_IBL_MAX_RETRY_BACKOFF_FRAMES: u64 = 4;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::graphics) struct RealtimeIblTimeSliceConfig {
    pmrem_mip_count: u8,
    capture_faces_per_frame: u8,
}

impl RealtimeIblTimeSliceConfig {
    pub(in crate::graphics) fn try_new(
        pmrem_mip_count: u8,
        capture_faces_per_frame: u8,
    ) -> Option<Self> {
        (pmrem_mip_count > 0
            && capture_faces_per_frame > 0
            && capture_faces_per_frame <= CUBE_FACE_COUNT)
            .then_some(Self {
                pmrem_mip_count,
                capture_faces_per_frame,
            })
    }

    fn topology_cache_capacity(self) -> usize {
        let face_batches = usize::from(CUBE_FACE_COUNT.div_ceil(self.capture_faces_per_frame));
        let stages = face_batches * 2 + usize::from(self.pmrem_mip_count) * 2 - 1;
        stages * 2
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(in crate::graphics) enum IblRealtimeBufferSlot {
    A,
    B,
}

impl IblRealtimeBufferSlot {
    const fn other(self) -> Self {
        match self {
            Self::A => Self::B,
            Self::B => Self::A,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(in crate::graphics) struct CubeFaceRange {
    pub first: u8,
    pub count: u8,
}

impl CubeFaceRange {
    pub const ALL: Self = Self::new(0, CUBE_FACE_COUNT);

    pub const fn new(first: u8, count: u8) -> Self {
        Self { first, count }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(in crate::graphics) struct CubeMipRange {
    pub first: u8,
    pub count: u8,
}

impl CubeMipRange {
    pub const fn new(first: u8, count: u8) -> Self {
        Self { first, count }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(in crate::graphics) enum RealtimeIblOperation {
    CaptureSky(CubeFaceRange),
    GenerateSourceMip {
        mip_level: u8,
    },
    Prefilter {
        mips: CubeMipRange,
        faces: CubeFaceRange,
    },
    ProjectDiffuseSh9,
}

fn realtime_ibl_failure_operation(operation: RealtimeIblOperation) -> RealtimeIblFailureOperation {
    match operation {
        RealtimeIblOperation::CaptureSky(_) => RealtimeIblFailureOperation::CaptureSky,
        RealtimeIblOperation::GenerateSourceMip { .. } => {
            RealtimeIblFailureOperation::GenerateSourceMip
        }
        RealtimeIblOperation::Prefilter { .. } => RealtimeIblFailureOperation::Prefilter,
        RealtimeIblOperation::ProjectDiffuseSh9 => RealtimeIblFailureOperation::ProjectDiffuseSh9,
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::graphics) enum RealtimeIblSliceAttempt {
    Succeeded,
    Failed(RealtimeIblFailureKind),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::graphics) struct RealtimeIblPrefilterDispatchSlice {
    pub mip_level: u8,
    pub first_face: u8,
    pub face_count: u8,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::graphics) struct RealtimeIblBatchToken {
    generation: u64,
    state: u8,
    substep: u8,
    attempt_frame_number: u64,
}

impl RealtimeIblBatchToken {
    pub(in crate::graphics) fn generation(self) -> u64 {
        self.generation
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(in crate::graphics) struct RealtimeIblFrameBatch {
    token: RealtimeIblBatchToken,
    ready_slot: IblRealtimeBufferSlot,
    work_slot: IblRealtimeBufferSlot,
    operations: [RealtimeIblOperation; 1],
    topology_cache_capacity: usize,
}

impl RealtimeIblFrameBatch {
    pub(in crate::graphics) fn token(&self) -> RealtimeIblBatchToken {
        self.token
    }

    pub(in crate::graphics) fn logical_state(&self) -> u8 {
        self.token.state
    }

    pub(in crate::graphics) fn ready_slot(&self) -> IblRealtimeBufferSlot {
        self.ready_slot
    }

    pub(in crate::graphics) fn work_slot(&self) -> IblRealtimeBufferSlot {
        self.work_slot
    }

    pub(in crate::graphics) fn operations(&self) -> &[RealtimeIblOperation] {
        &self.operations
    }

    pub(in crate::graphics) fn operation(&self) -> RealtimeIblOperation {
        *self
            .operations
            .first()
            .expect("realtime IBL frame batches contain exactly one operation")
    }

    pub(in crate::graphics) fn topology_cache_capacity(&self) -> usize {
        self.topology_cache_capacity
    }

    pub(in crate::graphics) fn completes_generation(&self) -> bool {
        matches!(
            self.operations.as_slice(),
            [RealtimeIblOperation::ProjectDiffuseSh9]
        )
    }

    pub(in crate::graphics) fn prefilter_dispatch_slices(
        &self,
    ) -> Vec<RealtimeIblPrefilterDispatchSlice> {
        self.operations
            .iter()
            .filter_map(|operation| match operation {
                RealtimeIblOperation::Prefilter { mips, faces } => Some((*mips, *faces)),
                _ => None,
            })
            .flat_map(|(mips, faces)| {
                (mips.first..mips.first.saturating_add(mips.count)).map(move |mip_level| {
                    RealtimeIblPrefilterDispatchSlice {
                        mip_level,
                        first_face: faces.first,
                        face_count: faces.count,
                    }
                })
            })
            .collect()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::graphics) enum RealtimeIblCompletion {
    Advanced,
    Published,
    RetryScheduled,
    Failed,
    Stale,
}

// This ticket keeps partial work private to the non-sampled slot. A newer
// source revision replaces the ticket, while the last published environment
// stays stable until a complete successor has passed its terminal SH9 stage.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct EnvironmentGenerationTicket {
    generation: u64,
    key: IblBakeKey,
    stage: EnvironmentGenerationStage,
    consecutive_failure_count: u8,
    retry_not_before_frame: Option<u64>,
}

impl EnvironmentGenerationTicket {
    fn new(generation: u64, key: IblBakeKey) -> Self {
        Self {
            generation,
            key,
            stage: EnvironmentGenerationStage::CaptureSky { first_face: 0 },
            consecutive_failure_count: 0,
            retry_not_before_frame: None,
        }
    }

    fn operation(self, config: RealtimeIblTimeSliceConfig) -> RealtimeIblOperation {
        self.stage.operation(config)
    }

    fn logical_state(self) -> u8 {
        self.stage.logical_state()
    }

    fn substep(self) -> u8 {
        self.stage.substep()
    }

    fn is_terminal(self) -> bool {
        matches!(self.stage, EnvironmentGenerationStage::ProjectDiffuseSh9)
    }

    fn advance(&mut self, config: RealtimeIblTimeSliceConfig) {
        self.stage = self.stage.advance(config);
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum EnvironmentGenerationStage {
    CaptureSky { first_face: u8 },
    GenerateSourceMip { mip_level: u8 },
    Prefilter { mip_level: u8, first_face: u8 },
    ProjectDiffuseSh9,
}

impl EnvironmentGenerationStage {
    fn operation(self, config: RealtimeIblTimeSliceConfig) -> RealtimeIblOperation {
        match self {
            Self::CaptureSky { first_face } => {
                RealtimeIblOperation::CaptureSky(CubeFaceRange::new(
                    first_face,
                    config
                        .capture_faces_per_frame
                        .min(CUBE_FACE_COUNT - first_face),
                ))
            }
            Self::GenerateSourceMip { mip_level } => {
                RealtimeIblOperation::GenerateSourceMip { mip_level }
            }
            Self::Prefilter {
                mip_level,
                first_face,
            } => RealtimeIblOperation::Prefilter {
                mips: CubeMipRange::new(mip_level, 1),
                faces: if mip_level == 0 {
                    CubeFaceRange::new(
                        first_face,
                        config
                            .capture_faces_per_frame
                            .min(CUBE_FACE_COUNT - first_face),
                    )
                } else {
                    CubeFaceRange::ALL
                },
            },
            Self::ProjectDiffuseSh9 => RealtimeIblOperation::ProjectDiffuseSh9,
        }
    }

    fn logical_state(self) -> u8 {
        match self {
            Self::CaptureSky { .. } => 0,
            Self::GenerateSourceMip { .. } => 1,
            Self::Prefilter { mip_level, .. } => 2_u8.saturating_add(mip_level),
            Self::ProjectDiffuseSh9 => u8::MAX,
        }
    }

    fn substep(self) -> u8 {
        match self {
            Self::CaptureSky { first_face } | Self::Prefilter { first_face, .. } => first_face,
            _ => 0,
        }
    }

    fn advance(self, config: RealtimeIblTimeSliceConfig) -> Self {
        match self {
            Self::CaptureSky { first_face } => {
                let next = first_face.saturating_add(config.capture_faces_per_frame);
                if next < CUBE_FACE_COUNT {
                    Self::CaptureSky { first_face: next }
                } else if config.pmrem_mip_count > 1 {
                    Self::GenerateSourceMip { mip_level: 1 }
                } else {
                    Self::Prefilter {
                        mip_level: 0,
                        first_face: 0,
                    }
                }
            }
            Self::GenerateSourceMip { mip_level } => {
                if mip_level.saturating_add(1) < config.pmrem_mip_count {
                    Self::GenerateSourceMip {
                        mip_level: mip_level.saturating_add(1),
                    }
                } else {
                    Self::Prefilter {
                        mip_level: 0,
                        first_face: 0,
                    }
                }
            }
            Self::Prefilter {
                mip_level: 0,
                first_face,
            } => {
                let next = first_face.saturating_add(config.capture_faces_per_frame);
                if next < CUBE_FACE_COUNT {
                    Self::Prefilter {
                        mip_level: 0,
                        first_face: next,
                    }
                } else if config.pmrem_mip_count > 1 {
                    Self::Prefilter {
                        mip_level: 1,
                        first_face: 0,
                    }
                } else {
                    Self::ProjectDiffuseSh9
                }
            }
            Self::Prefilter { mip_level, .. } => {
                if mip_level.saturating_add(1) < config.pmrem_mip_count {
                    Self::Prefilter {
                        mip_level: mip_level.saturating_add(1),
                        first_face: 0,
                    }
                } else {
                    Self::ProjectDiffuseSh9
                }
            }
            Self::ProjectDiffuseSh9 => Self::ProjectDiffuseSh9,
        }
    }
}

pub(in crate::graphics) struct RealtimeIblTimeSliceScheduler {
    config: RealtimeIblTimeSliceConfig,
    generation: u64,
    ticket: Option<EnvironmentGenerationTicket>,
    published_key: Option<IblBakeKey>,
    ready_slot: IblRealtimeBufferSlot,
    current_frame: Option<u64>,
    current_batch: Option<RealtimeIblFrameBatch>,
    failure_report: Option<RealtimeIblFailureReport>,
    terminal_failure_key: Option<IblBakeKey>,
}

impl RealtimeIblTimeSliceScheduler {
    pub(in crate::graphics) fn new(config: RealtimeIblTimeSliceConfig) -> Self {
        Self {
            config,
            generation: 0,
            ticket: None,
            published_key: None,
            ready_slot: IblRealtimeBufferSlot::A,
            current_frame: None,
            current_batch: None,
            failure_report: None,
            terminal_failure_key: None,
        }
    }

    pub(in crate::graphics) fn request_rebake(&mut self, key: IblBakeKey) -> bool {
        if self.ticket.is_some_and(|ticket| ticket.key == key) {
            return false;
        }
        if self.ticket.is_none() && self.terminal_failure_key == Some(key) {
            return false;
        }
        if self.published_key == Some(key) && self.ticket.is_none() {
            self.failure_report = None;
            self.terminal_failure_key = None;
            return false;
        }

        self.failure_report = None;
        self.terminal_failure_key = None;
        self.generation = self.generation.wrapping_add(1);
        self.ticket = (self.published_key != Some(key))
            .then(|| EnvironmentGenerationTicket::new(self.generation, key));
        self.current_frame = None;
        self.current_batch = None;
        true
    }

    pub(in crate::graphics) fn begin_frame(
        &mut self,
        frame_number: u64,
    ) -> Option<RealtimeIblFrameBatch> {
        let ticket = self.ticket?;
        if ticket
            .retry_not_before_frame
            .is_some_and(|retry_frame| !frame_sequence_has_reached(frame_number, retry_frame))
        {
            return None;
        }
        if self.current_frame == Some(frame_number) {
            return self.current_batch.clone();
        }
        let batch = RealtimeIblFrameBatch {
            token: RealtimeIblBatchToken {
                generation: ticket.generation,
                state: ticket.logical_state(),
                substep: ticket.substep(),
                attempt_frame_number: frame_number,
            },
            ready_slot: self.ready_slot,
            work_slot: self.ready_slot.other(),
            operations: [ticket.operation(self.config)],
            topology_cache_capacity: self.config.topology_cache_capacity(),
        };
        self.current_frame = Some(frame_number);
        self.current_batch = Some(batch.clone());
        Some(batch)
    }

    pub(in crate::graphics) fn complete_attempt(
        &mut self,
        token: RealtimeIblBatchToken,
        attempt: RealtimeIblSliceAttempt,
    ) -> RealtimeIblCompletion {
        if token.generation != self.generation
            || self
                .current_batch
                .as_ref()
                .map(RealtimeIblFrameBatch::token)
                != Some(token)
        {
            return RealtimeIblCompletion::Stale;
        }
        let attempt_frame = self
            .current_frame
            .expect("a current realtime IBL batch must own its frame number");
        self.current_batch = None;
        self.current_frame = None;

        let Some(ticket) = self.ticket.as_mut() else {
            return RealtimeIblCompletion::Stale;
        };
        if ticket.generation != token.generation {
            return RealtimeIblCompletion::Stale;
        }
        if let RealtimeIblSliceAttempt::Failed(failure_kind) = attempt {
            ticket.consecutive_failure_count = ticket.consecutive_failure_count.saturating_add(1);
            let terminal =
                ticket.consecutive_failure_count >= REALTIME_IBL_MAX_CONSECUTIVE_FAILURES;
            let retry_not_before_frame = (!terminal).then(|| {
                attempt_frame
                    .wrapping_add(retry_backoff_frames(ticket.consecutive_failure_count) + 1)
            });
            ticket.retry_not_before_frame = retry_not_before_frame;
            let failed_key = ticket.key;
            self.failure_report = Some(RealtimeIblFailureReport {
                bake_key: failed_key,
                generation: ticket.generation,
                logical_state: token.state,
                substep: token.substep,
                operation: realtime_ibl_failure_operation(ticket.operation(self.config)),
                failure_kind,
                failed_attempt_count: ticket.consecutive_failure_count,
                retry_not_before_frame,
                terminal,
                last_good_available: self.published_key.is_some(),
            });
            if terminal {
                self.ticket = None;
                self.terminal_failure_key = Some(failed_key);
                return RealtimeIblCompletion::Failed;
            }
            return RealtimeIblCompletion::RetryScheduled;
        }

        ticket.consecutive_failure_count = 0;
        ticket.retry_not_before_frame = None;
        self.failure_report = None;
        if ticket.is_terminal() {
            self.ready_slot = self.ready_slot.other();
            self.published_key = Some(ticket.key);
            self.ticket = None;
            RealtimeIblCompletion::Published
        } else {
            ticket.advance(self.config);
            RealtimeIblCompletion::Advanced
        }
    }

    #[cfg(test)]
    pub(in crate::graphics) fn complete_frame(
        &mut self,
        token: RealtimeIblBatchToken,
        gpu_succeeded: bool,
    ) -> RealtimeIblCompletion {
        self.complete_attempt(
            token,
            if gpu_succeeded {
                RealtimeIblSliceAttempt::Succeeded
            } else {
                RealtimeIblSliceAttempt::Failed(RealtimeIblFailureKind::Submission)
            },
        )
    }

    pub(in crate::graphics) fn published_key(&self) -> Option<IblBakeKey> {
        self.published_key
    }

    pub(in crate::graphics) fn pending_key(&self) -> Option<IblBakeKey> {
        self.ticket.map(|ticket| ticket.key)
    }

    pub(in crate::graphics) fn ready_slot(&self) -> IblRealtimeBufferSlot {
        self.ready_slot
    }

    pub(in crate::graphics) fn has_published_environment(&self) -> bool {
        self.published_key.is_some()
    }

    pub(in crate::graphics) fn is_rebake_pending(&self) -> bool {
        self.ticket.is_some()
    }

    pub(in crate::graphics) fn readiness(&self) -> RealtimeIblReadiness {
        if self.failure_report.is_some_and(|report| report.terminal) {
            if self.published_key.is_some() {
                RealtimeIblReadiness::FailedLastGood
            } else {
                RealtimeIblReadiness::FailedFallback
            }
        } else {
            match (self.published_key.is_some(), self.ticket.is_some()) {
                (false, false) => RealtimeIblReadiness::Fallback,
                (false, true) => RealtimeIblReadiness::Baking,
                (true, false) => RealtimeIblReadiness::Ready,
                (true, true) => RealtimeIblReadiness::RefreshingLastGood,
            }
        }
    }

    pub(in crate::graphics) fn failure_report(&self) -> Option<RealtimeIblFailureReport> {
        self.failure_report
    }
}

fn retry_backoff_frames(failed_attempt_count: u8) -> u64 {
    let exponent = u32::from(failed_attempt_count.saturating_sub(1));
    REALTIME_IBL_INITIAL_RETRY_BACKOFF_FRAMES
        .checked_shl(exponent)
        .unwrap_or(REALTIME_IBL_MAX_RETRY_BACKOFF_FRAMES)
        .min(REALTIME_IBL_MAX_RETRY_BACKOFF_FRAMES)
}

pub(in crate::graphics) fn frame_sequence_age(
    frame_number: u64,
    earlier_frame: u64,
) -> Option<u64> {
    // Half-range ordering keeps short-lived retry and freshness distances unambiguous across wrap.
    let age = frame_number.wrapping_sub(earlier_frame);
    (age <= i64::MAX as u64).then_some(age)
}

fn frame_sequence_has_reached(frame_number: u64, target_frame: u64) -> bool {
    frame_sequence_age(frame_number, target_frame).is_some()
}

#[cfg(test)]
mod tests;
