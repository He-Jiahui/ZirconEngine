use std::fmt;
use std::sync::{Arc, Mutex, MutexGuard};

use crate::core::framework::bridge::{BridgeError, PluginInterface};

use super::{FrozenBridgeTable, WeakBridge};

/// Cloneable consumer-side bridge handle bound after the runtime catalog has
/// merged and finalized every plugin contribution.
pub struct BridgeImport<T: ?Sized> {
    binding: Arc<Mutex<Option<WeakBridge<T>>>>,
}

impl<T: ?Sized> Clone for BridgeImport<T> {
    fn clone(&self) -> Self {
        Self {
            binding: Arc::clone(&self.binding),
        }
    }
}

impl<T: ?Sized> fmt::Debug for BridgeImport<T> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let bound = self
            .binding
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .is_some();
        formatter
            .debug_struct("BridgeImport")
            .field("bound", &bound)
            .finish()
    }
}

impl<T> BridgeImport<T>
where
    T: PluginInterface + ?Sized,
{
    pub(crate) fn new() -> (Self, InterfaceImport) {
        let binding = Arc::new(Mutex::new(None));
        let imported = Self {
            binding: Arc::clone(&binding),
        };
        let erased = InterfaceImport {
            interface_id: T::INTERFACE_ID.to_string(),
            update: Arc::new(move |table| {
                let mut binding = binding
                    .lock()
                    .unwrap_or_else(|poisoned| poisoned.into_inner());
                *binding = table.map(FrozenBridgeTable::resolve_weak::<T>);
            }),
        };
        (imported, erased)
    }

    pub fn call<R>(&self, callback: impl FnOnce(&T) -> R) -> Result<R, BridgeError> {
        let bridge = self
            .lock_binding()
            .as_ref()
            .cloned()
            .ok_or(BridgeError::Absent)?;
        bridge.call(callback)
    }

    pub fn is_enabled(&self) -> bool {
        self.lock_binding()
            .as_ref()
            .is_some_and(WeakBridge::is_enabled)
    }

    fn lock_binding(&self) -> MutexGuard<'_, Option<WeakBridge<T>>> {
        self.binding
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}

#[derive(Clone)]
pub(crate) struct InterfaceImport {
    interface_id: String,
    update: Arc<dyn Fn(Option<&FrozenBridgeTable>) + Send + Sync>,
}

impl InterfaceImport {
    pub(crate) fn interface_id(&self) -> &str {
        &self.interface_id
    }

    pub(crate) fn bind(&self, table: &FrozenBridgeTable) {
        (self.update)(Some(table));
    }

    pub(crate) fn unbind(&self) {
        (self.update)(None);
    }
}

impl fmt::Debug for InterfaceImport {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("InterfaceImport")
            .field("interface_id", &self.interface_id)
            .finish_non_exhaustive()
    }
}
