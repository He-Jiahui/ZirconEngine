use std::collections::BTreeMap;
use std::marker::PhantomData;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Mutex, OnceLock};

use zircon_runtime_interface::{
    ZrByteBufferRef, ZrByteSlice, ZrComponentDescV1, ZrEventTypeId, ZrHostApiV3, ZrHostAssetApiV1,
    ZrHostDiagnosticsApiV1, ZrHostEcsApiV1, ZrHostEventApiV1, ZrOwnedByteBuffer,
    ZrRuntimePluginHandle, ZrStatus, ZrStatusCode, ZrSystemRegistrationV1,
};

use crate::plugin::{ComponentTypeDescriptor, PluginModuleId, RuntimeExtensionRegistry};
use crate::scene::ecs::{SystemRef, SystemStage};

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
            NativeHostApiV3RegistrationContext {
                registry: registry as *mut RuntimeExtensionRegistry as usize,
                owner,
            },
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

#[derive(Clone, Copy)]
struct NativeHostApiV3RegistrationContext {
    registry: usize,
    owner: PluginModuleId,
}

unsafe extern "C" fn native_host_register_system_v1(
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
    status(ZrStatusCode::UnsupportedVersion)
}

unsafe extern "C" fn native_host_asset_request_v1(
    _handle: ZrRuntimePluginHandle,
    _request: ZrByteSlice,
    _output: *mut ZrOwnedByteBuffer,
) -> ZrStatus {
    status(ZrStatusCode::UnsupportedVersion)
}

unsafe extern "C" fn native_host_event_emit_v1(
    _handle: ZrRuntimePluginHandle,
    _event_type: ZrEventTypeId,
    _payload: *const u8,
    _len: usize,
) -> ZrStatus {
    status(ZrStatusCode::UnsupportedVersion)
}

unsafe extern "C" fn native_host_event_drain_v1(
    _handle: ZrRuntimePluginHandle,
    _event_type: ZrEventTypeId,
    _buffer: ZrByteBufferRef,
) -> ZrStatus {
    status(ZrStatusCode::UnsupportedVersion)
}

unsafe extern "C" fn native_host_diagnostics_emit_v1(
    _handle: ZrRuntimePluginHandle,
    _target: ZrByteSlice,
    _message: ZrByteSlice,
) -> ZrStatus {
    ZrStatus::ok()
}

unsafe extern "C" fn native_host_diagnostics_metric_v1(
    _handle: ZrRuntimePluginHandle,
    _path: ZrByteSlice,
    _value: f64,
    _unit: ZrByteSlice,
) -> ZrStatus {
    ZrStatus::ok()
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
        .register_native_system::<(), _>(context.owner, id, stage, move |()| {
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
    lock_contexts().get(&handle.raw()).copied()
}

fn lock_contexts(
) -> std::sync::MutexGuard<'static, BTreeMap<u64, NativeHostApiV3RegistrationContext>> {
    contexts()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

fn contexts() -> &'static Mutex<BTreeMap<u64, NativeHostApiV3RegistrationContext>> {
    static CONTEXTS: OnceLock<Mutex<BTreeMap<u64, NativeHostApiV3RegistrationContext>>> =
        OnceLock::new();
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
    use super::*;

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
        assert_eq!(registry.components().len(), 1);
        assert_eq!(registry.components()[0].type_id, "weather.native_component");
        assert_eq!(registry.components()[0].plugin_id, "weather");
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
}
