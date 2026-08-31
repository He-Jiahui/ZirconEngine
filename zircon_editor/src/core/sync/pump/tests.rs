use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::mpsc;
use std::sync::{Arc, Barrier};
use std::time::{Duration, Instant};

use zircon_runtime::scene::{DefaultLevelManager, NodeKind, World};
use zircon_runtime_interface::world_sync::{WatchKey, WatchRegistration, WatchToken, WorldFact};
use zircon_runtime_interface::{
    GatewaySessionIdentity, ZrRuntimeOperationHandle, ZrRuntimeOperationResultV1,
    ZrRuntimeOperationStatusV2, ZrRuntimeOperationSubmitRequestV1,
};

use crate::core::editor_event::ViewInstanceId;
use crate::core::editor_message::{
    EditorMessagePayload, EditorTopic, EditorViewInvalidationMask, SharedEditorMessageBus,
};
use crate::core::gateway::{
    EditorRuntimeGateway, EditorRuntimeGatewayHandle, GatewayError, InProcessGateway,
    RuntimeCapabilities,
};

use super::{
    world_replacement_epoch_advanced, WorldSyncPump, WorldSyncPumpError,
    WorldSyncShutdownWatchDisposition, TOPIC_WORLD_FACT,
};

struct BlockingWatchGateway {
    entered: Arc<Barrier>,
    release: Arc<Barrier>,
    token: WatchToken,
}

impl EditorRuntimeGateway for BlockingWatchGateway {
    fn session_handle(&self) -> zircon_runtime_interface::ZrRuntimeSessionHandle {
        zircon_runtime_interface::ZrRuntimeSessionHandle::new(1)
    }

    fn session_identity(&self) -> zircon_runtime_interface::GatewaySessionIdentity {
        zircon_runtime_interface::GatewaySessionIdentity::new(1, self.session_handle(), 1, None)
    }

    fn capabilities(&self) -> Arc<RuntimeCapabilities> {
        Arc::new(RuntimeCapabilities::editor_default())
    }

    fn watch_world(&self, _registration: WatchRegistration) -> Result<WatchToken, GatewayError> {
        self.entered.wait();
        self.release.wait();
        Ok(self.token)
    }

    fn unwatch_world(&self, _token: WatchToken) -> Result<bool, GatewayError> {
        Ok(true)
    }

    fn submit_operation(
        &self,
        _request: ZrRuntimeOperationSubmitRequestV1,
    ) -> Result<ZrRuntimeOperationHandle, GatewayError> {
        Err(GatewayError::CapabilityMissing {
            capability: "runtime.operation.submit",
        })
    }

    fn poll_operation(
        &self,
        _handle: ZrRuntimeOperationHandle,
    ) -> Result<ZrRuntimeOperationStatusV2, GatewayError> {
        Err(GatewayError::CapabilityMissing {
            capability: "runtime.operation.poll",
        })
    }

    fn harvest_operation(
        &self,
        _handle: ZrRuntimeOperationHandle,
    ) -> Result<ZrRuntimeOperationResultV1, GatewayError> {
        Err(GatewayError::CapabilityMissing {
            capability: "runtime.operation.harvest",
        })
    }
}

struct TrackingWatchGateway {
    token: WatchToken,
    identity: GatewaySessionIdentity,
    watch_calls: Arc<AtomicUsize>,
    unwatch_calls: Arc<AtomicUsize>,
    unwatch_error: Option<GatewayError>,
}

struct BlockingDrainGateway {
    entered: Arc<Barrier>,
    release: Arc<Barrier>,
}

impl EditorRuntimeGateway for BlockingDrainGateway {
    fn session_handle(&self) -> zircon_runtime_interface::ZrRuntimeSessionHandle {
        zircon_runtime_interface::ZrRuntimeSessionHandle::new(3)
    }

    fn session_identity(&self) -> zircon_runtime_interface::GatewaySessionIdentity {
        zircon_runtime_interface::GatewaySessionIdentity::new(3, self.session_handle(), 1, None)
    }

    fn capabilities(&self) -> Arc<RuntimeCapabilities> {
        Arc::new(RuntimeCapabilities::editor_default())
    }

    fn drain_world_invalidations(
        &self,
    ) -> Result<Vec<zircon_runtime_interface::world_sync::InvalidationBatch>, GatewayError> {
        self.entered.wait();
        self.release.wait();
        Ok(Vec::new())
    }

