use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;

use zircon_runtime::scene::{LevelSystem, World};
use zircon_runtime_interface::{
    ZrRuntimeOperationHandle, ZrRuntimeOperationResultV1, ZrRuntimeOperationStatusV2,
    ZrRuntimeOperationSubmitRequestV1, ZrRuntimeSessionHandle,
};

use crate::core::gateway::{EditorRuntimeGateway, GatewayError};

pub(super) struct CallbackThenErrorGateway {
    level: LevelSystem,
    fail_next_world_write: AtomicBool,
    duplicate_first_write_callback: bool,
    dispatch_calls: Option<Arc<AtomicUsize>>,
    callback_attempts: Option<Arc<AtomicUsize>>,
}

impl CallbackThenErrorGateway {
    pub(super) fn new(level: LevelSystem) -> Self {
        Self {
            level,
            fail_next_world_write: AtomicBool::new(true),
            duplicate_first_write_callback: false,
            dispatch_calls: None,
            callback_attempts: None,
        }
    }

    pub(super) fn duplicate_first_write_then_error(
        level: LevelSystem,
        dispatch_calls: Arc<AtomicUsize>,
        callback_attempts: Arc<AtomicUsize>,
    ) -> Self {
        Self {
            level,
            fail_next_world_write: AtomicBool::new(true),
            duplicate_first_write_callback: true,
            dispatch_calls: Some(dispatch_calls),
            callback_attempts: Some(callback_attempts),
        }
    }

    fn record_dispatch(&self) {
        if let Some(dispatch_calls) = self.dispatch_calls.as_ref() {
            dispatch_calls.fetch_add(1, Ordering::SeqCst);
        }
    }

    fn record_callback_attempt(&self) {
        if let Some(callback_attempts) = self.callback_attempts.as_ref() {
            callback_attempts.fetch_add(1, Ordering::SeqCst);
        }
    }
}

impl EditorRuntimeGateway for CallbackThenErrorGateway {
    fn session_handle(&self) -> ZrRuntimeSessionHandle {
        ZrRuntimeSessionHandle::invalid()
    }

    fn session_identity(&self) -> zircon_runtime_interface::GatewaySessionIdentity {
        zircon_runtime_interface::GatewaySessionIdentity::detached()
    }

    fn with_world(&self, read: &mut dyn FnMut(&World)) -> Result<(), GatewayError> {
        self.record_dispatch();
        self.level.with_world(read);
        Ok(())
    }

    fn with_world_mut(&self, write: &mut dyn FnMut(&mut World)) -> Result<(), GatewayError> {
        self.record_dispatch();
        self.level.with_world_mut(|world| {
            self.record_callback_attempt();
            write(world);
            if self.duplicate_first_write_callback
                && self.fail_next_world_write.load(Ordering::Acquire)
            {
                self.record_callback_attempt();
                write(world);
            }
        });
        if self.fail_next_world_write.swap(false, Ordering::AcqRel) {
            Err(GatewayError::Protocol {
                message: "gateway reported after executing a world write".to_owned(),
            })
        } else {
            Ok(())
        }
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
