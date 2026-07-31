use std::fmt;
use std::sync::Arc;

pub(crate) struct RuntimeProviderRegistration<P: ?Sized> {
    debug_name: &'static str,
    provider_id: String,
    priority: i32,
    provider: Arc<P>,
}

impl<P: ?Sized> Clone for RuntimeProviderRegistration<P> {
    fn clone(&self) -> Self {
        Self {
            debug_name: self.debug_name,
            provider_id: self.provider_id.clone(),
            priority: self.priority,
            provider: Arc::clone(&self.provider),
        }
    }
}

impl<P: ?Sized> RuntimeProviderRegistration<P> {
    pub(crate) fn new(
        debug_name: &'static str,
        provider_id: impl Into<String>,
        provider: Arc<P>,
    ) -> Self {
        Self {
            debug_name,
            provider_id: provider_id.into(),
            priority: 0,
            provider,
        }
    }

    pub(crate) fn provider_id(&self) -> &str {
        &self.provider_id
    }

    pub(crate) const fn priority(&self) -> i32 {
        self.priority
    }

    pub(crate) fn with_priority(mut self, priority: i32) -> Self {
        self.priority = priority;
        self
    }

    pub(crate) fn provider(&self) -> &P {
        self.provider.as_ref()
    }

    pub(crate) fn provider_arc(&self) -> Arc<P> {
        Arc::clone(&self.provider)
    }
}

impl<P: ?Sized> fmt::Debug for RuntimeProviderRegistration<P> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct(self.debug_name)
            .field("provider_id", &self.provider_id)
            .field("priority", &self.priority)
            .finish_non_exhaustive()
    }
}

macro_rules! define_runtime_provider_registration {
    (
        $(#[$meta:meta])*
        $vis:vis struct $name:ident for $provider_trait:path;
    ) => {
        $(#[$meta])*
        #[derive(Clone)]
        $vis struct $name {
            registration: RuntimeProviderRegistration<dyn $provider_trait>,
        }

        impl $name {
            pub fn new(
                provider_id: impl Into<String>,
                provider: std::sync::Arc<dyn $provider_trait>,
            ) -> Self {
                Self {
                    registration: RuntimeProviderRegistration::new(
                        stringify!($name),
                        provider_id,
                        provider,
                    ),
                }
            }

            pub fn provider_id(&self) -> &str {
                self.registration.provider_id()
            }

            pub const fn priority(&self) -> i32 {
                self.registration.priority()
            }

            pub fn with_priority(mut self, priority: i32) -> Self {
                self.registration = self.registration.with_priority(priority);
                self
            }

            pub fn provider(&self) -> &dyn $provider_trait {
                self.registration.provider()
            }

            pub(crate) fn provider_arc(&self) -> std::sync::Arc<dyn $provider_trait> {
                self.registration.provider_arc()
            }
        }

        impl std::fmt::Debug for $name {
            fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                std::fmt::Debug::fmt(&self.registration, formatter)
            }
        }
    };
}

pub(crate) use define_runtime_provider_registration;
