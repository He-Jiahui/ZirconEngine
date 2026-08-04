use std::fmt;
use std::mem::{align_of, size_of, MaybeUninit};
use std::panic::{catch_unwind, resume_unwind, AssertUnwindSafe};

use crate::scene::{EntityId, World};

use super::{Command, ErasedCommand};

const INLINE_COMMAND_BYTES: usize = 192;
const INLINE_COMMAND_ALIGNMENT: usize = 64;
const INLINE_COMMAND_BYTE_BUDGET: usize = 4 * 1024 * 1024;

#[repr(C, align(64))]
struct InlineCommandPayload {
    bytes: [MaybeUninit<u8>; INLINE_COMMAND_BYTES],
}

impl InlineCommandPayload {
    fn new() -> Self {
        Self {
            bytes: [MaybeUninit::uninit(); INLINE_COMMAND_BYTES],
        }
    }

    fn as_mut_ptr(&mut self) -> *mut u8 {
        self.bytes.as_mut_ptr().cast()
    }
}

struct InlineCommand {
    payload: InlineCommandPayload,
    apply: unsafe fn(*mut u8, &mut World),
    drop_payload: unsafe fn(*mut u8),
    armed: bool,
}

impl InlineCommand {
    fn new<C>(command: C) -> Self
    where
        C: Command,
    {
        debug_assert!(can_inline::<C>(0));

        let mut inline = Self {
            payload: InlineCommandPayload::new(),
            apply: apply_inline::<C>,
            drop_payload: drop_inline::<C>,
            armed: false,
        };

        // The payload starts at offset zero of a 64-byte-aligned slot. `can_inline`
        // admits only payloads that fit this slot and require no stricter alignment.
        unsafe {
            inline.payload.as_mut_ptr().cast::<C>().write(command);
        }
        inline.armed = true;
        inline
    }

    fn apply(&mut self, world: &mut World) {
        let apply = self.apply;
        self.armed = false;

        // `armed` means the slot holds one initialized `C`. Clear it before calling
        // user code so an unwind cannot make `Drop` run on the moved payload again.
        unsafe {
            apply(self.payload.as_mut_ptr(), world);
        }
    }
}

impl Drop for InlineCommand {
    fn drop(&mut self) {
        if self.armed {
            // An armed slot contains one initialized payload that was never moved into
            // `Command::apply`; dropping it releases abandoned and panic-discarded work.
            unsafe {
                (self.drop_payload)(self.payload.as_mut_ptr());
            }
        }
    }
}

unsafe fn apply_inline<C>(payload: *mut u8, world: &mut World)
where
    C: Command,
{
    // Callers provide the aligned, initialized storage created by `InlineCommand::new`.
    let command = unsafe { payload.cast::<C>().read() };
    command.apply(world);
}

unsafe fn drop_inline<C>(payload: *mut u8)
where
    C: Command,
{
    // Callers invoke this only for the aligned, initialized storage still owned by a slot.
    unsafe {
        payload.cast::<C>().drop_in_place();
    }
}

enum QueuedCommand {
    Inline(InlineCommand, usize),
    Fallback(Box<dyn ErasedCommand>, usize),
}

impl QueuedCommand {
    fn storage(&self) -> QueuedCommandStorage {
        match self {
            Self::Inline(_, bytes) => QueuedCommandStorage::Inline(*bytes),
            Self::Fallback(_, bytes) => QueuedCommandStorage::Fallback(*bytes),
        }
    }

    fn apply(self, world: &mut World) {
        match self {
            Self::Inline(mut command, _) => command.apply(world),
            Self::Fallback(command, _) => command.apply_boxed(world),
        }
    }
}

#[derive(Clone, Copy)]
enum QueuedCommandStorage {
    Inline(usize),
    Fallback(usize),
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct CommandQueueMetrics {
    queued_inline_commands: usize,
    queued_fallback_commands: usize,
    queued_inline_bytes: usize,
    queued_inline_slot_bytes: usize,
    queued_fallback_bytes: usize,
    queue_storage_growths: usize,
    fallback_payload_allocations: usize,
    fallback_payload_releases: usize,
    inline_payload_releases: usize,
    inline_dispatch_calls: usize,
    fallback_dispatch_calls: usize,
    discarded_inline_commands: usize,
    discarded_fallback_commands: usize,
}

impl CommandQueueMetrics {
    pub fn queued_inline_commands(&self) -> usize {
        self.queued_inline_commands
    }

