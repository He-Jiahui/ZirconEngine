use std::any::Any;
use std::sync::Arc;
use std::thread::ThreadId;
use std::time::Instant;

use zircon_runtime::core::TaskGraphShutdownReport;

use crate::entry::product_composition::ownership::ProductOwnership;
use crate::entry::runtime_library::RuntimeSession;

use super::ProductCloseError;

pub(in crate::entry) enum RetainedRuntime {
    Owned(RuntimeSession),
    Shared(Arc<RuntimeSession>),
}

pub(in crate::entry) struct RetainedPacket {
    pub(super) product: Option<ProductOwnership>,
    pub(super) runtime: Option<RetainedRuntime>,
    pub(super) host_pins: Vec<Box<dyn Any + Send + Sync>>,
    graph_report: Option<TaskGraphShutdownReport>,
}

impl RetainedPacket {
    pub(in crate::entry) fn product(product: ProductOwnership) -> Self {
        Self {
            product: Some(product),
            runtime: None,
            host_pins: Vec::new(),
            graph_report: None,
        }
    }

    pub(in crate::entry) fn runtime(session: RuntimeSession) -> Self {
        Self {
            product: None,
            runtime: Some(RetainedRuntime::Owned(session)),
            host_pins: Vec::new(),
            graph_report: None,
        }
    }

    pub(in crate::entry) fn with_runtime(mut self, session: Arc<RuntimeSession>) -> Self {
        self.runtime = Some(RetainedRuntime::Shared(session));
        self
    }

    pub(in crate::entry) fn with_host_pin<T: Any + Send + Sync>(mut self, pin: T) -> Self {
        self.host_pins.push(Box::new(pin));
        self
    }

    pub(super) fn required_runtime_thread(&self) -> Option<ThreadId> {
        self.runtime.as_ref().map(|runtime| match runtime {
            RetainedRuntime::Owned(session) => session.creator_thread(),
            RetainedRuntime::Shared(session) => session.creator_thread(),
        })
    }

    pub(super) fn report(&self) -> Option<&TaskGraphShutdownReport> {
        self.graph_report.as_ref()
    }

    pub(in crate::entry) fn close_until(
        &mut self,
        deadline: Instant,
    ) -> Result<Option<TaskGraphShutdownReport>, ProductCloseError> {
        if self
            .required_runtime_thread()
            .is_some_and(|thread| thread != std::thread::current().id())
        {
            return Err(ProductCloseError::WrongRuntimeThread);
        }
        if self.graph_report.is_none() {
            if let Some(product) = &mut self.product {
                match product.close_until(deadline) {
                    Ok(report) => self.graph_report = Some(report),
                    Err(error) => return Err(error),
                }
            }
        }
        if let Some(runtime) = self.runtime.take() {
            let mut session = match runtime {
                RetainedRuntime::Owned(session) => session,
                RetainedRuntime::Shared(shared) => match Arc::try_unwrap(shared) {
                    Ok(session) => session,
                    Err(shared) => {
                        self.runtime = Some(RetainedRuntime::Shared(shared));
                        return Err(ProductCloseError::RuntimeShared);
                    }
                },
            };
            if let Err(error) = session.try_destroy() {
                self.runtime = Some(RetainedRuntime::Owned(session));
                return Err(ProductCloseError::Runtime(Arc::new(error)));
            }
        }
        Ok(self.graph_report.clone())
    }

    pub(super) fn take_runtime_only(&mut self) -> Option<RuntimeSession> {
        if self.product.is_some() || !self.host_pins.is_empty() {
            return None;
        }
        match self.runtime.take()? {
            RetainedRuntime::Owned(session) => Some(session),
            shared => {
                self.runtime = Some(shared);
                None
            }
        }
    }
}
