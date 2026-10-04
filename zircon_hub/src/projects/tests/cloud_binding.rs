use super::*;

#[test]
fn binding_preserves_distinct_local_and_remote_identity_and_rejects_invalid_ids() {
    let local = ProjectGuid::new();
    let scope = CloudAccountScope {
        environment: CloudBindingEnvironment {
            issuer: "https://identity.example".into(),
            client_id: "hub".into(),
            service_url: "https://service.example".into(),
        },
        subject: "user-1".into(),
    };
    let remote = "00000000-0000-4000-8000-000000000002".to_owned();
    let binding = CloudProjectBinding::new(
        scope.clone(),
        "e:/projects/local".into(),
        local,
        "00000000-0000-4000-8000-000000000001".into(),
        remote.clone(),
    )
    .unwrap();
    assert_eq!(binding.local_project_guid, local);
    assert_eq!(binding.project_id, remote);
    assert_eq!(binding.account, scope);
    assert_eq!(
        serde_json::from_value::<CloudProjectBinding>(serde_json::to_value(&binding).unwrap())
            .unwrap(),
        binding
    );
    assert!(CloudProjectBinding::new(
        scope,
        "e:/projects/local".into(),
        local,
        "NOT-A-UUID".into(),
        remote,
    )
    .is_none());
}
