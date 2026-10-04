use super::*;
use crate::account::tests::{broker, set_account};
use axum::{
    body::Bytes,
    extract::Path,
    http::{HeaderMap, StatusCode},
    routing::{get, put},
    Router,
};
use sha2::{Digest, Sha256};
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};

const ORGANIZATION: &str = "00000000-0000-4000-8000-000000000001";
const PROJECT: &str = "00000000-0000-4000-8000-000000000002";
const BLOB_ROUTE: &str = "/v1/organizations/{organization}/projects/{project}/cloud/blobs/{digest}";

fn sha256(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

#[tokio::test]
async fn blob_download_is_scoped_authenticated_and_byte_preserving() {
    let bytes = b"\0not-json\xff".to_vec();
    let expected = sha256(&bytes);
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let broker = broker(format!("http://{}", listener.local_addr().unwrap()));
    set_account(&broker, "alice", 7).await;
    let expected_route_digest = expected.clone();
    let router = Router::new().route(
        BLOB_ROUTE,
        get(
            move |Path((organization, project, actual_digest)): Path<(String, String, String)>,
                  headers: HeaderMap| {
                let bytes = bytes.clone();
                let expected_route_digest = expected_route_digest.clone();
                async move {
                    assert_eq!(organization, ORGANIZATION);
                    assert_eq!(project, PROJECT);
                    assert_eq!(actual_digest, expected_route_digest);
                    assert_eq!(
                        headers.get("authorization").unwrap().to_str().unwrap(),
                        "Bearer fixture-access"
                    );
                    bytes
                }
            },
        ),
    );
    let server = tokio::spawn(async move { axum::serve(listener, router).await.unwrap() });
    let actual = broker
        .download_cloud_blob("7", ORGANIZATION, PROJECT, &expected)
        .await
        .unwrap();
    assert_eq!(actual.as_slice(), b"\0not-json\xff");
    server.abort();
}

#[tokio::test]
async fn rejects_untrusted_scope_digest_response_and_oversize_before_exposing_bytes() {
    let requests = Arc::new(AtomicUsize::new(0));
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let broker = broker(format!("http://{}", listener.local_addr().unwrap()));
    set_account(&broker, "alice", 7).await;
    let router = Router::new().route(
        BLOB_ROUTE,
        get({
            let requests = requests.clone();
            move |Path((_organization, _project, digest)): Path<(String, String, String)>| {
                let requests = requests.clone();
                async move {
                    requests.fetch_add(1, Ordering::SeqCst);
                    if digest == "a".repeat(64) {
                        (StatusCode::OK, b"wrong bytes".to_vec())
                    } else if digest == "b".repeat(64) {
                        (StatusCode::OK, vec![0; MAX_BLOB_BYTES + 1])
                    } else {
                        (StatusCode::FORBIDDEN, Vec::new())
                    }
                }
            }
        }),
    );
    let server = tokio::spawn(async move { axum::serve(listener, router).await.unwrap() });
    for (organization, project, digest) in [
        ("../other", PROJECT, "a".repeat(64)),
        (ORGANIZATION, "../other", "a".repeat(64)),
        (ORGANIZATION, PROJECT, "A".repeat(64)),
    ] {
        assert!(matches!(
            broker
                .download_cloud_blob("7", organization, project, &digest)
                .await,
            Err(AccountError::ServiceFailure)
        ));
    }
    assert_eq!(requests.load(Ordering::SeqCst), 0);
    for digest in ["a".repeat(64), "b".repeat(64), "c".repeat(64)] {
        assert!(matches!(
            broker
                .download_cloud_blob("7", ORGANIZATION, PROJECT, &digest)
                .await,
            Err(AccountError::ServiceFailure)
        ));
    }
    assert_eq!(requests.load(Ordering::SeqCst), 3);
    server.abort();
}

#[tokio::test]
async fn late_blob_from_previous_account_generation_is_discarded() {
    let entered = Arc::new(tokio::sync::Notify::new());
    let release = Arc::new(tokio::sync::Notify::new());
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let broker = broker(format!("http://{}", listener.local_addr().unwrap()));
    set_account(&broker, "alice", 7).await;
    let router = Router::new().route(
        BLOB_ROUTE,
        get({
            let entered = entered.clone();
            let release = release.clone();
            move || {
                let entered = entered.clone();
                let release = release.clone();
                async move {
                    entered.notify_one();
                    release.notified().await;
                    b"old account bytes".to_vec()
                }
            }
        }),
    );
    let server = tokio::spawn(async move { axum::serve(listener, router).await.unwrap() });
    let reader = broker.clone();
    let query = tokio::spawn(async move {
        reader
            .download_cloud_blob("7", ORGANIZATION, PROJECT, &sha256(b"old account bytes"))
            .await
    });
    entered.notified().await;
    set_account(&broker, "bob", 8).await;
    release.notify_one();
    assert!(matches!(query.await.unwrap(), Err(AccountError::Cancelled)));
    server.abort();
}

#[tokio::test]
async fn upload_sends_exact_content_addressed_bytes_and_can_retry_same_blob() {
    let body = b"\0binary\xff".to_vec();
    let expected = sha256(&body);
    let received = Arc::new(std::sync::Mutex::new(Vec::new()));
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let broker = broker(format!("http://{}", listener.local_addr().unwrap()));
    set_account(&broker, "alice", 7).await;
    let expected_route_digest = expected.clone();
    let router = Router::new().route(
        BLOB_ROUTE,
        put({
            let received = received.clone();
            move |Path((organization, project, actual_digest)): Path<(String, String, String)>,
                  headers: HeaderMap,
                  body: Bytes| {
                let received = received.clone();
                let expected_route_digest = expected_route_digest.clone();
                async move {
                    assert_eq!(organization, ORGANIZATION);
                    assert_eq!(project, PROJECT);
                    assert_eq!(actual_digest, expected_route_digest);
                    assert_eq!(
                        headers.get("authorization").unwrap().to_str().unwrap(),
                        "Bearer fixture-access"
                    );
                    assert_eq!(
                        headers.get("content-type").unwrap().to_str().unwrap(),
                        "application/octet-stream"
                    );
                    received.lock().unwrap().push(body.to_vec());
                    StatusCode::NO_CONTENT
                }
            }
        }),
    );
    let server = tokio::spawn(async move { axum::serve(listener, router).await.unwrap() });
    for _ in 0..2 {
        broker
            .upload_cloud_blob("7", ORGANIZATION, PROJECT, &expected, body.clone())
            .await
            .unwrap();
    }
    assert_eq!(*received.lock().unwrap(), vec![body.clone(), body]);
    server.abort();
}

#[tokio::test]
async fn upload_rejects_untrusted_scope_payload_digest_and_size_without_network() {
    let requests = Arc::new(AtomicUsize::new(0));
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let broker = broker(format!("http://{}", listener.local_addr().unwrap()));
    set_account(&broker, "alice", 7).await;
    let router = Router::new().route(
        BLOB_ROUTE,
        put({
            let requests = requests.clone();
            move || {
                requests.fetch_add(1, Ordering::SeqCst);
                async { StatusCode::NO_CONTENT }
            }
        }),
    );
    let server = tokio::spawn(async move { axum::serve(listener, router).await.unwrap() });
    for (organization, project, digest, bytes) in [
        ("../other", PROJECT, sha256(b"valid"), b"valid".to_vec()),
        (
            ORGANIZATION,
            "../other",
            sha256(b"valid"),
            b"valid".to_vec(),
        ),
        (ORGANIZATION, PROJECT, "A".repeat(64), b"valid".to_vec()),
        (ORGANIZATION, PROJECT, sha256(b"valid"), b"forged".to_vec()),
        (
            ORGANIZATION,
            PROJECT,
            sha256(b"valid"),
            vec![0; MAX_BLOB_BYTES + 1],
        ),
    ] {
        assert!(matches!(
            broker
                .upload_cloud_blob("7", organization, project, &digest, bytes)
                .await,
            Err(AccountError::ServiceFailure)
        ));
    }
    assert!(matches!(
        broker
            .upload_cloud_blob(
                "6",
                ORGANIZATION,
                PROJECT,
                &sha256(b"valid"),
                b"valid".to_vec()
            )
            .await,
        Err(AccountError::Cancelled)
    ));
    assert_eq!(requests.load(Ordering::SeqCst), 0);
    server.abort();
}

#[tokio::test]
async fn upload_distinguishes_final_rejections_from_uncertain_server_results() {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let broker = broker(format!("http://{}", listener.local_addr().unwrap()));
    set_account(&broker, "alice", 7).await;
    let forbidden = sha256(b"forbidden");
    let server_error = sha256(b"server-error");
    let unexpected_success = sha256(b"unexpected-success");
    let forbidden_route = forbidden.clone();
    let server_error_route = server_error.clone();
    let router = Router::new().route(
        BLOB_ROUTE,
        put(
            move |Path((_, _, digest)): Path<(String, String, String)>, _: Bytes| {
                let forbidden = forbidden_route.clone();
                let server_error = server_error_route.clone();
                async move {
                    if digest == forbidden {
                        StatusCode::FORBIDDEN
                    } else if digest == server_error {
                        StatusCode::INTERNAL_SERVER_ERROR
                    } else {
                        StatusCode::OK
                    }
                }
            },
        ),
    );
    let server = tokio::spawn(async move { axum::serve(listener, router).await.unwrap() });
    assert!(matches!(
        broker
            .upload_cloud_blob(
                "7",
                ORGANIZATION,
                PROJECT,
                &forbidden,
                b"forbidden".to_vec()
            )
            .await,
        Err(AccountError::ServiceFailure)
    ));
    for (digest, bytes) in [
        (server_error, b"server-error".to_vec()),
        (unexpected_success, b"unexpected-success".to_vec()),
    ] {
        assert!(matches!(
            broker
                .upload_cloud_blob("7", ORGANIZATION, PROJECT, &digest, bytes)
                .await,
            Err(AccountError::OutcomeUnknown)
        ));
    }
    server.abort();
}

#[tokio::test]
async fn upload_timeout_or_late_old_generation_is_not_reported_as_committed() {
    let entered = Arc::new(tokio::sync::Notify::new());
    let release = Arc::new(tokio::sync::Notify::new());
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let broker = broker(format!("http://{}", listener.local_addr().unwrap()));
    set_account(&broker, "alice", 7).await;
    let router = Router::new().route(
        BLOB_ROUTE,
        put({
            let entered = entered.clone();
            let release = release.clone();
            move |_: Bytes| {
                let entered = entered.clone();
                let release = release.clone();
                async move {
                    entered.notify_one();
                    release.notified().await;
                    StatusCode::NO_CONTENT
                }
            }
        }),
    );
    let server = tokio::spawn(async move { axum::serve(listener, router).await.unwrap() });
    let reader = broker.clone();
    let query = tokio::spawn(async move {
        reader
            .upload_cloud_blob(
                "7",
                ORGANIZATION,
                PROJECT,
                &sha256(b"new bytes"),
                b"new bytes".to_vec(),
            )
            .await
    });
    entered.notified().await;
    set_account(&broker, "bob", 8).await;
    release.notify_one();
    assert!(matches!(
        query.await.unwrap(),
        Err(AccountError::OutcomeUnknown)
    ));
    server.abort();
    let _ = server.await;
    let unavailable = broker
        .upload_cloud_blob(
            "8",
            ORGANIZATION,
            PROJECT,
            &sha256(b"retry"),
            b"retry".to_vec(),
        )
        .await;
    assert!(matches!(unavailable, Err(AccountError::OutcomeUnknown)));
}