    fn submit_operation(
        &self,
        _request: ZrRuntimeOperationSubmitRequestV1,
    ) -> Result<ZrRuntimeOperationHandle, GatewayError> {
        Err(GatewayError::CapabilityMissing {
            capability: "runtime.operation.submit",
        })
    }

    fn poll_operation(
        &self,
        _handle: ZrRuntimeOperationHandle,
    ) -> Result<ZrRuntimeOperationStatusV2, GatewayError> {
        Err(GatewayError::CapabilityMissing {
            capability: "runtime.operation.poll",
        })
    }

    fn harvest_operation(
        &self,
        _handle: ZrRuntimeOperationHandle,
    ) -> Result<ZrRuntimeOperationResultV1, GatewayError> {
        Err(GatewayError::CapabilityMissing {
            capability: "runtime.operation.harvest",
        })
    }
}

struct RecordingDrainGateway {
    calls: Arc<AtomicUsize>,
}

impl EditorRuntimeGateway for RecordingDrainGateway {
    fn session_handle(&self) -> zircon_runtime_interface::ZrRuntimeSessionHandle {
        zircon_runtime_interface::ZrRuntimeSessionHandle::new(4)
    }

    fn session_identity(&self) -> zircon_runtime_interface::GatewaySessionIdentity {
        zircon_runtime_interface::GatewaySessionIdentity::new(4, self.session_handle(), 1, None)
    }

    fn capabilities(&self) -> Arc<RuntimeCapabilities> {
        Arc::new(RuntimeCapabilities::editor_default())
    }

    fn drain_world_invalidations(
        &self,
    ) -> Result<Vec<zircon_runtime_interface::world_sync::InvalidationBatch>, GatewayError> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        Ok(Vec::new())
    }

    fn submit_operation(
        &self,
        _request: ZrRuntimeOperationSubmitRequestV1,
    ) -> Result<ZrRuntimeOperationHandle, GatewayError> {
        Err(GatewayError::CapabilityMissing {
            capability: "runtime.operation.submit",
        })
    }

    fn poll_operation(
        &self,
        _handle: ZrRuntimeOperationHandle,
    ) -> Result<ZrRuntimeOperationStatusV2, GatewayError> {
        Err(GatewayError::CapabilityMissing {
            capability: "runtime.operation.poll",
        })
    }

    fn harvest_operation(
        &self,
        _handle: ZrRuntimeOperationHandle,
    ) -> Result<ZrRuntimeOperationResultV1, GatewayError> {
        Err(GatewayError::CapabilityMissing {
            capability: "runtime.operation.harvest",
        })
    }
}

impl TrackingWatchGateway {
    fn new(token: WatchToken) -> Self {
        Self::with_identity(
            token,
            GatewaySessionIdentity::new(
                2,
                zircon_runtime_interface::ZrRuntimeSessionHandle::new(2),
                1,
                None,
            ),
        )
    }

    fn with_identity(token: WatchToken, identity: GatewaySessionIdentity) -> Self {
        Self {
            token,
            identity,
            watch_calls: Arc::new(AtomicUsize::new(0)),
            unwatch_calls: Arc::new(AtomicUsize::new(0)),
            unwatch_error: None,
        }
    }

    fn with_failing_unwatch(
        token: WatchToken,
        identity: GatewaySessionIdentity,
        error: GatewayError,
    ) -> Self {
        let mut gateway = Self::with_identity(token, identity);
        gateway.unwatch_error = Some(error);
        gateway
    }
}

impl EditorRuntimeGateway for TrackingWatchGateway {
    fn session_handle(&self) -> zircon_runtime_interface::ZrRuntimeSessionHandle {
        self.identity.runtime_session()
    }

    fn session_identity(&self) -> zircon_runtime_interface::GatewaySessionIdentity {
        self.identity.clone()
    }

    fn capabilities(&self) -> Arc<RuntimeCapabilities> {
        Arc::new(RuntimeCapabilities::editor_default())
    }

    fn watch_world(&self, _registration: WatchRegistration) -> Result<WatchToken, GatewayError> {
        self.watch_calls.fetch_add(1, Ordering::SeqCst);
        Ok(self.token)
    }

    fn unwatch_world(&self, _token: WatchToken) -> Result<bool, GatewayError> {
        self.unwatch_calls.fetch_add(1, Ordering::SeqCst);
        if let Some(error) = &self.unwatch_error {
            return Err(error.clone());
        }
        Ok(true)
    }

