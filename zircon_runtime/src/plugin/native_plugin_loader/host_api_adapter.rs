use std::marker::PhantomData;
use std::str::Utf8Error;
use std::sync::{Arc, OnceLock};

use zircon_runtime_interface::{
    ZrByteBufferRef, ZrByteSlice, ZrComponentDescV1, ZrEventTypeId, ZrHostApiV3, ZrHostApiV4,
    ZrHostAssetApiV1, ZrHostBridgeApiV1, ZrHostDiagnosticsApiV1, ZrHostEcsApiV1, ZrHostEcsApiV2,
    ZrHostEventApiV1, ZrNativeSystemAccessV1, ZrOwnedByteBuffer, ZrRuntimePluginHandle, ZrStatus,
    ZrStatusCode, ZrSystemRegistrationV1, ZrSystemRegistrationV2,
    ZR_NATIVE_SYSTEM_ACCESS_DOMAIN_COMPONENT_V1, ZR_NATIVE_SYSTEM_ACCESS_DOMAIN_RESOURCE_V1,
    ZR_NATIVE_SYSTEM_ACCESS_MODE_READ_V1, ZR_NATIVE_SYSTEM_ACCESS_MODE_WRITE_V1,
    ZR_NATIVE_SYSTEM_THREAD_AFFINITY_MAIN_THREAD_ONLY_V1,
    ZR_NATIVE_SYSTEM_THREAD_AFFINITY_WORKER_SAFE_V1,
};

use crate::core::framework::bridge::{BridgeInterfaceStatus, InterfaceSlot};
use crate::core::framework::scene::ComponentTypeDescriptor;
use crate::plugin::{
    FrozenBridgeTable, PluginModuleId, RuntimeExtensionRegistry, RuntimeExtensionRegistryError,
};
use crate::scene::ecs::{
    ChangeTickWindow, SystemParam, SystemParamAccess, SystemParamError, SystemRef, SystemStage,
};
use crate::scene::World;

use super::bridge_method_bindings::{
    NativeBridgeCall, NativeBridgeMethodDescriptor, NativeBridgeMethodFn,
};
use super::ffi_panic_guard::catch_native_host_api_panic;
use super::loaded_native_plugin::NativePluginCallbackLease;
use super::registration_manifest::{
    NativePluginRegistrationThreadAffinity, NativeSystemAccessAuthority,
    NativeSystemAccessAuthorityError, NativeSystemAccessContractError, NativeSystemAccessPlan,
};

mod context_registry;

use context_registry::HostContextRegistry;

type NativeHostApiAdapterResult<T> = std::result::Result<T, NativeHostApiAdapterError>;

const MAX_NATIVE_SYSTEM_ACCESS_ENTRIES: usize = 4_096;

#[derive(Debug)]
enum NativeHostApiAdapterError {
    InvalidPluginModuleOwner {
        source: RuntimeExtensionRegistryError,
    },
    InvalidUtf8 {
        source: Utf8Error,
    },
    UnknownSystemStage {
        stage: u32,
    },
    InvalidSystemSet {
        source: RuntimeExtensionRegistryError,
    },
    RegisterSystem {
        source: RuntimeExtensionRegistryError,
    },
    UnknownPluginModuleOwner {
        owner: PluginModuleId,
    },
    RegisterComponent {
        source: RuntimeExtensionRegistryError,
    },
    InvalidV4RuntimeModuleName {
        module_name: String,
    },
    InvalidV4RegistrationAbiVersion {
        actual: u32,
    },
    InvalidV4RegistrationSize {
        actual: usize,
    },
    EmptyV4AccessContract,
    InvalidV4StringListPointer {
        field: &'static str,
        count: usize,
    },
    InvalidV4AccessPointer {
        count: usize,
    },
    TooManyV4Accesses {
        count: usize,
    },
    InvalidV4AccessAbiVersion {
        actual: u32,
    },
    InvalidV4AccessSize {
        actual: usize,
    },
    InvalidV4AccessMode {
        mode: u32,
    },
    InvalidV4AccessDomain {
        domain: u32,
    },
    InvalidV4ThreadAffinity {
        affinity: u32,
    },
    InvalidV4SystemAccess {
        source: NativeSystemAccessContractError,
    },
    UnauthorizedV4SystemAccess {
        source: NativeSystemAccessAuthorityError,
    },
}

