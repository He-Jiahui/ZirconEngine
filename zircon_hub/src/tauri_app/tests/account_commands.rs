use super::*;
use serde_json::json;

#[test]
fn cloud_head_desktop_action_is_read_only_and_generation_scoped() {
    let request: AccountActionRequest = serde_json::from_value(json!({
        "action": "cloud-head", "backendEpoch": "hub-current", "generation": "7",
        "organization": "00000000-0000-4000-8000-000000000001",
        "project": "00000000-0000-4000-8000-000000000002"
    }))
    .unwrap();
    assert_eq!(request.backend_epoch(), "hub-current");
    let context = request.service_response_context().unwrap();
    assert_eq!(context.generation, "7");
    assert!(!context.mutation);
    assert!(serde_json::from_value::<AccountActionRequest>(json!({
        "action": "cloud-head", "backendEpoch": "hub-current", "generation": "7",
        "organization": "00000000-0000-4000-8000-000000000001",
        "project": "00000000-0000-4000-8000-000000000002", "operationId": "unexpected"
    }))
    .is_err());
}

#[test]
fn cloud_sync_actions_accept_only_native_scope_revision_and_operation_identity() {
    let stage: AccountActionRequest = serde_json::from_value(json!({
        "action": "cloud-stage-download", "backendEpoch": "hub-current", "generation": "7",
        "expectedRevision": "12"
    }))
    .unwrap();
    let context = stage.service_response_context().unwrap();
    assert_eq!(context.generation, "7");
    assert!(!context.mutation);

    let push: AccountActionRequest = serde_json::from_value(json!({
        "action": "cloud-push", "backendEpoch": "hub-current", "generation": "7",
        "operationId": "00000000-0000-4000-8000-000000000003", "baseRevision": "12"
    }))
    .unwrap();
    assert!(push.service_response_context().unwrap().mutation);
    for extra in [
        ("projectRoot", json!("E:/Projects/Game")),
        ("path", json!("Content/scene.zui")),
        ("manifest", json!({"files": []})),
        ("files", json!([])),
        ("stageId", json!("00000000-0000-4000-8000-000000000004")),
    ] {
        let mut request = json!({
            "action": "cloud-stage-download", "backendEpoch": "hub-current", "generation": "7",
            "expectedRevision": "12"
        });
        request[extra.0] = extra.1;
        assert!(serde_json::from_value::<AccountActionRequest>(request).is_err());
    }
}

#[test]
fn catalog_install_bridge_uses_the_existing_epoch_and_generation_contract() {
    for (request, mutation) in [
        (
            json!({"action":"catalog","backendEpoch":"current","generation":"7","query":"plugin"}),
            false,
        ),
        (
            json!({"action":"catalog-entitlements","backendEpoch":"current","generation":"7","organization":"org"}),
            false,
        ),
        (
            json!({"action":"package-inventory","schemaVersion":2,"targetMode":"editor_host","backendEpoch":"current","generation":"7","organization":"org"}),
            false,
        ),
        (
            json!({"action":"catalog-license","backendEpoch":"current","generation":"7","organization":"org","operationId":"op","expectedPolicyRevision":"2","packageId":"pkg","revision":"1","licenseId":"license"}),
            true,
        ),
        (
            json!({"action":"package-install","schemaVersion":2,"targetMode":"editor_host","backendEpoch":"current","generation":"7","organization":"org","operationId":"op","packageId":"pkg","revision":"1","expectedInventoryRevision":"0"}),
            true,
        ),
    ] {
        let parsed: AccountActionRequest = serde_json::from_value(request.clone()).unwrap();
        assert_eq!(parsed.backend_epoch(), "current");
        let context = parsed.service_response_context().unwrap();
        assert_eq!(context.generation, "7");
        assert_eq!(context.mutation, mutation);
        let mut unsafe_request = request;
        unsafe_request["artifactUrl"] = json!("https://untrusted");
        assert!(serde_json::from_value::<AccountActionRequest>(unsafe_request).is_err());
    }

    for action in ["package-inventory", "package-install"] {
        let mut valid = json!({
            "action": action,
            "schemaVersion": 2,
            "targetMode": "editor_host",
            "backendEpoch": "current",
            "generation": "7",
            "organization": "org",
            "operationId": "op",
            "packageId": "pkg",
            "revision": "1",
            "expectedInventoryRevision": "0"
        });
        if action == "package-inventory" {
            for key in [
                "operationId",
                "packageId",
                "revision",
                "expectedInventoryRevision",
            ] {
                valid.as_object_mut().unwrap().remove(key);
            }
        }
        for target_mode in ["editor_host", "client_runtime"] {
            let mut supported = valid.clone();
            supported["targetMode"] = json!(target_mode);
            assert!(serde_json::from_value::<AccountActionRequest>(supported).is_ok());
        }
        for schema_version in [None, Some(1), Some(3)] {
            let mut unsupported = valid.clone();
            match schema_version {
                Some(version) => unsupported["schemaVersion"] = json!(version),
                None => {
                    unsupported.as_object_mut().unwrap().remove("schemaVersion");
                }
            }
            assert!(serde_json::from_value::<AccountActionRequest>(unsupported).is_err());
        }
        for target_mode in [None, Some("server_runtime")] {
            let mut unsupported = valid.clone();
            match target_mode {
                Some(mode) => unsupported["targetMode"] = json!(mode),
                None => {
                    unsupported.as_object_mut().unwrap().remove("targetMode");
                }
            }
            assert!(serde_json::from_value::<AccountActionRequest>(unsupported).is_err());
        }
    }
}

