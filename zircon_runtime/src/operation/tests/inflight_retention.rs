use std::sync::mpsc::{self, Receiver, SyncSender};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use crate::core::framework::scene::WorldHandle;
use crate::operation::RuntimeOperationSnapshot;
use crate::scene::{LevelMetadata, LevelSystem, World};

use super::*;

fn test_level() -> LevelSystem {
    LevelSystem::new(
        WorldHandle::new(1),
        Arc::new(Mutex::new(World::empty())),
        LevelMetadata::default(),
    )
}

struct BlockingPrepareHandler {
    started: SyncSender<()>,
    release: Mutex<Receiver<()>>,
}

const OWNER_BYTES: usize = 512;

impl RuntimeOperationHandler for BlockingPrepareHandler {
    fn snapshot(
        &self,
        _context: RuntimeOperationContext<'_>,
        payload: serde_json::Value,
    ) -> Result<serde_json::Value, RuntimeOperationHandlerError> {
        Ok(payload)
    }

    fn prepare(
        &self,
        snapshot: serde_json::Value,
    ) -> Result<RuntimeOperationPrepared, RuntimeOperationHandlerError> {
        self.started
            .send(())
            .expect("blocking prepare start observer remains alive");
        self.release
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .recv()
            .expect("blocking prepare release remains alive");
        Ok(RuntimeOperationPrepared::new(snapshot.clone(), snapshot))
    }

    fn snapshot_owned(
        &self,
        _context: RuntimeOperationContext<'_>,
        payload: serde_json::Value,
    ) -> Result<RuntimeOperationSnapshot, RuntimeOperationHandlerError> {
        Ok(RuntimeOperationSnapshot::with_owner_state(
            payload,
            Box::new(vec![0_u8; OWNER_BYTES]),
            OWNER_BYTES,
        ))
    }

    fn apply(
        &self,
        _context: RuntimeOperationContext<'_>,
        _command: serde_json::Value,
    ) -> Result<(), RuntimeOperationHandlerError> {
        Ok(())
    }
}

#[test]
fn cancelled_preparing_task_remains_non_evictable_until_worker_completion() {
    let (started_sender, started_receiver) = mpsc::sync_channel(1);
    let (release_sender, release_receiver) = mpsc::sync_channel(1);
    let mut service = RuntimeOperationService::with_limits(RuntimeOperationLimits {
        max_tasks: 1,
        max_in_flight_prepares: 1,
        max_retained_bytes: 1_024,
        max_owner_applies_per_tick: 1,
        terminal_result_ttl: Duration::from_secs(60),
    });
    service
        .register_handler(
            "test.blocking-prepare",
            Arc::new(BlockingPrepareHandler {
                started: started_sender,
                release: Mutex::new(release_receiver),
            }),
        )
        .unwrap();

    let request = || {
        ZrRuntimeOperationSubmitRequestV1::new(
            ZIRCON_RUNTIME_ABI_VERSION_V1,
            "test.blocking-prepare",
            serde_json::Value::Null,
        )
    };
    let first = service.submit(request()).unwrap();
    let runtime = CoreRuntime::new();
    let level = test_level();
    service.tick(&runtime.handle(), &level);
    started_receiver
        .recv_timeout(Duration::from_secs(2))
        .expect("worker prepare starts within the bounded test deadline");
    let worker_reserved_bytes = service.lock_state().retained_bytes;

    service.cancel(first).unwrap();
    assert_eq!(
        service.lock_state().retained_bytes,
        worker_reserved_bytes,
        "cancel cannot release bytes while the worker still owns the immutable snapshot"
    );
    let pressure_admission = service.submit(request());
    release_sender
        .send(())
        .expect("cancelled worker can finish and publish its completion");

    let pressure_rejected = matches!(
        &pressure_admission,
        Err(RuntimeOperationServiceError::TaskCapacityReached { maximum: 1 })
    );
    let mut post_completion_admission = None;
    let completion_deadline = Instant::now() + Duration::from_secs(2);
    while Instant::now() < completion_deadline {
        service.tick(&runtime.handle(), &level);
        if pressure_rejected {
            match service.submit(request()) {
                Ok(handle) => {
                    post_completion_admission = Some(handle);
                    break;
                }
                Err(RuntimeOperationServiceError::TaskCapacityReached { maximum: 1 }) => {}
                Err(error) => panic!("unexpected post-completion admission error: {error}"),
            }
        }
        if post_completion_admission.is_none() {
            std::thread::sleep(Duration::from_millis(1));
        }
    }

    assert!(pressure_rejected);
    assert!(post_completion_admission.is_some());
}