    fn submit_operation(
        &self,
        _request: ZrRuntimeOperationSubmitRequestV1,
    ) -> Result<ZrRuntimeOperationHandle, GatewayError> {
        Err(GatewayError::CapabilityMissing {
            capability: "runtime.operation.submit",
        })
    }

    fn poll_operation(
        &self,
        _handle: ZrRuntimeOperationHandle,
    ) -> Result<ZrRuntimeOperationStatusV2, GatewayError> {
        Err(GatewayError::CapabilityMissing {
            capability: "runtime.operation.poll",
        })
    }

    fn harvest_operation(
        &self,
        _handle: ZrRuntimeOperationHandle,
    ) -> Result<ZrRuntimeOperationResultV1, GatewayError> {
        Err(GatewayError::CapabilityMissing {
            capability: "runtime.operation.harvest",
        })
    }
}

fn hierarchy_watch_registration() -> WatchRegistration {
    WatchRegistration::new(WatchKey::WorldStructure)
}

fn hierarchy_watch_view() -> ViewInstanceId {
    ViewInstanceId::new("hierarchy")
}

#[test]
fn pump_forwards_facts_and_projects_dirty_tokens_once_per_runtime_drain() {
    let level = DefaultLevelManager::default().create_level(World::empty(), Default::default());
    let gateway = EditorRuntimeGatewayHandle::new(Arc::new(InProcessGateway::for_authoring_level(
        level.clone(),
    )));
    let bus = SharedEditorMessageBus::default();
    let subscriber = bus
        .register_subscriber([EditorTopic::parse(TOPIC_WORLD_FACT).unwrap()])
        .unwrap();
    let hierarchy = ViewInstanceId::new("hierarchy");
    let mut pump = WorldSyncPump::default();

    let token = pump
        .watch_view(
            &gateway,
            WatchRegistration::new(WatchKey::WorldStructure),
            hierarchy.clone(),
            EditorViewInvalidationMask::TREE_STRUCTURE,
        )
        .unwrap();

    let mut spawned = None;
    gateway
        .with_world_mut(&mut |world| {
            spawned = Some(
                world
                    .spawn_node(NodeKind::Empty)
                    .expect("test scene spawn should succeed"),
            )
        })
        .unwrap();
    let spawned = spawned.expect("world mutation should return the new entity");

    let report = pump.pump(&gateway, &bus).unwrap();
    assert!(report.transport_available());
    assert_eq!(report.drain_gateway_generation(), Some(0));
    assert_eq!(report.batches(), 1);
    assert_eq!(report.published_facts(), 1);
    assert_eq!(report.matched_tokens(), 1);
    assert_eq!(report.dirty_views(), 1);
    assert_eq!(report.last_generation(), Some(level.world_generation()));
    assert_eq!(
        bus.drain_dirty().mask_for(&hierarchy),
        Some(EditorViewInvalidationMask::TREE_STRUCTURE)
    );

    let deliveries = bus.drain_deliveries(subscriber);
    assert_eq!(deliveries.len(), 1);
    assert_eq!(deliveries[0].topic().as_str(), TOPIC_WORLD_FACT);
    match deliveries[0].message().payload() {
        EditorMessagePayload::Custom { schema_id, payload } => {
            assert_eq!(schema_id.as_str(), "zircon.editor.world_fact.v1");
            assert_eq!(
                serde_json::from_value::<WorldFact>(payload.clone()).unwrap(),
                WorldFact::Spawned(spawned)
            );
        }
        unexpected => panic!("world fact must use the custom fact envelope, got {unexpected:?}"),
    }

    let next = pump.pump(&gateway, &bus).unwrap();
    assert_eq!(next.batches(), 0);
    assert_eq!(next.published_facts(), 0);
    assert_eq!(next.matched_tokens(), 0);
    assert_eq!(next.dirty_views(), 0);
    assert!(bus.drain_dirty().is_empty());
    assert!(bus.drain_deliveries(subscriber).is_empty());

    assert!(pump.unwatch_view(&gateway, &token).unwrap());
}