    pub fn queued_fallback_commands(&self) -> usize {
        self.queued_fallback_commands
    }

    pub fn queued_inline_bytes(&self) -> usize {
        self.queued_inline_bytes
    }

    /// Actual occupied inline-slot storage, including the fixed slot cost for
    /// small payloads. This is the quantity bounded by the queue byte budget.
    pub fn queued_inline_slot_bytes(&self) -> usize {
        self.queued_inline_slot_bytes
    }

    pub fn queued_fallback_bytes(&self) -> usize {
        self.queued_fallback_bytes
    }

    /// Counts backing-vector growth, not one allocation per inline payload.
    pub fn queue_storage_growths(&self) -> usize {
        self.queue_storage_growths
    }

    /// Counts explicit boxed fallback allocations only.
    pub fn fallback_payload_allocations(&self) -> usize {
        self.fallback_payload_allocations
    }

    /// Counts consumed or panic-discarded boxed fallback payloads.
    pub fn fallback_payload_releases(&self) -> usize {
        self.fallback_payload_releases
    }

    /// Counts consumed or panic-discarded inline payloads.
    pub fn inline_payload_releases(&self) -> usize {
        self.inline_payload_releases
    }

    pub fn inline_dispatch_calls(&self) -> usize {
        self.inline_dispatch_calls
    }

    pub fn fallback_dispatch_calls(&self) -> usize {
        self.fallback_dispatch_calls
    }

    pub fn discarded_inline_commands(&self) -> usize {
        self.discarded_inline_commands
    }

    pub fn discarded_fallback_commands(&self) -> usize {
        self.discarded_fallback_commands
    }

    fn queued(&mut self, storage: QueuedCommandStorage) {
        match storage {
            QueuedCommandStorage::Inline(bytes) => {
                self.queued_inline_commands += 1;
                self.queued_inline_bytes += bytes;
                self.queued_inline_slot_bytes += INLINE_COMMAND_BYTES;
            }
            QueuedCommandStorage::Fallback(bytes) => {
                self.queued_fallback_commands += 1;
                self.queued_fallback_bytes += bytes;
                self.fallback_payload_allocations += 1;
            }
        }
    }

    fn queue_storage_grew(&mut self) {
        self.queue_storage_growths += 1;
    }

    fn dispatched(&mut self, storage: QueuedCommandStorage) {
        match storage {
            QueuedCommandStorage::Inline(bytes) => {
                self.queued_inline_commands -= 1;
                self.queued_inline_bytes -= bytes;
                self.queued_inline_slot_bytes -= INLINE_COMMAND_BYTES;
                self.inline_payload_releases += 1;
                self.inline_dispatch_calls += 1;
            }
            QueuedCommandStorage::Fallback(bytes) => {
                self.queued_fallback_commands -= 1;
                self.queued_fallback_bytes -= bytes;
                self.fallback_payload_releases += 1;
                self.fallback_dispatch_calls += 1;
            }
        }
    }

    fn discard_queued(&mut self) {
        self.discarded_inline_commands += self.queued_inline_commands;
        self.discarded_fallback_commands += self.queued_fallback_commands;
        self.inline_payload_releases += self.queued_inline_commands;
        self.fallback_payload_releases += self.queued_fallback_commands;
        self.queued_inline_commands = 0;
        self.queued_fallback_commands = 0;
        self.queued_inline_bytes = 0;
        self.queued_inline_slot_bytes = 0;
        self.queued_fallback_bytes = 0;
    }

    fn merge_from(&mut self, other: Self) {
        self.queued_inline_commands += other.queued_inline_commands;
        self.queued_fallback_commands += other.queued_fallback_commands;
        self.queued_inline_bytes += other.queued_inline_bytes;
        self.queued_inline_slot_bytes += other.queued_inline_slot_bytes;
        self.queued_fallback_bytes += other.queued_fallback_bytes;
        self.queue_storage_growths += other.queue_storage_growths;
        self.fallback_payload_allocations += other.fallback_payload_allocations;
        self.fallback_payload_releases += other.fallback_payload_releases;
        self.inline_payload_releases += other.inline_payload_releases;
        self.inline_dispatch_calls += other.inline_dispatch_calls;
        self.fallback_dispatch_calls += other.fallback_dispatch_calls;
        self.discarded_inline_commands += other.discarded_inline_commands;
        self.discarded_fallback_commands += other.discarded_fallback_commands;
    }
}

#[derive(Default)]
pub struct CommandQueue {
    commands: Vec<QueuedCommand>,
    metrics: CommandQueueMetrics,
}

impl CommandQueue {
    pub fn with_capacity(command_capacity: usize) -> Self {
        Self {
            commands: Vec::with_capacity(command_capacity),
            metrics: CommandQueueMetrics {
                queue_storage_growths: usize::from(command_capacity > 0),
                ..CommandQueueMetrics::default()
            },
        }
    }

