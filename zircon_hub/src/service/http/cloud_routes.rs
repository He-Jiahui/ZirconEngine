use super::*;
use axum::{
    body::{to_bytes, Body, Bytes},
    http::{header::CONTENT_TYPE, StatusCode},
    response::IntoResponse,
};
use tokio::sync::OwnedSemaphorePermit;

struct TransferBytes {
    bytes: Vec<u8>,
    _permit: OwnedSemaphorePermit,
}

impl AsRef<[u8]> for TransferBytes {
    fn as_ref(&self) -> &[u8] {
        &self.bytes
    }
}

fn admit(state: &AppState) -> Result<OwnedSemaphorePermit, ServiceError> {
    state
        .cloud_requests
        .clone()
        .try_acquire_owned()
        .map_err(|_| ServiceError::Capacity)
}

pub(super) async fn head(
    State(state): State<AppState>,
    Path((organization, project)): Path<(String, String)>,
    headers: HeaderMap,
) -> Result<impl IntoResponse, ServiceError> {
    let permit = admit(&state)?;
    let principal = principal(&state, &headers).await?;
    let bytes = state
        .database
        .execute(move |connection| {
            let snapshot = cloud::head(connection, &principal, &organization, &project)?;
            let bytes = serde_json::to_vec(&snapshot).map_err(|_| ServiceError::Storage)?;
            Ok(TransferBytes {
                bytes,
                _permit: permit,
            })
        })
        .await?;
    Ok((
        [
            (CONTENT_TYPE, "application/json"),
            (header::CACHE_CONTROL, "no-store"),
        ],
        Body::from(Bytes::from_owner(bytes)),
    ))
}

pub(super) async fn usage(
    State(state): State<AppState>,
    Path((organization, project)): Path<(String, String)>,
    headers: HeaderMap,
) -> Result<Json<cloud::Usage>, ServiceError> {
    let principal = principal(&state, &headers).await?;
    state
        .database
        .execute(move |connection| cloud::usage(connection, &principal, &organization, &project))
        .await
        .map(Json)
}

pub(super) async fn commit(
    State(state): State<AppState>,
    Path((organization, project)): Path<(String, String)>,
    request: Request,
) -> Result<impl IntoResponse, ServiceError> {
    let permit = admit(&state)?;
    let principal = principal(&state, request.headers()).await?;
    if request
        .headers()
        .get(CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.split(';').next())
        != Some("application/json")
    {
        return Err(ServiceError::InvalidRequest);
    }
    let bytes = to_bytes(request.into_body(), cloud::MAX_MANIFEST_BYTES + 65536)
        .await
        .map_err(|_| ServiceError::Capacity)?;
    let store = state.cloud.clone();
    let (status, bytes) = state
        .database
        .execute(move |connection| {
            let request: cloud::CommitRequest =
                serde_json::from_slice(&bytes).map_err(|_| ServiceError::InvalidRequest)?;
            let outcome = cloud::commit(
                connection,
                &principal,
                &store,
                &organization,
                &project,
                request,
            )?;
            let status = if matches!(outcome, cloud::CommitOutcome::Conflict { .. }) {
                StatusCode::CONFLICT
            } else {
                StatusCode::OK
            };
            let bytes = serde_json::to_vec(&outcome).map_err(|_| ServiceError::Storage)?;
            Ok((
                status,
                TransferBytes {
                    bytes,
                    _permit: permit,
                },
            ))
        })
        .await?;
    Ok((
        status,
        [
            (CONTENT_TYPE, "application/json"),
            (header::CACHE_CONTROL, "no-store"),
        ],
        Body::from(Bytes::from_owner(bytes)),
    ))
}

pub(super) async fn retention(
    State(state): State<AppState>,
    Path((organization, project)): Path<(String, String)>,
    headers: HeaderMap,
) -> Result<impl IntoResponse, ServiceError> {
    let principal = principal(&state, &headers).await?;
    let retention = state
        .database
        .execute(move |connection| {
            cloud::retention(connection, &principal, &organization, &project)
        })
        .await?;
    Ok(([(header::CACHE_CONTROL, "no-store")], Json(retention)))
}

pub(super) async fn update_retention(
    State(state): State<AppState>,
    Path((organization, project)): Path<(String, String)>,
    headers: HeaderMap,
    Json(request): Json<cloud::RetentionRequest>,
) -> Result<impl IntoResponse, ServiceError> {
    let permit = admit(&state)?;
    let principal = principal(&state, &headers).await?;
    let outcome = state
        .database
        .execute(move |connection| {
            let _permit = permit;
            cloud::update_retention(connection, &principal, &organization, &project, request)
        })
        .await?;
    let status = if matches!(&outcome, cloud::RetentionOutcome::RetentionConflict { .. }) {
        StatusCode::CONFLICT
    } else {
        StatusCode::OK
    };
    Ok((status, [(header::CACHE_CONTROL, "no-store")], Json(outcome)))
}

pub(super) async fn maintenance(
    State(state): State<AppState>,
    Path((organization, project)): Path<(String, String)>,
    headers: HeaderMap,
    Json(request): Json<cloud::MaintenanceRequest>,
) -> Result<impl IntoResponse, ServiceError> {
    let permit = admit(&state)?;
    let principal = principal(&state, &headers).await?;
    let store = state.cloud.clone();
    let report = state
        .database
        .execute(move |connection| {
            let _permit = permit;
            cloud::maintain(
                connection,
                &principal,
                &store,
                &organization,
                &project,
                request,
            )
        })
        .await?;
    Ok(([(header::CACHE_CONTROL, "no-store")], Json(report)))
}

pub(super) async fn upload(
    State(state): State<AppState>,
    Path((organization, project, digest)): Path<(String, String, String)>,
    request: Request,
) -> Result<StatusCode, ServiceError> {
    let permit = admit(&state)?;
    let principal = principal(&state, request.headers()).await?;
    let body = to_bytes(request.into_body(), cloud::MAX_BLOB_BYTES)
        .await
        .map_err(|_| ServiceError::Capacity)?;
    let store = state.cloud.clone();
    state
        .database
        .execute(move |connection| {
            let _permit = permit;
            cloud::upload(
                connection,
                &principal,
                &store,
                &organization,
                &project,
                &digest,
                &body,
            )
        })
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

pub(super) async fn download(
    State(state): State<AppState>,
    Path((organization, project, digest)): Path<(String, String, String)>,
    headers: HeaderMap,
) -> Result<impl IntoResponse, ServiceError> {
    let permit = admit(&state)?;
    let principal = principal(&state, &headers).await?;
    let store = state.cloud.clone();
    let bytes = state
        .database
        .execute(move |connection| {
            let bytes = cloud::read_blob(
                connection,
                &principal,
                &store,
                &organization,
                &project,
                &digest,
            )?;
            Ok(TransferBytes {
                bytes,
                _permit: permit,
            })
        })
        .await?;
    Ok((
        [
            (CONTENT_TYPE, "application/octet-stream"),
            (header::CACHE_CONTROL, "no-store"),
        ],
        Body::from(Bytes::from_owner(bytes)),
    ))
}

#[cfg(test)]
#[path = "cloud_routes/tests/cases.rs"]
mod tests;
