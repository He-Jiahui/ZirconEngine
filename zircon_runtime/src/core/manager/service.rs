use std::fmt;
use std::marker::PhantomData;
use std::sync::Arc;

use crate::core::runtime::{CoreHandle, RegisteredServiceIdentity, RegistryName};
use crate::core::CoreError;

pub struct ManagerServiceHandle<T: ?Sized> {
    pub index: u32,
    pub generation: u32,
    pub service: RegistryName,
    marker: PhantomData<fn() -> T>,
}

impl<T: ?Sized> ManagerServiceHandle<T> {
    pub(crate) fn from_identity(identity: RegisteredServiceIdentity) -> Self {
        Self {
            index: identity.index(),
            generation: identity.generation(),
            service: identity.service().clone(),
            marker: PhantomData,
        }
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
        .map(ManagerServiceHandle::from_identity)
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
        let identity = handle.into_identity();
        let registered =
            self.resolve_registered_manager::<RegisteredManagerService<T>>(&identity)?;
        Ok(registered.shared())
    }
}
