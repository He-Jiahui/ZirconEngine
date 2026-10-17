use super::*;
use std::future::Future;

struct MemoryCredentials(std::sync::Mutex<Option<SavedSession>>);
impl CredentialStore for MemoryCredentials {
    fn load(&self) -> Result<Option<SavedSession>, AccountError> {
        Ok(self
            .0
            .lock()
            .unwrap()
            .clone()
            .filter(|saved| !saved.revoke_only))
    }
    fn save(
        &self,
        session: &SavedSession,
        expected: Option<&SavedSession>,
    ) -> Result<(), AccountError> {
        let mut value = self.0.lock().unwrap();
        if value.as_ref().is_some_and(|saved| saved.revoke_only) {
            return Err(AccountError::RevocationPending);
        }
        if expected.is_some_and(|expected| {
            !value
                .as_ref()
                .is_some_and(|current| current.refresh_token == expected.refresh_token)
        }) {
            return Err(AccountError::Cancelled);
        }
        *value = Some(session.clone());
        Ok(())
    }
    fn clear(&self, expected: Option<&SavedSession>) -> Result<(), AccountError> {
        let mut value = self.0.lock().unwrap();
        if expected.is_some_and(|expected| {
            !value
                .as_ref()
                .is_some_and(|current| current.refresh_token == expected.refresh_token)
        }) {
            return Ok(());
        }
        if !value.as_ref().is_some_and(|saved| saved.revoke_only) {
            value.take();
        }
        Ok(())
    }
    fn begin_logout(&self) -> Result<Option<SavedSession>, AccountError> {
        let mut value = self.0.lock().unwrap();
        if let Some(saved) = value.as_mut() {
            saved.revoke_only = true;
        }
        Ok(value.clone())
    }
    fn pending_revocation(&self) -> Result<Option<SavedSession>, AccountError> {
        Ok(self
            .0
            .lock()
            .unwrap()
            .clone()
            .filter(|saved| saved.revoke_only))
    }
    fn finish_revocation(&self, revoked: &SavedSession) -> Result<(), AccountError> {
        let mut value = self.0.lock().unwrap();
        if value
            .as_ref()
            .is_some_and(|saved| saved.revoke_only && saved.refresh_token == revoked.refresh_token)
        {
            value.take();
        }
        Ok(())
    }
}

pub(super) fn broker(service_url: String) -> Arc<AccountBroker> {
    let (cancel, _) = watch::channel(0);
    Arc::new(AccountBroker {
        config: AccountConfig {
            issuer: "http://127.0.0.1:1/realm".into(),
            client_id: "hub".into(),
            service_url,
            callback_port: 8480,
            allow_loopback_http: true,
            operation_journal_path: None,
        },
        credentials: Arc::new(MemoryCredentials(std::sync::Mutex::new(None))),
        state: Mutex::new(State {
            generation: 0,
            authenticated: None,
            view: AccountView::default(),
        }),
        operation: Mutex::new(()),
        mutation: Mutex::new(()),
        cancel,
    })
}

pub(super) async fn set_account(broker: &AccountBroker, subject: &str, generation: u64) {
    let mut state = broker.state.lock().await;
    state.generation = generation;
    state.view = AccountView {
        configured: true,
        status: "signed-in".into(),
        issuer: Some(broker.config.issuer.clone()),
        subject: Some(subject.into()),
        display_name: Some(subject.into()),
        generation: generation.to_string(),
        error: None,
    };
    state.authenticated = Some(oidc::Authenticated {
        subject: subject.into(),
        name: subject.into(),
        nonce: "fixture".into(),
        tokens: serde_json::from_value(
            serde_json::json!({"access_token":"fixture-access","token_type":"Bearer"}),
        )
        .unwrap(),
    });
}

#[test]
fn a_refresh_completed_after_logout_cannot_restore_the_old_credential() {
    let saved = SavedSession {
        issuer: "issuer".into(),
        subject: "alice".into(),
        nonce: "nonce".into(),
        refresh_token: "old".into(),
        revoke_only: false,
    };
    let credentials = MemoryCredentials(std::sync::Mutex::new(Some(saved.clone())));
    let revoked = credentials.begin_logout().unwrap().unwrap();
    assert!(credentials.save(&saved, Some(&saved)).is_err());
    credentials.finish_revocation(&revoked).unwrap();
    assert!(matches!(
        credentials.save(&saved, Some(&saved)),
        Err(AccountError::Cancelled)
    ));
    assert!(credentials.load().unwrap().is_none());
    let mut next = saved.clone();
    next.refresh_token = "new-account-token".into();
    credentials.save(&next, None).unwrap();
    credentials.clear(Some(&saved)).unwrap();
    assert_eq!(
        credentials.load().unwrap().unwrap().refresh_token,
        "new-account-token"
    );
}