#[test]
fn pump_reports_each_world_replacement_epoch_once_for_the_current_gateway() {
    let level = DefaultLevelManager::default().create_level(World::empty(), Default::default());
    let gateway = EditorRuntimeGatewayHandle::new(Arc::new(InProcessGateway::for_authoring_level(
        level.clone(),
    )));
    let bus = SharedEditorMessageBus::default();
    let mut pump = WorldSyncPump::default();
    pump.watch_view(
        &gateway,
        hierarchy_watch_registration(),
        hierarchy_watch_view(),
        EditorViewInvalidationMask::TREE_STRUCTURE,
    )
    .unwrap();

    level.replace_world_and_reset_runtime_state(World::empty());
    let first_epoch = level.capture_world_replacement_epoch();
    let first = pump.pump(&gateway, &bus).unwrap();
    assert_eq!(first.advanced_world_replacement_epoch(), Some(first_epoch));
    assert!(pump.acknowledge_world_replacement(first_epoch));
    assert_eq!(
        pump.pump(&gateway, &bus)
            .unwrap()
            .advanced_world_replacement_epoch(),
        None
    );

    level.replace_world_and_reset_runtime_state(World::empty());
    let second_epoch = level.capture_world_replacement_epoch();
    assert!(second_epoch > first_epoch);
    assert_eq!(
        pump.pump(&gateway, &bus)
            .unwrap()
            .advanced_world_replacement_epoch(),
        Some(second_epoch)
    );
}

#[test]
fn unacknowledged_world_replacement_is_reported_again_on_the_next_pump() {
    let level = DefaultLevelManager::default().create_level(World::empty(), Default::default());
    let gateway = EditorRuntimeGatewayHandle::new(Arc::new(InProcessGateway::for_authoring_level(
        level.clone(),
    )));
    let bus = SharedEditorMessageBus::default();
    let mut pump = WorldSyncPump::default();
    pump.watch_view(
        &gateway,
        hierarchy_watch_registration(),
        hierarchy_watch_view(),
        EditorViewInvalidationMask::TREE_STRUCTURE,
    )
    .unwrap();
    level.replace_world_and_reset_runtime_state(World::empty());
    let epoch = level.capture_world_replacement_epoch();

    assert_eq!(
        pump.pump(&gateway, &bus)
            .unwrap()
            .advanced_world_replacement_epoch(),
        Some(epoch)
    );
    assert_eq!(
        pump.pump(&gateway, &bus)
            .unwrap()
            .advanced_world_replacement_epoch(),
        Some(epoch)
    );
    assert!(pump.acknowledge_world_replacement(epoch));
    assert_eq!(
        pump.pump(&gateway, &bus)
            .unwrap()
            .advanced_world_replacement_epoch(),
        None
    );
}

#[test]
fn pump_uses_a_pre_watch_world_replacement_only_as_the_gateway_epoch_baseline() {
    let level = DefaultLevelManager::default().create_level(World::empty(), Default::default());
    level.replace_world_and_reset_runtime_state(World::empty());
    level.with_world_mut(|world| {
        for _ in 0..10 {
            world
                .spawn_node(zircon_runtime::scene::NodeKind::Empty)
                .expect("advance the initial world beyond the fact age budget");
        }
    });
    let gateway = EditorRuntimeGatewayHandle::new(Arc::new(InProcessGateway::for_authoring_level(
        level.clone(),
    )));
    let bus = SharedEditorMessageBus::default();
    let mut pump = WorldSyncPump::default();
    pump.watch_view(
        &gateway,
        hierarchy_watch_registration(),
        hierarchy_watch_view(),
        EditorViewInvalidationMask::TREE_STRUCTURE,
    )
    .unwrap();

    assert_eq!(
        pump.pump(&gateway, &bus)
            .unwrap()
            .advanced_world_replacement_epoch(),
        None,
        "a world installed before the editor watch is the initial Play world, not a scene transition"
    );

    level.replace_world_and_reset_runtime_state(World::empty());
    assert_eq!(
        pump.pump(&gateway, &bus)
            .unwrap()
            .advanced_world_replacement_epoch(),
        Some(level.capture_world_replacement_epoch())
    );
}

#[test]
fn world_replacement_epoch_validation_rejects_zero_and_regression() {
    assert!(world_replacement_epoch_advanced(None, 0).is_err());
    assert!(world_replacement_epoch_advanced(Some(9), 8).is_err());
    assert!(!world_replacement_epoch_advanced(Some(9), 9).unwrap());
    assert!(world_replacement_epoch_advanced(Some(9), 10).unwrap());
}

