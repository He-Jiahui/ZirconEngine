use std::sync::atomic::{AtomicU64, Ordering};

use crate::core::framework::render::RenderPhase;

use super::cached_mesh_draw_commands::MeshDrawCommandCacheStats;
use super::mesh_draw_command::MeshDrawCommand;

pub(crate) const MESH_PASS_COMMAND_BUCKET_COUNT: usize = 10;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct MeshPassCommandRange {
    start: usize,
    end: usize,
}

impl MeshPassCommandRange {
    pub(crate) const EMPTY: Self = Self { start: 0, end: 0 };

    pub(crate) fn new(start: usize, end: usize) -> Self {
        debug_assert!(start <= end);
        Self { start, end }
    }

    pub(crate) const fn start(self) -> usize {
        self.start
    }

    pub(crate) const fn end(self) -> usize {
        self.end
    }

    pub(crate) const fn len(self) -> usize {
        self.end - self.start
    }

    pub(crate) const fn is_empty(self) -> bool {
        self.start == self.end
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct MeshPassCommandRanges {
    ranges: [MeshPassCommandRange; MESH_PASS_COMMAND_BUCKET_COUNT],
}

impl MeshPassCommandRanges {
    pub(crate) const fn range(self, bucket: MeshPassCommandBucket) -> MeshPassCommandRange {
        self.ranges[bucket.index()]
    }

    pub(crate) const fn total_len(self) -> usize {
        let mut total = 0;
        let mut index = 0;
        while index < MESH_PASS_COMMAND_BUCKET_COUNT {
            total += self.ranges[index].len();
            index += 1;
        }
        total
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub(crate) enum MeshPassCommandBucket {
    DepthPrepass = 0,
    Shadow = 1,
    Opaque = 2,
    AlphaMask = 3,
    AdvancedPbrOpaque = 4,
    Transmission = 5,
    Transparent = 6,
    HalfResolutionTransparent = 7,
    Velocity = 8,
    TaaReactiveMask = 9,
}

impl MeshPassCommandBucket {
    pub(crate) const ALL: [Self; MESH_PASS_COMMAND_BUCKET_COUNT] = [
        Self::DepthPrepass,
        Self::Shadow,
        Self::Opaque,
        Self::AlphaMask,
        Self::AdvancedPbrOpaque,
        Self::Transmission,
        Self::Transparent,
        Self::HalfResolutionTransparent,
        Self::Velocity,
        Self::TaaReactiveMask,
    ];

    pub(crate) const fn index(self) -> usize {
        self as usize
    }

    pub(crate) fn classify(
        command: &MeshDrawCommand,
        half_resolution_mesh_pass_available: bool,
    ) -> Option<Self> {
        match command.phase {
            RenderPhase::Prepass => Some(Self::DepthPrepass),
            RenderPhase::Shadow => Some(Self::Shadow),
            RenderPhase::Opaque3d => Some(Self::Opaque),
            RenderPhase::AlphaMask3d => Some(Self::AlphaMask),
            RenderPhase::Transparent3d if is_late_forward_opaque(command) => {
                Some(Self::AdvancedPbrOpaque)
            }
            RenderPhase::Transparent3d if is_transmission(command) => Some(Self::Transmission),
            RenderPhase::Transparent3d if command.uses_half_resolution_transparency() => {
                if half_resolution_mesh_pass_available {
                    Some(Self::HalfResolutionTransparent)
                } else {
                    Some(Self::Transparent)
                }
            }
            RenderPhase::Transparent3d => Some(Self::Transparent),
            RenderPhase::PostProcess
                if command.pipeline_kind
                    == super::mesh_draw_command::MeshPassPipelineKind::Velocity =>
            {
                Some(Self::Velocity)
            }
            RenderPhase::PostProcess
                if matches!(
                    command.pipeline_kind,
                    super::mesh_draw_command::MeshPassPipelineKind::TaaReactiveMask
                        | super::mesh_draw_command::MeshPassPipelineKind::TaaReactiveMaterialMask
                ) =>
            {
                Some(Self::TaaReactiveMask)
            }
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct MeshPassCommandSealError {
    unsupported_phase: RenderPhase,
}

impl MeshPassCommandSealError {
    pub(crate) const fn unsupported_phase(self) -> RenderPhase {
        self.unsupported_phase
    }
}

/// Append-only owner for one generation of mesh commands.
///
/// The arena intentionally exposes no command slices before `seal`: producers may append and
/// truncate to a checkpoint, while replay and indirect planning can only consume the sealed view.
pub(crate) struct MeshPassCommandBuildArena {
    arena_id: u64,
    commands: Vec<MeshDrawCommand>,
    cache_stats: MeshDrawCommandCacheStats,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct MeshPassCommandArenaCheckpoint {
    arena_id: u64,
    command_len: usize,
    cache_stats: MeshDrawCommandCacheStats,
}

static NEXT_MESH_PASS_COMMAND_ARENA_ID: AtomicU64 = AtomicU64::new(1);

fn next_arena_id() -> u64 {
    NEXT_MESH_PASS_COMMAND_ARENA_ID
        .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |current| {
            current.checked_add(1)
        })
        .expect("mesh command arena ID space is exhausted")
}

impl Default for MeshPassCommandBuildArena {
    fn default() -> Self {
        Self::new()
    }
}

impl MeshPassCommandBuildArena {
    pub(crate) fn with_capacity(capacity: usize) -> Self {
        Self {
            arena_id: next_arena_id(),
            commands: Vec::with_capacity(capacity),
            cache_stats: MeshDrawCommandCacheStats::default(),
        }
    }

    pub(crate) fn new() -> Self {
        Self::with_capacity(0)
    }

    pub(crate) fn len(&self) -> usize {
        self.commands.len()
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.commands.is_empty()
    }

    pub(crate) fn checkpoint(&self) -> MeshPassCommandArenaCheckpoint {
        MeshPassCommandArenaCheckpoint {
            arena_id: self.arena_id,
            command_len: self.commands.len(),
            cache_stats: self.cache_stats,
        }
    }

    pub(crate) fn truncate_to_checkpoint(&mut self, checkpoint: MeshPassCommandArenaCheckpoint) {
        assert!(
            checkpoint.arena_id == self.arena_id,
            "mesh command arena checkpoint belongs to another arena"
        );
        assert!(
            checkpoint.command_len <= self.commands.len(),
            "mesh command arena checkpoint must belong to the current arena"
        );
        self.commands.truncate(checkpoint.command_len);
        self.cache_stats = checkpoint.cache_stats;
    }

    pub(crate) fn push(&mut self, command: MeshDrawCommand) {
        self.commands.push(command);
    }

    pub(crate) fn extend<I>(&mut self, commands: I)
    where
        I: IntoIterator<Item = MeshDrawCommand>,
    {
        self.commands.extend(commands);
    }

    pub(crate) fn cache_stats_mut(&mut self) -> &mut MeshDrawCommandCacheStats {
        &mut self.cache_stats
    }

    pub(crate) fn accumulate_cache_stats(&mut self, stats: MeshDrawCommandCacheStats) {
        self.cache_stats.accumulate(stats);
    }

    pub(crate) fn seal(
        mut self,
        half_resolution_mesh_pass_available: bool,
    ) -> Result<MeshPassCommandArena, MeshPassCommandSealError> {
        if let Some(command) = self.commands.iter().find(|command| {
            MeshPassCommandBucket::classify(command, half_resolution_mesh_pass_available).is_none()
        }) {
            return Err(MeshPassCommandSealError {
                unsupported_phase: command.phase,
            });
        }

        // `sort_by` is intentionally stable: packed sort keys are not yet proven collision-free.
        self.commands.sort_by(|left, right| {
            let left_bucket =
                MeshPassCommandBucket::classify(left, half_resolution_mesh_pass_available)
                    .expect("unsupported mesh command was checked before sorting");
            let right_bucket =
                MeshPassCommandBucket::classify(right, half_resolution_mesh_pass_available)
                    .expect("unsupported mesh command was checked before sorting");
            left_bucket
                .index()
                .cmp(&right_bucket.index())
                .then_with(|| left.sort_key.cmp(&right.sort_key))
        });

        let mut ranges = [MeshPassCommandRange::EMPTY; MESH_PASS_COMMAND_BUCKET_COUNT];
        let mut cursor = 0;
        for bucket in MeshPassCommandBucket::ALL {
            let start = cursor;
            while cursor < self.commands.len()
                && MeshPassCommandBucket::classify(
                    &self.commands[cursor],
                    half_resolution_mesh_pass_available,
                ) == Some(bucket)
            {
                cursor += 1;
            }
            ranges[bucket.index()] = MeshPassCommandRange::new(start, cursor);
        }
        debug_assert_eq!(cursor, self.commands.len());

        Ok(MeshPassCommandArena {
            commands: self.commands,
            ranges: MeshPassCommandRanges { ranges },
            cache_stats: self.cache_stats,
        })
    }
}

/// Sealed, read-only command storage shared by replay, indirect planning, and diagnostics.
pub(crate) struct MeshPassCommandArena {
    commands: Vec<MeshDrawCommand>,
    ranges: MeshPassCommandRanges,
    cache_stats: MeshDrawCommandCacheStats,
}

impl MeshPassCommandArena {
    pub(crate) fn commands(&self) -> &[MeshDrawCommand] {
        &self.commands
    }

    pub(crate) fn bucket(&self, bucket: MeshPassCommandBucket) -> &[MeshDrawCommand] {
        let range = self.ranges.range(bucket);
        &self.commands[range.start()..range.end()]
    }

    pub(crate) const fn ranges(&self) -> MeshPassCommandRanges {
        self.ranges
    }

    pub(crate) const fn cache_stats(&self) -> MeshDrawCommandCacheStats {
        self.cache_stats
    }
}

fn is_late_forward_opaque(command: &MeshDrawCommand) -> bool {
    command.pipeline_key().requires_forward_path() && !is_transmission(command)
}

fn is_transmission(command: &MeshDrawCommand) -> bool {
    command.pipeline_key().pbr_transmission
}

#[cfg(test)]
#[path = "tests/command_arena.rs"]
mod tests;
