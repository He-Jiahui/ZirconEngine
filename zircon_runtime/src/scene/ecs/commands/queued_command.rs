use std::collections::BTreeSet;

use crate::scene::World;

use super::inline_command_arena::{InlineCommandArena, WorkerInlineCommandArena};
use super::{
    DeferredEntityRef, DeferredSpawnToken, DeferredStructuralKind, DeferredStructuralMetadata,
    ErasedCommand, ErasedQueuedStructuralCommand,
};

/// arena 中尚未消费的命令句柄；位置与同类型 vtable 一起移动，apply、stage 或 discard 只能发生一次。
pub(super) struct InlineCommand {
    arena: InlineCommandArenaLocation,
    block_index: usize,
    offset: usize,
    apply: unsafe fn(*mut u8, &mut World),
    structural: Option<InlineStructuralVtable>,
    drop_payload: unsafe fn(*mut u8),
}

#[derive(Clone, Copy)]
pub(super) struct InlineStructuralVtable {
    metadata: unsafe fn(*const u8) -> DeferredStructuralMetadata,
    stage: unsafe fn(*mut u8, &mut crate::scene::world::DeferredStructuralBatch, &mut World),
}

#[derive(Clone, Copy)]
enum InlineCommandArenaLocation {
    Queue,
    Worker(usize),
}

impl InlineStructuralVtable {
    pub(super) fn new(
        metadata: unsafe fn(*const u8) -> DeferredStructuralMetadata,
        stage: unsafe fn(*mut u8, &mut crate::scene::world::DeferredStructuralBatch, &mut World),
    ) -> Self {
        Self { metadata, stage }
    }
}

impl InlineCommand {
    pub(super) fn new(
        block_index: usize,
        offset: usize,
        apply: unsafe fn(*mut u8, &mut World),
        structural: Option<InlineStructuralVtable>,
        drop_payload: unsafe fn(*mut u8),
    ) -> Self {
        Self {
            arena: InlineCommandArenaLocation::Queue,
            block_index,
            offset,
            apply,
            structural,
            drop_payload,
        }
    }

    fn remap_to_worker(&mut self, worker_arena_index: usize, block_offset: usize) {
        debug_assert!(matches!(self.arena, InlineCommandArenaLocation::Queue));
        self.arena = InlineCommandArenaLocation::Worker(worker_arena_index);
        self.block_index += block_offset;
    }

    fn remap_worker_arena(&mut self, from: usize, to: usize) {
        if matches!(self.arena, InlineCommandArenaLocation::Worker(index) if index == from) {
            self.arena = InlineCommandArenaLocation::Worker(to);
        }
    }

    fn remap_appended_arena(
        &mut self,
        queue_block_offset: usize,
        worker_arena_remaps: &[(usize, usize)],
    ) {
        match self.arena {
            InlineCommandArenaLocation::Queue => self.block_index += queue_block_offset,
            InlineCommandArenaLocation::Worker(source_index) => {
                let (destination_index, block_offset) = worker_arena_remaps[source_index];
                self.arena = InlineCommandArenaLocation::Worker(destination_index);
                self.block_index += block_offset;
            }
        }
    }

    fn apply(
        self,
        arena: &mut InlineCommandArena,
        worker_arenas: &mut [WorkerInlineCommandArena],
        world: &mut World,
    ) {
        let arena = match self.arena {
            InlineCommandArenaLocation::Queue => arena,
            InlineCommandArenaLocation::Worker(index) => &mut worker_arenas[index].arena,
        };
        // SAFETY: InlineCommandArena 在对齐槽位写入已初始化的命令并创建同类型 vtable；CommandQueue 先标记 Consumed，arena 在本次提交结束前有效。
        unsafe {
            (self.apply)(arena.payload_ptr(self.block_index, self.offset), world);
        }
    }

    fn structural_metadata(
        &self,
        arena: &InlineCommandArena,
        worker_arenas: &[WorkerInlineCommandArena],
    ) -> Option<DeferredStructuralMetadata> {
        let structural = self.structural?;
        let arena = match self.arena {
            InlineCommandArenaLocation::Queue => arena,
            InlineCommandArenaLocation::Worker(index) => &worker_arenas[index].arena,
        };
        // SAFETY: 元数据回调只借读对齐且已初始化的结构命令；arena 块和该命令的类型绑定在提交期间保持不变。
        Some(unsafe {
            (structural.metadata)(arena.payload_ptr_const(self.block_index, self.offset))
        })
    }

    fn stage_structural(
        self,
        arena: &mut InlineCommandArena,
        worker_arenas: &mut [WorkerInlineCommandArena],
        batch: &mut crate::scene::world::DeferredStructuralBatch,
        world: &mut World,
    ) {
        let structural = self
            .structural
            .expect("only structural commands may enter a structural segment");
        let arena = match self.arena {
            InlineCommandArenaLocation::Queue => arena,
            InlineCommandArenaLocation::Worker(index) => &mut worker_arenas[index].arena,
        };
        // SAFETY: 结构命令从对齐且已初始化的同类型 arena 槽位移出交给批事务，队列此前已把该条目标记 Consumed。
        unsafe {
            (structural.stage)(
                arena.payload_ptr(self.block_index, self.offset),
                batch,
                world,
            );
        }
    }

    fn discard(
        self,
        arena: &mut InlineCommandArena,
        worker_arenas: &mut [WorkerInlineCommandArena],
    ) {
        let arena = match self.arena {
            InlineCommandArenaLocation::Queue => arena,
            InlineCommandArenaLocation::Worker(index) => &mut worker_arenas[index].arena,
        };
        // SAFETY: 仅未消费的 inline 命令会进入 discard；其对齐槽位仍持有已初始化的原类型负载。
        unsafe {
            (self.drop_payload)(arena.payload_ptr(self.block_index, self.offset));
        }
    }
}

