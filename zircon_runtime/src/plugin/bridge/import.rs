use std::fmt;
use std::sync::Arc;

use arc_swap::ArcSwapOption;

use crate::core::framework::bridge::{BridgeError, PluginInterface};

use super::{FrozenBridgeTable, WeakBridge};

/// Cloneable consumer-side bridge handle bound after the runtime catalog has
/// merged and finalized every plugin contribution.
pub struct BridgeImport<T: ?Sized> {
    binding: Arc<ArcSwapOption<WeakBridge<T>>>,
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
        let bound = self.binding.load().is_some();
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
        let binding = Arc::new(ArcSwapOption::empty());
        let imported = Self {
            binding: Arc::clone(&binding),
        };
        let erased = InterfaceImport {
            interface_id: T::INTERFACE_ID.to_string(),
            update: Arc::new(move |table| {
                binding.store(
                    table
                        .map(FrozenBridgeTable::resolve_weak::<T>)
                        .map(Arc::new),
                );
            }),
        };
        (imported, erased)
    }

    pub fn call<R>(&self, callback: impl FnOnce(&T) -> R) -> Result<R, BridgeError> {
        let binding = self.binding.load();
        let bridge = binding.as_ref().ok_or(BridgeError::Absent)?;
        bridge.call(callback)
    }

    pub fn is_enabled(&self) -> bool {
        self.binding
            .load()
            .as_deref()
            .is_some_and(WeakBridge::is_enabled)
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