fn signed_in(generation: &str) -> AccountView {
    AccountView {
        configured: true,
        status: "signed-in".into(),
        issuer: Some("https://identity.example/realm".into()),
        subject: Some("current-user".into()),
        display_name: Some("Current User".into()),
        generation: generation.into(),
        error: None,
    }
}

#[test]
fn recovery_snapshot_read_failure_is_explicit_and_payload_free() {
    let mut snapshot = finish_account_action("hub-current".into(), signed_in("7"), Ok(None), None);
    apply_recovery(&mut snapshot, Err(AccountError::OperationStore));
    assert_eq!(
        snapshot.operations_error.as_deref(),
        Some("account_operation_store_unavailable")
    );
    assert!(snapshot.operations.is_empty());
    apply_recovery(
        &mut snapshot,
        Ok((Vec::new(), "18446744073709551615".into())),
    );
    assert_eq!(snapshot.operations_revision, "18446744073709551615");
    for action in ["reconcile", "retry", "acknowledge"] {
        let request: AccountActionRequest = serde_json::from_value(json!({"action":action,"backendEpoch":"hub-current","generation":"7","operationId":"00000000-0000-4000-8000-000000000003"})).unwrap();
        assert_eq!(request.service_response_context().unwrap().generation, "7");
        assert!(serde_json::from_value::<AccountActionRequest>(json!({"action":action,"backendEpoch":"hub-current","generation":"7","operationId":"00000000-0000-4000-8000-000000000003","mutation":{"action":"create-project","name":"Changed"}})).is_err());
    }
}

#[test]
fn account_commands_accept_the_camel_case_desktop_contract() {
    for action in ["sign-in", "refresh", "logout", "cancel"] {
        let request: AccountActionRequest = serde_json::from_value(json!({
            "action": action,
            "backendEpoch": "hub-current"
        }))
        .unwrap();
        assert_eq!(request.backend_epoch(), "hub-current");
        assert!(request.service_response_context().is_none());
    }
    let request: AccountActionRequest = serde_json::from_value(json!({
        "action": "mutate",
        "backendEpoch": "hub-current",
        "generation": "19",
        "organization": "org-1",
        "operationId": "op-1",
        "expectedPolicyRevision": "27",
        "mutation": {"kind": "create-project", "name": "Project"}
    }))
    .unwrap();
    let context = request.service_response_context().unwrap();
    assert_eq!(context.generation, "19");
    assert!(context.mutation);
    assert!(matches!(
        request,
        AccountActionRequest::Mutate {
            operation_id,
            expected_policy_revision,
            ..
        } if operation_id == "op-1" && expected_policy_revision == "27"
    ));
    let request: AccountActionRequest = serde_json::from_value(json!({
        "action": "issued-invitations",
        "backendEpoch": "hub-current",
        "generation": "19",
        "organization": "00000000-0000-4000-8000-000000000003",
        "after": "00000000-0000-4000-8000-000000000004"
    }))
    .unwrap();
    let context = request.service_response_context().unwrap();
    assert_eq!(context.generation, "19");
    assert!(!context.mutation);
    assert!(matches!(
        request,
        AccountActionRequest::IssuedInvitations { organization, after, .. }
            if organization == "00000000-0000-4000-8000-000000000003"
                && after.as_deref() == Some("00000000-0000-4000-8000-000000000004")
    ));
}

