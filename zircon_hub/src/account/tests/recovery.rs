use super::*;
use crate::account::{
    credential::{CredentialStore, SavedSession},
    AccountConfig, State,
};
use std::{future::Future, sync::Arc};
use tokio::sync::{watch, Mutex};

#[test]
fn cloud_receipt_conflict_is_terminal_without_becoming_a_publish() {
    use crate::account::cloud::manifest::{FileEntry, Manifest};

    let organization = "00000000-0000-4000-8000-000000000001";
    let project = "00000000-0000-4000-8000-000000000002";
    let manifest = Manifest {
        schema_version: 1,
        engine: "Zircon".into(),
        package_lock_digest: "a".repeat(64),
        package_lock: None,
        ignore_policy: "zircon-project-v1".into(),
        source_revision: None,
        files: vec![FileEntry {
            path: "Content/scene.zui".into(),
            digest: "b".repeat(64),
            bytes: 5,
        }],
    };
    let digest = manifest.canonical_digest().unwrap();
    let payload = OperationPayload::CloudCommit {
        organization: organization.into(),
        project: project.into(),
        base_revision: "0".into(),
        manifest,
    };
    let conflict = serde_json::json!({
        "status":"committed",
        "result":{
            "status":"conflict","organizationId":organization,"projectId":project,
            "baseRevision":"0","currentRevision":"1","manifestDigest":digest
        }
    });
    let (status, projected) = project_operation_receipt(&payload, &conflict).unwrap();
    assert_eq!(status, OperationStatus::Conflict);
    assert_eq!(projected["status"], "conflict");
    assert_eq!(projected["result"]["status"], "conflict");

    let mut wrong = conflict;
    wrong["result"]["projectId"] = serde_json::json!(organization);
    assert!(matches!(
        project_operation_receipt(&payload, &wrong),
        Err(AccountError::OutcomeUnknown)
    ));
}

struct UnusedCredentials;
impl CredentialStore for UnusedCredentials {
    fn load(&self) -> Result<Option<SavedSession>, AccountError> {
        unreachable!()
    }
    fn save(&self, _: &SavedSession, _: Option<&SavedSession>) -> Result<(), AccountError> {
        unreachable!()
    }
    fn clear(&self, _: Option<&SavedSession>) -> Result<(), AccountError> {
        unreachable!()
    }
    fn begin_logout(&self) -> Result<Option<SavedSession>, AccountError> {
        unreachable!()
    }
    fn pending_revocation(&self) -> Result<Option<SavedSession>, AccountError> {
        unreachable!()
    }
    fn finish_revocation(&self, _: &SavedSession) -> Result<(), AccountError> {
        unreachable!()
    }
}

#[tokio::test]
async fn retry_excludes_acknowledgement_before_it_can_read_the_account_or_journal() {
    let (cancel, _) = watch::channel(0);
    let broker = AccountBroker {
        config: AccountConfig {
            issuer: "http://127.0.0.1:1/realm".into(),
            client_id: "hub".into(),
            service_url: "http://127.0.0.1:1".into(),
            callback_port: 8480,
            allow_loopback_http: true,
            operation_journal_path: None,
        },
        credentials: Arc::new(UnusedCredentials),
        state: Mutex::new(State {
            generation: 7,
            authenticated: None,
            view: AccountView {
                generation: "7".into(),
                ..Default::default()
            },
        }),
        operation: Mutex::new(()),
        mutation: Mutex::new(()),
        cancel,
    };
    let state = broker.state.lock().await;
    let mut retry = Box::pin(broker.retry_operation("7", "00000000-0000-4000-8000-000000000003"));
    std::future::poll_fn(|context| {
        assert!(retry.as_mut().poll(context).is_pending());
        std::task::Poll::Ready(())
    })
    .await;
    assert!(broker.mutation.try_lock().is_err());
    assert!(matches!(
        broker
            .acknowledge_operation("7", "00000000-0000-4000-8000-000000000003")
            .await,
        Err(AccountError::Busy)
    ));
    drop(retry);
    assert!(broker.mutation.try_lock().is_ok());
    drop(state);
}
