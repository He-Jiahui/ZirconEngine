use std::fmt;
use std::marker::PhantomData;
use std::sync::Arc;

use crate::core::runtime::{CoreHandle, CoreWeak, RegisteredServiceIdentity, RegistryName};
use crate::core::CoreError;

pub struct ManagerServiceHandle<T: ?Sized> {
    pub(crate) index: u32,
    pub(crate) generation: u32,
    pub(crate) service: RegistryName,
    runtime: CoreWeak,
    marker: PhantomData<fn() -> T>,
}

impl<T: ?Sized> ManagerServiceHandle<T> {
    pub(crate) fn from_identity(runtime: CoreWeak, identity: RegisteredServiceIdentity) -> Self {
        Self {
            index: identity.index(),
            generation: identity.generation(),
            service: identity.service().clone(),
            runtime,
            marker: PhantomData,
        }
    }

    pub fn service_name(&self) -> &RegistryName {
        &self.service
    }

    fn belongs_to(&self, core: &CoreHandle) -> bool {
        std::ptr::eq(self.runtime.inner.as_ptr(), Arc::as_ptr(&core.inner))
    }

    fn into_identity(self) -> RegisteredServiceIdentity {
        RegisteredServiceIdentity::new(self.index, self.generation, self.service)
    }
}

impl<T: ?Sized> Clone for ManagerServiceHandle<T> {
    fn clone(&self) -> Self {
        Self {
            index: self.index,
            generation: self.generation,
            service: self.service.clone(),
            runtime: self.runtime.clone(),
            marker: PhantomData,
        }
    }
}

impl<T: ?Sized> fmt::Debug for ManagerServiceHandle<T> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ManagerServiceHandle")
            .field("index", &self.index)
            .field("generation", &self.generation)
            .field("service", &self.service)
            .finish()
    }
}

impl<T: ?Sized> PartialEq for ManagerServiceHandle<T> {
    fn eq(&self, other: &Self) -> bool {
        self.index == other.index
            && self.generation == other.generation
            && self.service == other.service
            && std::ptr::eq(self.runtime.inner.as_ptr(), other.runtime.inner.as_ptr())
    }
}

impl<T: ?Sized> Eq for ManagerServiceHandle<T> {}

pub struct RegisteredManagerService<T: ?Sized + Send + Sync + 'static> {
    inner: Arc<T>,
}

impl<T: ?Sized + Send + Sync + 'static> RegisteredManagerService<T> {
    pub fn new(inner: Arc<T>) -> Self {
        Self { inner }
    }

    fn shared(&self) -> Arc<T> {
        Arc::clone(&self.inner)
    }
}

impl<T: ?Sized + Send + Sync + 'static> fmt::Debug for RegisteredManagerService<T> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.debug_struct("RegisteredManagerService").finish()
    }
}

pub trait ManagerServiceResolver {
    fn resolve<T: ?Sized + Send + Sync + 'static>(
        &self,
        handle: ManagerServiceHandle<T>,
    ) -> Result<Arc<T>, CoreError>;
}

pub fn manager_service_handle<T: ?Sized>(
    core: &CoreHandle,
    service_name: &str,
) -> Result<ManagerServiceHandle<T>, CoreError> {
    core.registered_manager_identity(service_name)
        .map(|identity| ManagerServiceHandle::from_identity(core.downgrade(), identity))
}

pub fn resolve_manager_service<T: ?Sized + Send + Sync + 'static>(
    core: &CoreHandle,
    handle: ManagerServiceHandle<T>,
) -> Result<Arc<T>, CoreError> {
    ManagerServiceResolver::resolve(core, handle)
}

impl ManagerServiceResolver for CoreHandle {
    fn resolve<T: ?Sized + Send + Sync + 'static>(
        &self,
        handle: ManagerServiceHandle<T>,
    ) -> Result<Arc<T>, CoreError> {
        if !handle.belongs_to(self) {
            return Err(CoreError::ServiceUnavailable(
                handle.service_name().to_string(),
            ));
        }
        let identity = handle.into_identity();
        let registered =
            self.resolve_registered_manager::<RegisteredManagerService<T>>(&identity)?;
        Ok(registered.shared())
    }
}
