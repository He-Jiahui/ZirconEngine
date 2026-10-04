use super::*;
use crate::projects::CloudBindingEnvironment;

#[tokio::test]
async fn switched_recovery_snapshot_never_invokes_local_cleanup() {
    let environment = CloudBindingEnvironment {
        issuer: "https://identity.example".into(),
        client_id: "hub".into(),
        service_url: "https://service.example".into(),
    };
    let scope = CloudAccountScope {
        environment: environment.clone(),
        subject: "alice".into(),
    };
    let alice = AccountView {
        status: "signed-in".into(),
        issuer: Some(environment.issuer.clone()),
        subject: Some("alice".into()),
        generation: "7".into(),
        ..AccountView::default()
    };
    assert!(recovery_snapshot_matches_scope(&alice, "7", &scope));

    let bob = AccountView {
        subject: Some("bob".into()),
        generation: "8".into(),
        ..alice
    };
    let mut cleanup_called = false;
    let result = guarded_recovery_cleanup(
        std::future::ready((bob, Ok((Vec::new(), Vec::new(), "0".into())))),
        "7",
        &scope,
        |_, _| {
            cleanup_called = true;
            std::future::ready(Ok::<_, String>(()))
        },
    )
    .await;
    assert_eq!(result, Err("account_session_expired".into()));
    assert!(!cleanup_called);

    let mut cleanup_called = false;
    let result = guarded_recovery_cleanup(
        std::future::ready((
            AccountView {
                status: "signed-in".into(),
                issuer: Some(environment.issuer.clone()),
                subject: Some("alice".into()),
                generation: "7".into(),
                ..AccountView::default()
            },
            Err(AccountError::OperationStore),
        )),
        "7",
        &scope,
        |_, _| {
            cleanup_called = true;
            std::future::ready(Ok::<_, String>(()))
        },
    )
    .await;
    assert_eq!(result, Err("account_operation_store_unavailable".into()));
    assert!(!cleanup_called);
}
