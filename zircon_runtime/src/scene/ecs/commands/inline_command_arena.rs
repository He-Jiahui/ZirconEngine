use std::mem::{MaybeUninit, align_of, size_of};

use crate::scene::World;

use super::queued_command::{InlineCommand, InlineStructuralVtable};
use super::{Command, DeferredStructuralMetadata, DeferredSystemKey, QueuedStructuralCommand};

const MAX_INLINE_COMMAND_BYTES: usize = 192;
const INLINE_COMMAND_ALIGNMENT: usize = 64;
const INLINE_COMMAND_BLOCK_BYTES: usize = 64 * 1024;
const INLINE_COMMAND_BYTE_BUDGET: usize = 4 * 1024 * 1024;
const INLINE_COMMAND_BLOCK_LIMIT: usize = INLINE_COMMAND_BYTE_BUDGET / INLINE_COMMAND_BLOCK_BYTES;

#[repr(C, align(64))]
struct InlineCommandBlock {
    bytes: [MaybeUninit<u8>; INLINE_COMMAND_BLOCK_BYTES],
}

impl InlineCommandBlock {
    fn new() -> Self {
        Self {
            bytes: [MaybeUninit::uninit(); INLINE_COMMAND_BLOCK_BYTES],
        }
    }

    fn as_mut_ptr(&mut self) -> *mut u8 {
        self.bytes.as_mut_ptr().cast()
    }

    fn as_ptr(&self) -> *const u8 {
        self.bytes.as_ptr().cast()
    }
}

#[derive(Clone, Copy)]
struct InlineAllocationPlan {
    starts_new_block: bool,
    offset: usize,
    next_offset: usize,
    storage_bytes: usize,
}

/// Reusable aligned payload storage. Queue entries keep block offsets rather
/// than pointers so moving the block vector cannot invalidate queued payloads.
#[derive(Default)]
pub(super) struct InlineCommandArena {
    blocks: Vec<InlineCommandBlock>,
    next_offset: usize,
}

pub(super) struct WorkerInlineCommandArena {
    pub(super) key: DeferredSystemKey,
    pub(super) arena: InlineCommandArena,
}

impl WorkerInlineCommandArena {
    pub(super) fn matches(&self, key: &DeferredSystemKey) -> bool {
        self.key == *key
    }
}

impl InlineCommandArena {
    pub(super) fn with_command_capacity(command_capacity: usize) -> Self {
        Self {
            blocks: Vec::with_capacity(usize::from(command_capacity > 0)),
            next_offset: 0,
        }
    }

    pub(super) fn try_push<C>(&mut self, command: C) -> Result<(InlineCommand, usize, bool), C>
    where
        C: Command,
    {
        self.try_push_with(command, None)
    }

    pub(super) fn try_push_structural<C>(
        &mut self,
        command: C,
    ) -> Result<(InlineCommand, usize, bool), C>
    where
        C: QueuedStructuralCommand,
    {
        self.try_push_with(
            command,
            Some(InlineStructuralVtable::new(
                structural_inline_metadata::<C>,
                stage_inline_structural::<C>,
            )),
        )
    }

    fn try_push_with<C>(
        &mut self,
        command: C,
        structural: Option<InlineStructuralVtable>,
    ) -> Result<(InlineCommand, usize, bool), C>
    where
        C: Command,
    {
        let Some(plan) = self.allocation_plan::<C>() else {
            return Err(command);
        };
        let storage_grew = if plan.starts_new_block {
            let storage_grew = self.blocks.len() == self.blocks.capacity();
            self.blocks.push(InlineCommandBlock::new());
            storage_grew
        } else {
            false
        };
        let block_index = self.blocks.len() - 1;
        unsafe {
            self.payload_ptr(block_index, plan.offset)
                .cast::<C>()
                .write(command);
        }
        self.next_offset = plan.next_offset;
        Ok((
            InlineCommand::new(
                block_index,
                plan.offset,
                apply_inline::<C>,
                structural,
                drop_inline::<C>,
            ),
            plan.storage_bytes,
            storage_grew,
        ))
    }

