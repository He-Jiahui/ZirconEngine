use super::{RuntimeOperationContext, RuntimeOperationHandlerError};

pub trait RuntimeOperationHandler: Send + Sync {
    fn execute(
        &self,
        context: RuntimeOperationContext<'_>,
        payload: serde_json::Value,
    ) -> Result<serde_json::Value, RuntimeOperationHandlerError>;
}
