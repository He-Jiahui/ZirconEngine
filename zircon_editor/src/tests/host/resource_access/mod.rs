use crossbeam_channel::unbounded;
use zircon_runtime::core::framework::asset::ResourceManager;
use zircon_runtime_interface::resource::{
    MaterialMarker, ModelMarker, ResourceDiagnostic, ResourceEvent, ResourceHandle, ResourceId,
    ResourceKind, ResourceLocator, ResourceRecord, ResourceState,
};

#[test]
fn resolve_ready_handle_returns_typed_handle_from_resource_server() {
    let locator = ResourceLocator::parse("res://models/triangle.obj").unwrap();
    let expected_id = ResourceId::from_locator(&locator);
    let server = FakeResourceServer::new(vec![status(
        &locator.to_string(),
        ResourceKind::Model,
        ResourceState::Ready,
    )]);

    let handle =
        crate::ui::host::resource_access::resolve_ready_handle::<ModelMarker>(&server, &locator)
            .expect("ready model handle");

    assert_eq!(handle, ResourceHandle::<ModelMarker>::new(expected_id));
}

#[test]
fn resolve_ready_handle_surfaces_non_ready_state_and_diagnostics() {
    let locator = ResourceLocator::parse("res://materials/default.material.toml").unwrap();
    let server = FakeResourceServer::new(vec![ResourceRecord {
        id: ResourceId::from_locator(&locator),
        kind: ResourceKind::Material,
        primary_locator: locator.clone(),
        artifact_locator: Some(
            ResourceLocator::parse("lib://materials/default.material.bin").unwrap(),
        ),
        revision: 2,
        state: ResourceState::Error,
        dependency_ids: Vec::new(),
        diagnostics: vec![ResourceDiagnostic::error("shader compile failed")],
        source_hash: String::new(),
        importer_id: String::new(),
        importer_version: 0,
        config_hash: String::new(),
    }]);

    let error =
        crate::ui::host::resource_access::resolve_ready_handle::<MaterialMarker>(&server, &locator)
            .expect_err("error state should be rejected");

    assert!(error.contains("res://materials/default.material.toml"));
    assert!(error.contains("Error"));
    assert!(error.contains("shader compile failed"));
}

#[derive(Clone, Debug)]
struct FakeResourceServer {
    records: Vec<ResourceRecord>,
}

impl FakeResourceServer {
    fn new(records: Vec<ResourceRecord>) -> Self {
        Self { records }
    }
}

impl ResourceManager for FakeResourceServer {
    fn resolve_resource_id(&self, locator: &str) -> Option<String> {
        self.records
            .iter()
            .find(|record| record.primary_locator.to_string() == locator)
            .map(|record| record.id.to_string())
    }

    fn resource_status(&self, locator: &str) -> Option<ResourceRecord> {
        self.records
            .iter()
            .find(|record| record.primary_locator.to_string() == locator)
            .cloned()
    }

    fn list_resources(&self) -> Vec<ResourceRecord> {
        self.records.clone()
    }

    fn resource_revision(&self, locator: &str) -> Option<u64> {
        self.records
            .iter()
            .find(|record| record.primary_locator.to_string() == locator)
            .map(|record| record.revision)
    }

    fn subscribe_resource_changes(&self) -> zircon_runtime::core::ChannelReceiver<ResourceEvent> {
        let (_sender, receiver) = unbounded();
        receiver
    }
}

fn status(locator: &str, kind: ResourceKind, state: ResourceState) -> ResourceRecord {
    let locator = ResourceLocator::parse(locator).unwrap();
    ResourceRecord {
        id: ResourceId::from_locator(&locator),
        kind,
        primary_locator: locator,
        artifact_locator: None,
        revision: 1,
        state,
        dependency_ids: Vec::new(),
        diagnostics: Vec::new(),
        source_hash: String::new(),
        importer_id: String::new(),
        importer_version: 0,
        config_hash: String::new(),
    }
}
