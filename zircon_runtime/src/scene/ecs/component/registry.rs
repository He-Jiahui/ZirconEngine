use std::any::{type_name, TypeId};
use std::collections::HashMap;
use std::fmt;

use crate::scene::ecs::StorageType;

use super::id::ComponentId;
use super::marker::Component;
use super::table_column::TableColumnLayout;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ComponentDescriptor {
    pub id: ComponentId,
    pub type_name: String,
    pub storage_type: StorageType,
    pub source: ComponentDescriptorSource,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ComponentDescriptorSource {
    RustType { type_id: TypeId },
    DynamicPlugin { component_type_id: String },
}

/// A descriptor-owned capability carried out of an isolated World. Its
/// numeric component id remains local to that World and is never reused by a
/// commit target.
#[derive(Clone)]
pub(crate) struct TransferredComponentDescriptor {
    descriptor: ComponentDescriptor,
    table_column_layout: Option<TableColumnLayout>,
}

/// A transaction-local import log for descriptors transferred out of an
/// isolated preflight World. It carries only affected descriptors and never
/// mutates the target registry until publication is known to be infallible.
pub(crate) struct PreflightedTransferredDescriptorImports {
    base_descriptor_count: usize,
    pending: Vec<PendingTransferredDescriptor>,
    rust_ids_by_type_id: HashMap<TypeId, ComponentId>,
    dynamic_ids_by_type_id: HashMap<String, ComponentId>,
    reused_descriptor_resolves: usize,
}

struct PendingTransferredDescriptor {
    descriptor: ComponentDescriptor,
    table_column_layout: Option<TableColumnLayout>,
}

impl PreflightedTransferredDescriptorImports {
    fn new(base_descriptor_count: usize) -> Self {
        Self {
            base_descriptor_count,
            pending: Vec::new(),
            rust_ids_by_type_id: HashMap::new(),
            dynamic_ids_by_type_id: HashMap::new(),
            reused_descriptor_resolves: 0,
        }
    }

    pub(crate) fn imported_descriptor_count(&self) -> usize {
        self.pending.len()
    }

    pub(crate) fn reused_descriptor_resolve_count(&self) -> usize {
        self.reused_descriptor_resolves
    }

    fn component_id_for_source(&self, source: &ComponentDescriptorSource) -> Option<ComponentId> {
        match source {
            ComponentDescriptorSource::RustType { type_id } => {
                self.rust_ids_by_type_id.get(type_id).copied()
            }
            ComponentDescriptorSource::DynamicPlugin { component_type_id } => {
                self.dynamic_ids_by_type_id.get(component_type_id).copied()
            }
        }
    }

    fn descriptor(&self, component_id: ComponentId) -> Option<&ComponentDescriptor> {
        component_id
            .index()
            .checked_sub(self.base_descriptor_count)
            .and_then(|index| self.pending.get(index))
            .map(|pending| &pending.descriptor)
    }

    fn stage(&mut self, transfer: &TransferredComponentDescriptor) -> ComponentId {
        let component_id = ComponentId::new(self.base_descriptor_count + self.pending.len());
        let mut descriptor = transfer.descriptor.clone();
        descriptor.id = component_id;
        match &descriptor.source {
            ComponentDescriptorSource::RustType { type_id } => {
                let previous = self.rust_ids_by_type_id.insert(*type_id, component_id);
                debug_assert!(previous.is_none());
            }
            ComponentDescriptorSource::DynamicPlugin { component_type_id } => {
                let previous = self
                    .dynamic_ids_by_type_id
                    .insert(component_type_id.clone(), component_id);
                debug_assert!(previous.is_none());
            }
        }
        self.pending.push(PendingTransferredDescriptor {
            descriptor,
            table_column_layout: transfer.table_column_layout.clone(),
        });
        component_id
    }

    fn stage_dynamic(&mut self, component_type_id: &str) -> ComponentId {
        let component_id = ComponentId::new(self.base_descriptor_count + self.pending.len());
        let component_type_id = component_type_id.to_string();
        let previous = self
            .dynamic_ids_by_type_id
            .insert(component_type_id.clone(), component_id);
        debug_assert!(previous.is_none());
        self.pending.push(PendingTransferredDescriptor {
            descriptor: ComponentDescriptor {
                id: component_id,
                type_name: component_type_id.clone(),
                storage_type: StorageType::SparseSet,
                source: ComponentDescriptorSource::DynamicPlugin { component_type_id },
            },
            table_column_layout: None,
        });
        component_id
    }
}

#[derive(Clone, Default, PartialEq, Eq)]
pub struct ComponentRegistry {
    descriptors: Vec<ComponentDescriptor>,
    rust_ids_by_type_id: HashMap<TypeId, ComponentId>,
    dynamic_ids_by_type_id: HashMap<String, ComponentId>,
    table_column_layouts: HashMap<ComponentId, TableColumnLayout>,
}

impl ComponentRegistry {
    pub(crate) fn generation(&self) -> u64 {
        self.descriptors.len() as u64
    }

    pub fn component_id<T>(&mut self) -> ComponentId
    where
        T: Component,
    {
        let type_id = TypeId::of::<T>();
        if let Some(id) = self.rust_ids_by_type_id.get(&type_id).copied() {
            return id;
        }
        let storage_type = T::STORAGE_TYPE;
        let id = self.insert_descriptor(
            type_name::<T>().to_string(),
            storage_type,
            ComponentDescriptorSource::RustType { type_id },
        );
        self.rust_ids_by_type_id.insert(type_id, id);
        if storage_type == StorageType::Table {
            self.table_column_layouts
                .insert(id, TableColumnLayout::of::<T>());
        }
        id
    }

    pub fn dynamic_component_id(&mut self, component_type_id: &str) -> ComponentId {
        if let Some(id) = self.dynamic_ids_by_type_id.get(component_type_id).copied() {
            return id;
        }
        let id = self.insert_descriptor(
            component_type_id.to_string(),
            StorageType::SparseSet,
            ComponentDescriptorSource::DynamicPlugin {
                component_type_id: component_type_id.to_string(),
            },
        );
        self.dynamic_ids_by_type_id
            .insert(component_type_id.to_string(), id);
        id
    }

    pub fn registered_component_id<T>(&self) -> Option<ComponentId>
    where
        T: Component,
    {
        self.rust_ids_by_type_id.get(&TypeId::of::<T>()).copied()
    }

    pub fn registered_dynamic_component_id(&self, component_type_id: &str) -> Option<ComponentId> {
        self.dynamic_ids_by_type_id.get(component_type_id).copied()
    }

    pub fn descriptor(&self, id: ComponentId) -> Option<&ComponentDescriptor> {
        self.descriptors.get(id.index())
    }

    pub(crate) fn rust_type_for_id(&self, id: ComponentId) -> Option<(TypeId, &str)> {
        let descriptor = self.descriptor(id)?;
        match &descriptor.source {
            ComponentDescriptorSource::RustType { type_id } => {
                Some((*type_id, descriptor.type_name.as_str()))
            }
            ComponentDescriptorSource::DynamicPlugin { .. } => None,
        }
    }

    pub fn descriptors(&self) -> &[ComponentDescriptor] {
        &self.descriptors
    }

    pub(crate) fn table_column_layout(&self, id: ComponentId) -> Option<&TableColumnLayout> {
        self.table_column_layouts.get(&id)
    }

    pub(crate) fn table_column_layouts_for_ids(
        &self,
        component_ids: &[ComponentId],
    ) -> Option<Vec<(ComponentId, TableColumnLayout)>> {
        let mut layouts = Vec::with_capacity(component_ids.len());
        for component_id in component_ids {
            let layout = self.table_column_layout(*component_id)?.clone();
            layouts.push((*component_id, layout));
        }
        Some(layouts)
    }

    pub(crate) fn transferred_descriptor(
        &self,
        component_id: ComponentId,
    ) -> Option<TransferredComponentDescriptor> {
        let descriptor = self.descriptor(component_id)?.clone();
        let table_column_layout = match descriptor.storage_type {
            StorageType::Table => Some(self.table_column_layout(component_id)?.clone()),
            StorageType::SparseSet => None,
        };
        Some(TransferredComponentDescriptor {
            descriptor,
            table_column_layout,
        })
    }

    /// Starts a transaction-local descriptor import. The returned delta only
    /// owns descriptors absent from this registry, so preflight work stays
    /// proportional to the affected component types.
    pub(crate) fn begin_transferred_descriptor_imports(
        &self,
    ) -> PreflightedTransferredDescriptorImports {
        PreflightedTransferredDescriptorImports::new(self.descriptors.len())
    }

    /// Resolves a transferred descriptor against the live base registry plus
    /// a transaction-local delta without publishing either a descriptor or a
    /// table layout into the target.
    pub(crate) fn preflight_transferred_descriptor_import(
        &self,
        imports: &mut PreflightedTransferredDescriptorImports,
        transfer: &TransferredComponentDescriptor,
    ) -> Option<ComponentId> {
        if let Some(component_id) =
            self.component_id_for_transferred_source(&transfer.descriptor.source)
        {
            let descriptor = self.descriptor(component_id)?;
            if descriptor_matches_transfer(descriptor, &transfer.descriptor) {
                imports.reused_descriptor_resolves += 1;
                return Some(component_id);
            }
            return None;
        }

        if let Some(component_id) = imports.component_id_for_source(&transfer.descriptor.source) {
            let descriptor = imports.descriptor(component_id)?;
            if descriptor_matches_transfer(descriptor, &transfer.descriptor) {
                imports.reused_descriptor_resolves += 1;
                return Some(component_id);
            }
            return None;
        }

        if transfer.descriptor.storage_type == StorageType::Table
            && transfer.table_column_layout.is_none()
        {
            return None;
        }
        Some(imports.stage(transfer))
    }

    /// Reserves the target-local id for a dynamic descriptor inside the same
    /// transaction delta used by transferred component rows. This does not
    /// mutate the registry, and a later transferred row for the same type will
    /// resolve to this exact id.
    pub(crate) fn preflight_dynamic_descriptor_import(
        &self,
        imports: &mut PreflightedTransferredDescriptorImports,
        component_type_id: &str,
    ) -> ComponentId {
        if let Some(component_id) = self.registered_dynamic_component_id(component_type_id) {
            imports.reused_descriptor_resolves += 1;
            return component_id;
        }
        if let Some(component_id) = imports
            .dynamic_ids_by_type_id
            .get(component_type_id)
            .copied()
        {
            imports.reused_descriptor_resolves += 1;
            return component_id;
        }
        imports.stage_dynamic(component_type_id)
    }

    /// Publishes a descriptor import that was completely preflighted before
    /// any target entity or storage row became visible.
    pub(crate) fn publish_preflighted_transferred_descriptor_imports(
        &mut self,
        imports: PreflightedTransferredDescriptorImports,
    ) {
        debug_assert_eq!(self.descriptors.len(), imports.base_descriptor_count);
        self.descriptors.reserve(imports.pending.len());
        self.rust_ids_by_type_id
            .reserve(imports.rust_ids_by_type_id.len());
        self.dynamic_ids_by_type_id
            .reserve(imports.dynamic_ids_by_type_id.len());
        self.table_column_layouts.reserve(imports.pending.len());
        for pending in imports.pending {
            let component_id = pending.descriptor.id;
            debug_assert_eq!(component_id.index(), self.descriptors.len());
            match &pending.descriptor.source {
                ComponentDescriptorSource::RustType { type_id } => {
                    let previous = self.rust_ids_by_type_id.insert(*type_id, component_id);
                    debug_assert!(previous.is_none());
                }
                ComponentDescriptorSource::DynamicPlugin { component_type_id } => {
                    let previous = self
                        .dynamic_ids_by_type_id
                        .insert(component_type_id.clone(), component_id);
                    debug_assert!(previous.is_none());
                }
            }
            if let Some(layout) = pending.table_column_layout {
                let previous = self.table_column_layouts.insert(component_id, layout);
                debug_assert!(previous.is_none());
            }
            self.descriptors.push(pending.descriptor);
        }
    }

    fn component_id_for_transferred_source(
        &self,
        source: &ComponentDescriptorSource,
    ) -> Option<ComponentId> {
        match source {
            ComponentDescriptorSource::RustType { type_id } => {
                self.rust_ids_by_type_id.get(type_id).copied()
            }
            ComponentDescriptorSource::DynamicPlugin { component_type_id } => {
                self.dynamic_ids_by_type_id.get(component_type_id).copied()
            }
        }
    }

    fn insert_descriptor(
        &mut self,
        type_name: String,
        storage_type: StorageType,
        source: ComponentDescriptorSource,
    ) -> ComponentId {
        let id = ComponentId::new(self.descriptors.len());
        self.descriptors.push(ComponentDescriptor {
            id,
            type_name,
            storage_type,
            source,
        });
        id
    }
}

fn descriptor_matches_transfer(
    existing: &ComponentDescriptor,
    transferred: &ComponentDescriptor,
) -> bool {
    existing.type_name == transferred.type_name
        && existing.storage_type == transferred.storage_type
        && existing.source == transferred.source
}

impl fmt::Debug for ComponentRegistry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ComponentRegistry")
            .field("descriptors", &self.descriptors)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TableValue;

    impl Component for TableValue {}

    struct SparseValue;

    impl Component for SparseValue {
        const STORAGE_TYPE: StorageType = StorageType::SparseSet;
    }

    #[test]
    fn rust_table_components_receive_their_registered_dense_column_layout() {
        let mut registry = ComponentRegistry::default();
        let table_id = registry.component_id::<TableValue>();
        let sparse_id = registry.component_id::<SparseValue>();
        let dynamic_id = registry.dynamic_component_id("plugin.example.dynamic");

        assert!(registry
            .table_column_layout(table_id)
            .is_some_and(TableColumnLayout::matches::<TableValue>));
        assert!(registry.table_column_layout(sparse_id).is_none());
        assert!(registry.table_column_layout(dynamic_id).is_none());
    }

    #[test]
    fn table_column_layout_batches_preserve_the_signature_component_order() {
        let mut registry = ComponentRegistry::default();
        let first = registry.component_id::<TableValue>();
        let second = registry.component_id::<AnotherTableValue>();
        let sparse = registry.component_id::<SparseValue>();

        let layouts = registry
            .table_column_layouts_for_ids(&[second, first])
            .expect("registered table components must have layouts");

        assert_eq!(
            layouts
                .iter()
                .map(|(component_id, _)| *component_id)
                .collect::<Vec<_>>(),
            vec![second, first]
        );
        assert!(registry
            .table_column_layouts_for_ids(&[first, sparse])
            .is_none());
    }

    #[test]
    fn transferred_rust_table_descriptor_imports_its_column_layout_once() {
        let mut source = ComponentRegistry::default();
        let source_id = source.component_id::<TableValue>();
        let transfer = source
            .transferred_descriptor(source_id)
            .expect("registered source descriptor should transfer");

        let mut target = ComponentRegistry::default();
        let mut imports = target.begin_transferred_descriptor_imports();
        let imported = target
            .preflight_transferred_descriptor_import(&mut imports, &transfer)
            .expect("target should preflight a compatible table descriptor");
        let resolved_again = target
            .preflight_transferred_descriptor_import(&mut imports, &transfer)
            .expect("target should reuse its local descriptor identity during preflight");
        target.publish_preflighted_transferred_descriptor_imports(imports);

        assert_eq!(imported, resolved_again);
        assert_eq!(
            target.rust_type_for_id(imported),
            Some((TypeId::of::<TableValue>(), type_name::<TableValue>()))
        );
        assert!(target
            .table_column_layout(imported)
            .is_some_and(TableColumnLayout::matches::<TableValue>));
    }

    #[test]
    fn dynamic_descriptor_preflight_reserves_one_unpublished_target_local_id() {
        let mut registry = ComponentRegistry::default();
        let mut imports = registry.begin_transferred_descriptor_imports();

        let first = registry
            .preflight_dynamic_descriptor_import(&mut imports, "weather.Component.CloudLayer");
        let second = registry
            .preflight_dynamic_descriptor_import(&mut imports, "weather.Component.CloudLayer");

        assert_eq!(first, second);
        assert!(registry
            .registered_dynamic_component_id("weather.Component.CloudLayer")
            .is_none());
        assert_eq!(imports.imported_descriptor_count(), 1);

        registry.publish_preflighted_transferred_descriptor_imports(imports);
        assert_eq!(
            registry.registered_dynamic_component_id("weather.Component.CloudLayer"),
            Some(first)
        );
    }

    #[test]
    fn transferred_descriptor_preflight_defers_import_until_publication() {
        let mut source = ComponentRegistry::default();
        let source_id = source.component_id::<TableValue>();
        let transfer = source
            .transferred_descriptor(source_id)
            .expect("registered source descriptor should transfer");

        let mut target = ComponentRegistry::default();
        target.component_id::<SparseValue>();
        let base_descriptor_count = target.descriptors().len();
        let mut imports = target.begin_transferred_descriptor_imports();

        let imported = target
            .preflight_transferred_descriptor_import(&mut imports, &transfer)
            .expect("compatible table descriptor should preflight");
        let reused = target
            .preflight_transferred_descriptor_import(&mut imports, &transfer)
            .expect("repeated descriptor should resolve inside the same preflight");

        assert_eq!(imported, reused);
        assert_eq!(imports.imported_descriptor_count(), 1);
        assert_eq!(imports.reused_descriptor_resolve_count(), 1);
        assert_eq!(target.descriptors().len(), base_descriptor_count);
        assert!(target.table_column_layout(imported).is_none());

        target.publish_preflighted_transferred_descriptor_imports(imports);

        assert_eq!(target.descriptors().len(), base_descriptor_count + 1);
        assert!(target
            .table_column_layout(imported)
            .is_some_and(TableColumnLayout::matches::<TableValue>));
    }

    #[test]
    fn transferred_descriptor_preflight_rejects_conflicts_without_mutating_the_base() {
        let mut target = ComponentRegistry::default();
        target.dynamic_component_id("plugin.example.Probe");
        let base_descriptor_count = target.descriptors().len();
        let conflicting = TransferredComponentDescriptor {
            descriptor: ComponentDescriptor {
                id: ComponentId::new(99),
                type_name: "incompatible".to_string(),
                storage_type: StorageType::SparseSet,
                source: ComponentDescriptorSource::DynamicPlugin {
                    component_type_id: "plugin.example.Probe".to_string(),
                },
            },
            table_column_layout: None,
        };
        let mut imports = target.begin_transferred_descriptor_imports();

        assert!(target
            .preflight_transferred_descriptor_import(&mut imports, &conflicting)
            .is_none());
        assert_eq!(imports.imported_descriptor_count(), 0);
        assert_eq!(target.descriptors().len(), base_descriptor_count);
    }

    struct AnotherTableValue;

    impl Component for AnotherTableValue {}
}
