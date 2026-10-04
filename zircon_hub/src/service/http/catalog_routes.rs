use super::*;

pub(super) mod artifacts;

pub(super) async fn catalog_entitlements(
    State(state): State<AppState>,
    Path(organization): Path<String>,
    headers: HeaderMap,
    Query(query): Query<PageQuery>,
) -> Result<Json<Page<catalog::Entitlement>>, ServiceError> {
    let principal = principal(&state, &headers).await?;
    state
        .database
        .execute(move |connection| {
            catalog::entitlements(connection, &principal, &organization, query.after)
        })
        .await
        .map(Json)
}

pub(super) async fn catalog_artifact(
    State(state): State<AppState>,
    Path((organization, package, revision)): Path<(String, String, String)>,
    headers: HeaderMap,
) -> Result<Json<catalog::Release>, ServiceError> {
    let principal = principal(&state, &headers).await?;
    state
        .database
        .execute(move |connection| {
            let policy = catalog::CatalogPolicy::load(state.catalog_policy_file.as_deref())?;
            catalog::authorized_artifact(
                connection,
                &principal,
                &policy,
                &organization,
                &package,
                &revision,
            )
        })
        .await
        .map(Json)
}

pub(super) async fn catalog_list(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<catalog::CatalogQuery>,
) -> Result<Json<catalog::CatalogPage>, ServiceError> {
    let _principal = principal(&state, &headers).await?;
    state
        .database
        .execute(move |connection| {
            let policy = catalog::CatalogPolicy::load(state.catalog_policy_file.as_deref())?;
            catalog::list(connection, &policy, query)
        })
        .await
        .map(Json)
}

pub(super) async fn catalog_publish(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(request): Json<catalog::PublishRequest>,
) -> Result<Json<catalog::Publication>, ServiceError> {
    let principal = principal(&state, &headers).await?;
    state
        .database
        .execute(move |connection| {
            let policy = catalog::CatalogPolicy::load(state.catalog_policy_file.as_deref())?;
            catalog::publish(connection, &principal, &policy, request)
        })
        .await
        .map(Json)
}

pub(super) async fn catalog_license(
    State(state): State<AppState>,
    Path(organization): Path<String>,
    headers: HeaderMap,
    Json(request): Json<catalog::AcceptLicense>,
) -> Result<Json<catalog::Publication>, ServiceError> {
    let principal = principal(&state, &headers).await?;
    state
        .database
        .execute(move |connection| {
            let policy = catalog::CatalogPolicy::load(state.catalog_policy_file.as_deref())?;
            catalog::accept_license(connection, &principal, &policy, &organization, request)
        })
        .await
        .map(Json)
}
