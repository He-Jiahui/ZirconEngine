use thiserror::Error;
use zircon_editor::core::gateway::GatewayError;

#[derive(Debug, Error)]
pub(super) enum NavigationOperationCommandError {
    #[error("navigation runtime gateway failed: {source}")]
    Gateway {
        #[source]
        source: GatewayError,
    },
    #[error("navigation runtime operation failed: {message}")]
    Failed { message: String },
    #[error("navigation runtime operation protocol failed: {message}")]
    Protocol { message: String },
    #[error("navigation runtime operation did not complete within the polling budget")]
    PollBudgetExhausted,
}