#[tokio::test]
async fn admitted_write_survives_restart_identity_change_rejection_and_receipt_reconciliation() {
    use axum::{
        routing::{get, post},
        Json, Router,
    };
    use operations::{OperationIdentity, OperationJournal, OperationStatus};
    use std::sync::atomic::{AtomicUsize, Ordering};
    const OP: &str = "00000000-0000-4000-8000-000000000003";
    const ORG: &str = "00000000-0000-4000-8000-000000000004";
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let url = format!("http://{}", listener.local_addr().unwrap());
    let directory = std::env::temp_dir().join(format!(
        "hub-operation-recovery-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let path = directory.join("operations.dat");
    let mut original = broker(url.clone());
    Arc::get_mut(&mut original)
        .unwrap()
        .config
        .operation_journal_path = Some(path.clone());
    set_account(&original, "alice", 7).await;
    let identity = OperationIdentity::new(&original.config, "alice");
    let attempts = Arc::new(AtomicUsize::new(0));
    let router = Router::new().route("/v1/organizations", post({
        let attempts = attempts.clone(); let path = path.clone(); let identity = identity.clone();
        move |Json(body): Json<serde_json::Value>| {
            let attempts = attempts.clone(); let path = path.clone(); let identity = identity.clone();
            async move {
                assert_eq!(body["operationId"], OP); assert_eq!(body["name"], "Team");
                // Dispatch is only allowed after the exact operation is visible on disk.
                assert_eq!(OperationJournal::new(path).get(&identity, OP).unwrap().operation_id, OP);
                if attempts.fetch_add(1, Ordering::SeqCst) == 0 {
                    (axum::http::StatusCode::OK, "x".repeat(262145))
                } else {
                    (axum::http::StatusCode::FORBIDDEN, "{\"error\":\"forbidden\"}".into())
                }
            }
        }
    })).route("/v1/operations/{operation}", get(|| async { Json(serde_json::json!({"status":"committed","result":{"id":ORG,"name":"Team","policyRevision":"1"}})) }));
    let server = tokio::spawn(async move { axum::serve(listener, router).await.unwrap() });
    assert!(matches!(
        original
            .service_request(
                "7",
                ServiceRequest::CreateOrganization {
                    operation_id: OP.into(),
                    name: "Team".into()
                }
            )
            .await,
        Err(AccountError::OutcomeUnknown)
    ));
    original.logout().await.unwrap();
    assert!(original.recovery_snapshot().await.1.unwrap().0.is_empty());
    drop(original);
    let mut restarted = broker(url);
    Arc::get_mut(&mut restarted)
        .unwrap()
        .config
        .operation_journal_path = Some(path.clone());
    set_account(&restarted, "bob", 9).await;
    assert!(restarted.recovery_snapshot().await.1.unwrap().0.is_empty());
    assert!(restarted.retry_operation("9", OP).await.is_err());
    assert_eq!(attempts.load(Ordering::SeqCst), 1);
    set_account(&restarted, "alice", 11).await;
    assert_eq!(
        restarted.recovery_snapshot().await.1.unwrap().0[0].status,
        OperationStatus::Unknown
    );
    assert!(matches!(
        restarted.retry_operation("11", OP).await,
        Err(AccountError::OutcomeUnknown)
    ));
    assert!(matches!(
        restarted.acknowledge_operation("11", OP).await,
        Err(AccountError::OutcomeUnknown)
    ));
    let receipt = restarted.reconcile_operation("11", OP).await.unwrap();
    assert_eq!(receipt["status"], "committed");
    assert_eq!(
        restarted.recovery_snapshot().await.1.unwrap().0[0].status,
        OperationStatus::Committed
    );
    restarted.acknowledge_operation("11", OP).await.unwrap();
    assert!(restarted.recovery_snapshot().await.1.unwrap().0.is_empty());
    server.abort();
    let _ = server.await;
    std::fs::remove_dir_all(directory).unwrap();
}

#[tokio::test]
async fn stale_generation_and_broken_journal_never_dispatch_or_acknowledge() {
    use axum::{routing::post, Router};
    use std::sync::atomic::{AtomicUsize, Ordering};
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let requests = Arc::new(AtomicUsize::new(0));
    let router = Router::new().route(
        "/v1/organizations",
        post({
            let requests = requests.clone();
            move || {
                let requests = requests.clone();
                async move {
                    requests.fetch_add(1, Ordering::SeqCst);
                    "{}"
                }
            }
        }),
    );
    let directory = std::env::temp_dir().join(format!(
        "hub-broken-journal-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir_all(&directory).unwrap();
    let path = directory.join("operations.dat");
    std::fs::write(&path, b"unreadable journal").unwrap();
    let mut broker = broker(format!("http://{}", listener.local_addr().unwrap()));
    Arc::get_mut(&mut broker)
        .unwrap()
        .config
        .operation_journal_path = Some(path.clone());
    set_account(&broker, "alice", 7).await;
    let server = tokio::spawn(async move { axum::serve(listener, router).await.unwrap() });
    let request = ServiceRequest::CreateOrganization {
        operation_id: "00000000-0000-4000-8000-000000000003".into(),
        name: "Team".into(),
    };
    assert!(matches!(
        broker.service_request("6", request.clone()).await,
        Err(AccountError::Cancelled)
    ));
    assert!(matches!(
        broker.service_request("7", request).await,
        Err(AccountError::OperationStore)
    ));
    assert!(matches!(
        broker
            .acknowledge_operation("6", "00000000-0000-4000-8000-000000000003")
            .await,
        Err(AccountError::Cancelled)
    ));
    assert_eq!(requests.load(Ordering::SeqCst), 0);
    assert!(broker.recovery_snapshot().await.1.is_err());
    assert_eq!(std::fs::read(path).unwrap(), b"unreadable journal");
    server.abort();
    let _ = server.await;
    std::fs::remove_dir_all(directory).unwrap();
}

#[tokio::test]
async fn revoked_only_credential_survives_failure_and_retries_after_broker_restart() {
    use axum::{routing::post, Router};
    use std::sync::atomic::{AtomicUsize, Ordering};
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let issuer = format!("http://{}/realm", listener.local_addr().unwrap());
    let credentials = Arc::new(MemoryCredentials(std::sync::Mutex::new(Some(
        SavedSession {
            issuer: issuer.clone(),
            subject: "alice".into(),
            nonce: "fixture".into(),
            refresh_token: "refresh-fixture".into(),
            revoke_only: false,
        },
    ))));
    let attempts = Arc::new(AtomicUsize::new(0));
    let router = Router::new().route(
        "/realm/protocol/openid-connect/logout",
        post({
            let attempts = attempts.clone();
            move || {
                let attempts = attempts.clone();
                async move {
                    if attempts.fetch_add(1, Ordering::SeqCst) == 0 {
                        axum::http::StatusCode::SERVICE_UNAVAILABLE
                    } else {
                        axum::http::StatusCode::NO_CONTENT
                    }
                }
            }
        }),
    );
    let server = tokio::spawn(async move { axum::serve(listener, router).await.unwrap() });
    let mut first = broker("http://127.0.0.1:1".into());
    let first_mut = Arc::get_mut(&mut first).unwrap();
    first_mut.config.issuer = issuer.clone();
    first_mut.credentials = credentials.clone();
    assert!(matches!(
        first.logout().await,
        Err(AccountError::RevocationPending)
    ));
    assert!(credentials.load().unwrap().is_none());
    assert!(credentials.pending_revocation().unwrap().is_some());
    drop(first);
    let mut restarted = broker("http://127.0.0.1:1".into());
    let restarted_mut = Arc::get_mut(&mut restarted).unwrap();
    restarted_mut.config.issuer = issuer;
    restarted_mut.credentials = credentials.clone();
    assert!(matches!(
        restarted.authenticate(true).await,
        Err(AccountError::SessionExpired)
    ));
    assert_eq!(attempts.load(Ordering::SeqCst), 2);
    assert!(credentials.pending_revocation().unwrap().is_none());
    assert!(credentials.load().unwrap().is_none());
    let dto = serde_json::to_string(&restarted.view().await).unwrap();
    assert!(!dto.contains("refresh-fixture"));
    server.abort();
    let _ = server.await;
}

#[tokio::test]
async fn logout_cancels_login_waiting_for_the_initial_state_lock() {
    let broker = broker("http://127.0.0.1:1".into());
    let state = broker.state.lock().await;
    let authentication = broker.authenticate(false);
    tokio::pin!(authentication);
    // Polling the real login future once advances it to the held state lock.
    std::future::poll_fn(|cx| {
        assert!(authentication.as_mut().poll(cx).is_pending());
        std::task::Poll::Ready(())
    })
    .await;
    let cancellation_generation = *broker.cancel.borrow();
    let logout = broker.logout();
    tokio::pin!(logout);
    std::future::poll_fn(|cx| {
        assert!(logout.as_mut().poll(cx).is_pending());
        std::task::Poll::Ready(())
    })
    .await;
    // Logout must wait for the state lock that orders package commit authorization.
    assert_eq!(*broker.cancel.borrow(), cancellation_generation);
    drop(state);
    // Both futures must keep running so logout can publish cancellation after acquiring state.
    let (authentication, logout) = tokio::time::timeout(std::time::Duration::from_secs(1), async {
        tokio::join!(authentication, logout)
    })
    .await
    .unwrap();
    assert!(matches!(authentication, Err(AccountError::Cancelled)));
    let view = logout.unwrap();
    assert_eq!(view.status, "signed-out");
    assert!(view.subject.is_none());
    assert!(broker.credentials.load().unwrap().is_none());
    assert_eq!(*broker.cancel.borrow(), cancellation_generation + 1);
}

#[tokio::test]
async fn logout_is_not_blocked_by_a_slow_service_response_and_late_data_is_rejected() {
    use axum::{routing::get, Json, Router};
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let broker = broker(format!("http://{}", listener.local_addr().unwrap()));
    let entered = Arc::new(tokio::sync::Notify::new());
    let release = Arc::new(tokio::sync::Notify::new());
    let router = Router::new().route(
        "/v1/organizations",
        get({
            let entered = entered.clone();
            let release = release.clone();
            move || {
                let entered = entered.clone();
                let release = release.clone();
                async move {
                    entered.notify_one();
                    release.notified().await;
                    Json(serde_json::json!({"items":[{"id":"old-account"}],"nextCursor":null}))
                }
            }
        }),
    );
    let server = tokio::spawn(async move { axum::serve(listener, router).await.unwrap() });
    broker.state.lock().await.authenticated = Some(oidc::Authenticated {
        subject: "alice".into(),
        name: "Alice".into(),
        nonce: "fixture".into(),
        tokens: serde_json::from_value(
            serde_json::json!({"access_token":"fixture-access","token_type":"Bearer"}),
        )
        .unwrap(),
    });
    let task_broker = broker.clone();
    let query = tokio::spawn(async move { task_broker.organizations().await });
    entered.notified().await;
    assert!(
        tokio::time::timeout(std::time::Duration::from_secs(1), broker.logout())
            .await
            .unwrap()
            .is_ok()
    );
    release.notify_one();
    assert!(matches!(query.await.unwrap(), Err(AccountError::Cancelled)));
    server.abort();
}

#[tokio::test]
async fn logout_cancels_pending_operation_and_clears_credentials_even_if_provider_is_down() {
    let credentials = Arc::new(MemoryCredentials(std::sync::Mutex::new(Some(
        SavedSession {
            issuer: "http://127.0.0.1:1/realm".into(),
            subject: "alice".into(),
            nonce: "fixture-nonce".into(),
            refresh_token: "fixture-refresh".into(),
            revoke_only: false,
        },
    ))));
    let (cancel, _) = watch::channel(0);
    let broker = Arc::new(AccountBroker {
        config: AccountConfig {
            issuer: "http://127.0.0.1:1/realm".into(),
            client_id: "hub".into(),
            service_url: "http://127.0.0.1:1".into(),
            callback_port: 8480,
            allow_loopback_http: true,
            operation_journal_path: None,
        },
        credentials: credentials.clone(),
        state: Mutex::new(State {
            generation: 0,
            authenticated: None,
            view: AccountView {
                configured: true,
                status: "signed-in".into(),
                subject: Some("alice".into()),
                ..Default::default()
            },
        }),
        operation: Mutex::new(()),
        mutation: Mutex::new(()),
        cancel,
    });
    let guard = broker.operation.lock().await;
    let mut cancellation = broker.cancel.subscribe();
    let task_broker = broker.clone();
    let task = tokio::spawn(async move { task_broker.logout().await });
    tokio::time::timeout(std::time::Duration::from_secs(1), cancellation.changed())
        .await
        .unwrap()
        .unwrap();
    drop(guard);
    assert!(task.await.unwrap().is_err());
    assert!(credentials.load().unwrap().is_none());
    assert!(credentials.pending_revocation().unwrap().is_some());
    assert!(matches!(
        broker.logout().await,
        Err(AccountError::RevocationPending)
    ));
    assert!(credentials.pending_revocation().unwrap().is_some());
    let view = broker.view().await;
    assert_eq!(view.status, "signed-out");
    assert!(view.subject.is_none());
    let json = serde_json::to_string(&view).unwrap();
    assert!(!json.contains("fixture-refresh"));
    assert!(!json.contains("fixture-nonce"));
}
