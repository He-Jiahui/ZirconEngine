use std::collections::BTreeMap;
use std::marker::PhantomData;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Mutex, OnceLock};

use zircon_runtime_interface::{
    ZrByteBufferRef, ZrByteSlice, ZrComponentDescV1, ZrEventTypeId, ZrHostApiV3, ZrHostAssetApiV1,
    ZrHostBridgeApiV1, ZrHostDiagnosticsApiV1, ZrHostEcsApiV1, ZrHostEventApiV1, ZrOwnedByteBuffer,
    ZrRuntimePluginHandle, ZrStatus, ZrStatusCode, ZrSystemRegistrationV1,
};

use crate::plugin::{
    BridgeInterfaceStatus, ComponentTypeDescriptor, FrozenBridgeTable, InterfaceSlot,
    PluginModuleId, RuntimeExtensionRegistry,
};
use crate::scene::ecs::{
    ChangeTickWindow, SystemParam, SystemParamAccess, SystemParamError, SystemRef, SystemStage,
};
use crate::scene::World;

use super::bridge_method_bindings::{
    NativeBridgeCall, NativeBridgeMethodDescriptor, NativeBridgeMethodFn,
};
use super::ffi_panic_guard::catch_native_host_api_panic;

#[cfg(test)]
use super::bridge_method_bindings::{
    native_bridge_method_descriptors_from_manifest, NativeBridgeMethodBinding,
    NativeBridgeMethodManifestError,
};
#[cfg(test)]
use crate::plugin::{PluginInterfaceMethodManifest, PluginPackageManifest};

pub struct NativeHostApiV3RegistrationScope<'registry> {
    handle: ZrRuntimePluginHandle,
    _registry: PhantomData<&'registry mut RuntimeExtensionRegistry>,
}

impl<'registry> NativeHostApiV3RegistrationScope<'registry> {
    pub fn new(
        registry: &'registry mut RuntimeExtensionRegistry,
        module_name: impl Into<String>,
    ) -> Result<Self, String> {
        let owner = registry
            .intern_plugin_module(module_name)
            .map_err(|error| error.to_string())?;
        let handle = ZrRuntimePluginHandle::new(next_host_handle());
        lock_contexts().insert(
            handle.raw(),
            NativeHostApiV3Context::Registration(NativeHostApiV3RegistrationContext {
                registry: registry as *mut RuntimeExtensionRegistry as usize,
                owner,
            }),
        );
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

impl Drop for NativeHostApiV3RegistrationScope<'_> {
    fn drop(&mut self) {
        lock_contexts().remove(&self.handle.raw());
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
        let handle = ZrRuntimePluginHandle::new(next_host_handle());
        let methods = methods
            .into_iter()
            .map(|(slot, method_slot, method)| ((slot.raw(), method_slot), method))
            .collect();
        lock_contexts().insert(
            handle.raw(),
            NativeHostApiV3Context::BridgeCall(NativeHostBridgeCallContext { table, methods }),
        );
        Self { handle }
    }

    pub fn from_method_descriptors(
        table: FrozenBridgeTable,
        descriptors: impl IntoIterator<Item = NativeBridgeMethodDescriptor>,
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
        Ok(Self::with_methods(table, methods))
    }

    pub const fn handle(&self) -> ZrRuntimePluginHandle {
        self.handle
    }

    pub fn method_count(&self) -> usize {
        match lock_contexts().get(&self.handle.raw()) {
            Some(NativeHostApiV3Context::BridgeCall(context)) => context.methods.len(),
            Some(NativeHostApiV3Context::Registration(_)) | None => 0,
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
        lock_contexts().remove(&self.handle.raw());
    }
}

#[derive(Clone, Copy)]
struct NativeHostApiV3RegistrationContext {
    registry: usize,
    owner: PluginModuleId,
}

#[derive(Clone)]
struct NativeHostBridgeCallContext {
    table: FrozenBridgeTable,
    methods: BTreeMap<(u32, u32), NativeBridgeMethodFn>,
}

#[derive(Clone)]
enum NativeHostApiV3Context {
    Registration(NativeHostApiV3RegistrationContext),
    BridgeCall(NativeHostBridgeCallContext),
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
    let Some(snapshot) = context.table.interface_snapshot(slot) else {
        return status(ZrStatusCode::NotFound);
    };
    if snapshot.status != BridgeInterfaceStatus::Enabled {
        context.table.record_not_enabled_call(slot);
        return status(ZrStatusCode::BridgeNotEnabled);
    }
    let Some(method) = context.methods.get(&(interface_slot, method_slot)).copied() else {
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
) -> Result<(), String> {
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
                .map_err(|error| error.to_string())
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
    builder.register().map_err(|error| error.to_string())
}

struct NativeDynamicAccess;

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
) -> Result<(), String> {
    let type_id = read_utf8(descriptor.type_id)?;
    let display_name = read_utf8(descriptor.display_name)?;
    let registry = registry_from_context(context);
    let plugin_id = registry
        .plugin_module_name(context.owner)
        .and_then(plugin_id_from_runtime_module_name)
        .ok_or_else(|| format!("unknown plugin module owner {}", context.owner.raw()))?
        .to_string();
    registry
        .register_component(ComponentTypeDescriptor::new(
            type_id,
            plugin_id,
            display_name,
        ))
        .map_err(|error| error.to_string())
}

unsafe fn registry_from_context<'a>(
    context: NativeHostApiV3RegistrationContext,
) -> &'a mut RuntimeExtensionRegistry {
    &mut *(context.registry as *mut RuntimeExtensionRegistry)
}

