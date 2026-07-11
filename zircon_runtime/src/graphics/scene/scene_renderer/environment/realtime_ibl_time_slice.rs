use crate::core::framework::render::IblBakeKey;

const CUBE_FACE_COUNT: u8 = 6;

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
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::graphics) struct CubeMipRange {
    pub first: u8,
    pub count: u8,
}

impl CubeMipRange {
    pub const fn new(first: u8, count: u8) -> Self {
        Self { first, count }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::graphics) enum RealtimeIblOperation {
    CaptureSky(CubeFaceRange),
    CaptureCloud(CubeFaceRange),
    GenerateSourceMips,
    Prefilter {
        mips: CubeMipRange,
        faces: CubeFaceRange,
    },
    ProjectDiffuseSh9,
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
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(in crate::graphics) struct RealtimeIblFrameBatch {
    token: RealtimeIblBatchToken,
    full_update: bool,
    ready_slot: IblRealtimeBufferSlot,
    work_slot: IblRealtimeBufferSlot,
    operations: Vec<RealtimeIblOperation>,
}

impl RealtimeIblFrameBatch {
    pub(in crate::graphics) fn token(&self) -> RealtimeIblBatchToken {
        self.token
    }

    pub(in crate::graphics) fn logical_state(&self) -> u8 {
        self.token.state
    }

    pub(in crate::graphics) fn is_full_update(&self) -> bool {
        self.full_update
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
    Retry,
    Stale,
}

pub(in crate::graphics) struct RealtimeIblTimeSliceScheduler {
    config: RealtimeIblTimeSliceConfig,
    generation: u64,
    pending_key: Option<IblBakeKey>,
    published_key: Option<IblBakeKey>,
    ready_slot: IblRealtimeBufferSlot,
    state: u8,
    substep: u8,
    current_frame: Option<u64>,
    current_batch: Option<RealtimeIblFrameBatch>,
}

impl RealtimeIblTimeSliceScheduler {
    pub(in crate::graphics) fn new(config: RealtimeIblTimeSliceConfig) -> Self {
        Self {
            config,
            generation: 0,
            pending_key: None,
            published_key: None,
            ready_slot: IblRealtimeBufferSlot::A,
            state: 0,
            substep: 0,
            current_frame: None,
            current_batch: None,
        }
    }

    pub(in crate::graphics) fn request_rebake(&mut self, key: IblBakeKey) -> bool {
        if self.pending_key == Some(key) {
            return false;
        }
        if self.published_key == Some(key) && self.pending_key.is_none() {
            return false;
        }
        self.generation = self.generation.wrapping_add(1);
        self.pending_key = (self.published_key != Some(key)).then_some(key);
        self.state = 0;
        self.substep = 0;
        self.current_frame = None;
        self.current_batch = None;
        true
    }

    pub(in crate::graphics) fn begin_frame(
        &mut self,
        frame_number: u64,
    ) -> Option<RealtimeIblFrameBatch> {
        self.pending_key?;
        if self.current_frame == Some(frame_number) {
            return self.current_batch.clone();
        }
        let full_update = self.published_key.is_none();
        let operations = if full_update {
            self.full_update_operations()
        } else {
            self.sliced_operations()
        };
        let batch = RealtimeIblFrameBatch {
            token: RealtimeIblBatchToken {
                generation: self.generation,
                state: self.state,
                substep: self.substep,
            },
            full_update,
            ready_slot: self.ready_slot,
            work_slot: self.ready_slot.other(),
            operations,
        };
        self.current_frame = Some(frame_number);
        self.current_batch = Some(batch.clone());
        Some(batch)
    }

    pub(in crate::graphics) fn complete_frame(
        &mut self,
        token: RealtimeIblBatchToken,
        gpu_succeeded: bool,
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
        self.current_batch = None;
        self.current_frame = None;
        if !gpu_succeeded {
            return RealtimeIblCompletion::Retry;
        }
        if self.published_key.is_none() || self.advance_sliced_state() {
            self.ready_slot = self.ready_slot.other();
            self.published_key = self.pending_key.take();
            self.state = 0;
            self.substep = 0;
            RealtimeIblCompletion::Published
        } else {
            RealtimeIblCompletion::Advanced
        }
    }

    pub(in crate::graphics) fn published_key(&self) -> Option<IblBakeKey> {
        self.published_key
    }

    pub(in crate::graphics) fn pending_key(&self) -> Option<IblBakeKey> {
        self.pending_key
    }

    pub(in crate::graphics) fn ready_slot(&self) -> IblRealtimeBufferSlot {
        self.ready_slot
    }

    pub(in crate::graphics) fn is_rebake_pending(&self) -> bool {
        self.pending_key.is_some()
    }

    fn full_update_operations(&self) -> Vec<RealtimeIblOperation> {
        vec![
            RealtimeIblOperation::CaptureSky(CubeFaceRange::ALL),
            RealtimeIblOperation::CaptureCloud(CubeFaceRange::ALL),
            RealtimeIblOperation::GenerateSourceMips,
            RealtimeIblOperation::Prefilter {
                mips: CubeMipRange::new(0, self.config.pmrem_mip_count),
                faces: CubeFaceRange::ALL,
            },
            RealtimeIblOperation::ProjectDiffuseSh9,
        ]
    }

    fn sliced_operations(&self) -> Vec<RealtimeIblOperation> {
        let all_faces = CubeFaceRange::ALL;
        match self.state {
            0 => vec![RealtimeIblOperation::CaptureSky(self.current_face_range())],
            1 => vec![RealtimeIblOperation::CaptureCloud(
                self.current_face_range(),
            )],
            2 => vec![RealtimeIblOperation::GenerateSourceMips],
            3..=5 => vec![RealtimeIblOperation::Prefilter {
                mips: CubeMipRange::new(0, 1),
                faces: CubeFaceRange::new((self.state - 3) * 2, 2),
            }],
            6..=8 => self.prefilter_if_available(self.state - 5, 1, all_faces),
            9 => self.prefilter_if_available(4, 2, all_faces),
            10 => self.prefilter_if_available(6, u8::MAX, all_faces),
            _ => vec![RealtimeIblOperation::ProjectDiffuseSh9],
        }
    }

    fn prefilter_if_available(
        &self,
        first_mip: u8,
        mip_count: u8,
        faces: CubeFaceRange,
    ) -> Vec<RealtimeIblOperation> {
        let mips = self.clamped_mip_range(first_mip, mip_count);
        if mips.count == 0 {
            Vec::new()
        } else {
            vec![RealtimeIblOperation::Prefilter { mips, faces }]
        }
    }

    fn current_face_range(&self) -> CubeFaceRange {
        CubeFaceRange::new(
            self.substep,
            self.config
                .capture_faces_per_frame
                .min(CUBE_FACE_COUNT - self.substep),
        )
    }

    fn clamped_mip_range(&self, first: u8, count: u8) -> CubeMipRange {
        CubeMipRange::new(
            first.min(self.config.pmrem_mip_count),
            count.min(self.config.pmrem_mip_count.saturating_sub(first)),
        )
    }

    fn advance_sliced_state(&mut self) -> bool {
        if self.state <= 1 {
            let next = self
                .substep
                .saturating_add(self.config.capture_faces_per_frame);
            if next < CUBE_FACE_COUNT {
                self.substep = next;
                return false;
            }
            self.substep = 0;
        }
        if self.state == 11 {
            return true;
        }
        self.state += 1;
        false
    }
}

#[cfg(test)]
mod tests;
