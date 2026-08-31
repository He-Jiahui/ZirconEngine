use std::sync::mpsc::{self, Receiver, SyncSender};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use super::*;

struct BlockingPrepareHandler {
    started: SyncSender<()>,
    release: Mutex<Receiver<()>>,
}

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
    let mut world = World::empty();
    service.tick(&runtime.handle(), &mut world);
    started_receiver
        .recv_timeout(Duration::from_secs(2))
        .expect("worker prepare starts within the bounded test deadline");

    service.cancel(first).unwrap();
    let pressure_admission = service.submit(request());
    release_sender
        .send(())
        .expect("cancelled worker can finish and publish its completion");

    let pressure_rejected = matches!(
        &pressure_admission,
        Err(RuntimeOperationServiceError::TaskCapacityReached { maximum: 1 })
    );
    let mut post_completion_admission = None;
    for _ in 0..1_024 {
        service.tick(&runtime.handle(), &mut world);
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
        std::thread::yield_now();
    }

    assert!(pressure_rejected);
    assert!(post_completion_admission.is_some());
}
