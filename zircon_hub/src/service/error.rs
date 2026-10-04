use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};

#[derive(Debug, thiserror::Error)]
pub enum ServiceError {
    #[error("configuration_invalid")]
    Configuration,
    #[error("identity_unavailable")]
    IdentityUnavailable,
    #[error("unauthorized")]
    Unauthorized,
    #[error("forbidden")]
    Forbidden,
    #[error("policy_conflict")]
    Conflict,
    #[error("invalid_request")]
    InvalidRequest,
    #[error("storage_unavailable")]
    Storage,
    #[error("capacity_exceeded")]
    Capacity,
    #[error("operation_outcome_unknown")]
    OutcomeUnknown,
    #[error("operation_id_conflict")]
    OperationConflict,
}

impl ServiceError {
    pub fn code(&self) -> &'static str {
        match self {
            Self::Configuration => "configuration_invalid",
            Self::IdentityUnavailable => "identity_unavailable",
            Self::Unauthorized => "unauthorized",
            Self::Forbidden => "forbidden",
            Self::Conflict => "policy_conflict",
            Self::InvalidRequest => "invalid_request",
            Self::Storage => "storage_unavailable",
            Self::Capacity => "capacity_exceeded",
            Self::OutcomeUnknown => "operation_outcome_unknown",
            Self::OperationConflict => "operation_id_conflict",
        }
    }
}

impl From<rusqlite::Error> for ServiceError {
    fn from(_: rusqlite::Error) -> Self {
        Self::Storage
    }
}

impl IntoResponse for ServiceError {
    fn into_response(self) -> Response {
        let status = match self {
            Self::Unauthorized => StatusCode::UNAUTHORIZED,
            Self::Forbidden => StatusCode::FORBIDDEN,
            Self::Conflict | Self::OperationConflict => StatusCode::CONFLICT,
            Self::InvalidRequest => StatusCode::BAD_REQUEST,
            Self::Capacity => StatusCode::TOO_MANY_REQUESTS,
            _ => StatusCode::SERVICE_UNAVAILABLE,
        };
        (status, Json(serde_json::json!({"error": self.code()}))).into_response()
    }
}
