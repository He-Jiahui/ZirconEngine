use super::*;

#[test]
fn cloud_head_route_is_read_only_and_uses_canonical_qualified_ids() {
    let organization = "00000000-0000-4000-8000-000000000001";
    let project = "00000000-0000-4000-8000-000000000002";
    let prepared = ServiceRequest::CloudHead {
        organization: organization.into(),
        project: project.into(),
    }
    .parts()
    .unwrap();
    assert_eq!(
        prepared.path,
        format!("/v1/organizations/{organization}/projects/{project}/cloud/head")
    );
    assert!(prepared.body.is_none());
    assert!(prepared.operation.is_none());
    assert_eq!(
        prepared.cloud_scope,
        Some((organization.into(), project.into()))
    );
    for (organization, project) in [
        ("../escape", project),
        (organization, "bad?query"),
        (organization, "00000000-0000-4000-8000-00000000000A"),
    ] {
        assert!(ServiceRequest::CloudHead {
            organization: organization.into(),
            project: project.into()
        }
        .parts()
        .is_err());
    }
}

#[test]
fn cloud_commit_prepares_a_canonical_bounded_journal_payload() {
    let organization = "00000000-0000-4000-8000-000000000001";
    let project = "00000000-0000-4000-8000-000000000002";
    let operation_id = "00000000-0000-4000-8000-000000000003";
    let manifest = serde_json::json!({
        "schemaVersion":1,"engine":"Zircon","packageLockDigest":"a".repeat(64),
        "ignorePolicy":"zircon-project-v1","sourceRevision":null,
        "files":[
            {"path":"Content/z.zr","digest":"b".repeat(64),"bytes":1},
            {"path":"Content/a.zr","digest":"c".repeat(64),"bytes":2}
        ]
    });
    let prepared = ServiceRequest::CloudCommit {
        organization: organization.into(),
        project: project.into(),
        operation_id: operation_id.into(),
        base_revision: "0".into(),
        manifest: manifest.clone(),
    }
    .parts()
    .unwrap();
    assert_eq!(
        prepared.path,
        format!("/v1/organizations/{organization}/projects/{project}/cloud/commit")
    );
    let body: Value = serde_json::from_slice(prepared.body.as_ref().unwrap()).unwrap();
    assert_eq!(body["operationId"], operation_id);
    assert_eq!(body["manifest"]["files"][0]["path"], "Content/a.zr");
    assert!(
        matches!(prepared.operation, Some((id, OperationPayload::CloudCommit { .. })) if id == operation_id)
    );
    for (organization, project, base_revision) in [
        ("../escape", project, "0"),
        (organization, "bad?query", "0"),
        (organization, project, "00"),
    ] {
        assert!(ServiceRequest::CloudCommit {
            organization: organization.into(),
            project: project.into(),
            operation_id: operation_id.into(),
            base_revision: base_revision.into(),
            manifest: manifest.clone(),
        }
        .parts()
        .is_err());
    }
    assert!(ServiceRequest::CloudCommit {
        organization: organization.into(),
        project: project.into(),
        operation_id: "not-a-uuid".into(),
        base_revision: "0".into(),
        manifest,
    }
    .parts()
    .is_err());
}

#[tokio::test]
async fn cloud_commit_transport_preserves_typed_http_conflict() {
    use axum::{http::StatusCode, routing::post, Json, Router};

    let response = serde_json::json!({
        "status":"conflict",
        "organizationId":"00000000-0000-4000-8000-000000000001",
        "projectId":"00000000-0000-4000-8000-000000000002",
        "baseRevision":"0","currentRevision":"1",
        "manifestDigest":"a".repeat(64)
    });
    let router = Router::new()
        .route(
            "/commit",
            post(move || {
                let response = response.clone();
                async move { (StatusCode::CONFLICT, Json(response)) }
            }),
        )
        .route(
            "/reused",
            post(|| async {
                (
                    StatusCode::CONFLICT,
                    Json(serde_json::json!({"error":"operation_id_conflict"})),
                )
            }),
        );
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let base = format!("http://{}", listener.local_addr().unwrap());
    let server = tokio::spawn(async move { axum::serve(listener, router).await.unwrap() });
    let http = oidc::http().unwrap();
    let request = http.post(format!("{base}/commit")).build().unwrap();
    let result = send_service_request(&http, request, true, true, SERVICE_RESPONSE_LIMIT)
        .await
        .unwrap();
    let reused = http.post(format!("{base}/reused")).build().unwrap();
    let reused = send_service_request(&http, reused, true, true, SERVICE_RESPONSE_LIMIT).await;
    server.abort();
    let _ = server.await;
    assert!(cloud::commit::is_conflict(&result));
    assert!(matches!(reused, Err(AccountError::OperationConflict)));
    for first_attempt in [true, false] {
        assert_eq!(
            mutation_status(&reused, true, first_attempt),
            OperationStatus::OperationIdConflict
        );
    }
    assert_eq!(
        mutation_status(&Err(AccountError::ServiceFailure), true, false),
        OperationStatus::Unknown
    );
}

