use std::ops::Deref;
use std::sync::Arc;

use zircon_runtime_interface::{
    ZrByteBufferRef, ZrByteSlice, ZrHostApiV3, ZrHostAssetApiV1, ZrHostBridgeApiV1,
    ZrHostDiagnosticsApiV1, ZrHostEcsApiV1, ZrHostEventApiV1, ZrRuntimePluginHandle, ZrStatus,
    ZrStatusCode,
};

use crate::core::framework::bridge::{BridgeInterfaceStatus, InterfaceSlot};
use crate::plugin::FrozenBridgeTable;

use super::super::bridge_method_bindings::{
    NativeBridgeCall, NativeBridgeMethodDescriptor, NativeBridgeMethodFn,
};
use super::super::ffi_panic_guard::catch_native_host_api_panic;
use super::super::loaded_native_plugin::NativePluginLibraryGenerationOwner;
use super::context_handles::{
    context_snapshot, insert_context, remove_context, NativeHostApiV3Context,
};

pub(super) struct NativeHostBridgeCallContext {
    pub(super) table: FrozenBridgeTable,
    pub(super) methods: DenseBridgeMethodTable,
    pub(super) library_owner: Option<NativePluginLibraryGenerationOwner>,
}

/// Pins the single registry-owned context allocation for the complete callback dispatch.
pub(super) struct NativeHostBridgeCallContextPin {
    context: Arc<NativeHostApiV3Context>,
}

impl NativeHostBridgeCallContextPin {
    pub(super) fn new(context: Arc<NativeHostApiV3Context>) -> Self {
        debug_assert!(matches!(
            context.as_ref(),
            NativeHostApiV3Context::BridgeCall(_)
        ));
        Self { context }
    }
}

impl Deref for NativeHostBridgeCallContextPin {
    type Target = NativeHostBridgeCallContext;

    fn deref(&self) -> &Self::Target {
        match self.context.as_ref() {
            NativeHostApiV3Context::BridgeCall(context) => context,
            NativeHostApiV3Context::RegistrationV4(_) => {
                unreachable!("bridge call context pin must retain a bridge context")
            }
        }
    }
}

/// Immutable callback dispatch storage indexed by the already-resolved interface and method
/// slots. The manifest parser may use ordered maps while building descriptors, but a stable ABI
/// call must not pay a tree lookup after its scope has been frozen.
pub(super) struct DenseBridgeMethodTable {
    interfaces: DenseBridgeSlotDirectory<DenseBridgeMethodRow>,
    method_count: usize,
}

impl DenseBridgeMethodTable {
    pub(super) fn from_entries(
        entries: impl IntoIterator<Item = (u32, u32, NativeBridgeMethodFn)>,
    ) -> Self {
        let mut interfaces = DenseBridgeSlotDirectory::default();
        let mut method_count = 0;

        for (interface_slot, method_slot, method) in entries {
            if interfaces.get(interface_slot).is_none() {
                interfaces.insert(interface_slot, DenseBridgeMethodRow::default());
            }
            let row = interfaces
                .get_mut(interface_slot)
                .expect("inserted bridge interface row must remain addressable");
            method_count += usize::from(row.insert(method_slot, method));
        }

        Self {
            interfaces,
            method_count,
        }
    }

    pub(super) fn get(
        &self,
        interface_slot: u32,
        method_slot: u32,
    ) -> Option<NativeBridgeMethodFn> {
        self.interfaces.get(interface_slot)?.get(method_slot)
    }

    pub(super) const fn len(&self) -> usize {
        self.method_count
    }

    #[cfg(test)]
    pub(super) fn metrics(&self) -> DenseBridgeMethodTableMetrics {
        DenseBridgeMethodTableMetrics {
            interface_rows: self.interfaces.len(),
            method_rows: self.interfaces.len(),
            occupied_methods: self.method_count,
            tree_probes: 0,
        }
    }
}

struct DenseBridgeMethodRow {
    methods: DenseBridgeSlotDirectory<NativeBridgeMethodFn>,
}

impl Default for DenseBridgeMethodRow {
    fn default() -> Self {
        Self {
            methods: DenseBridgeSlotDirectory::default(),
        }
    }
}

impl DenseBridgeMethodRow {
    fn insert(&mut self, method_slot: u32, method: NativeBridgeMethodFn) -> bool {
        self.methods.insert(method_slot, method).is_none()
    }

