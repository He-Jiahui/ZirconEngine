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

pub(crate) async fn upload(
    State(state): State<AppState>,
    Path((package, revision)): Path<(String, String)>,
    request: Request,
) -> Result<StatusCode, ServiceError> {
    let permit = state
        .catalog_requests
        .clone()
        .try_acquire_owned()
        .map_err(|_| ServiceError::Capacity)?;
    let principal = principal(&state, request.headers()).await?;
    if request
        .headers()
        .get(CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        != Some("application/octet-stream")
    {
        return Err(ServiceError::InvalidRequest);
    }
    let bytes = to_bytes(request.into_body(), catalog::artifacts::MAX_ARTIFACT_BYTES)
        .await
        .map_err(|_| ServiceError::Capacity)?;
    state
        .database
        .execute(move |connection| {
            let _permit = permit;
            let policy = catalog::CatalogPolicy::load(state.catalog_policy_file.as_deref())?;
            catalog::artifacts::upload(connection, &principal, &policy, &package, &revision, &bytes)
        })
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

pub(crate) async fn download(
    State(state): State<AppState>,
    Path((organization, package, revision)): Path<(String, String, String)>,
    headers: HeaderMap,
) -> Result<impl IntoResponse, ServiceError> {
    let permit = state
        .catalog_requests
        .clone()
        .try_acquire_owned()
        .map_err(|_| ServiceError::Capacity)?;
    let principal = principal(&state, &headers).await?;
    let bytes = state
        .database
        .execute(move |connection| {
            let policy = catalog::CatalogPolicy::load(state.catalog_policy_file.as_deref())?;
            let bytes = catalog::artifacts::download(
                connection,
                &principal,
                &policy,
                &organization,
                &package,
                &revision,
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
            (header::X_CONTENT_TYPE_OPTIONS, "nosniff"),
        ],
        Body::from(Bytes::from_owner(bytes)),
    ))
}

pub(crate) async fn manifest(
    State(state): State<AppState>,
    Path((organization, package, revision)): Path<(String, String, String)>,
    headers: HeaderMap,
) -> Result<impl IntoResponse, ServiceError> {
    let principal = principal(&state, &headers).await?;
    let envelope = state
        .database
        .execute(move |connection| {
            let policy = catalog::CatalogPolicy::load(state.catalog_policy_file.as_deref())?;
            catalog::artifacts::envelope(
                connection,
                &principal,
                &policy,
                &organization,
                &package,
                &revision,
            )
        })
        .await?;
    Ok((
        [
            (CONTENT_TYPE, "application/jwt"),
            (header::CACHE_CONTROL, "no-store"),
            (header::X_CONTENT_TYPE_OPTIONS, "nosniff"),
        ],
        envelope,
    ))
}
