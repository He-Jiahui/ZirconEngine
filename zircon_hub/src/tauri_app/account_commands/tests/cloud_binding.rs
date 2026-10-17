use super::*;
use serde_json::json;

#[test]
fn binding_actions_require_native_selection_guard_and_do_not_expose_commit() {
    let attach: AccountActionRequest = serde_json::from_value(json!({
        "action": "attach-cloud-project", "backendEpoch": "hub-current", "generation": "7",
        "organization": "00000000-0000-4000-8000-000000000001",
        "project": "00000000-0000-4000-8000-000000000002",
        "selectedProjectId": "E:/Projects/Local"
    }))
    .unwrap();
    assert_eq!(attach.backend_epoch(), "hub-current");
    assert!(!attach.service_response_context().unwrap().mutation);
    assert!(serde_json::from_value::<AccountActionRequest>(json!({
        "action": "attach-cloud-project", "backendEpoch": "hub-current", "generation": "7",
        "organization": "00000000-0000-4000-8000-000000000001",
        "project": "00000000-0000-4000-8000-000000000002"
    }))
    .is_err());
    assert!(serde_json::from_value::<AccountActionRequest>(json!({
        "action": "cloud-commit", "backendEpoch": "hub-current", "generation": "7"
    }))
    .is_err());
}

#[test]
fn binding_scope_requires_same_signed_in_issuer_subject_and_generation() {
    let environment = CloudBindingEnvironment {
        issuer: "https://identity.example".into(),
        client_id: "hub".into(),
        service_url: "https://service.example".into(),
    };
    let mut view = AccountView {
        configured: true,
        status: "signed-in".into(),
        issuer: Some(environment.issuer.clone()),
        subject: Some("alice".into()),
        generation: "7".into(),
        ..AccountView::default()
    };
    assert!(binding_account_scope(&view, "7", Some(&environment)).is_some());
    assert!(binding_account_scope(&view, "6", Some(&environment)).is_none());
    view.issuer = Some("https://other.example".into());
    assert!(binding_account_scope(&view, "7", Some(&environment)).is_none());
    view.issuer = Some(environment.issuer.clone());
    view.status = "signed-out".into();
    assert!(binding_account_scope(&view, "7", Some(&environment)).is_none());
}
