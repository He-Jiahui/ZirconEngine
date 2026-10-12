use std::any::Any;
use std::marker::PhantomData;
use std::sync::Arc;

use crate::core::manager::RegisteredManagerService;

use super::{
    downcast_resolved_service, ensure_service_resolution_available, validate_service_identity,
    CoreError, CoreHandle, CoreWeak, RegisteredServiceIdentity, ServiceCallAdmission,
    ServiceCallGuard, ServiceHandle, ServiceKind, ServiceObject, RESOLUTION_STACK_FRAME_CAPACITY,
};

trait BorrowedProjection<T: ?Sized>: Send + Sync {
    fn accepts(&self, source: &ServiceObject) -> bool;
    fn project<'a>(&self, source: &'a ServiceObject) -> &'a T;
}

struct DirectProjection<S, T: ?Sized> {
    project: for<'a> fn(&'a S) -> &'a T,
}

impl<S: Any + Send + Sync, T: ?Sized + Send + Sync> BorrowedProjection<T>
    for DirectProjection<S, T>
{
    fn accepts(&self, source: &ServiceObject) -> bool {
        source.is::<S>()
    }

    fn project<'a>(&self, source: &'a ServiceObject) -> &'a T {
        (self.project)(source.downcast_ref::<S>().expect("admitted source type"))
    }
}

struct ManagerProjection<S: ?Sized, T: ?Sized> {
    project: for<'a> fn(&'a S) -> &'a T,
}

impl<S: ?Sized + Send + Sync + 'static, T: ?Sized + Send + Sync> BorrowedProjection<T>
    for ManagerProjection<S, T>
{
    fn accepts(&self, source: &ServiceObject) -> bool {
        source.is::<RegisteredManagerService<S>>()
    }

    fn project<'a>(&self, source: &'a ServiceObject) -> &'a T {
        let registered = source
            .downcast_ref::<RegisteredManagerService<S>>()
            .expect("admitted registered manager type");
        (self.project)(registered.borrowed())
    }
}

pub(super) struct TypedProjection<T: ?Sized> {
    project: Box<dyn BorrowedProjection<T>>,
}

// Only the registry owns projection code. This packet never owns the physical
// object; every invocation reacquires its original slot and generation.
struct AliasBinding {
    core: CoreWeak,
    source: RegisteredServiceIdentity,
    target: RegisteredServiceIdentity,
    projection: ServiceObject,
}

pub(super) enum GuardService<T: ?Sized> {
    Owned(Arc<T>),
    Projected {
        source: ServiceObject,
        projection: Arc<TypedProjection<T>>,
    },
}

impl<T: ?Sized> GuardService<T> {
    pub(super) fn borrowed(&self) -> &T {
        match self {
            Self::Owned(service) => service.as_ref(),
            Self::Projected { source, projection } => projection.project.project(source),
        }
    }
}

/// An idle forwarding token. It owns no service, projection, or native code pin.
pub(crate) struct ServiceAliasSource<T: ?Sized> {
    core: CoreWeak,
    source: RegisteredServiceIdentity,
    target: RegisteredServiceIdentity,
    marker: PhantomData<fn() -> T>,
}

impl<T: ?Sized> Clone for ServiceAliasSource<T> {
    fn clone(&self) -> Self {
        Self {
            core: self.core.clone(),
            source: self.source.clone(),
            target: self.target.clone(),
            marker: PhantomData,
        }
    }
}

impl<T: ?Sized> std::fmt::Debug for ServiceAliasSource<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ServiceAliasSource")
            .field("source", &self.source)
            .field("target", &self.target)
            .finish_non_exhaustive()
    }
}

impl<T: ?Sized + Send + Sync + 'static> ServiceAliasSource<T> {
    pub(crate) fn enter(&self) -> Result<ServiceCallGuard<T>, CoreError> {
        let core = self.core.upgrade().ok_or(CoreError::RuntimeUnavailable)?;
        core.enter_alias(&self.target, Some(&self.source))
    }
}