    fn allocation_plan<C>(&self) -> Option<InlineAllocationPlan>
    where
        C: Command,
    {
        if size_of::<C>() > MAX_INLINE_COMMAND_BYTES || align_of::<C>() > INLINE_COMMAND_ALIGNMENT {
            return None;
        }

        if self.blocks.is_empty() {
            return Some(InlineAllocationPlan {
                starts_new_block: true,
                offset: 0,
                next_offset: size_of::<C>(),
                storage_bytes: size_of::<C>(),
            });
        }

        let offset = align_up(self.next_offset, align_of::<C>())?;
        let next_offset = offset.checked_add(size_of::<C>())?;
        if next_offset <= INLINE_COMMAND_BLOCK_BYTES {
            return Some(InlineAllocationPlan {
                starts_new_block: false,
                offset,
                next_offset,
                storage_bytes: next_offset - self.next_offset,
            });
        }
        if self.blocks.len() >= INLINE_COMMAND_BLOCK_LIMIT {
            return None;
        }

        Some(InlineAllocationPlan {
            starts_new_block: true,
            offset: 0,
            next_offset: size_of::<C>(),
            storage_bytes: INLINE_COMMAND_BLOCK_BYTES - self.next_offset + size_of::<C>(),
        })
    }

    pub(super) fn append(&mut self, other: &mut Self) -> (usize, bool, usize) {
        if other.blocks.is_empty() {
            return (0, false, 0);
        }
        if self.blocks.is_empty() {
            if self.blocks.capacity() >= other.blocks.len() {
                self.blocks.append(&mut other.blocks);
                self.next_offset = other.next_offset;
                other.next_offset = 0;
            } else {
                std::mem::swap(self, other);
            }
            return (0, false, 0);
        }

        let block_offset = self.blocks.len();
        let leading_padding = INLINE_COMMAND_BLOCK_BYTES - self.next_offset;
        let required_blocks = block_offset.saturating_add(other.blocks.len());
        let storage_grew = required_blocks > self.blocks.capacity();
        // Moving whole blocks preserves the source queue's validated layout and
        // never allocates or dispatches commands one by one.
        self.blocks.append(&mut other.blocks);
        self.next_offset = other.next_offset;
        other.next_offset = 0;
        (block_offset, storage_grew, leading_padding)
    }

    pub(super) fn reset(&mut self) {
        self.blocks.clear();
        self.next_offset = 0;
    }

    /// Releases backing storage only after every inline payload has been
    /// consumed or discarded. Normal queue resets deliberately retain it.
    pub(super) fn trim_idle_storage(&mut self) -> usize {
        if !self.blocks.is_empty() {
            return 0;
        }
        let released_bytes = self
            .blocks
            .capacity()
            .saturating_mul(size_of::<InlineCommandBlock>());
        self.blocks = Vec::new();
        self.next_offset = 0;
        released_bytes
    }

    pub(super) unsafe fn payload_ptr(&mut self, block_index: usize, offset: usize) -> *mut u8 {
        let block = self
            .blocks
            .get_mut(block_index)
            .expect("inline command block index must remain valid");
        assert!(
            offset <= INLINE_COMMAND_BLOCK_BYTES,
            "inline command offset must remain within its block"
        );
        unsafe { block.as_mut_ptr().add(offset) }
    }

    pub(super) fn payload_ptr_const(&self, block_index: usize, offset: usize) -> *const u8 {
        let block = self
            .blocks
            .get(block_index)
            .expect("inline command block index must remain valid");
        assert!(
            offset <= INLINE_COMMAND_BLOCK_BYTES,
            "inline command offset must remain within its block"
        );
        unsafe { block.as_ptr().add(offset) }
    }
}

fn align_up(value: usize, alignment: usize) -> Option<usize> {
    debug_assert!(alignment.is_power_of_two());
    value
        .checked_add(alignment - 1)
        .map(|value| value & !(alignment - 1))
}

unsafe fn apply_inline<C>(payload: *mut u8, world: &mut World)
where
    C: Command,
{
    // The arena plan guarantees an aligned, initialized `C` at this address.
    let command = unsafe { payload.cast::<C>().read() };
    command.apply(world);
}

unsafe fn structural_inline_metadata<C>(payload: *const u8) -> DeferredStructuralMetadata
where
    C: QueuedStructuralCommand,
{
    // The payload remains arena-owned until the structural segment consumes it.
    let command = unsafe { &*payload.cast::<C>() };
    command.structural_metadata()
}

unsafe fn stage_inline_structural<C>(
    payload: *mut u8,
    batch: &mut crate::scene::world::DeferredStructuralBatch,
    world: &mut World,
) where
    C: QueuedStructuralCommand,
{
    // Reading transfers the exactly-once payload owner to the batch transaction.
    let command = unsafe { payload.cast::<C>().read() };
    command.stage_into_batch(batch, world);
}

unsafe fn drop_inline<C>(payload: *mut u8)
where
    C: Command,
{
    // Discard calls this only while the arena still owns an initialized `C`.
    unsafe {
        payload.cast::<C>().drop_in_place();
    }
}