impl std::fmt::Display for NativeHostApiAdapterError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidPluginModuleOwner { source } => {
                write!(
                    formatter,
                    "native host API plugin module owner is invalid: {source}"
                )
            }
            Self::InvalidUtf8 { source } => {
                write!(
                    formatter,
                    "native host API string field is not valid UTF-8: {source}"
                )
            }
            Self::UnknownSystemStage { stage } => {
                write!(formatter, "unknown native system stage {stage}")
            }
            Self::InvalidSystemSet { source } => {
                write!(formatter, "native host API system set is invalid: {source}")
            }
            Self::RegisterSystem { source } => {
                write!(
                    formatter,
                    "native host API system registration failed: {source}"
                )
            }
            Self::UnknownPluginModuleOwner { owner } => {
                write!(formatter, "unknown plugin module owner {}", owner.raw())
            }
            Self::RegisterComponent { source } => {
                write!(
                    formatter,
                    "native host API component registration failed: {source}"
                )
            }
            Self::InvalidV4RuntimeModuleName { module_name } => write!(
                formatter,
                "native host API V4 requires a <plugin>.runtime module owner, got `{module_name}`"
            ),
            Self::InvalidV4RegistrationAbiVersion { actual } => {
                write!(
                    formatter,
                    "native host API V4 registration ABI must be 4, got {actual}"
                )
            }
            Self::InvalidV4RegistrationSize { actual } => write!(
                formatter,
                "native host API V4 registration size must be {}, got {actual}",
                std::mem::size_of::<ZrSystemRegistrationV2>()
            ),
            Self::EmptyV4AccessContract => formatter.write_str(
                "native host API V4 systems must declare at least one component or resource access",
            ),
            Self::InvalidV4StringListPointer { field, count } => write!(
                formatter,
                "native host API V4 {field} pointer is null for {count} entries"
            ),
            Self::InvalidV4AccessPointer { count } => write!(
                formatter,
                "native host API V4 access pointer is null for {count} entries"
            ),
            Self::TooManyV4Accesses { count } => write!(
                formatter,
                "native host API V4 access list has {count} entries, maximum is {MAX_NATIVE_SYSTEM_ACCESS_ENTRIES}"
            ),
            Self::InvalidV4AccessAbiVersion { actual } => {
                write!(
                    formatter,
                    "native host API V4 access ABI must be 1, got {actual}"
                )
            }
            Self::InvalidV4AccessSize { actual } => write!(
                formatter,
                "native host API V4 access size must be {}, got {actual}",
                std::mem::size_of::<ZrNativeSystemAccessV1>()
            ),
            Self::InvalidV4AccessMode { mode } => {
                write!(
                    formatter,
                    "native host API V4 access mode {mode} is unsupported"
                )
            }
            Self::InvalidV4AccessDomain { domain } => {
                write!(
                    formatter,
                    "native host API V4 access domain {domain} is unsupported"
                )
            }
            Self::InvalidV4ThreadAffinity { affinity } => write!(
                formatter,
                "native host API V4 thread affinity {affinity} is unsupported"
            ),
            Self::InvalidV4SystemAccess { source } => {
                write!(
                    formatter,
                    "native host API V4 access contract is invalid: {source}"
                )
            }
            Self::UnauthorizedV4SystemAccess { source } => {
                write!(
                    formatter,
                    "native host API V4 access is not authorized: {source}"
                )
            }
        }
    }
}

impl std::error::Error for NativeHostApiAdapterError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::InvalidPluginModuleOwner { source }
            | Self::InvalidSystemSet { source }
            | Self::RegisterSystem { source }
            | Self::RegisterComponent { source } => Some(source),
            Self::InvalidUtf8 { source } => Some(source),
            Self::InvalidV4SystemAccess { source } => Some(source),
            Self::UnauthorizedV4SystemAccess { source } => Some(source),
            Self::UnknownSystemStage { .. }
            | Self::UnknownPluginModuleOwner { .. }
            | Self::InvalidV4RuntimeModuleName { .. }
            | Self::InvalidV4RegistrationAbiVersion { .. }
            | Self::InvalidV4RegistrationSize { .. }
            | Self::EmptyV4AccessContract
            | Self::InvalidV4StringListPointer { .. }
            | Self::InvalidV4AccessPointer { .. }
            | Self::TooManyV4Accesses { .. }
            | Self::InvalidV4AccessAbiVersion { .. }
            | Self::InvalidV4AccessSize { .. }
            | Self::InvalidV4AccessMode { .. }
            | Self::InvalidV4AccessDomain { .. }
            | Self::InvalidV4ThreadAffinity { .. } => None,
        }
    }
}

pub struct NativeHostApiV3RegistrationScope<'registry> {
    handle: ZrRuntimePluginHandle,
    _registry: PhantomData<&'registry mut RuntimeExtensionRegistry>,
}

impl<'registry> NativeHostApiV3RegistrationScope<'registry> {
    pub fn new(
        registry: &'registry mut RuntimeExtensionRegistry,
        module_name: impl Into<String>,
    ) -> Result<Self, String> {
        Self::new_result(registry, module_name).map_err(|error| error.to_string())
    }

