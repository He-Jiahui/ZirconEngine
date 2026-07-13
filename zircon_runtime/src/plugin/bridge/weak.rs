use std::cell::RefCell;
use std::ops::Deref;
use std::sync::Arc;

use crate::core::framework::bridge::{BridgeError, InterfaceSlot, PluginInterface};

use super::table::FrozenBridgeTable;

pub struct WeakBridge<T: ?Sized> {
    table: FrozenBridgeTable,
    slot: Option<InterfaceSlot>,
    cached: RefCell<Option<(u32, Arc<T>)>>,
}

impl<T> WeakBridge<T>
where
    T: PluginInterface + ?Sized,
{
    pub(crate) fn new(table: FrozenBridgeTable, slot: Option<InterfaceSlot>) -> Self {
        Self {
            table,
            slot,
            cached: RefCell::new(None),
        }
    }

    pub fn owned(table: FrozenBridgeTable) -> Self {
        let slot = table.resolve_slot(T::INTERFACE_ID);
        Self::new(table, slot)
    }

    pub fn call<R>(&self, f: impl FnOnce(&T) -> R) -> Result<R, BridgeError> {
        match self.provider_with_slot() {
            Ok((slot, provider)) => {
                self.table.record_enabled_call(slot);
                Ok(f(&provider))
            }
            Err((Some(slot), error)) => {
                self.table.record_not_enabled_call(slot);
                Err(error)
            }
            Err((None, error)) => Err(error),
        }
    }

    pub fn pin(&self) -> Result<BridgeGuard<T>, BridgeError> {
        match self.provider_with_slot() {
            Ok((slot, target)) => {
                self.table.record_enabled_call(slot);
                Ok(BridgeGuard { target })
            }
            Err((Some(slot), error)) => {
                self.table.record_not_enabled_call(slot);
                Err(error)
            }
            Err((None, error)) => Err(error),
        }
    }

    pub fn is_enabled(&self) -> bool {
        self.provider().is_ok()
    }

    fn provider(&self) -> Result<Arc<T>, BridgeError> {
        self.provider_with_slot()
            .map(|(_, provider)| provider)
            .map_err(|(_, error)| error)
    }

    fn provider_with_slot(
        &self,
    ) -> Result<(InterfaceSlot, Arc<T>), (Option<InterfaceSlot>, BridgeError)> {
        let slot = self.slot.ok_or((None, BridgeError::Absent))?;
        let generation = self
            .table
            .entry(slot)
            .ok_or((Some(slot), BridgeError::Absent))?
            .generation();
        if let Some((cached_generation, cached)) = self.cached.borrow().as_ref() {
            if *cached_generation == generation && generation % 2 == 0 {
                return Ok((slot, cached.clone()));
            }
        }

        let (generation, provider) = self
            .table
            .provider::<T>(slot)
            .map_err(|error| (Some(slot), error))?;
        *self.cached.borrow_mut() = Some((generation, provider.clone()));
        Ok((slot, provider))
    }
}

pub struct BridgeGuard<T: ?Sized> {
    target: Arc<T>,
}

impl<T: ?Sized> Deref for BridgeGuard<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.target
    }
}