    pub fn push<C>(&mut self, command: C)
    where
        C: Command,
    {
        let payload_bytes = size_of::<C>();
        let command = if can_inline::<C>(self.metrics.queued_inline_slot_bytes) {
            QueuedCommand::Inline(InlineCommand::new(command), payload_bytes)
        } else {
            QueuedCommand::Fallback(Box::new(command), payload_bytes)
        };
        self.metrics.queued(command.storage());
        if self.commands.len() == self.commands.capacity() {
            self.metrics.queue_storage_grew();
        }
        self.commands.push(command);
    }

    pub fn apply(&mut self, world: &mut World) -> DeferredCommandReport {
        let applied_count = self.commands.len();
        world.clear_deferred_command_errors();

        let (commands, metrics) = (&mut self.commands, &mut self.metrics);
        let result = catch_unwind(AssertUnwindSafe(|| {
            for command in commands.drain(..) {
                let storage = command.storage();
                metrics.dispatched(storage);
                command.apply(world);
            }
        }));

        if let Err(payload) = result {
            metrics.discard_queued();
            resume_unwind(payload);
        }

        DeferredCommandReport::new(applied_count, world.take_deferred_command_errors())
    }

    pub fn len(&self) -> usize {
        self.commands.len()
    }

    pub fn is_empty(&self) -> bool {
        self.commands.is_empty()
    }

    pub fn metrics(&self) -> CommandQueueMetrics {
        self.metrics
    }

    pub(crate) fn append(&mut self, other: &mut Self) {
        self.commands.append(&mut other.commands);
        self.metrics.merge_from(other.metrics);
        other.metrics = CommandQueueMetrics::default();
    }
}

impl fmt::Debug for CommandQueue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CommandQueue")
            .field("len", &self.commands.len())
            .field("metrics", &self.metrics)
            .finish()
    }
}

impl Clone for CommandQueue {
    fn clone(&self) -> Self {
        Self::default()
    }
}

impl PartialEq for CommandQueue {
    fn eq(&self, _other: &Self) -> bool {
        true
    }
}

fn can_inline<C>(queued_inline_slot_bytes: usize) -> bool
where
    C: Command,
{
    size_of::<C>() <= INLINE_COMMAND_BYTES
        && align_of::<C>() <= INLINE_COMMAND_ALIGNMENT
        && queued_inline_slot_bytes
            <= INLINE_COMMAND_BYTE_BUDGET.saturating_sub(INLINE_COMMAND_BYTES)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DeferredCommandOperation {
    Spawn,
    Insert,
    InsertBundle,
    Remove,
    Despawn,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DeferredCommandError {
    operation: DeferredCommandOperation,
    entity: EntityId,
    message: String,
}

impl DeferredCommandError {
    pub fn new(
        operation: DeferredCommandOperation,
        entity: EntityId,
        message: impl Into<String>,
    ) -> Self {
        Self {
            operation,
            entity,
            message: message.into(),
        }
    }

    pub fn operation(&self) -> DeferredCommandOperation {
        self.operation
    }

    pub fn entity(&self) -> EntityId {
        self.entity
    }

    pub fn message(&self) -> &str {
        &self.message
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct DeferredCommandReport {
    applied_count: usize,
    errors: Vec<DeferredCommandError>,
}

impl DeferredCommandReport {
    pub fn new(applied_count: usize, errors: Vec<DeferredCommandError>) -> Self {
        Self {
            applied_count,
            errors,
        }
    }

    pub fn applied_count(&self) -> usize {
        self.applied_count
    }

    pub fn error_count(&self) -> usize {
        self.errors.len()
    }

    pub fn is_success(&self) -> bool {
        self.errors.is_empty()
    }

    pub fn errors(&self) -> &[DeferredCommandError] {
        &self.errors
    }
}