    fn get(&self, method_slot: u32) -> Option<NativeBridgeMethodFn> {
        self.methods.get(method_slot).copied()
    }
}

// ABI slots are arbitrary u32 values. A fixed four-level directory shares the common prefixes
// of dense registrations while keeping a sparse slot from turning into a slot-sized allocation.
const DENSE_BRIDGE_SLOT_DIRECTORY_BITS: u32 = 8;
const DENSE_BRIDGE_SLOT_DIRECTORY_LEVELS: u32 = u32::BITS / DENSE_BRIDGE_SLOT_DIRECTORY_BITS;
const DENSE_BRIDGE_SLOT_DIRECTORY_FANOUT: usize = 1_usize << DENSE_BRIDGE_SLOT_DIRECTORY_BITS;

struct DenseBridgeSlotDirectory<T> {
    root: DenseBridgeSlotDirectoryNode<T>,
    len: usize,
}

impl<T> Default for DenseBridgeSlotDirectory<T> {
    fn default() -> Self {
        Self {
            root: DenseBridgeSlotDirectoryNode::empty_at_depth(0),
            len: 0,
        }
    }
}

impl<T> DenseBridgeSlotDirectory<T> {
    fn get(&self, slot: u32) -> Option<&T> {
        self.root.get(slot, 0)
    }

    fn get_mut(&mut self, slot: u32) -> Option<&mut T> {
        self.root.get_mut(slot, 0)
    }

    fn insert(&mut self, slot: u32, value: T) -> Option<T> {
        let previous = self.root.insert(slot, 0, value);
        if previous.is_none() {
            self.len += 1;
        }
        previous
    }

    const fn len(&self) -> usize {
        self.len
    }
}

enum DenseBridgeSlotDirectoryNode<T> {
    Branch(Box<[Option<Box<DenseBridgeSlotDirectoryNode<T>>>; DENSE_BRIDGE_SLOT_DIRECTORY_FANOUT]>),
    Page(Box<[Option<T>; DENSE_BRIDGE_SLOT_DIRECTORY_FANOUT]>),
}

impl<T> DenseBridgeSlotDirectoryNode<T> {
    fn empty_at_depth(depth: u32) -> Self {
        if depth + 1 == DENSE_BRIDGE_SLOT_DIRECTORY_LEVELS {
            Self::Page(Box::new(std::array::from_fn(|_| None)))
        } else {
            Self::Branch(Box::new(std::array::from_fn(|_| None)))
        }
    }

    fn get(&self, slot: u32, depth: u32) -> Option<&T> {
        let index = dense_bridge_slot_directory_index(slot, depth);
        match self {
            Self::Branch(branches) => branches.get(index)?.as_deref()?.get(slot, depth + 1),
            Self::Page(values) => values.get(index)?.as_ref(),
        }
    }

    fn get_mut(&mut self, slot: u32, depth: u32) -> Option<&mut T> {
        let index = dense_bridge_slot_directory_index(slot, depth);
        match self {
            Self::Branch(branches) => branches
                .get_mut(index)?
                .as_deref_mut()?
                .get_mut(slot, depth + 1),
            Self::Page(values) => values.get_mut(index)?.as_mut(),
        }
    }

    fn insert(&mut self, slot: u32, depth: u32, value: T) -> Option<T> {
        let index = dense_bridge_slot_directory_index(slot, depth);
        match self {
            Self::Branch(branches) => branches[index]
                .get_or_insert_with(|| Box::new(Self::empty_at_depth(depth + 1)))
                .insert(slot, depth + 1, value),
            Self::Page(values) => values[index].replace(value),
        }
    }
}

fn dense_bridge_slot_directory_index(slot: u32, depth: u32) -> usize {
    debug_assert!(depth < DENSE_BRIDGE_SLOT_DIRECTORY_LEVELS);
    let shift = (DENSE_BRIDGE_SLOT_DIRECTORY_LEVELS - depth - 1) * DENSE_BRIDGE_SLOT_DIRECTORY_BITS;
    ((slot >> shift) & (DENSE_BRIDGE_SLOT_DIRECTORY_FANOUT as u32 - 1)) as usize
}

impl NativeHostBridgeCallContext {
    #[cfg(test)]
    pub(super) fn method_table_metrics(&self) -> DenseBridgeMethodTableMetrics {
        self.methods.metrics()
    }
}