    fn new_result(
        registry: &'registry mut RuntimeExtensionRegistry,
        module_name: impl Into<String>,
    ) -> NativeHostApiAdapterResult<Self> {
        let owner = registry
            .intern_plugin_module(module_name)
            .map_err(|source| NativeHostApiAdapterError::InvalidPluginModuleOwner { source })?;
        let handle = ZrRuntimePluginHandle::new(contexts().insert(Arc::new(
            NativeHostApiV3Context::Registration(NativeHostApiV3RegistrationContext {
                registry: registry as *mut RuntimeExtensionRegistry as usize,
                owner,
            }),
        )));
        Ok(Self {
            handle,
            _registry: PhantomData,
        })
    }

    pub const fn handle(&self) -> ZrRuntimePluginHandle {
        self.handle
    }

    pub fn api(&self) -> ZrHostApiV3 {
        ZrHostApiV3 {
            abi_version: 3,
            size_bytes: std::mem::size_of::<ZrHostApiV3>(),
            ecs: ZrHostEcsApiV1 {
                register_system: Some(native_host_register_system_v1),
                register_component: Some(native_host_register_component_v1),
                spawn_command: Some(native_host_spawn_command_v1),
            },
            asset: ZrHostAssetApiV1 {
                request: Some(native_host_asset_request_v1),
            },
            event: ZrHostEventApiV1 {
                emit: Some(native_host_event_emit_v1),
                drain: Some(native_host_event_drain_v1),
            },
            bridge: ZrHostBridgeApiV1 {
                call: Some(native_host_bridge_call_v1),
            },
            diagnostics: ZrHostDiagnosticsApiV1 {
                emit: Some(native_host_diagnostics_emit_v1),
                metric: Some(native_host_diagnostics_metric_v1),
            },
        }
    }
}

/// Host-owned authority inputs for a V4 native registration scope.
///
/// Component IDs are read from the runtime registry at each registration so components declared
/// earlier in the same entry callback become immediately usable. Resource IDs and capability
/// grants are host policy, never plugin-provided inputs.
#[derive(Clone, Debug, Default)]
pub struct NativeHostApiV4RegistrationPolicy {
    granted_capabilities: Vec<String>,
    known_resource_ids: Vec<String>,
}

impl NativeHostApiV4RegistrationPolicy {
    pub fn new<C, R, CS, RS>(granted_capabilities: C, known_resource_ids: R) -> Self
    where
        C: IntoIterator<Item = CS>,
        R: IntoIterator<Item = RS>,
        CS: Into<String>,
        RS: Into<String>,
    {
        let mut granted_capabilities = granted_capabilities
            .into_iter()
            .map(Into::into)
            .collect::<Vec<String>>();
        let mut known_resource_ids = known_resource_ids
            .into_iter()
            .map(Into::into)
            .collect::<Vec<String>>();
        granted_capabilities.sort();
        granted_capabilities.dedup();
        known_resource_ids.sort();
        known_resource_ids.dedup();
        Self {
            granted_capabilities,
            known_resource_ids,
        }
    }
}

pub struct NativeHostApiV4RegistrationScope<'registry> {
    handle: ZrRuntimePluginHandle,
    _registry: PhantomData<&'registry mut RuntimeExtensionRegistry>,
}

impl<'registry> NativeHostApiV4RegistrationScope<'registry> {
    pub fn new(
        registry: &'registry mut RuntimeExtensionRegistry,
        module_name: impl Into<String>,
        policy: NativeHostApiV4RegistrationPolicy,
    ) -> Result<Self, String> {
        Self::new_result(registry, module_name, policy).map_err(|error| error.to_string())
    }

    fn new_result(
        registry: &'registry mut RuntimeExtensionRegistry,
        module_name: impl Into<String>,
        policy: NativeHostApiV4RegistrationPolicy,
    ) -> NativeHostApiAdapterResult<Self> {
        let module_name = module_name.into();
        let plugin_id = plugin_id_from_runtime_module_name(&module_name)
            .ok_or_else(|| NativeHostApiAdapterError::InvalidV4RuntimeModuleName {
                module_name: module_name.clone(),
            })?
            .to_string();
        let owner = registry
            .intern_plugin_module(module_name)
            .map_err(|source| NativeHostApiAdapterError::InvalidPluginModuleOwner { source })?;
        let handle = ZrRuntimePluginHandle::new(contexts().insert(Arc::new(
            NativeHostApiV3Context::RegistrationV4(NativeHostApiV4RegistrationContext {
                registry: registry as *mut RuntimeExtensionRegistry as usize,
                owner,
                plugin_id,
                policy,
            }),
        )));
        Ok(Self {
            handle,
            _registry: PhantomData,
        })
    }

    pub const fn handle(&self) -> ZrRuntimePluginHandle {
        self.handle
    }

