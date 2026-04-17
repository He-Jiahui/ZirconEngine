use crossbeam_channel::unbounded;
use zircon_manager::{
    ResourceChangeRecord, ResourceManager, ResourceStateRecord, ResourceStatusRecord,
};
use zircon_resource::{
    MaterialMarker, ModelMarker, ResourceHandle, ResourceId, ResourceKind, ResourceLocator,
};

#[test]
fn resolve_ready_handle_returns_typed_handle_from_resource_server() {
    let locator = ResourceLocator::parse("res://models/triangle.obj").unwrap();
    let expected_id = ResourceId::from_locator(&locator);
    let server = FakeResourceServer::new(vec![status(
        &locator.to_string(),
        ResourceKind::Model,
        ResourceStateRecord::Ready,
    )]);

    let handle =
        crate::host::resource_access::resolve_ready_handle::<ModelMarker>(&server, &locator)
            .expect("ready model handle");

    assert_eq!(handle, ResourceHandle::<ModelMarker>::new(expected_id));
}

#[test]
fn resolve_ready_handle_surfaces_non_ready_state_and_diagnostics() {
    let locator = ResourceLocator::parse("res://materials/default.material.toml").unwrap();
    let server = FakeResourceServer::new(vec![ResourceStatusRecord {
        id: ResourceId::from_locator(&locator).to_string(),
        locator: locator.to_string(),
        kind: ResourceKind::Material,
        artifact_locator: Some("lib://materials/default.material.bin".to_string()),
        revision: 2,
        state: ResourceStateRecord::Error,
        dependency_ids: Vec::new(),
        diagnostics: vec!["shader compile failed".to_string()],
    }]);

    let error =
        crate::host::resource_access::resolve_ready_handle::<MaterialMarker>(&server, &locator)
            .expect_err("error state should be rejected");

    assert!(error.contains("res://materials/default.material.toml"));
    assert!(error.contains("Error"));
    assert!(error.contains("shader compile failed"));
}

#[derive(Clone, Debug)]
struct FakeResourceServer {
    records: Vec<ResourceStatusRecord>,
}

impl FakeResourceServer {
    fn new(records: Vec<ResourceStatusRecord>) -> Self {
        Self { records }
    }
}

impl ResourceManager for FakeResourceServer {
    fn resolve_resource_id(&self, locator: &str) -> Option<String> {
        self.records
            .iter()
            .find(|record| record.locator == locator)
            .map(|record| record.id.clone())
    }

    fn resource_status(&self, locator: &str) -> Option<ResourceStatusRecord> {
        self.records
            .iter()
            .find(|record| record.locator == locator)
            .cloned()
    }

    fn list_resources(&self) -> Vec<ResourceStatusRecord> {
        self.records.clone()
    }

    fn resource_revision(&self, locator: &str) -> Option<u64> {
        self.records
            .iter()
            .find(|record| record.locator == locator)
            .map(|record| record.revision)
    }

    fn subscribe_resource_changes(&self) -> zircon_core::ChannelReceiver<ResourceChangeRecord> {
        let (_sender, receiver) = unbounded();
        receiver
    }
}

fn status(locator: &str, kind: ResourceKind, state: ResourceStateRecord) -> ResourceStatusRecord {
    let locator = ResourceLocator::parse(locator).unwrap();
    ResourceStatusRecord {
        id: ResourceId::from_locator(&locator).to_string(),
        locator: locator.to_string(),
        kind,
        artifact_locator: None,
        revision: 1,
        state,
        dependency_ids: Vec::new(),
        diagnostics: Vec::new(),
    }
}