#[cfg(test)]
pub(super) struct DenseBridgeMethodTableMetrics {
    pub(super) interface_rows: usize,
    pub(super) method_rows: usize,
    pub(super) occupied_methods: usize,
    pub(super) tree_probes: usize,
}

#[derive(Clone)]
pub struct NativeHostBridgeCallScope {
    handle: ZrRuntimePluginHandle,
    _registration: Arc<NativeHostBridgeCallRegistration>,
}

struct NativeHostBridgeCallRegistration {
    handle: ZrRuntimePluginHandle,
}

impl NativeHostBridgeCallScope {
    pub fn new(table: FrozenBridgeTable) -> Self {
        Self::with_methods_and_owner(table, std::iter::empty(), None)
    }

    /// Builds a bridge scope without a dynamic-library generation owner.
    ///
    /// # Safety
    ///
    /// Every supplied callback must remain valid until the last clone of the returned scope is
    /// dropped. Native ABI callbacks loaded from a dynamic library must use the live-host
    /// generation-owned construction path instead.
    pub unsafe fn with_methods(
        table: FrozenBridgeTable,
        methods: impl IntoIterator<Item = (InterfaceSlot, u32, NativeBridgeMethodFn)>,
    ) -> Self {
        Self::with_methods_and_owner(table, methods, None)
    }

    pub(in super::super) fn with_methods_and_owner(
        table: FrozenBridgeTable,
        methods: impl IntoIterator<Item = (InterfaceSlot, u32, NativeBridgeMethodFn)>,
        library_owner: Option<NativePluginLibraryGenerationOwner>,
    ) -> Self {
        let methods = DenseBridgeMethodTable::from_entries(
            methods
                .into_iter()
                .map(|(slot, method_slot, method)| (slot.raw(), method_slot, method)),
        );
        let handle = ZrRuntimePluginHandle::new(insert_context(
            NativeHostApiV3Context::BridgeCall(NativeHostBridgeCallContext {
                table,
                methods,
                library_owner,
            }),
        ));
        Self {
            handle,
            _registration: Arc::new(NativeHostBridgeCallRegistration { handle }),
        }
    }

    /// Builds a bridge scope from descriptors without a dynamic-library generation owner.
    ///
    /// # Safety
    ///
    /// Every descriptor callback must remain valid until the last clone of the returned scope is
    /// dropped. Native ABI descriptors loaded from a dynamic library must use the live-host
    /// generation-owned construction path instead.
    pub unsafe fn from_method_descriptors(
        table: FrozenBridgeTable,
        descriptors: impl IntoIterator<Item = NativeBridgeMethodDescriptor>,
    ) -> Result<Self, crate::plugin::RuntimeExtensionRegistryError> {
        Self::from_method_descriptors_with_owner(table, descriptors, None)
    }

    pub(in super::super) fn from_method_descriptors_with_owner(
        table: FrozenBridgeTable,
        descriptors: impl IntoIterator<Item = NativeBridgeMethodDescriptor>,
        library_owner: Option<NativePluginLibraryGenerationOwner>,
    ) -> Result<Self, crate::plugin::RuntimeExtensionRegistryError> {
        let methods = descriptors
            .into_iter()
            .map(|descriptor| {
                let slot = table
                    .resolve_slot(descriptor.interface_id())
                    .ok_or_else(|| {
                        crate::plugin::RuntimeExtensionRegistryError::MissingPluginInterface(
                            descriptor.interface_id().to_string(),
                        )
                    })?;
                Ok((slot, descriptor.method_slot(), descriptor.method()))
            })
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Self::with_methods_and_owner(table, methods, library_owner))
    }

    pub(in super::super) fn from_method_descriptor_refs_with_owner<'a>(
        table: FrozenBridgeTable,
        descriptors: impl IntoIterator<Item = &'a NativeBridgeMethodDescriptor>,
        library_owner: Option<NativePluginLibraryGenerationOwner>,
    ) -> Result<Self, crate::plugin::RuntimeExtensionRegistryError> {
        let methods = descriptors
            .into_iter()
            .map(|descriptor| {
                let slot = table
                    .resolve_slot(descriptor.interface_id())
                    .ok_or_else(|| {
                        crate::plugin::RuntimeExtensionRegistryError::MissingPluginInterface(
                            descriptor.interface_id().to_string(),
                        )
                    })?;
                Ok((slot, descriptor.method_slot(), descriptor.method()))
            })
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Self::with_methods_and_owner(table, methods, library_owner))
    }

    pub const fn handle(&self) -> ZrRuntimePluginHandle {
        self.handle
    }

    pub fn method_count(&self) -> usize {
        match context_snapshot(self.handle.raw()).as_deref() {
            Some(NativeHostApiV3Context::BridgeCall(context)) => context.methods.len(),
            Some(NativeHostApiV3Context::RegistrationV4(_)) | None => 0,
        }
    }

    pub fn api(&self) -> ZrHostApiV3 {
        ZrHostApiV3 {
            abi_version: 3,
            size_bytes: std::mem::size_of::<ZrHostApiV3>(),
            ecs: ZrHostEcsApiV1::empty(),
            asset: ZrHostAssetApiV1::empty(),
            event: ZrHostEventApiV1::empty(),
            bridge: ZrHostBridgeApiV1 {
                call: Some(native_host_bridge_call_v1),
            },
            diagnostics: ZrHostDiagnosticsApiV1::empty(),
        }
    }
}