    pub fn api(&self) -> ZrHostApiV4 {
        ZrHostApiV4 {
            abi_version: 4,
            size_bytes: std::mem::size_of::<ZrHostApiV4>(),
            ecs: ZrHostEcsApiV2 {
                register_system: Some(native_host_register_system_v2),
                register_component: Some(native_host_register_component_v1),
                spawn_command: Some(native_host_spawn_command_v1),
            },
            asset: ZrHostAssetApiV1 {
                request: Some(native_host_asset_request_v1),
            },
            event: ZrHostEventApiV1 {
                emit: Some(native_host_event_emit_v1),
                drain: Some(native_host_event_drain_v1),
            },
            bridge: ZrHostBridgeApiV1 {
                call: Some(native_host_bridge_call_v1),
            },
            diagnostics: ZrHostDiagnosticsApiV1 {
                emit: Some(native_host_diagnostics_emit_v1),
                metric: Some(native_host_diagnostics_metric_v1),
            },
        }
    }
}

impl Drop for NativeHostApiV3RegistrationScope<'_> {
    fn drop(&mut self) {
        contexts().remove(self.handle.raw());
    }
}

impl Drop for NativeHostApiV4RegistrationScope<'_> {
    fn drop(&mut self) {
        contexts().remove(self.handle.raw());
    }
}

pub struct NativeHostBridgeCallScope {
    handle: ZrRuntimePluginHandle,
}

impl NativeHostBridgeCallScope {
    pub fn new(table: FrozenBridgeTable) -> Self {
        Self::with_methods(table, std::iter::empty())
    }

    pub fn with_methods(
        table: FrozenBridgeTable,
        methods: impl IntoIterator<Item = (InterfaceSlot, u32, NativeBridgeMethodFn)>,
    ) -> Self {
        Self::with_methods_and_owner(table, methods, None)
    }

    pub(super) fn with_methods_and_owner(
        table: FrozenBridgeTable,
        methods: impl IntoIterator<Item = (InterfaceSlot, u32, NativeBridgeMethodFn)>,
        callback_owner: Option<NativePluginCallbackLease>,
    ) -> Self {
        let methods = DenseBridgeMethodTable::from_entries(
            methods
                .into_iter()
                .map(|(slot, method_slot, method)| (slot.raw(), method_slot, method)),
        );
        let handle = ZrRuntimePluginHandle::new(contexts().insert(Arc::new(
            NativeHostApiV3Context::BridgeCall(Arc::new(NativeHostBridgeCallContext {
                table,
                methods,
                _callback_owner: callback_owner,
            })),
        )));
        Self { handle }
    }

    pub fn from_method_descriptors(
        table: FrozenBridgeTable,
        descriptors: impl IntoIterator<Item = NativeBridgeMethodDescriptor>,
    ) -> Result<Self, crate::plugin::RuntimeExtensionRegistryError> {
        Self::from_method_descriptors_with_owner(table, descriptors, None)
    }

    pub(super) fn from_method_descriptors_with_owner(
        table: FrozenBridgeTable,
        descriptors: impl IntoIterator<Item = NativeBridgeMethodDescriptor>,
        callback_owner: Option<NativePluginCallbackLease>,
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
        Ok(Self::with_methods_and_owner(table, methods, callback_owner))
    }

    pub const fn handle(&self) -> ZrRuntimePluginHandle {
        self.handle
    }