#[test]
fn account_commands_reject_unknown_actions_fields_and_snake_case_fields() {
    for value in [
        json!({"action": "request-url", "backendEpoch": "hub-current"}),
        json!({"action": "sign-in", "backendEpoch": "hub-current", "url": "https://other.example"}),
        json!({"action": "sign-in", "backend_epoch": "hub-current"}),
        json!({"action": "organizations", "backendEpoch": "hub-current", "generation": 19}),
        json!({"action": "receipt", "backendEpoch": "hub-current", "generation": "19", "operation_id": "op-1"}),
    ] {
        assert!(serde_json::from_value::<AccountActionRequest>(value).is_err());
    }
}

#[tokio::test]
async fn unconfigured_commands_preserve_the_unavailable_view_and_configuration_error() {
    for configuration_error in [None, Some("account_configuration_invalid")] {
        let state = AccountCommandState {
            broker: None,
            environment: None,
            unavailable: AccountView {
                error: configuration_error.map(str::to_owned),
                ..AccountView::default()
            },
        };
        let snapshot = execute_account_action(
            AccountActionRequest::SignIn {
                backend_epoch: "hub-current".into(),
            },
            "hub-current".into(),
            state.broker.clone(),
            state.unavailable.clone(),
        )
        .await
        .unwrap();
        assert!(!snapshot.account.configured);
        assert_eq!(snapshot.account.status, "unavailable");
        assert_eq!(snapshot.account.error.as_deref(), configuration_error);
        assert_eq!(
            snapshot.error.as_deref(),
            Some(configuration_error.unwrap_or("account_not_configured"))
        );
        assert!(snapshot.data.is_none());
    }
}

#[tokio::test]
async fn stale_epoch_is_rejected_before_account_action_dispatch() {
    let state = AccountCommandState {
        broker: None,
        environment: None,
        unavailable: AccountView::default(),
    };
    for action in ["sign-in", "refresh", "logout", "cancel"] {
        let request = serde_json::from_value(json!({
            "action": action,
            "backendEpoch": "hub-retired"
        }))
        .unwrap();
        assert!(matches!(
            execute_account_action(
                request,
                "hub-current".into(),
                state.broker.clone(),
                state.unavailable.clone(),
            )
            .await,
            Err(AccountCommandError::StaleEpoch)
        ));
    }
    assert_eq!(state.unavailable.generation, "0");
}

#[test]
fn authentication_and_revocation_errors_keep_the_cleared_account_snapshot() {
    for error in [
        AccountError::ProviderUnavailable,
        AccountError::CredentialStore,
        AccountError::InvalidIdentity,
        AccountError::SessionExpired,
        AccountError::Cancelled,
    ] {
        let expected_error = error.to_string();
        let cleared_view = AccountView {
            configured: true,
            status: "signed-out".into(),
            generation: "21".into(),
            ..AccountView::default()
        };
        let snapshot = finish_account_action("hub-current".into(), cleared_view, Err(error), None);
        let serialized = serde_json::to_value(snapshot).unwrap();
        assert_eq!(serialized["account"]["status"], "signed-out");
        assert_eq!(serialized["account"]["generation"], "21");
        assert!(serialized["account"]["subject"].is_null());
        assert!(serialized["account"]["error"].is_null());
        assert_eq!(serialized["error"], expected_error);
        assert!(serialized["data"].is_null());
    }
}

#[test]
fn service_response_data_cannot_be_published_with_a_newer_account_view() {
    for mutation in [false, true] {
        let snapshot = finish_account_action(
            "hub-current".into(),
            signed_in("21"),
            Ok(Some(json!({"items": [{"id": "old-account-record"}]}))),
            Some(&ServiceResponseContext {
                generation: "19".into(),
                mutation,
            }),
        );
        assert_eq!(snapshot.account.generation, "21");
        assert!(snapshot.data.is_none());
        assert_eq!(
            snapshot.error.as_deref(),
            Some(if mutation {
                "account_service_outcome_unknown"
            } else {
                "account_login_cancelled"
            })
        );
    }
}

#[test]
fn service_response_requires_a_matching_signed_in_generation() {
    let context = ServiceResponseContext {
        generation: "19".into(),
        mutation: false,
    };
    let expected_data = json!({"items": [{"id": "current-account-record"}]});
    let accepted = finish_account_action(
        "hub-current".into(),
        signed_in("19"),
        Ok(Some(expected_data.clone())),
        Some(&context),
    );
    assert_eq!(accepted.data, Some(expected_data.clone()));
    assert!(accepted.error.is_none());

    let mut signed_out = signed_in("19");
    signed_out.status = "signed-out".into();
    let rejected = finish_account_action(
        "hub-current".into(),
        signed_out,
        Ok(Some(expected_data)),
        Some(&context),
    );
    assert!(rejected.data.is_none());
    assert_eq!(rejected.error.as_deref(), Some("account_login_cancelled"));
}
