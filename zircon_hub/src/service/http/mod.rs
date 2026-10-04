use super::{
    catalog, cloud,
    error::ServiceError,
    identity::{OidcVerifier, Principal},
    organization::{
        self, CreateOrganization, Invitation, IssuedInvitation, MutationRequest, Organization,
        Page, PageQuery, Receipt,
    },
    storage::{
        receipt::{self, OperationStatus},
        Database,
    },
};
use axum::{
    extract::{DefaultBodyLimit, Path, Query, Request, State},
    http::{header, HeaderMap},
    middleware::{self, Next},
    response::Response,
    routing::{get, post},
    Json, Router,
};
use std::sync::Arc;
use tokio::sync::Semaphore;
mod catalog_routes;
mod cloud_routes;
use catalog_routes::{
    catalog_artifact, catalog_entitlements, catalog_license, catalog_list, catalog_publish,
};

#[derive(Clone)]
struct AppState {
    identity: OidcVerifier,
    database: Database,
    requests: Arc<Semaphore>,
    catalog_policy_file: Option<std::path::PathBuf>,
    cloud: Arc<cloud::BlobStore>,
    cloud_requests: Arc<Semaphore>,
    catalog_requests: Arc<Semaphore>,
}

pub fn router(
    identity: OidcVerifier,
    database: Database,
    catalog_policy_file: Option<std::path::PathBuf>,
    cloud: Arc<cloud::BlobStore>,
) -> Router {
    let state = AppState {
        identity,
        database,
        requests: Arc::new(Semaphore::new(32)),
        catalog_policy_file,
        cloud,
        cloud_requests: Arc::new(Semaphore::new(4)),
        catalog_requests: Arc::new(Semaphore::new(2)),
    };
    Router::new()
        .route(
            "/health",
            get(|| async { Json(serde_json::json!({"status":"alive","protocolVersion":1})) }),
        )
        .route("/v1/organizations", get(list).post(create))
        .route("/v1/organizations/{organization}/mutations", post(mutate))
        .route("/v1/organizations/{organization}/members", get(members))
        .route(
            "/v1/organizations/{organization}/invitations",
            get(issued_invitations),
        )
        .route("/v1/organizations/{organization}/projects", get(projects))
        .route("/v1/invitations", get(invitations))
        .route("/v1/operations/{operation}", get(operation))
        .route("/v1/catalog", get(catalog_list).post(catalog_publish))
        .route(
            "/v1/catalog/{package}/{revision}/artifact",
            axum::routing::put(catalog_routes::artifacts::upload),
        )
        .route(
            "/v1/organizations/{organization}/catalog/{package}/{revision}",
            get(catalog_artifact),
        )
        .route(
            "/v1/organizations/{organization}/catalog/{package}/{revision}/artifact",
            get(catalog_routes::artifacts::download),
        )
        .route(
            "/v1/organizations/{organization}/catalog/{package}/{revision}/manifest",
            get(catalog_routes::artifacts::manifest),
        )
        .route(
            "/v1/organizations/{organization}/licenses",
            get(catalog_entitlements).post(catalog_license),
        )
        .route(
            "/v1/organizations/{organization}/projects/{project}/cloud/head",
            get(cloud_routes::head),
        )
        .route(
            "/v1/organizations/{organization}/projects/{project}/cloud/usage",
            get(cloud_routes::usage),
        )
        .route(
            "/v1/organizations/{organization}/projects/{project}/cloud/retention",
            get(cloud_routes::retention).post(cloud_routes::update_retention),
        )
        .route(
            "/v1/organizations/{organization}/projects/{project}/cloud/maintenance",
            post(cloud_routes::maintenance),
        )
        .route(
            "/v1/organizations/{organization}/projects/{project}/cloud/commit",
            post(cloud_routes::commit),
        )
        .route(
            "/v1/organizations/{organization}/projects/{project}/cloud/blobs/{digest}",
            get(cloud_routes::download).put(cloud_routes::upload),
        )
        .layer(DefaultBodyLimit::max(65536))
        .layer(middleware::from_fn_with_state(
            state.clone(),
            bounded_request,
        ))
        .with_state(state)
}

