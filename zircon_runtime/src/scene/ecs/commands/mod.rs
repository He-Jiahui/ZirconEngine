#[path = "command.rs"]
mod command;
#[path = "command_metrics.rs"]
mod command_metrics;
#[path = "command_queue.rs"]
mod command_queue;
#[path = "commands/mod.rs"]
mod commands;
#[path = "inline_command_arena.rs"]
mod inline_command_arena;
#[path = "queued_command.rs"]
mod queued_command;
#[path = "structural.rs"]
mod structural;
#[path = "worker_command_buffer.rs"]
mod worker_command_buffer;

pub(crate) use command::ErasedCommand;
pub use command::{
    Command, DeferredCommandError, DeferredCommandOperation, DeferredCommandReport,
    DeferredCommandTarget, DeferredEntity, DeferredEntityRef, DeferredSpawnToken,
    DeferredSystemKey, FnCommand,
};
pub use command_metrics::CommandQueueMetrics;
pub use command_queue::CommandQueue;
pub use commands::{Commands, CommandsParam, EntityCommands};
pub(crate) use structural::{
    DeferredStructuralKind, DeferredStructuralMetadata, ErasedQueuedStructuralCommand,
    QueuedStructuralCommand,
};
pub use worker_command_buffer::{WorkerCommandBuffer, WorkerCommandBufferMergeError};