    pub fn method_count(&self) -> usize {
        match contexts().get(self.handle.raw()).as_deref() {
            Some(NativeHostApiV3Context::BridgeCall(context)) => context.methods.len(),
            Some(NativeHostApiV3Context::Registration(_))
            | Some(NativeHostApiV3Context::RegistrationV4(_))
            | None => 0,
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

impl Drop for NativeHostBridgeCallScope {
    fn drop(&mut self) {
        contexts().remove(self.handle.raw());
    }
}

#[derive(Clone, Copy)]
struct NativeHostApiV3RegistrationContext {
    registry: usize,
    owner: PluginModuleId,
}

#[derive(Clone)]
struct NativeHostApiV4RegistrationContext {
    registry: usize,
    owner: PluginModuleId,
    plugin_id: String,
    policy: NativeHostApiV4RegistrationPolicy,
}

impl NativeHostApiV4RegistrationContext {
    const fn v3_context(&self) -> NativeHostApiV3RegistrationContext {
        NativeHostApiV3RegistrationContext {
            registry: self.registry,
            owner: self.owner,
        }
    }
}

struct NativeHostBridgeCallContext {
    table: FrozenBridgeTable,
    methods: DenseBridgeMethodTable,
    _callback_owner: Option<NativePluginCallbackLease>,
}

/// Immutable callback dispatch storage indexed by the already-resolved interface and method
/// slots. The manifest parser may use ordered maps while building descriptors, but a stable ABI
/// call must not pay a tree lookup after its scope has been frozen.
struct DenseBridgeMethodTable {
    interfaces: DenseBridgeSlotDirectory<DenseBridgeMethodRow>,
    method_count: usize,
}

impl DenseBridgeMethodTable {
    fn from_entries(entries: impl IntoIterator<Item = (u32, u32, NativeBridgeMethodFn)>) -> Self {
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

    fn get(&self, interface_slot: u32, method_slot: u32) -> Option<NativeBridgeMethodFn> {
        self.interfaces.get(interface_slot)?.get(method_slot)
    }

    const fn len(&self) -> usize {
        self.method_count
    }

    #[cfg(test)]
    fn metrics(&self) -> DenseBridgeMethodTableMetrics {
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
    fn method_table_metrics(&self) -> DenseBridgeMethodTableMetrics {
        self.methods.metrics()
    }
}

#[cfg(test)]
struct DenseBridgeMethodTableMetrics {
    interface_rows: usize,
    method_rows: usize,
    occupied_methods: usize,
    tree_probes: usize,
}

#[derive(Clone)]
enum NativeHostApiV3Context {
    Registration(NativeHostApiV3RegistrationContext),
    RegistrationV4(NativeHostApiV4RegistrationContext),
    BridgeCall(Arc<NativeHostBridgeCallContext>),
}

unsafe extern "C" fn native_host_register_system_v1(
    handle: ZrRuntimePluginHandle,
    registration: *const ZrSystemRegistrationV1,
) -> ZrStatus {
    catch_native_host_api_panic(|| unsafe {
        native_host_register_system_v1_inner(handle, registration)
    })
}

unsafe fn native_host_register_system_v1_inner(
    handle: ZrRuntimePluginHandle,
    registration: *const ZrSystemRegistrationV1,
) -> ZrStatus {
    if registration.is_null() {
        return status(ZrStatusCode::InvalidArgument);
    }
    let Some(context) = context_for(handle) else {
        return status(ZrStatusCode::NotFound);
    };
    let registration = unsafe { &*registration };
    let result = unsafe { register_system_from_abi(context, handle, registration) };
    match result {
        Ok(()) => ZrStatus::ok(),
        Err(_) => status(ZrStatusCode::Error),
    }
}

unsafe extern "C" fn native_host_register_system_v2(
    handle: ZrRuntimePluginHandle,
    registration: *const ZrSystemRegistrationV2,
) -> ZrStatus {
    catch_native_host_api_panic(|| unsafe {
        native_host_register_system_v2_inner(handle, registration)
    })
}

unsafe fn native_host_register_system_v2_inner(
    handle: ZrRuntimePluginHandle,
    registration: *const ZrSystemRegistrationV2,
) -> ZrStatus {
    if registration.is_null() {
        return status(ZrStatusCode::InvalidArgument);
    }
    let Some(context) = context_for_v4(handle) else {
        return status(ZrStatusCode::NotFound);
    };
    let registration = unsafe { &*registration };
    let result = unsafe { register_system_from_abi_v4(context, handle, registration) };
    match result {
        Ok(()) => ZrStatus::ok(),
        Err(_) => status(ZrStatusCode::Error),
    }
}

#[allow(clippy::needless_pass_by_value)]
unsafe extern "C" fn native_host_register_component_v1(
    handle: ZrRuntimePluginHandle,
    descriptor: *const ZrComponentDescV1,
) -> ZrStatus {
    catch_native_host_api_panic(|| unsafe {
        native_host_register_component_v1_inner(handle, descriptor)
    })
}

unsafe fn native_host_register_component_v1_inner(
    handle: ZrRuntimePluginHandle,
    descriptor: *const ZrComponentDescV1,
) -> ZrStatus {
    if descriptor.is_null() {
        return status(ZrStatusCode::InvalidArgument);
    }
    let Some(context) = context_for(handle) else {
        return status(ZrStatusCode::NotFound);
    };
    let descriptor = unsafe { &*descriptor };
    let result = unsafe { register_component_from_abi(context, descriptor) };
    match result {
        Ok(()) => ZrStatus::ok(),
        Err(_) => status(ZrStatusCode::Error),
    }
}

unsafe extern "C" fn native_host_spawn_command_v1(
    _handle: ZrRuntimePluginHandle,
    _payload: *const u8,
    _len: usize,
) -> ZrStatus {
    catch_native_host_api_panic(|| status(ZrStatusCode::UnsupportedVersion))
}

unsafe extern "C" fn native_host_asset_request_v1(
    _handle: ZrRuntimePluginHandle,
    _request: ZrByteSlice,
    _output: *mut ZrOwnedByteBuffer,
) -> ZrStatus {
    catch_native_host_api_panic(|| status(ZrStatusCode::UnsupportedVersion))
}

unsafe extern "C" fn native_host_event_emit_v1(
    _handle: ZrRuntimePluginHandle,
    _event_type: ZrEventTypeId,
    _payload: *const u8,
    _len: usize,
) -> ZrStatus {
    catch_native_host_api_panic(|| status(ZrStatusCode::UnsupportedVersion))
}

unsafe extern "C" fn native_host_event_drain_v1(
    _handle: ZrRuntimePluginHandle,
    _event_type: ZrEventTypeId,
    _buffer: ZrByteBufferRef,
) -> ZrStatus {
    catch_native_host_api_panic(|| status(ZrStatusCode::UnsupportedVersion))
}

unsafe extern "C" fn native_host_bridge_call_v1(
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
    context.table.record_enabled_call(slot);
    method.call(NativeBridgeCall {
        interface_slot,
        method_slot,
        payload: ZrByteSlice { data: payload, len },
        output,
    })
}

unsafe extern "C" fn native_host_diagnostics_emit_v1(
    _handle: ZrRuntimePluginHandle,
    _target: ZrByteSlice,
    _message: ZrByteSlice,
) -> ZrStatus {
    catch_native_host_api_panic(ZrStatus::ok)
}

unsafe extern "C" fn native_host_diagnostics_metric_v1(
    _handle: ZrRuntimePluginHandle,
    _path: ZrByteSlice,
    _value: f64,
    _unit: ZrByteSlice,
) -> ZrStatus {
    catch_native_host_api_panic(ZrStatus::ok)
}

unsafe fn register_system_from_abi(
    context: NativeHostApiV3RegistrationContext,
    handle: ZrRuntimePluginHandle,
    registration: &ZrSystemRegistrationV1,
) -> NativeHostApiAdapterResult<()> {
    let id = read_utf8(registration.system_id)?;
    let stage = stage_from_abi(registration.stage)?;
    let set_names = read_byte_slices(registration.set_names, registration.set_count)?;
    let before = read_byte_slices(registration.before, registration.before_count)?;
    let after = read_byte_slices(registration.after, registration.after_count)?;
    let invoke = registration.invoke;
    let user_data = registration.user_data;
    let registry = registry_from_context(context);
    let sets = set_names
        .into_iter()
        .map(|set_name| {
            registry
                .intern_system_set(set_name)
                .map_err(|source| NativeHostApiAdapterError::InvalidSystemSet { source })
        })
        .collect::<Result<Vec<_>, _>>()?;

    let mut builder = registry
        .register_native_system::<NativeDynamicAccess, _>(context.owner, id, stage, move |()| {
            if let Some(invoke) = invoke {
                let _ = unsafe { invoke(handle, user_data, ZrByteSlice::empty()) };
            }
        })
        .with_order(registration.order);
    for set in sets {
        builder = builder.in_set(set);
    }
    for system_id in before {
        builder = builder.before(SystemRef::System(system_id));
    }
    for system_id in after {
        builder = builder.after(SystemRef::System(system_id));
    }
    builder
        .register()
        .map_err(|source| NativeHostApiAdapterError::RegisterSystem { source })
}

unsafe fn register_system_from_abi_v4(
    context: NativeHostApiV4RegistrationContext,
    handle: ZrRuntimePluginHandle,
    registration: &ZrSystemRegistrationV2,
) -> NativeHostApiAdapterResult<()> {
    if registration.abi_version != 4 {
        return Err(NativeHostApiAdapterError::InvalidV4RegistrationAbiVersion {
            actual: registration.abi_version,
        });
    }
    if registration.size_bytes != std::mem::size_of::<ZrSystemRegistrationV2>() {
        return Err(NativeHostApiAdapterError::InvalidV4RegistrationSize {
            actual: registration.size_bytes,
        });
    }
    // V3's empty declaration is the legacy conservative fallback. V4 is the opt-in exact
    // contract, so it must never enter that fallback through an empty access array.
    if registration.access_count == 0 {
        return Err(NativeHostApiAdapterError::EmptyV4AccessContract);
    }

    let id = read_utf8(registration.system_id)?;
    let stage = stage_from_abi(registration.stage)?;
    let set_names = unsafe {
        read_v4_byte_slices("set_names", registration.set_names, registration.set_count)
    }?;
    let before =
        unsafe { read_v4_byte_slices("before", registration.before, registration.before_count) }?;
    let after =
        unsafe { read_v4_byte_slices("after", registration.after, registration.after_count) }?;
    let affinity = v4_thread_affinity_from_abi(registration.thread_affinity)?;
    let accesses =
        unsafe { read_v4_system_accesses(registration.accesses, registration.access_count) }?;
    let access_plan = Arc::new(
        NativeSystemAccessPlan::from_manifest(
            affinity,
            &accesses,
            &context.policy.granted_capabilities,
        )
        .map_err(|source| NativeHostApiAdapterError::InvalidV4SystemAccess { source })?,
    );
    let registry = registry_from_context(context.v3_context());
    let authority = NativeSystemAccessAuthority::new(
        context.plugin_id,
        registry
            .components()
            .iter()
            .map(|component| component.type_id.clone()),
        context.policy.known_resource_ids,
        context.policy.granted_capabilities,
    );
    authority
        .authorize(access_plan.as_ref())
        .map_err(|source| NativeHostApiAdapterError::UnauthorizedV4SystemAccess { source })?;
    let sets = set_names
        .into_iter()
        .map(|set_name| {
            registry
                .intern_system_set(set_name)
                .map_err(|source| NativeHostApiAdapterError::InvalidSystemSet { source })
        })
        .collect::<Result<Vec<_>, _>>()?;
    let invoke = registration.invoke;
    let user_data = registration.user_data;
    let access_build = Arc::clone(&access_plan);
    let mut builder = registry
        .register_external_native_system(
            context.owner,
            id,
            stage,
            access_plan.affinity(),
            move |world| {
                access_build
                    .compile(world)
                    .map_err(|error| error.to_string())
            },
            move || {
                if let Some(invoke) = invoke {
                    let _ = unsafe { invoke(handle, user_data, ZrByteSlice::empty()) };
                }
            },
        )
        .with_order(registration.order);
    for set in sets {
        builder = builder.in_set(set);
    }
    for system_id in before {
        builder = builder.before(SystemRef::System(system_id));
    }
    for system_id in after {
        builder = builder.after(SystemRef::System(system_id));
    }
    builder
        .register()
        .map_err(|source| NativeHostApiAdapterError::RegisterSystem { source })
}

pub(super) struct NativeDynamicAccess;

impl SystemParam for NativeDynamicAccess {
    type State = ();
    type Item<'world> = ();

    fn init_state(
        _world: &mut World,
        access: &mut SystemParamAccess,
    ) -> Result<Self::State, SystemParamError> {
        // Native ABI callbacks can reach host state through opaque handles, so the scheduler
        // must treat them as conservative world writers until a typed access ABI exists.
        access.add_conservative_world_access();
        Ok(())
    }

    unsafe fn get_param<'world>(
        _world: *mut World,
        _state: &'world mut Self::State,
        _ticks: ChangeTickWindow,
    ) -> Self::Item<'world> {
    }
}

unsafe fn register_component_from_abi(
    context: NativeHostApiV3RegistrationContext,
    descriptor: &ZrComponentDescV1,
) -> NativeHostApiAdapterResult<()> {
    let type_id = read_utf8(descriptor.type_id)?;
    let display_name = read_utf8(descriptor.display_name)?;
    let registry = registry_from_context(context);
    let plugin_id = registry
        .plugin_module_name(context.owner)
        .and_then(plugin_id_from_runtime_module_name)
        .ok_or(NativeHostApiAdapterError::UnknownPluginModuleOwner {
            owner: context.owner,
        })?
        .to_string();
    registry
        .register_component(ComponentTypeDescriptor::new(
            type_id,
            plugin_id,
            display_name,
        ))
        .map_err(|source| NativeHostApiAdapterError::RegisterComponent { source })
}

unsafe fn registry_from_context<'a>(
    context: NativeHostApiV3RegistrationContext,
) -> &'a mut RuntimeExtensionRegistry {
    &mut *(context.registry as *mut RuntimeExtensionRegistry)
}

fn stage_from_abi(stage: u32) -> NativeHostApiAdapterResult<SystemStage> {
    SystemStage::ORDER
        .get(stage as usize)
        .copied()
        .ok_or(NativeHostApiAdapterError::UnknownSystemStage { stage })
}

unsafe fn read_byte_slices(
    values: *const ZrByteSlice,
    count: usize,
) -> NativeHostApiAdapterResult<Vec<String>> {
    if values.is_null() || count == 0 {
        return Ok(Vec::new());
    }
    unsafe { std::slice::from_raw_parts(values, count) }
        .iter()
        .copied()
        .map(|slice| unsafe { read_utf8(slice) })
        .collect()
}

unsafe fn read_v4_byte_slices(
    field: &'static str,
    values: *const ZrByteSlice,
    count: usize,
) -> NativeHostApiAdapterResult<Vec<String>> {
    if count == 0 {
        return Ok(Vec::new());
    }
    if values.is_null() {
        return Err(NativeHostApiAdapterError::InvalidV4StringListPointer { field, count });
    }
    unsafe { read_byte_slices(values, count) }
}

unsafe fn read_v4_system_accesses(
    values: *const ZrNativeSystemAccessV1,
    count: usize,
) -> NativeHostApiAdapterResult<Vec<String>> {
    if count == 0 {
        return Ok(Vec::new());
    }
    if values.is_null() {
        return Err(NativeHostApiAdapterError::InvalidV4AccessPointer { count });
    }
    if count > MAX_NATIVE_SYSTEM_ACCESS_ENTRIES {
        return Err(NativeHostApiAdapterError::TooManyV4Accesses { count });
    }
    unsafe { std::slice::from_raw_parts(values, count) }
        .iter()
        .map(|access| {
            if access.abi_version != 1 {
                return Err(NativeHostApiAdapterError::InvalidV4AccessAbiVersion {
                    actual: access.abi_version,
                });
            }
            if access.size_bytes != std::mem::size_of::<ZrNativeSystemAccessV1>() {
                return Err(NativeHostApiAdapterError::InvalidV4AccessSize {
                    actual: access.size_bytes,
                });
            }
            let mode = match access.mode {
                ZR_NATIVE_SYSTEM_ACCESS_MODE_READ_V1 => "read",
                ZR_NATIVE_SYSTEM_ACCESS_MODE_WRITE_V1 => "write",
                mode => return Err(NativeHostApiAdapterError::InvalidV4AccessMode { mode }),
            };
            let domain = match access.domain {
                ZR_NATIVE_SYSTEM_ACCESS_DOMAIN_COMPONENT_V1 => "component",
                ZR_NATIVE_SYSTEM_ACCESS_DOMAIN_RESOURCE_V1 => "resource",
                domain => return Err(NativeHostApiAdapterError::InvalidV4AccessDomain { domain }),
            };
            let stable_id = unsafe { read_utf8(access.stable_id) }?;
            Ok(format!("{mode}:{domain}:{stable_id}"))
        })
        .collect()
}

fn v4_thread_affinity_from_abi(
    affinity: u32,
) -> NativeHostApiAdapterResult<NativePluginRegistrationThreadAffinity> {
    match affinity {
        ZR_NATIVE_SYSTEM_THREAD_AFFINITY_MAIN_THREAD_ONLY_V1 => {
            Ok(NativePluginRegistrationThreadAffinity::MainThreadOnly)
        }
        ZR_NATIVE_SYSTEM_THREAD_AFFINITY_WORKER_SAFE_V1 => {
            Ok(NativePluginRegistrationThreadAffinity::WorkerSafe)
        }
        affinity => Err(NativeHostApiAdapterError::InvalidV4ThreadAffinity { affinity }),
    }
}

unsafe fn read_utf8(slice: ZrByteSlice) -> NativeHostApiAdapterResult<String> {
    std::str::from_utf8(unsafe { slice.as_slice() })
        .map(str::to_string)
        .map_err(|source| NativeHostApiAdapterError::InvalidUtf8 { source })
}

fn plugin_id_from_runtime_module_name(module_name: &str) -> Option<&str> {
    module_name.strip_suffix(".runtime")
}

fn context_for(handle: ZrRuntimePluginHandle) -> Option<NativeHostApiV3RegistrationContext> {
    if !handle.is_valid() {
        return None;
    }
    match contexts().get(handle.raw()).as_deref()? {
        NativeHostApiV3Context::Registration(context) => Some(*context),
        NativeHostApiV3Context::RegistrationV4(context) => Some(context.v3_context()),
        NativeHostApiV3Context::BridgeCall(_) => None,
    }
}

fn context_for_v4(handle: ZrRuntimePluginHandle) -> Option<NativeHostApiV4RegistrationContext> {
    if !handle.is_valid() {
        return None;
    }
    match contexts().get(handle.raw()).as_deref()? {
        NativeHostApiV3Context::RegistrationV4(context) => Some(context.clone()),
        NativeHostApiV3Context::Registration(_) | NativeHostApiV3Context::BridgeCall(_) => None,
    }
}

fn bridge_context_for(
    handle: ZrRuntimePluginHandle,
) -> Result<Arc<NativeHostBridgeCallContext>, ZrStatusCode> {
    if !handle.is_valid() {
        return Err(ZrStatusCode::NotFound);
    }
    match contexts().get(handle.raw()).as_deref() {
        Some(NativeHostApiV3Context::Registration(_))
        | Some(NativeHostApiV3Context::RegistrationV4(_)) => Err(ZrStatusCode::UnsupportedVersion),
        Some(NativeHostApiV3Context::BridgeCall(context)) => Ok(context.clone()),
        None => Err(ZrStatusCode::NotFound),
    }
}

fn contexts() -> &'static HostContextRegistry<NativeHostApiV3Context> {
    static CONTEXTS: OnceLock<HostContextRegistry<NativeHostApiV3Context>> = OnceLock::new();
    CONTEXTS.get_or_init(HostContextRegistry::default)
}

fn status(code: ZrStatusCode) -> ZrStatus {
    ZrStatus::new(code, ZrByteSlice::empty())
}

#[cfg(test)]
#[path = "host_api_adapter/tests.rs"]
mod tests;