fn stage_from_abi(stage: u32) -> Result<SystemStage, String> {
    SystemStage::ORDER
        .get(stage as usize)
        .copied()
        .ok_or_else(|| format!("unknown native system stage {stage}"))
}

unsafe fn read_byte_slices(
    values: *const ZrByteSlice,
    count: usize,
) -> Result<Vec<String>, String> {
    if values.is_null() || count == 0 {
        return Ok(Vec::new());
    }
    unsafe { std::slice::from_raw_parts(values, count) }
        .iter()
        .copied()
        .map(|slice| unsafe { read_utf8(slice) })
        .collect()
}

unsafe fn read_utf8(slice: ZrByteSlice) -> Result<String, String> {
    std::str::from_utf8(unsafe { slice.as_slice() })
        .map(str::to_string)
        .map_err(|error| error.to_string())
}

fn plugin_id_from_runtime_module_name(module_name: &str) -> Option<&str> {
    module_name.strip_suffix(".runtime")
}

fn context_for(handle: ZrRuntimePluginHandle) -> Option<NativeHostApiV3RegistrationContext> {
    if !handle.is_valid() {
        return None;
    }
    match lock_contexts().get(&handle.raw()).cloned()? {
        NativeHostApiV3Context::Registration(context) => Some(context),
        NativeHostApiV3Context::BridgeCall(_) => None,
    }
}

fn bridge_context_for(
    handle: ZrRuntimePluginHandle,
) -> Result<NativeHostBridgeCallContext, ZrStatusCode> {
    if !handle.is_valid() {
        return Err(ZrStatusCode::NotFound);
    }
    match lock_contexts().get(&handle.raw()).cloned() {
        Some(NativeHostApiV3Context::Registration(_)) => Err(ZrStatusCode::UnsupportedVersion),
        Some(NativeHostApiV3Context::BridgeCall(context)) => Ok(context),
        None => Err(ZrStatusCode::NotFound),
    }
}

fn lock_contexts() -> std::sync::MutexGuard<'static, BTreeMap<u64, NativeHostApiV3Context>> {
    contexts()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

fn contexts() -> &'static Mutex<BTreeMap<u64, NativeHostApiV3Context>> {
    static CONTEXTS: OnceLock<Mutex<BTreeMap<u64, NativeHostApiV3Context>>> = OnceLock::new();
    CONTEXTS.get_or_init(|| Mutex::new(BTreeMap::new()))
}

fn next_host_handle() -> u64 {
    static NEXT_HOST_HANDLE: AtomicU64 = AtomicU64::new(1);
    NEXT_HOST_HANDLE.fetch_add(1, Ordering::Relaxed)
}