#[test]
fn expired_preparing_task_releases_worker_bytes_once_after_completion() {
    let (started_sender, started_receiver) = mpsc::sync_channel(1);
    let (release_sender, release_receiver) = mpsc::sync_channel(1);
    let mut service = RuntimeOperationService::with_limits(RuntimeOperationLimits {
        max_tasks: 1,
        max_in_flight_prepares: 1,
        max_retained_bytes: 1_024,
        max_owner_applies_per_tick: 1,
        terminal_result_ttl: Duration::from_secs(60),
    });
    service
        .register_handler(
            "test.blocking-expiry",
            Arc::new(BlockingPrepareHandler {
                started: started_sender,
                release: Mutex::new(release_receiver),
            }),
        )
        .unwrap();
    let request = || {
        ZrRuntimeOperationSubmitRequestV1::new(
            ZIRCON_RUNTIME_ABI_VERSION_V1,
            "test.blocking-expiry",
            serde_json::Value::Null,
        )
    };
    let first = service
        .submit_with_deadline(request(), Some(Instant::now() + Duration::from_secs(5)))
        .unwrap();
    let runtime = CoreRuntime::new();
    let level = test_level();
    service.tick(&runtime.handle(), &level);
    started_receiver
        .recv_timeout(Duration::from_secs(2))
        .expect("worker prepare starts within the bounded test deadline");
    let worker_reserved_bytes = service.lock_state().retained_bytes;

    service.set_deadline_state_for_test(
        first,
        Some(Instant::now() - Duration::from_millis(1)),
        true,
    );
    service.tick(&runtime.handle(), &level);
    assert_eq!(
        service.poll(first).unwrap().phase(),
        Some(zircon_runtime_interface::ZrRuntimeOperationPhase::Expired)
    );
    assert_eq!(
        service.lock_state().retained_bytes,
        worker_reserved_bytes,
        "expiry cannot release bytes while the worker still owns the immutable snapshot"
    );
    assert!(matches!(
        service.submit(request()),
        Err(RuntimeOperationServiceError::TaskCapacityReached { maximum: 1 })
    ));

    release_sender
        .send(())
        .expect("expired worker can finish and publish its stale completion");
    let completion_deadline = Instant::now() + Duration::from_secs(2);
    while Instant::now() < completion_deadline {
        service.tick(&runtime.handle(), &level);
        if service.lock_state().retained_bytes == 0 {
            break;
        }
        std::thread::sleep(Duration::from_millis(1));
    }
    assert_eq!(service.lock_state().retained_bytes, 0);
    assert_eq!(service.lock_state().in_flight_prepares, 0);
}

#[test]
fn real_worker_channel_loss_releases_nonzero_owner_bytes_once_and_allows_admission() {
    let (started_sender, started_receiver) = mpsc::sync_channel(1);
    let (release_sender, release_receiver) = mpsc::sync_channel(1);
    let mut service = RuntimeOperationService::with_limits(RuntimeOperationLimits {
        max_tasks: 1,
        max_in_flight_prepares: 1,
        max_retained_bytes: 1_024,
        max_owner_applies_per_tick: 1,
        terminal_result_ttl: Duration::from_secs(60),
    });
    service
        .register_handler(
            "test.real-worker-channel-loss",
            Arc::new(BlockingPrepareHandler {
                started: started_sender,
                release: Mutex::new(release_receiver),
            }),
        )
        .unwrap();
    let request = || {
        ZrRuntimeOperationSubmitRequestV1::new(
            ZIRCON_RUNTIME_ABI_VERSION_V1,
            "test.real-worker-channel-loss",
            serde_json::Value::Null,
        )
    };
    let first = service.submit(request()).unwrap();
    let runtime = CoreRuntime::new();
    let level = test_level();
    service.tick(&runtime.handle(), &level);
    started_receiver
        .recv_timeout(Duration::from_secs(2))
        .expect("real worker reaches its prepare barrier");
    let worker_reserved_bytes = service.lock_state().retained_bytes;
    assert!(worker_reserved_bytes >= OWNER_BYTES);

    service.drop_prepare_completion_receivers_for_test();
    release_sender
        .send(())
        .expect("real worker can finish after its completion receiver is dropped");

    let terminal_deadline = Instant::now() + Duration::from_secs(2);
    while Instant::now() < terminal_deadline {
        service.tick(&runtime.handle(), &level);
        if service
            .poll(first)
            .unwrap()
            .phase()
            .is_some_and(|phase| phase.is_terminal())
        {
            break;
        }
        std::thread::sleep(Duration::from_millis(1));
    }
    let status = service.poll(first).unwrap();
    assert_eq!(
        status.phase(),
        Some(zircon_runtime_interface::ZrRuntimeOperationPhase::Failed)
    );
    assert_eq!(
        status.detail_kind(),
        Some(zircon_runtime_interface::ZrRuntimeOperationDetailKindV2::WorkerChannelLost)
    );
    assert_eq!(service.lock_state().retained_bytes, 0);
    assert_eq!(service.lock_state().in_flight_prepares, 0);

    service.tick(&runtime.handle(), &level);
    assert_eq!(service.lock_state().retained_bytes, 0);
    assert_eq!(service.lock_state().in_flight_prepares, 0);
    service
        .harvest(first)
        .expect("worker-channel-loss failure can be harvested before replacement admission");
    let replacement = service
        .submit(request())
        .expect("capacity is available after exactly-once channel-loss cleanup");
    assert_eq!(
        service.poll(replacement).unwrap().phase(),
        Some(zircon_runtime_interface::ZrRuntimeOperationPhase::Queued)
    );
}