impl Drop for NativeHostBridgeCallRegistration {
    fn drop(&mut self) {
        remove_context(self.handle.raw());
    }
}

pub(super) fn bridge_context_for(
    handle: ZrRuntimePluginHandle,
) -> Result<NativeHostBridgeCallContextPin, ZrStatusCode> {
    if !handle.is_valid() {
        return Err(ZrStatusCode::NotFound);
    }
    let context = context_snapshot(handle.raw()).ok_or(ZrStatusCode::NotFound)?;
    match context.as_ref() {
        NativeHostApiV3Context::RegistrationV4(_) => {
            return Err(ZrStatusCode::UnsupportedVersion);
        }
        NativeHostApiV3Context::BridgeCall(_) => {}
    }
    Ok(NativeHostBridgeCallContextPin::new(context))
}

pub(super) unsafe extern "C" fn native_host_bridge_call_v1(
    handle: ZrRuntimePluginHandle,
    interface_slot: u32,
    method_slot: u32,
    payload: *const u8,
    len: usize,
    output: ZrByteBufferRef,
) -> ZrStatus {
    catch_native_host_api_panic(|| unsafe {
        native_host_bridge_call_v1_inner(handle, interface_slot, method_slot, payload, len, output)
    })
}

unsafe fn native_host_bridge_call_v1_inner(
    handle: ZrRuntimePluginHandle,
    interface_slot: u32,
    method_slot: u32,
    payload: *const u8,
    len: usize,
    output: ZrByteBufferRef,
) -> ZrStatus {
    if payload.is_null() && len != 0 {
        return status(ZrStatusCode::InvalidArgument);
    }
    let context = match bridge_context_for(handle) {
        Ok(context) => context,
        Err(code) => return status(code),
    };
    let slot = InterfaceSlot::from_raw(interface_slot);
    let Some(entry) = context.table.entry(slot) else {
        return status(ZrStatusCode::NotFound);
    };
    if entry.status() != BridgeInterfaceStatus::Enabled {
        context.table.record_not_enabled_call(slot);
        return status(ZrStatusCode::BridgeNotEnabled);
    }
    let Some(method) = context.methods.get(interface_slot, method_slot) else {
        return status(ZrStatusCode::NotFound);
    };
    let callback_lease = match context.library_owner.as_ref() {
        Some(owner) => match owner.acquire_callback() {
            Ok(lease) => Some(lease),
            Err(_) => {
                context.table.record_not_enabled_call(slot);
                return status(ZrStatusCode::BridgeNotEnabled);
            }
        },
        None => None,
    };
    context.table.record_enabled_call(slot);
    let started_at = callback_lease
        .as_ref()
        .and_then(|lease| lease.begin_callback_measurement());
    let status = method.call(NativeBridgeCall {
        interface_slot,
        method_slot,
        payload: ZrByteSlice { data: payload, len },
        output,
    });
    if let Some(lease) = callback_lease.as_ref() {
        lease.complete_callback_measurement(started_at);
    }
    status
}

fn status(code: ZrStatusCode) -> ZrStatus {
    ZrStatus::new(code, ZrByteSlice::empty())
}

#[cfg(test)]
mod tests;