#[test]
fn catalog_requests_reject_path_and_revision_aliases_and_keep_search_structured() {
    let request = ServiceRequest::Catalog {
        after: None,
        query: Some("name&after=other".into()),
    }
    .parts()
    .unwrap();
    assert_eq!(request.path, "/v1/catalog");
    assert_eq!(request.query.as_deref(), Some("name&after=other"));
    assert!(ServiceRequest::Catalog {
        after: None,
        query: Some("x".repeat(257))
    }
    .parts()
    .is_err());
    for organization in ["../other", "bad?query"] {
        assert!(ServiceRequest::CatalogEntitlements {
            organization: organization.into(),
            after: None
        }
        .parts()
        .is_err());
    }
    assert!(ServiceRequest::CatalogArtifact {
        organization: "00000000-0000-4000-8000-000000000003".into(),
        package_id: "00000000-0000-4000-8000-000000000004".into(),
        revision: "01".into()
    }
    .parts()
    .is_err());
}

#[test]
fn catalog_entitlements_request_uses_the_license_route() {
    let request = ServiceRequest::CatalogEntitlements {
        organization: "00000000-0000-4000-8000-000000000002".into(),
        after: None,
    };

    let prepared = request.parts().unwrap();

    assert_eq!(
        prepared.path,
        "/v1/organizations/00000000-0000-4000-8000-000000000002/licenses"
    );
}

#[test]
fn native_service_paths_cannot_escape_the_whitelisted_resource() {
    for organization in ["../other", "a?token=x", "a/b", "https://other.example"] {
        assert!(ServiceRequest::Members {
            organization: organization.into(),
            after: None
        }
        .parts()
        .is_err());
        assert!(ServiceRequest::IssuedInvitations {
            organization: organization.into(),
            after: None
        }
        .parts()
        .is_err());
    }
    let request = ServiceRequest::IssuedInvitations {
        organization: "00000000-0000-4000-8000-000000000003".into(),
        after: Some("00000000-0000-4000-8000-000000000004".into()),
    }
    .parts()
    .unwrap();
    assert_eq!(
        request.path,
        "/v1/organizations/00000000-0000-4000-8000-000000000003/invitations"
    );
    assert_eq!(
        request.after.as_deref(),
        Some("00000000-0000-4000-8000-000000000004")
    );
}

#[tokio::test]
async fn committed_response_over_budget_preserves_unknown_outcome() {
    use axum::{routing::post, Router};
    use std::sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    };

    let commits = Arc::new(AtomicUsize::new(0));
    let router = Router::new().route(
        "/mutation",
        post({
            let commits = commits.clone();
            move || {
                let commits = commits.clone();
                async move {
                    commits.fetch_add(1, Ordering::SeqCst);
                    vec![b'x'; SERVICE_RESPONSE_LIMIT + 1]
                }
            }
        }),
    );
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let url = format!("http://{}/mutation", listener.local_addr().unwrap());
    let server = tokio::spawn(async move { axum::serve(listener, router).await.unwrap() });
    let response = oidc::http().unwrap().post(url).send().await.unwrap();
    let result = read_service_body(response, true, SERVICE_RESPONSE_LIMIT).await;
    server.abort();
    let _ = server.await;
    assert_eq!(commits.load(Ordering::SeqCst), 1);
    assert!(matches!(result, Err(AccountError::OutcomeUnknown)));
}

#[tokio::test]
async fn response_budget_accepts_the_exact_limit_and_rejects_oversized_reads() {
    use axum::{routing::get, Router};

    let router = Router::new()
        .route(
            "/exact",
            get(|| async { vec![b'x'; SERVICE_RESPONSE_LIMIT] }),
        )
        .route(
            "/over",
            get(|| async { vec![b'x'; SERVICE_RESPONSE_LIMIT + 1] }),
        );
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let base = format!("http://{}", listener.local_addr().unwrap());
    let server = tokio::spawn(async move { axum::serve(listener, router).await.unwrap() });
    let http = oidc::http().unwrap();
    let exact = http.get(format!("{base}/exact")).send().await.unwrap();
    let exact = read_service_body(exact, false, SERVICE_RESPONSE_LIMIT).await;
    let over = http.get(format!("{base}/over")).send().await.unwrap();
    let over = read_service_body(over, false, SERVICE_RESPONSE_LIMIT).await;
    server.abort();
    let _ = server.await;
    assert_eq!(exact.unwrap().len(), SERVICE_RESPONSE_LIMIT);
    assert!(matches!(over, Err(AccountError::ServiceFailure)));
}