async fn bounded_request(
    State(state): State<AppState>,
    request: Request,
    next: Next,
) -> Result<Response, ServiceError> {
    // Browser origins cannot invoke this privileged local API; the desktop broker uses native HTTP.
    if request.headers().contains_key(header::ORIGIN) {
        return Err(ServiceError::Forbidden);
    }
    let _permit = state
        .requests
        .try_acquire()
        .map_err(|_| ServiceError::Capacity)?;
    tokio::time::timeout(std::time::Duration::from_secs(15), next.run(request))
        .await
        .map_err(|_| ServiceError::OutcomeUnknown)
}

async fn principal(state: &AppState, headers: &HeaderMap) -> Result<Principal, ServiceError> {
    let bearer = headers
        .get(header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.strip_prefix("Bearer "))
        .ok_or(ServiceError::Unauthorized)?;
    state.identity.verify(bearer).await
}

async fn list(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<PageQuery>,
) -> Result<Json<Page<Organization>>, ServiceError> {
    let principal = principal(&state, &headers).await?;
    state
        .database
        .execute(move |connection| organization::list(connection, &principal, query))
        .await
        .map(Json)
}

async fn create(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(request): Json<CreateOrganization>,
) -> Result<Json<Organization>, ServiceError> {
    let principal = principal(&state, &headers).await?;
    state
        .database
        .execute(move |connection| {
            organization::create(connection, &principal, &request.operation_id, &request.name)
        })
        .await
        .map(Json)
}

async fn invitations(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<PageQuery>,
) -> Result<Json<Page<Invitation>>, ServiceError> {
    let principal = principal(&state, &headers).await?;
    state
        .database
        .execute(move |connection| organization::invitations(connection, &principal, query))
        .await
        .map(Json)
}

async fn issued_invitations(
    State(state): State<AppState>,
    Path(organization): Path<String>,
    headers: HeaderMap,
    Query(query): Query<PageQuery>,
) -> Result<Json<Page<IssuedInvitation>>, ServiceError> {
    let principal = principal(&state, &headers).await?;
    state
        .database
        .execute(move |connection| {
            organization::issued_invitations(connection, &principal, &organization, query)
        })
        .await
        .map(Json)
}

async fn operation(
    State(state): State<AppState>,
    Path(operation): Path<String>,
    headers: HeaderMap,
) -> Result<Json<OperationStatus>, ServiceError> {
    let principal = principal(&state, &headers).await?;
    state
        .database
        .execute(move |connection| {
            let status = receipt::lookup(connection, &principal, &operation)?;
            if let OperationStatus::Committed { result } = &status {
                cloud::authorize_receipt(connection, &principal, result)?;
            }
            Ok(status)
        })
        .await
        .map(Json)
}

async fn members(
    State(state): State<AppState>,
    Path(organization): Path<String>,
    headers: HeaderMap,
    Query(query): Query<PageQuery>,
) -> Result<Json<Page<organization::Member>>, ServiceError> {
    let principal = principal(&state, &headers).await?;
    state
        .database
        .execute(move |connection| {
            organization::members(connection, &principal, &organization, query)
        })
        .await
        .map(Json)
}

async fn projects(
    State(state): State<AppState>,
    Path(organization): Path<String>,
    headers: HeaderMap,
    Query(query): Query<PageQuery>,
) -> Result<Json<Page<organization::Project>>, ServiceError> {
    let principal = principal(&state, &headers).await?;
    state
        .database
        .execute(move |connection| {
            organization::projects(connection, &principal, &organization, query)
        })
        .await
        .map(Json)
}

async fn mutate(
    State(state): State<AppState>,
    Path(organization): Path<String>,
    headers: HeaderMap,
    Json(request): Json<MutationRequest>,
) -> Result<Json<Receipt>, ServiceError> {
    let principal = principal(&state, &headers).await?;
    state
        .database
        .execute(move |connection| {
            organization::mutate(connection, &principal, &organization, request)
        })
        .await
        .map(Json)
}