#[test]
fn repeated_view_watch_registration_reuses_the_existing_runtime_token() {
    let level = DefaultLevelManager::default().create_level(World::empty(), Default::default());
    let gateway =
        EditorRuntimeGatewayHandle::new(Arc::new(InProcessGateway::for_authoring_level(level)));
    let bus = SharedEditorMessageBus::default();
    let hierarchy = ViewInstanceId::new("hierarchy");
    let mut pump = WorldSyncPump::default();

    let first = pump
        .watch_view(
            &gateway,
            WatchRegistration::new(WatchKey::WorldStructure),
            hierarchy.clone(),
            EditorViewInvalidationMask::TREE_STRUCTURE,
        )
        .unwrap();
    let repeated = pump
        .watch_view(
            &gateway,
            WatchRegistration::new(WatchKey::WorldStructure),
            hierarchy.clone(),
            EditorViewInvalidationMask::TREE_STRUCTURE,
        )
        .unwrap();

    assert_eq!(repeated, first);
    assert_eq!(pump.watches().len(), 1);

    gateway
        .with_world_mut(&mut |world| {
            world
                .spawn_node(NodeKind::Empty)
                .expect("test scene spawn should succeed");
        })
        .unwrap();
    let report = pump.pump(&gateway, &bus).unwrap();
    assert_eq!(report.matched_tokens(), 1);
    assert_eq!(report.dirty_views(), 1);
    assert_eq!(
        bus.drain_dirty().mask_for(&hierarchy),
        Some(EditorViewInvalidationMask::TREE_STRUCTURE)
    );
}

#[test]
fn watch_registration_receipt_keeps_the_token_with_its_issuing_gateway_identity() {
    let level = DefaultLevelManager::default().create_level(World::empty(), Default::default());
    let gateway =
        EditorRuntimeGatewayHandle::new(Arc::new(InProcessGateway::for_authoring_level(level)));
    let mut pump = WorldSyncPump::default();

    let issued = pump
        .watch_view_with_identity(
            &gateway,
            hierarchy_watch_registration(),
            hierarchy_watch_view(),
            EditorViewInvalidationMask::TREE_STRUCTURE,
        )
        .expect("watch registration should report its issuing session");

    assert_eq!(issued.identity(), &gateway.identity());
    gateway
        .replace(Arc::new(TrackingWatchGateway::new(issued.token())))
        .expect("gateway replacement should succeed");
    assert_ne!(issued.identity(), &gateway.identity());
}

#[test]
fn live_token_collision_preserves_the_existing_editor_binding() {
    let mut pump = WorldSyncPump::default();
    let token = WatchToken::new(7);
    let hierarchy = ViewInstanceId::new("hierarchy");
    pump.watches
        .bind(
            token,
            WatchRegistration::new(WatchKey::WorldStructure),
            hierarchy.clone(),
            EditorViewInvalidationMask::TREE_STRUCTURE,
        )
        .expect("fixture token should bind");

    let error = pump
        .reject_live_watch_token(token)
        .expect_err("a reused runtime token must be rejected");

    assert!(
        matches!(error, WorldSyncPumpError::TokenCollision { token: actual } if actual == token)
    );
    assert_eq!(pump.watches().binding(token).unwrap().view(), &hierarchy);
}

#[test]
fn unwatch_removes_the_editor_projection_without_suppressing_runtime_facts() {
    let level = DefaultLevelManager::default().create_level(World::empty(), Default::default());
    let gateway =
        EditorRuntimeGatewayHandle::new(Arc::new(InProcessGateway::for_authoring_level(level)));
    let bus = SharedEditorMessageBus::default();
    let subscriber = bus
        .register_subscriber([EditorTopic::parse(TOPIC_WORLD_FACT).unwrap()])
        .unwrap();
    let mut pump = WorldSyncPump::default();
    let token = pump
        .watch_view(
            &gateway,
            WatchRegistration::new(WatchKey::WorldStructure),
            ViewInstanceId::new("hierarchy"),
            EditorViewInvalidationMask::TREE_STRUCTURE,
        )
        .unwrap();
    assert!(pump.unwatch_view(&gateway, &token).unwrap());

    gateway
        .with_world_mut(&mut |world| {
            world
                .spawn_node(NodeKind::Empty)
                .expect("test scene spawn should succeed");
        })
        .unwrap();

    let report = pump.pump(&gateway, &bus).unwrap();
    assert_eq!(report.published_facts(), 1);
    assert_eq!(report.matched_tokens(), 0);
    assert_eq!(report.dirty_views(), 0);
    assert!(bus.drain_dirty().is_empty());
    assert_eq!(bus.drain_deliveries(subscriber).len(), 1);
}