impl CoreHandle {
    /// Binds an exact declared dependency view to the original owned service. The
    /// projection borrows its input; it cannot return an Arc or capture one.
    /// Calls admit both exact generations together. Retain an idle handle,
    /// rather than a call guard, between operations. Alias chains are rejected.
    pub fn register_service_alias<S: Any + Send + Sync, T: ?Sized + Send + Sync + 'static>(
        &self,
        source_name: &str,
        source_kind: ServiceKind,
        target_name: &str,
        target_kind: ServiceKind,
        project: for<'a> fn(&'a S) -> &'a T,
    ) -> Result<ServiceObject, CoreError> {
        let source = self.registered_service_identity(source_name, source_kind)?;
        self.bind_alias(
            source,
            target_name,
            target_kind,
            Box::new(DirectProjection { project }),
        )
    }

    pub(crate) fn register_registered_manager_alias<
        S: ?Sized + Send + Sync + 'static,
        T: ?Sized + Send + Sync + 'static,
    >(
        &self,
        source: RegisteredServiceIdentity,
        target_name: &str,
        project: for<'a> fn(&'a S) -> &'a T,
    ) -> Result<ServiceObject, CoreError> {
        self.bind_alias(
            source,
            target_name,
            ServiceKind::Manager,
            Box::new(ManagerProjection { project }),
        )
    }

    fn bind_alias<T: ?Sized + Send + Sync + 'static>(
        &self,
        source: RegisteredServiceIdentity,
        target_name: &str,
        target_kind: ServiceKind,
        project: Box<dyn BorrowedProjection<T>>,
    ) -> Result<ServiceObject, CoreError> {
        self.ensure_identity_ready(&source)?;
        let target = self.registered_service_identity(target_name, target_kind)?;
        let source_instance = {
            let mut services = self.lock_services();
            let destination = services
                .get(target.service())
                .ok_or_else(|| CoreError::MissingService(target.service().to_string()))?;
            validate_service_identity(
                &target,
                destination.index,
                destination.generation,
                target_kind,
            )?;
            if source == target
                || !destination.dependencies.contains(source.service())
                || destination.lifecycle != super::LifecycleState::Initializing
                || destination.initialization_owner != Some(std::thread::current().id())
                || destination.instance.is_some()
            {
                return Err(CoreError::ServiceUnavailable(target_name.to_owned()));
            }
            let entry = services
                .get_mut(source.service())
                .ok_or_else(|| CoreError::MissingService(source.service().to_string()))?;
            validate_service_identity(
                &source,
                entry.index,
                entry.generation,
                source.service().service_kind(),
            )?;
            let instance = entry
                .instance
                .as_ref()
                .ok_or_else(|| CoreError::ServiceUnavailable(source.service().to_string()))?;
            if Self::is_service_alias(instance) {
                return Err(CoreError::ServiceUnavailable(source.service().to_string()));
            }
            if !project.accepts(instance) {
                return Err(CoreError::ServiceDowncast(source.service().to_string()));
            }
            entry.enter_call(source.service().as_str())?;
            entry.instance.as_ref().unwrap().clone()
        };
        let admitted = ServiceCallAdmission {
            core: self.clone(),
            identity: source.clone(),
            source: None,
        };
        let binding = AliasBinding {
            core: self.downgrade(),
            source,
            target,
            projection: Arc::new(TypedProjection { project }) as ServiceObject,
        };
        // Release the physical object before releasing its construction call.
        drop(source_instance);
        drop(admitted);
        Ok(Arc::new(binding) as ServiceObject)
    }

    pub(crate) fn is_service_alias(instance: &ServiceObject) -> bool {
        instance.is::<AliasBinding>()
    }

    pub(crate) fn resolve_alias_source<T: ?Sized + Send + Sync + 'static>(
        &self,
        name: &str,
        kind: ServiceKind,
    ) -> Result<ServiceAliasSource<T>, CoreError> {
        // Normal resolution performs the existing lazy initialization protocol.
        self.resolve_named_service(name, Some(kind))?;
        let identity = self.registered_service_identity(name, kind)?;
        let services = self.lock_services();
        let entry = services
            .get(identity.service())
            .ok_or_else(|| CoreError::MissingService(name.to_owned()))?;
        validate_service_identity(&identity, entry.index, entry.generation, kind)?;
        let binding = entry
            .instance
            .as_ref()
            .and_then(|value| value.downcast_ref::<AliasBinding>())
            .ok_or_else(|| CoreError::ServiceDowncast(name.to_owned()))?;
        self.validate_alias::<T>(binding, &identity)?;
        Ok(ServiceAliasSource {
            core: self.downgrade(),
            source: binding.source.clone(),
            target: identity,
            marker: PhantomData,
        })
    }

    pub(crate) fn try_resolve_manager_alias<T: ?Sized + Send + Sync + 'static>(
        &self,
        identity: &RegisteredServiceIdentity,
    ) -> Result<Option<ServiceCallGuard<T>>, CoreError> {
        self.ensure_identity_ready(identity)?;
        let is_alias = {
            let services = self.lock_services();
            let entry = services
                .get(identity.service())
                .ok_or_else(|| CoreError::MissingService(identity.service().to_string()))?;
            validate_service_identity(
                identity,
                entry.index,
                entry.generation,
                ServiceKind::Manager,
            )?;
            entry.instance.as_ref().is_some_and(Self::is_service_alias)
        };
        if is_alias {
            self.enter_alias(identity, None).map(Some)
        } else {
            Ok(None)
        }
    }

    fn validate_alias<T: ?Sized + Send + Sync + 'static>(
        &self,
        binding: &AliasBinding,
        identity: &RegisteredServiceIdentity,
    ) -> Result<(), CoreError> {
        if !std::ptr::eq(binding.core.inner.as_ptr(), Arc::as_ptr(&self.inner))
            || binding.target != *identity
        {
            return Err(CoreError::ServiceUnavailable(
                identity.service().to_string(),
            ));
        }
        if !binding.projection.is::<TypedProjection<T>>() {
            return Err(CoreError::ServiceDowncast(identity.service().to_string()));
        }
        Ok(())
    }

    fn enter_alias<T: ?Sized + Send + Sync + 'static>(
        &self,
        identity: &RegisteredServiceIdentity,
        expected_source: Option<&RegisteredServiceIdentity>,
    ) -> Result<ServiceCallGuard<T>, CoreError> {
        let (source, source_identity, projection) = {
            let mut services = self.lock_services();
            let target = services
                .get(identity.service())
                .ok_or_else(|| CoreError::MissingService(identity.service().to_string()))?;
            validate_service_identity(
                identity,
                target.index,
                target.generation,
                identity.service().service_kind(),
            )?;
            let binding = target
                .instance
                .as_ref()
                .and_then(|value| value.downcast_ref::<AliasBinding>())
                .ok_or_else(|| CoreError::ServiceDowncast(identity.service().to_string()))?;
            self.validate_alias::<T>(binding, identity)?;
            if expected_source.is_some_and(|expected| expected != &binding.source) {
                return Err(CoreError::ServiceUnavailable(
                    identity.service().to_string(),
                ));
            }
            let source_identity = binding.source.clone();
            let projection = Arc::downcast::<TypedProjection<T>>(binding.projection.clone())
                .map_err(|_| CoreError::ServiceDowncast(identity.service().to_string()))?;
            let target_count = target.next_call_count(identity.service().as_str())?;
            let owner = services
                .get(source_identity.service())
                .ok_or_else(|| CoreError::MissingService(source_identity.service().to_string()))?;
            validate_service_identity(
                &source_identity,
                owner.index,
                owner.generation,
                source_identity.service().service_kind(),
            )?;
            let source_count = owner.next_call_count(source_identity.service().as_str())?;
            let source = owner.instance.as_ref().ok_or_else(|| {
                CoreError::ServiceUnavailable(source_identity.service().to_string())
            })?;
            if Self::is_service_alias(source) {
                return Err(CoreError::ServiceUnavailable(
                    source_identity.service().to_string(),
                ));
            }
            if !projection.project.accepts(source) {
                return Err(CoreError::ServiceDowncast(
                    source_identity.service().to_string(),
                ));
            }
            let source = source.clone();
            // Neither count changes until both exact slots, types, and capacity pass.
            services
                .get_mut(identity.service())
                .unwrap()
                .in_flight_calls = target_count;
            if source_identity != *identity {
                services
                    .get_mut(source_identity.service())
                    .unwrap()
                    .in_flight_calls = source_count;
            }
            (source, source_identity, projection)
        };
        Ok(ServiceCallGuard {
            service: GuardService::Projected { source, projection },
            admission: ServiceCallAdmission {
                core: self.clone(),
                identity: identity.clone(),
                source: (source_identity != *identity).then_some(source_identity),
            },
        })
    }

    pub(super) fn enter_service<T: Any + Send + Sync>(
        &self,
        identity: &RegisteredServiceIdentity,
    ) -> Result<ServiceCallGuard<T>, CoreError> {
        self.ensure_identity_ready(identity)?;
        let service = {
            let mut services = self.lock_services();
            let entry = services
                .get_mut(identity.service())
                .ok_or_else(|| CoreError::MissingService(identity.service().to_string()))?;
            validate_service_identity(
                identity,
                entry.index,
                entry.generation,
                identity.service().service_kind(),
            )?;
            let instance = entry
                .instance
                .as_ref()
                .ok_or_else(|| CoreError::ServiceUnavailable(identity.service().to_string()))?;
            if Self::is_service_alias(instance) {
                drop(services);
                return self.enter_alias(identity, None);
            }
            if !instance.is::<T>() {
                return Err(CoreError::ServiceDowncast(identity.service().to_string()));
            }
            let service = downcast_resolved_service(identity.service().as_str(), instance.clone())?;
            entry.enter_call(identity.service().as_str())?;
            service
        };
        Ok(ServiceCallGuard {
            service: GuardService::Owned(service),
            admission: ServiceCallAdmission {
                core: self.clone(),
                identity: identity.clone(),
                source: None,
            },
        })
    }

    pub(super) fn ensure_identity_ready(
        &self,
        identity: &RegisteredServiceIdentity,
    ) -> Result<(), CoreError> {
        loop {
            let ready = {
                let services = self.lock_services();
                let entry = services
                    .get(identity.service())
                    .ok_or_else(|| CoreError::MissingService(identity.service().to_string()))?;
                validate_service_identity(
                    identity,
                    entry.index,
                    entry.generation,
                    identity.service().service_kind(),
                )?;
                ensure_service_resolution_available(identity.service().as_str(), entry.lifecycle)?;
                entry.instance.is_some()
            };
            if ready {
                return Ok(());
            }
            let mut stack = Vec::with_capacity(RESOLUTION_STACK_FRAME_CAPACITY);
            self.resolve_existing_service_inner(identity.service(), &mut stack)?;
        }
    }

    pub(super) fn checked_service_handle<T: Any + Send + Sync>(
        &self,
        identity: RegisteredServiceIdentity,
    ) -> Result<ServiceHandle<T>, CoreError> {
        self.ensure_identity_ready(&identity)?;
        let services = self.lock_services();
        let entry = services
            .get(identity.service())
            .ok_or_else(|| CoreError::MissingService(identity.service().to_string()))?;
        validate_service_identity(
            &identity,
            entry.index,
            entry.generation,
            identity.service().service_kind(),
        )?;
        let instance = entry
            .instance
            .as_ref()
            .ok_or_else(|| CoreError::ServiceUnavailable(identity.service().to_string()))?;
        if let Some(binding) = instance.downcast_ref::<AliasBinding>() {
            self.validate_alias::<T>(binding, &identity)?;
        } else if !instance.is::<T>() {
            return Err(CoreError::ServiceDowncast(identity.service().to_string()));
        }
        Ok(ServiceHandle::new(self.downgrade(), identity))
    }
}