pub(super) enum QueuedCommand {
    Inline {
        command: InlineCommand,
        payload_bytes: usize,
        storage_bytes: usize,
    },
    Fallback(Box<dyn ErasedCommand>, usize),
    StructuralFallback(Box<dyn ErasedQueuedStructuralCommand>, usize),
    Consumed,
}

impl QueuedCommand {
    pub(super) fn storage(&self) -> Option<QueuedCommandStorage> {
        match self {
            Self::Inline {
                payload_bytes,
                storage_bytes,
                ..
            } => Some(QueuedCommandStorage::Inline {
                payload_bytes: *payload_bytes,
                storage_bytes: *storage_bytes,
            }),
            Self::Fallback(_, bytes) | Self::StructuralFallback(_, bytes) => {
                Some(QueuedCommandStorage::Fallback(*bytes))
            }
            Self::Consumed => None,
        }
    }

    pub(super) fn apply(
        self,
        arena: &mut InlineCommandArena,
        worker_arenas: &mut [WorkerInlineCommandArena],
        world: &mut World,
    ) {
        match self {
            Self::Inline { command, .. } => command.apply(arena, worker_arenas, world),
            Self::Fallback(command, _) => command.apply_boxed(world),
            Self::StructuralFallback(_, _) => {
                unreachable!("structural commands must stage before generic apply")
            }
            Self::Consumed => unreachable!("consumed command cannot be applied twice"),
        }
    }

    pub(super) fn collect_spawn_tokens(
        &self,
        arena: &InlineCommandArena,
        worker_arenas: &[WorkerInlineCommandArena],
        tokens: &mut BTreeSet<DeferredSpawnToken>,
    ) {
        let Some(metadata) = self.structural_metadata(arena, worker_arenas) else {
            return;
        };
        if !matches!(
            metadata.kind(),
            DeferredStructuralKind::SpawnEmpty | DeferredStructuralKind::SpawnBundle
        ) {
            return;
        }
        if let DeferredEntityRef::Spawn(token) = metadata.target() {
            tokens.insert(token.clone());
        }
    }

    pub(super) fn structural_metadata(
        &self,
        arena: &InlineCommandArena,
        worker_arenas: &[WorkerInlineCommandArena],
    ) -> Option<DeferredStructuralMetadata> {
        match self {
            Self::Inline { command, .. } => command.structural_metadata(arena, worker_arenas),
            Self::Fallback(_, _) => None,
            Self::StructuralFallback(command, _) => Some(command.structural_metadata()),
            Self::Consumed => unreachable!("consumed command cannot expose structural metadata"),
        }
    }

    pub(super) fn stage_structural(
        self,
        arena: &mut InlineCommandArena,
        worker_arenas: &mut [WorkerInlineCommandArena],
        batch: &mut crate::scene::world::DeferredStructuralBatch,
        world: &mut World,
    ) {
        match self {
            Self::Inline { command, .. } => {
                command.stage_structural(arena, worker_arenas, batch, world)
            }
            Self::StructuralFallback(command, _) => command.stage_boxed(batch, world),
            Self::Fallback(_, _) => unreachable!("opaque command cannot stage structurally"),
            Self::Consumed => unreachable!("consumed command cannot stage twice"),
        }
    }

    pub(super) fn discard(
        self,
        arena: &mut InlineCommandArena,
        worker_arenas: &mut [WorkerInlineCommandArena],
    ) {
        match self {
            Self::Inline { command, .. } => command.discard(arena, worker_arenas),
            Self::Fallback(_, _) | Self::StructuralFallback(_, _) | Self::Consumed => {}
        }
    }

    pub(super) fn remap_inline_to_worker(
        &mut self,
        worker_arena_index: usize,
        block_offset: usize,
    ) {
        if let Self::Inline { command, .. } = self {
            command.remap_to_worker(worker_arena_index, block_offset);
        }
    }

    pub(super) fn remap_worker_arena(&mut self, from: usize, to: usize) {
        if let Self::Inline { command, .. } = self {
            command.remap_worker_arena(from, to);
        }
    }

    pub(super) fn remap_appended_arena(
        &mut self,
        queue_block_offset: usize,
        worker_arena_remaps: &[(usize, usize)],
    ) {
        if let Self::Inline { command, .. } = self {
            command.remap_appended_arena(queue_block_offset, worker_arena_remaps);
        }
    }

    pub(super) fn add_worker_inline_storage_prefix(
        &mut self,
        worker_arena_index: usize,
        storage_prefix: usize,
    ) -> bool {
        let Self::Inline {
            command,
            storage_bytes,
            ..
        } = self
        else {
            return false;
        };
        if matches!(
            command.arena,
            InlineCommandArenaLocation::Worker(index) if index == worker_arena_index
        ) {
            *storage_bytes += storage_prefix;
            return true;
        }
        false
    }

    pub(super) fn references_worker_arena(&self, worker_arena_index: usize) -> bool {
        matches!(
            self,
            Self::Inline { command, .. }
                if matches!(
                    command.arena,
                    InlineCommandArenaLocation::Worker(index) if index == worker_arena_index
                )
        )
    }

    pub(super) fn add_queue_inline_storage_prefix(&mut self, storage_prefix: usize) -> bool {
        let Self::Inline {
            command,
            storage_bytes,
            ..
        } = self
        else {
            return false;
        };
        if !matches!(command.arena, InlineCommandArenaLocation::Queue) {
            return false;
        }
        *storage_bytes += storage_prefix;
        true
    }
}

#[derive(Clone, Copy)]
pub(super) enum QueuedCommandStorage {
    Inline {
        payload_bytes: usize,
        storage_bytes: usize,
    },
    Fallback(usize),
}