#[test]
fn gateway_replacement_discards_retired_session_tokens_before_the_next_drain() {
    let first_level =
        DefaultLevelManager::default().create_level(World::empty(), Default::default());
    let second_level =
        DefaultLevelManager::default().create_level(World::empty(), Default::default());
    let gateway = EditorRuntimeGatewayHandle::new(Arc::new(InProcessGateway::for_authoring_level(
        first_level,
    )));
    let bus = SharedEditorMessageBus::default();
    let hierarchy = ViewInstanceId::new("hierarchy");
    let mut pump = WorldSyncPump::default();
    pump.watch_view(
        &gateway,
        WatchRegistration::new(WatchKey::WorldStructure),
        hierarchy.clone(),
        EditorViewInvalidationMask::TREE_STRUCTURE,
    )
    .unwrap();
    assert_eq!(pump.watches().len(), 1);

    gateway
        .replace(Arc::new(InProcessGateway::for_authoring_level(
            second_level,
        )))
        .unwrap();
    gateway
        .with_world_mut(&mut |world| {
            world
                .spawn_node(NodeKind::Empty)
                .expect("test scene spawn should succeed");
        })
        .unwrap();

    let report = pump.pump(&gateway, &bus).unwrap();
    assert_eq!(report.published_facts(), 1);
    assert_eq!(report.matched_tokens(), 0);
    assert!(bus.drain_dirty().is_empty());
    assert!(pump.watches().is_empty());
}

#[test]
fn replacement_during_world_drain_does_not_forward_the_old_lease_to_the_new_gateway() {
    let entered = Arc::new(Barrier::new(2));
    let release = Arc::new(Barrier::new(2));
    let gateway = EditorRuntimeGatewayHandle::new(Arc::new(BlockingDrainGateway {
        entered: entered.clone(),
        release: release.clone(),
    }));
    let worker_gateway = gateway.clone();
    let worker = std::thread::spawn(move || {
        WorldSyncPump::default()
            .pump(&worker_gateway, &SharedEditorMessageBus::default())
            .expect("the origin drain should complete")
    });

    entered.wait();
    let replacement_calls = Arc::new(AtomicUsize::new(0));
    gateway
        .replace(Arc::new(RecordingDrainGateway {
            calls: replacement_calls.clone(),
        }))
        .expect("replacement must publish while A is draining");
    release.wait();

    let report = worker.join().expect("join world-drain worker");
    assert!(report.transport_available());
    assert_eq!(report.stale_gateway_drains(), 1);
    assert_eq!(report.drain_gateway_generation(), Some(0));
    assert_eq!(
        report
            .drain_identity()
            .expect("stale drains must retain their actual origin identity")
            .gateway_generation(),
        0
    );
    assert_eq!(replacement_calls.load(Ordering::SeqCst), 0);
}

#[test]
fn watch_registration_cannot_bind_a_token_from_a_gateway_replaced_mid_registration() {
    let entered = Arc::new(Barrier::new(2));
    let release = Arc::new(Barrier::new(2));
    let token = WatchToken::new(71);
    let gateway = EditorRuntimeGatewayHandle::new(Arc::new(BlockingWatchGateway {
        entered: entered.clone(),
        release: release.clone(),
        token,
    }));
    let replacement_gateway = Arc::new(TrackingWatchGateway::new(token));
    let replacement_watch_calls = replacement_gateway.watch_calls.clone();
    let worker_gateway = gateway.clone();
    let worker = std::thread::spawn(move || {
        let mut pump = WorldSyncPump::default();
        pump.watch_view(
            &worker_gateway,
            hierarchy_watch_registration(),
            hierarchy_watch_view(),
            EditorViewInvalidationMask::TREE_STRUCTURE,
        )
    });

    entered.wait();
    let (replacement_started_sender, replacement_started) = mpsc::channel();
    let replacement_complete = Arc::new(AtomicBool::new(false));
    let replacement_complete_for_thread = replacement_complete.clone();
    let replacement_handle = gateway.clone();
    let replacement = std::thread::spawn(move || {
        replacement_started_sender
            .send(())
            .expect("replacement start receiver should remain alive");
        replacement_handle
            .replace(replacement_gateway)
            .expect("gateway replacement should succeed");
        replacement_complete_for_thread.store(true, Ordering::SeqCst);
    });
    replacement_started
        .recv()
        .expect("replacement thread should start");
    let deadline = Instant::now() + Duration::from_millis(100);
    while !replacement_complete.load(Ordering::SeqCst) && Instant::now() < deadline {
        std::thread::yield_now();
    }
    assert!(
        replacement_complete.load(Ordering::SeqCst),
        "an origin watch lease must keep the old endpoint alive without blocking replacement"
    );

    release.wait();
    let stale = worker.join().expect("watch worker should complete");
    replacement
        .join()
        .expect("replacement worker should complete");

    assert!(matches!(
        stale,
        Err(WorldSyncPumpError::Gateway(GatewayError::StaleGeneration {
            expected_generation: 0,
            current_generation: 1,
        }))
    ));

    let mut pump = WorldSyncPump::default();
    let current_token = pump
        .watch_view(
            &gateway,
            hierarchy_watch_registration(),
            hierarchy_watch_view(),
            EditorViewInvalidationMask::TREE_STRUCTURE,
        )
        .expect("current generation should receive its own watch");
    assert_eq!(current_token.token(), token);
    assert_eq!(replacement_watch_calls.load(Ordering::SeqCst), 1);
    assert_eq!(pump.watches().len(), 1);
}

