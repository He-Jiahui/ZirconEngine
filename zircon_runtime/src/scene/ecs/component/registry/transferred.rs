use std::any::TypeId;
use std::collections::HashMap;

use crate::scene::ecs::StorageType;

use super::super::id::ComponentId;
use super::super::table_column::TableColumnLayout;
use super::{ComponentDescriptor, ComponentDescriptorSource, ComponentRegistry};

/// A descriptor-owned capability carried out of an isolated World. Its
/// numeric component id remains local to that World and is never reused by a
/// commit target.
#[derive(Clone)]
pub(crate) struct TransferredComponentDescriptor {
    pub(super) descriptor: ComponentDescriptor,
    pub(super) table_column_layout: Option<TableColumnLayout>,
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

impl ComponentRegistry {
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
}

fn descriptor_matches_transfer(
    existing: &ComponentDescriptor,
    transferred: &ComponentDescriptor,
) -> bool {
    existing.type_name == transferred.type_name
        && existing.storage_type == transferred.storage_type
        && existing.source == transferred.source
}
