use std::fmt;
use std::sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard};

use zircon_runtime::scene::{EntityId, TypeRegistry};
use zircon_runtime::script::{
    VmReflectionCatalog, VmReflectionRegistrySnapshot, VmReflectionWorldAccess,
};
use zircon_runtime_interface::reflect::ReflectedValue;

use super::ReflectionHostError;
use crate::ScriptCallTable;

/// Package-owned reflection bridge captured by native backend callbacks.
#[derive(Clone)]
pub struct ReflectionHostModule {
    table: Arc<RwLock<Option<ScriptCallTable>>>,
    catalog: Arc<RwLock<Option<VmReflectionCatalog>>>,
    runtime_world_access: VmReflectionWorldAccess,
}

impl ReflectionHostModule {
    /// Constructs the package-owned reflection bridge from the runtime-issued access token.
    pub fn new(runtime_world_access: VmReflectionWorldAccess) -> Self {
        Self {
            table: Arc::new(RwLock::new(None)),
            catalog: Arc::new(RwLock::new(None)),
            runtime_world_access,
        }
    }

    /// Rebuilds the immutable dense table from the canonical public host and VM registry.
    pub fn install_type_registry(
        &self,
        registry: &TypeRegistry,
    ) -> Result<(), ReflectionHostError> {
        *self.table_write() = Some(ScriptCallTable::compile(&registry)?);
        *self.catalog_write() = None;
        Ok(())
    }

    /// Rebuilds the dense table with a fail-closed process-wide catalog revision guard.
    pub fn install_registry_snapshot(
        &self,
        snapshot: &VmReflectionRegistrySnapshot,
        catalog: &VmReflectionCatalog,
    ) -> Result<(), ReflectionHostError> {
        *self.table_write() = Some(ScriptCallTable::compile_snapshot(snapshot)?);
        *self.catalog_write() = Some(catalog.clone());
        Ok(())
    }

    /// Resolves a public reflected field once and returns its opaque numeric ABI token.
    pub fn resolve(&self, type_path: &str, member_name: &str) -> Result<u64, ReflectionHostError> {
        self.refresh_stale_table()?;
        self.with_table(|table| Ok(table.resolve(type_path, member_name)?.token()))
    }

    /// Reads a reflected field in the active script scene context using only numeric slots.
    pub fn read(
        &self,
        token: u64,
        entity: EntityId,
    ) -> Result<ReflectedValue, ReflectionHostError> {
        self.runtime_world_access
            .with_reflection_operation(|operation| {
                operation.with_world(|world| {
                    self.with_table(|table| {
                        table.read_token(token, world, entity).map_err(Into::into)
                    })
                })
            })
            .ok_or_else(|| {
                ReflectionHostError::RuntimeContext(
                    "script runtime reflection operation is not active".to_string(),
                )
            })?
    }

    /// Writes a reflected field in the active script scene context using only numeric slots.
    pub fn write(
        &self,
        token: u64,
        entity: EntityId,
        value: ReflectedValue,
    ) -> Result<bool, ReflectionHostError> {
        self.runtime_world_access
            .with_reflection_operation(|operation| {
                operation.with_world_mut(|world| {
                    self.with_table(|table| {
                        table
                            .write_token(token, world, entity, value)
                            .map_err(Into::into)
                    })
                })
            })
            .ok_or_else(|| {
                ReflectionHostError::RuntimeContext(
                    "script runtime reflection operation is not active".to_string(),
                )
            })?
    }

    /// Encodes a numeric reflected read for the string-only ZrVM native ABI.
    pub fn read_json(&self, token: u64, entity: EntityId) -> Result<String, ReflectionHostError> {
        Ok(serde_json::to_string(&self.read(token, entity)?)?)
    }

    /// Decodes and writes one reflected value received through the ZrVM native ABI.
    pub fn write_json(
        &self,
        token: u64,
        entity: EntityId,
        value_json: &str,
    ) -> Result<bool, ReflectionHostError> {
        let value = serde_json::from_str(value_json)?;
        self.write(token, entity, value)
    }

    /// Returns the number of package-loading name resolutions performed so far.
    pub fn resolution_count(&self) -> usize {
        self.table_read()
            .as_ref()
            .map(ScriptCallTable::resolution_count)
            .unwrap_or(0)
    }

    fn with_table<R>(
        &self,
        operation: impl FnOnce(&ScriptCallTable) -> Result<R, ReflectionHostError>,
    ) -> Result<R, ReflectionHostError> {
        let guard = self.table_read();
        let table = guard.as_ref().ok_or(ReflectionHostError::Uninitialized)?;
        operation(table)
    }

    fn refresh_stale_table(&self) -> Result<(), ReflectionHostError> {
        let mut table = self.table_write();
        let Some(installed) = table.as_ref() else {
            return Err(ReflectionHostError::Uninitialized);
        };
        if installed.can_resolve_names() {
            return Ok(());
        }
        let catalog = self
            .catalog_read()
            .as_ref()
            .cloned()
            .ok_or(ReflectionHostError::Uninitialized)?;
        let snapshot = catalog.current_snapshot()?;
        *table = Some(ScriptCallTable::compile_snapshot(&snapshot)?);
        Ok(())
    }

    fn table_read(&self) -> RwLockReadGuard<'_, Option<ScriptCallTable>> {
        self.table
            .read()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    fn table_write(&self) -> RwLockWriteGuard<'_, Option<ScriptCallTable>> {
        self.table
            .write()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    fn catalog_read(&self) -> RwLockReadGuard<'_, Option<VmReflectionCatalog>> {
        self.catalog
            .read()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    fn catalog_write(&self) -> RwLockWriteGuard<'_, Option<VmReflectionCatalog>> {
        self.catalog
            .write()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}

impl fmt::Debug for ReflectionHostModule {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ReflectionHostModule")
            .field("initialized", &self.table_read().is_some())
            .field(
                "catalog_revision",
                &self
                    .table_read()
                    .as_ref()
                    .and_then(ScriptCallTable::catalog_revision),
            )
            .field("resolution_count", &self.resolution_count())
            .finish()
    }
}
