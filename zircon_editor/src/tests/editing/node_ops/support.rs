use std::sync::atomic::{AtomicBool, Ordering};

use zircon_runtime::scene::{LevelSystem, World};
use zircon_runtime_interface::{
    ZrRuntimeOperationHandle, ZrRuntimeOperationResultV1, ZrRuntimeOperationStatusV2,
    ZrRuntimeOperationSubmitRequestV1, ZrRuntimeSessionHandle,
};

use crate::core::gateway::{EditorRuntimeGateway, GatewayError};

pub(super) struct CallbackThenErrorGateway {
    level: LevelSystem,
    fail_next_world_write: AtomicBool,
}

impl CallbackThenErrorGateway {
    pub(super) fn new(level: LevelSystem) -> Self {
        Self {
            level,
            fail_next_world_write: AtomicBool::new(true),
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
        self.level.with_world(read);
        Ok(())
    }

    fn with_world_mut(&self, write: &mut dyn FnMut(&mut World)) -> Result<(), GatewayError> {
        self.level.with_world_mut(write);
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