#[test]
fn stale_unwatch_after_gateway_replacement_cannot_revoke_a_current_session_token() {
    let token = WatchToken::new(73);
    let first_gateway = Arc::new(TrackingWatchGateway::new(token));
    let gateway = EditorRuntimeGatewayHandle::new(first_gateway);
    let mut pump = WorldSyncPump::default();
    let stale_token = pump
        .watch_view(
            &gateway,
            hierarchy_watch_registration(),
            hierarchy_watch_view(),
            EditorViewInvalidationMask::TREE_STRUCTURE,
        )
        .expect("first generation should register a watch");

    let current_gateway = Arc::new(TrackingWatchGateway::new(token));
    let current_unwatch_calls = current_gateway.unwatch_calls.clone();
    gateway
        .replace(current_gateway)
        .expect("gateway replacement should succeed");

    assert!(!pump.unwatch_view(&gateway, &stale_token).unwrap());
    assert_eq!(current_unwatch_calls.load(Ordering::SeqCst), 0);
    assert!(pump.watches().is_empty());
}

#[test]
fn reused_watch_token_cannot_cross_transport_epoch_or_project_boundary() {
    let raw_token = WatchToken::new(79);
    let raw_session = zircon_runtime_interface::ZrRuntimeSessionHandle::new(23);
    let first_gateway = Arc::new(TrackingWatchGateway::with_identity(
        raw_token,
        GatewaySessionIdentity::new(17, raw_session, 29, Some(Arc::from("E:/Projects/First"))),
    ));
    let gateway = EditorRuntimeGatewayHandle::new(first_gateway);
    let mut pump = WorldSyncPump::default();
    let stale_token = pump
        .watch_view(
            &gateway,
            hierarchy_watch_registration(),
            hierarchy_watch_view(),
            EditorViewInvalidationMask::TREE_STRUCTURE,
        )
        .expect("first transport should register a watch");

    let replacement = Arc::new(TrackingWatchGateway::with_identity(
        raw_token,
        GatewaySessionIdentity::new(17, raw_session, 30, Some(Arc::from("E:/Projects/Second"))),
    ));
    let replacement_unwatch_calls = replacement.unwatch_calls.clone();
    gateway
        .replace(replacement)
        .expect("replacement transport should publish");

    assert!(!pump.unwatch_view(&gateway, &stale_token).unwrap());
    assert_eq!(replacement_unwatch_calls.load(Ordering::SeqCst), 0);
    assert!(pump.watches().is_empty());
}

#[test]
fn explicit_shutdown_retires_every_local_watch_after_origin_cleanup() {
    let first_token = WatchToken::new(83);
    let gateway_impl = Arc::new(TrackingWatchGateway::new(first_token));
    let unwatch_calls = gateway_impl.unwatch_calls.clone();
    let gateway = EditorRuntimeGatewayHandle::new(gateway_impl);
    let mut pump = WorldSyncPump::default();
    pump.watch_view(
        &gateway,
        hierarchy_watch_registration(),
        hierarchy_watch_view(),
        EditorViewInvalidationMask::TREE_STRUCTURE,
    )
    .expect("live transport should register the first watch");
    pump.watches
        .bind(
            WatchToken::new(84),
            WatchRegistration::new(WatchKey::Subtree { root: 84 }),
            ViewInstanceId::new("inspector"),
            EditorViewInvalidationMask::PRESENTATION_DATA,
        )
        .expect("fixture should own a second live watch");

    let receipt = pump.shutdown(&gateway);

    assert_eq!(receipt.origin_identity(), Some(&gateway.identity()));
    assert_eq!(receipt.current_identity(), &gateway.identity());
    assert_eq!(receipt.unwatched_count(), 2);
    assert_eq!(receipt.stale_identity_count(), 0);
    assert_eq!(receipt.failed_count(), 0);
    assert_eq!(
        receipt
            .watches()
            .iter()
            .map(|watch| watch.token())
            .collect::<Vec<_>>(),
        vec![WatchToken::new(83), WatchToken::new(84)]
    );
    assert!(receipt.watches().iter().all(|watch| {
        matches!(
            watch.disposition(),
            WorldSyncShutdownWatchDisposition::Unwatched
        )
    }));
    assert_eq!(unwatch_calls.load(Ordering::SeqCst), 2);
    assert!(pump.watches().is_empty());
}

