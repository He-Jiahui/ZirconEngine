use std::error::Error;
use std::sync::{Arc, Mutex};
use std::thread::{self, ThreadId};
use std::time::Instant;

use zircon_runtime::core::TaskGraphShutdownReport;

use crate::entry::product_composition::ownership::ProductOwnership;
use crate::entry::runtime_library::RuntimeSession;

use super::{registry, ProductCloseError, RetainedPacket};

pub(super) struct Cleanup {
    packet: Option<RetainedPacket>,
    secondary: Option<ProductCloseError>,
    report: Option<TaskGraphShutdownReport>,
}

pub(in crate::entry) struct RetainedOwner {
    pub(super) bucket: ThreadId,
    pub(in crate::entry) primary: Arc<dyn Error + Send + Sync>,
    cleanup: Mutex<Cleanup>,
}

impl RetainedOwner {
    pub(in crate::entry) fn retain(
        primary: Arc<dyn Error + Send + Sync>,
        packet: RetainedPacket,
    ) -> Arc<Self> {
        let report = packet.report().cloned().or_else(|| {
            primary
                .downcast_ref::<ProductCloseError>()
                .and_then(|error| error.graph_report().cloned())
        });
        let owner = Arc::new(Self {
            bucket: thread::current().id(),
            primary,
            cleanup: Mutex::new(Cleanup {
                packet: Some(packet),
                secondary: None,
                report,
            }),
        });
        registry::insert(&owner);
        owner
    }

    pub(in crate::entry) fn pending(&self) -> bool {
        self.cleanup
            .try_lock()
            .map_or(true, |cleanup| cleanup.packet.is_some())
    }

    pub(super) fn has_runtime(&self) -> bool {
        self.cleanup.try_lock().map_or(true, |cleanup| {
            cleanup
                .packet
                .as_ref()
                .is_some_and(|packet| packet.runtime.is_some())
        })
    }

    pub(in crate::entry) fn observation(
        &self,
    ) -> Result<(Option<ProductCloseError>, Option<TaskGraphShutdownReport>), ProductCloseError>
    {
        let cleanup = self
            .cleanup
            .try_lock()
            .map_err(|_| ProductCloseError::Busy)?;
        Ok((cleanup.secondary.clone(), cleanup.report.clone()))
    }

    pub(in crate::entry) fn retry_until(
        self: &Arc<Self>,
        deadline: Instant,
    ) -> Result<Option<TaskGraphShutdownReport>, ProductCloseError> {
        let mut cleanup = self
            .cleanup
            .try_lock()
            .map_err(|_| ProductCloseError::Busy)?;
        let Some(packet) = cleanup.packet.as_mut() else {
            return Ok(cleanup.report.clone());
        };
        let result = packet.close_until(deadline);
        let report = packet.report().cloned().or_else(|| {
            result
                .as_ref()
                .err()
                .and_then(|error| error.graph_report().cloned())
        });
        cleanup.report = report;
        match result {
            Ok(report) => {
                cleanup.packet.take();
                cleanup.secondary = None;
                drop(cleanup);
                registry::release(self);
                Ok(report)
            }
            Err(error) => {
                cleanup.secondary = Some(error.clone());
                Err(error)
            }
        }
    }

    pub(in crate::entry) fn attach_product(
        self: &Arc<Self>,
        product: ProductOwnership,
    ) -> Result<(), ProductOwnership> {
        let Ok(mut cleanup) = self.cleanup.try_lock() else {
            return Err(product);
        };
        let Some(packet) = cleanup.packet.as_mut() else {
            return Err(product);
        };
        if packet.product.is_some()
            || packet
                .required_runtime_thread()
                .is_some_and(|id| id != thread::current().id())
        {
            return Err(product);
        }
        packet.product = Some(product);
        Ok(())
    }

    pub(in crate::entry) fn take_runtime_only(self: &Arc<Self>) -> Option<RuntimeSession> {
        let mut cleanup = self.cleanup.try_lock().ok()?;
        let packet = cleanup.packet.as_mut()?;
        if packet.required_runtime_thread() != Some(thread::current().id()) {
            return None;
        }
        let session = packet.take_runtime_only()?;
        cleanup.packet.take();
        drop(cleanup);
        registry::release(self);
        Some(session)
    }
}