fn status(code: ZrStatusCode) -> ZrStatus {
    ZrStatus::new(code, ZrByteSlice::empty())
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::*;
    use crate::core::framework::bridge::PluginInterface;
    use crate::plugin::PluginInterfaceManifest;

    trait NativeWeatherBridge: Send + Sync {}

    impl PluginInterface for dyn NativeWeatherBridge {
        const INTERFACE_ID: &'static str = "native.weather.bridge.v1";
    }

    struct NativeWeatherProvider;

    impl NativeWeatherBridge for NativeWeatherProvider {}

    #[test]
    fn native_host_api_v3_registers_systems_and_components_into_runtime_registry() {
        let mut registry = RuntimeExtensionRegistry::default();
        let scope =
            NativeHostApiV3RegistrationScope::new(&mut registry, "weather.runtime").unwrap();
        let api = scope.api();
        let set_names = [ZrByteSlice::from_static(b"weather.main")];
        let after = [ZrByteSlice::from_static(b"weather.bootstrap")];
        let system = ZrSystemRegistrationV1 {
            system_id: ZrByteSlice::from_static(b"weather.native_tick"),
            stage: SystemStage::ORDER
                .iter()
                .position(|stage| *stage == SystemStage::Update)
                .unwrap() as u32,
            order: 3,
            set_names: set_names.as_ptr(),
            set_count: set_names.len(),
            after: after.as_ptr(),
            after_count: after.len(),
            ..ZrSystemRegistrationV1::empty(3)
        };
        let component = ZrComponentDescV1 {
            type_id: ZrByteSlice::from_static(b"weather.native_component"),
            display_name: ZrByteSlice::from_static(b"Native Weather Component"),
            schema: ZrByteSlice::from_static(br#"{"fields":[]}"#),
            storage_kind: 1,
            ..ZrComponentDescV1::empty(3)
        };

        let system_status = unsafe { (api.ecs.register_system.unwrap())(scope.handle(), &system) };
        let component_status =
            unsafe { (api.ecs.register_component.unwrap())(scope.handle(), &component) };
        drop(scope);

        assert!(system_status.is_ok());
        assert!(component_status.is_ok());
        let systems = registry.plugin_systems().collect::<Vec<_>>();
        assert_eq!(systems.len(), 1);
        assert_eq!(systems[0].1.id, "weather.native_tick");
        assert_eq!(systems[0].1.stage, SystemStage::Update);
        assert_eq!(systems[0].1.order, 3);
        assert_eq!(systems[0].1.sets.len(), 1);
        assert_eq!(systems[0].1.constraints.len(), 1);
        let mut world = crate::scene::World::default();
        let native_system = systems[0]
            .1
            .build(&mut world)
            .expect("native ABI system should build into a schedule node");
        assert!(native_system.access().has_conservative_world_access());
        assert!(native_system
            .access()
            .conflicts_with(&crate::scene::ecs::SystemParamAccess::default()));
        assert_eq!(
            native_system
                .access()
                .conflict_kinds_with(&crate::scene::ecs::SystemParamAccess::default()),
            vec![crate::scene::ecs::SystemParamConflictKind::World]
        );
        assert_eq!(registry.components().len(), 1);
        assert_eq!(registry.components()[0].type_id, "weather.native_component");
        assert_eq!(registry.components()[0].plugin_id, "weather");
    }

    #[test]
    fn native_system_enters_schedule_as_conservative_node() {
        native_host_api_v3_registers_systems_and_components_into_runtime_registry();
    }

    #[test]
    fn native_host_api_v3_rejects_unknown_registration_handles() {
        let api = NativeHostApiV3RegistrationScope {
            handle: ZrRuntimePluginHandle::invalid(),
            _registry: PhantomData,
        }
        .api();
        let system = ZrSystemRegistrationV1 {
            system_id: ZrByteSlice::from_static(b"weather.native_tick"),
            ..ZrSystemRegistrationV1::empty(3)
        };

        let status = unsafe {
            (api.ecs.register_system.unwrap())(ZrRuntimePluginHandle::new(9999), &system)
        };

        assert_eq!(status.status_code(), ZrStatusCode::NotFound);
    }

    #[test]
    fn native_host_api_v3_exposes_bridge_domain_as_unsupported_until_connected() {
        let mut registry = RuntimeExtensionRegistry::default();
        let scope =
            NativeHostApiV3RegistrationScope::new(&mut registry, "weather.runtime").unwrap();
        let api = scope.api();

        let status = unsafe {
            (api.bridge.call.unwrap())(
                scope.handle(),
                1,
                2,
                std::ptr::null(),
                0,
                ZrByteBufferRef::empty(),
            )
        };

        assert_eq!(status.status_code(), ZrStatusCode::UnsupportedVersion);
    }

    #[test]
    fn native_host_bridge_call_scope_dispatches_registered_method() {
        let mut registry = RuntimeExtensionRegistry::default();
        let owner = registry.intern_plugin_module("weather.runtime").unwrap();
        registry
            .export_interface::<dyn NativeWeatherBridge>(owner, Arc::new(NativeWeatherProvider))
            .unwrap();
        let table = registry.frozen_bridge_table();
        let slot = table
            .resolve_slot(<dyn NativeWeatherBridge as PluginInterface>::INTERFACE_ID)
            .unwrap();
        let scope = NativeHostBridgeCallScope::with_methods(
            table.clone(),
            [(
                slot,
                7,
                NativeBridgeMethodFn::from_rust(native_bridge_test_method),
            )],
        );
        let api = scope.api();
        let payload = b"ping";

        let status = unsafe {
            (api.bridge.call.unwrap())(
                scope.handle(),
                slot.raw(),
                7,
                payload.as_ptr(),
                payload.len(),
                ZrByteBufferRef::empty(),
            )
        };

        assert_eq!(status.status_code(), ZrStatusCode::CapabilityDenied);
        assert_eq!(
            table.diagnostics(slot).unwrap().enabled_calls,
            debug_bridge_counter_value(1)
        );
    }

    #[test]
    fn native_host_bridge_call_scope_builds_method_table_from_interface_metadata() {
        let mut registry = RuntimeExtensionRegistry::default();
        let owner = registry.intern_plugin_module("weather.runtime").unwrap();
        registry
            .export_interface::<dyn NativeWeatherBridge>(owner, Arc::new(NativeWeatherProvider))
            .unwrap();
        let table = registry.frozen_bridge_table();
        let slot = table
            .resolve_slot(<dyn NativeWeatherBridge as PluginInterface>::INTERFACE_ID)
            .unwrap();
        let scope = NativeHostBridgeCallScope::from_method_descriptors(
            table.clone(),
            [NativeBridgeMethodDescriptor::new(
                <dyn NativeWeatherBridge as PluginInterface>::INTERFACE_ID,
                7,
                NativeBridgeMethodFn::from_rust(native_bridge_test_method),
            )],
        )
        .expect("native bridge method metadata should resolve");
        let api = scope.api();
        let payload = b"ping";

        let status = unsafe {
            (api.bridge.call.unwrap())(
                scope.handle(),
                slot.raw(),
                7,
                payload.as_ptr(),
                payload.len(),
                ZrByteBufferRef::empty(),
            )
        };

        assert_eq!(status.status_code(), ZrStatusCode::CapabilityDenied);
    }

    #[test]
    fn native_host_bridge_call_catches_plugin_method_panic() {
        let mut registry = RuntimeExtensionRegistry::default();
        let owner = registry.intern_plugin_module("weather.runtime").unwrap();
        registry
            .export_interface::<dyn NativeWeatherBridge>(owner, Arc::new(NativeWeatherProvider))
            .unwrap();
        let table = registry.frozen_bridge_table();
        let slot = table
            .resolve_slot(<dyn NativeWeatherBridge as PluginInterface>::INTERFACE_ID)
            .unwrap();
        let scope = NativeHostBridgeCallScope::with_methods(
            table.clone(),
            [(
                slot,
                7,
                NativeBridgeMethodFn::from_rust(native_bridge_panic_method),
            )],
        );
        let api = scope.api();

        let status = unsafe {
            (api.bridge.call.unwrap())(
                scope.handle(),
                slot.raw(),
                7,
                std::ptr::null(),
                0,
                ZrByteBufferRef::empty(),
            )
        };

        assert_eq!(status.status_code(), ZrStatusCode::Panic);
        assert_eq!(
            table.diagnostics(slot).unwrap().enabled_calls,
            debug_bridge_counter_value(1)
        );
    }

    #[test]
    fn native_bridge_method_descriptors_use_package_manifest_metadata() {
        let mut registry = RuntimeExtensionRegistry::default();
        let owner = registry.intern_plugin_module("weather.runtime").unwrap();
        registry
            .export_interface::<dyn NativeWeatherBridge>(owner, Arc::new(NativeWeatherProvider))
            .unwrap();
        let table = registry.frozen_bridge_table();
        let slot = table
            .resolve_slot(<dyn NativeWeatherBridge as PluginInterface>::INTERFACE_ID)
            .unwrap();
        let manifest = PluginPackageManifest::new("weather", "Weather").with_provided_interface(
            PluginInterfaceManifest::new(
                <dyn NativeWeatherBridge as PluginInterface>::INTERFACE_ID,
            )
            .with_method(PluginInterfaceMethodManifest::new("sample_temperature", 7)),
        );
        let descriptors = native_bridge_method_descriptors_from_manifest(
            &manifest,
            [NativeBridgeMethodBinding::new(
                <dyn NativeWeatherBridge as PluginInterface>::INTERFACE_ID,
                "sample_temperature",
                NativeBridgeMethodFn::from_rust(native_bridge_test_method),
            )],
        )
        .expect("native bridge descriptors from manifest");
        assert_eq!(descriptors.len(), 1);
        assert_eq!(
            descriptors[0].interface_id(),
            <dyn NativeWeatherBridge as PluginInterface>::INTERFACE_ID
        );
        assert_eq!(descriptors[0].method_slot(), 7);
        let scope = NativeHostBridgeCallScope::from_method_descriptors(table, descriptors).unwrap();
        let api = scope.api();
        let payload = b"ping";

        let status = unsafe {
            (api.bridge.call.unwrap())(
                scope.handle(),
                slot.raw(),
                7,
                payload.as_ptr(),
                payload.len(),
                ZrByteBufferRef::empty(),
            )
        };

        assert_eq!(status.status_code(), ZrStatusCode::CapabilityDenied);
    }

    #[test]
    fn native_bridge_method_descriptors_reject_missing_manifest_binding() {
        let manifest = PluginPackageManifest::new("weather", "Weather").with_provided_interface(
            PluginInterfaceManifest::new(
                <dyn NativeWeatherBridge as PluginInterface>::INTERFACE_ID,
            )
            .with_method(PluginInterfaceMethodManifest::new("sample_temperature", 7)),
        );

        let result = native_bridge_method_descriptors_from_manifest(&manifest, []);

        assert!(matches!(
            result,
            Err(NativeBridgeMethodManifestError::MissingBinding {
                interface_id,
                method_name
            }) if interface_id == <dyn NativeWeatherBridge as PluginInterface>::INTERFACE_ID
                && method_name == "sample_temperature"
        ));
    }

    #[test]
    fn native_host_bridge_call_scope_rejects_unknown_interface_metadata() {
        let table = RuntimeExtensionRegistry::default().frozen_bridge_table();

        let result = NativeHostBridgeCallScope::from_method_descriptors(
            table,
            [NativeBridgeMethodDescriptor::new(
                "native.missing.bridge.v1",
                1,
                NativeBridgeMethodFn::from_rust(native_bridge_test_method),
            )],
        );

        assert!(matches!(
            result,
            Err(crate::plugin::RuntimeExtensionRegistryError::MissingPluginInterface(
                interface_id
            )) if interface_id == "native.missing.bridge.v1"
        ));
    }

    #[test]
    fn native_host_bridge_call_maps_disabled_provider_to_bridge_not_enabled() {
        let mut registry = RuntimeExtensionRegistry::default();
        let owner = registry.intern_plugin_module("weather.runtime").unwrap();
        registry
            .export_interface::<dyn NativeWeatherBridge>(owner, Arc::new(NativeWeatherProvider))
            .unwrap();
        let table = registry.frozen_bridge_table();
        let slot = table
            .resolve_slot(<dyn NativeWeatherBridge as PluginInterface>::INTERFACE_ID)
            .unwrap();
        table.set_enabled(slot, false).unwrap();
        let scope = NativeHostBridgeCallScope::with_methods(
            table.clone(),
            [(
                slot,
                7,
                NativeBridgeMethodFn::from_rust(native_bridge_test_method),
            )],
        );
        let api = scope.api();

        let status = unsafe {
            (api.bridge.call.unwrap())(
                scope.handle(),
                slot.raw(),
                7,
                std::ptr::null(),
                0,
                ZrByteBufferRef::empty(),
            )
        };

        assert_eq!(status.status_code(), ZrStatusCode::BridgeNotEnabled);
        assert_eq!(
            table.diagnostics(slot).unwrap().not_enabled_calls,
            debug_bridge_counter_value(1)
        );
    }

    #[test]
    fn native_host_bridge_call_reports_absent_slot_and_missing_method() {
        let mut registry = RuntimeExtensionRegistry::default();
        let owner = registry.intern_plugin_module("weather.runtime").unwrap();
        registry
            .export_interface::<dyn NativeWeatherBridge>(owner, Arc::new(NativeWeatherProvider))
            .unwrap();
        let table = registry.frozen_bridge_table();
        let slot = table
            .resolve_slot(<dyn NativeWeatherBridge as PluginInterface>::INTERFACE_ID)
            .unwrap();
        let scope = NativeHostBridgeCallScope::new(table);
        let api = scope.api();

        let absent_slot_status = unsafe {
            (api.bridge.call.unwrap())(
                scope.handle(),
                99,
                7,
                std::ptr::null(),
                0,
                ZrByteBufferRef::empty(),
            )
        };
        let missing_method_status = unsafe {
            (api.bridge.call.unwrap())(
                scope.handle(),
                slot.raw(),
                99,
                std::ptr::null(),
                0,
                ZrByteBufferRef::empty(),
            )
        };

        assert_eq!(absent_slot_status.status_code(), ZrStatusCode::NotFound);
        assert_eq!(missing_method_status.status_code(), ZrStatusCode::NotFound);
    }

    #[test]
    fn native_host_api_v3_preserves_dotted_plugin_ids() {
        let mut registry = RuntimeExtensionRegistry::default();
        let scope = NativeHostApiV3RegistrationScope::new(&mut registry, "net.rpc.runtime")
            .expect("dotted plugin runtime module owner");
        let api = scope.api();
        let component = ZrComponentDescV1 {
            type_id: ZrByteSlice::from_static(b"net.rpc.NativePayload"),
            display_name: ZrByteSlice::from_static(b"Native RPC Payload"),
            ..ZrComponentDescV1::empty(3)
        };

        let status = unsafe { (api.ecs.register_component.unwrap())(scope.handle(), &component) };
        drop(scope);

        assert!(status.is_ok());
        assert_eq!(registry.components().len(), 1);
        assert_eq!(registry.components()[0].plugin_id, "net.rpc");
    }

    fn native_bridge_test_method(call: NativeBridgeCall) -> ZrStatus {
        let payload = unsafe { call.payload.as_slice() };
        if call.interface_slot == 0 && call.method_slot == 7 && payload == b"ping" {
            status(ZrStatusCode::CapabilityDenied)
        } else {
            status(ZrStatusCode::InvalidArgument)
        }
    }

    fn native_bridge_panic_method(_call: NativeBridgeCall) -> ZrStatus {
        panic!("native bridge method panic")
    }

    #[cfg(debug_assertions)]
    fn debug_bridge_counter_value(value: u64) -> u64 {
        value
    }

    #[cfg(not(debug_assertions))]
    fn debug_bridge_counter_value(_: u64) -> u64 {
        0
    }
}
