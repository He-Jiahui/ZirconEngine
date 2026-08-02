use super::{RuntimeOperationContext, RuntimeOperationHandlerError};

/// Worker-owned command and result produced before an owner-thread mutation.
pub struct RuntimeOperationPrepared {
    command: serde_json::Value,
    result: serde_json::Value,
}

impl RuntimeOperationPrepared {
    pub fn new(command: serde_json::Value, result: serde_json::Value) -> Self {
        Self { command, result }
    }

    pub(super) fn into_parts(self) -> (serde_json::Value, serde_json::Value) {
        (self.command, self.result)
    }
}

pub trait RuntimeOperationHandler: Send + Sync {
    /// Captures immutable, owned input on the runtime owner thread.
    fn snapshot(
        &self,
        context: RuntimeOperationContext<'_>,
        payload: serde_json::Value,
    ) -> Result<serde_json::Value, RuntimeOperationHandlerError>;

    /// Produces an owned apply command and its bounded terminal result off-thread.
    fn prepare(
        &self,
        snapshot: serde_json::Value,
    ) -> Result<RuntimeOperationPrepared, RuntimeOperationHandlerError>;

    /// Commits a previously prepared command on the runtime owner thread.
    fn apply(
        &self,
        context: RuntimeOperationContext<'_>,
        command: serde_json::Value,
    ) -> Result<(), RuntimeOperationHandlerError>;
}
