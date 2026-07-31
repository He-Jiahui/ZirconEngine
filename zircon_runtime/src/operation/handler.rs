use super::{RuntimeOperationContext, RuntimeOperationHandlerError};

pub trait RuntimeOperationHandler: Send + Sync {
    /// Validates and normalizes owned input without touching runtime-owned state.
    fn prepare(
        &self,
        payload: serde_json::Value,
    ) -> Result<serde_json::Value, RuntimeOperationHandlerError>;

    /// Applies prepared work on the runtime owner thread.
    fn apply(
        &self,
        context: RuntimeOperationContext<'_>,
        prepared: serde_json::Value,
    ) -> Result<serde_json::Value, RuntimeOperationHandlerError>;
}