#[test]
fn explicit_shutdown_never_sends_retired_watches_to_a_replacement_transport() {
    let token = WatchToken::new(85);
    let gateway = EditorRuntimeGatewayHandle::new(Arc::new(TrackingWatchGateway::new(token)));
    let mut pump = WorldSyncPump::default();
    pump.watch_view(
        &gateway,
        hierarchy_watch_registration(),
        hierarchy_watch_view(),
        EditorViewInvalidationMask::TREE_STRUCTURE,
    )
    .expect("first transport should register a watch");

    let replacement = Arc::new(TrackingWatchGateway::new(token));
    let replacement_unwatch_calls = replacement.unwatch_calls.clone();
    gateway
        .replace(replacement)
        .expect("replacement transport should publish");

    let receipt = pump.shutdown(&gateway);

    assert_eq!(receipt.stale_identity_count(), 1);
    assert_eq!(receipt.unwatched_count(), 0);
    assert_eq!(receipt.failed_count(), 0);
    assert!(matches!(
        receipt.watches()[0].disposition(),
        WorldSyncShutdownWatchDisposition::StaleIdentity
    ));
    assert_eq!(replacement_unwatch_calls.load(Ordering::SeqCst), 0);
    assert!(pump.watches().is_empty());
}

#[test]
fn explicit_shutdown_records_remote_unwatch_failures_after_local_retirement() {
    let token = WatchToken::new(86);
    let identity = GatewaySessionIdentity::new(
        47,
        zircon_runtime_interface::ZrRuntimeSessionHandle::new(47),
        1,
        None,
    );
    let gateway = EditorRuntimeGatewayHandle::new(Arc::new(
        TrackingWatchGateway::with_failing_unwatch(token, identity, GatewayError::SessionLost),
    ));
    let mut pump = WorldSyncPump::default();
    pump.watch_view(
        &gateway,
        hierarchy_watch_registration(),
        hierarchy_watch_view(),
        EditorViewInvalidationMask::TREE_STRUCTURE,
    )
    .expect("live transport should register a watch before failure");

    let receipt = pump.shutdown(&gateway);

    assert_eq!(receipt.unwatched_count(), 0);
    assert_eq!(receipt.stale_identity_count(), 0);
    assert_eq!(receipt.failed_count(), 1);
    assert!(matches!(
        receipt.watches()[0].disposition(),
        WorldSyncShutdownWatchDisposition::Failed(GatewayError::SessionLost)
    ));
    assert!(pump.watches().is_empty());
}

#[test]
fn shared_bus_exposes_a_borrowed_single_view_dirty_path() {
    let shared_bus_source = include_str!("../../editor_message/shared.rs");
    let bus_source = include_str!("../../editor_message/bus.rs");

    assert!(shared_bus_source.contains("pub fn mark_view_dirty_ref("));
    assert!(shared_bus_source.contains("self.lock().mark_view_dirty_ref(view, mask)"));
    assert!(bus_source.contains("pub fn mark_view_dirty_ref("));
    assert!(bus_source.contains("self.dirty.mark_ref(view, mask);"));
}

#[test]
fn pump_submits_each_projected_dirty_set_through_one_bus_lock_boundary() {
    let pump_source = include_str!("../pump.rs");
    let shared_bus_source = include_str!("../../editor_message/shared.rs");
    let bus_source = include_str!("../../editor_message/bus.rs");

    assert!(pump_source.contains("bus.mark_view_dirty_set(projection.dirty());"));
    assert!(shared_bus_source.contains("pub fn mark_view_dirty_set("));
    assert!(shared_bus_source.contains("self.lock().mark_view_dirty_set(dirty);"));
    assert!(bus_source.contains("pub fn mark_view_dirty_set("));
    assert!(bus_source.contains("for (view, mask) in dirty.iter()"));
}
