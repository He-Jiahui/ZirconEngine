use std::sync::Arc;

use zircon_runtime_interface::{
    ZrByteBufferRef, ZrByteSlice, ZrComponentDescV1, ZrEventTypeId, ZrHostAssetApiV1,
    ZrHostBridgeApiV1, ZrHostDiagnosticsApiV1, ZrHostEventApiV1, ZrOwnedByteBuffer,
    ZrRuntimePluginHandle, ZrStatus, ZrStatusCode, ZrSystemRegistrationV1, ZrSystemRegistrationV2,
};

use crate::core::framework::scene::ComponentTypeDescriptor;
use crate::plugin::{PluginModuleId, RuntimeExtensionRegistry};
use crate::scene::ecs::{
    ChangeTickWindow, SystemParam, SystemParamAccess, SystemParamError, SystemRef,
};
use crate::scene::World;

use super::super::ffi_panic_guard::catch_native_host_api_panic;
use super::super::registration_manifest::{NativeSystemAccessAuthority, NativeSystemAccessPlan};
use super::abi_decode::{
    read_byte_slices, read_utf8, read_v4_byte_slices, read_v4_system_accesses, stage_from_abi,
    v4_thread_affinity_from_abi, validate_v4_registration_header, NativeHostApiAdapterError,
    NativeHostApiAdapterResult,
};
use super::context_handles::{context_for, context_for_v4, NativeHostRegistrationScopeState};
use super::registration_policy::NativeHostApiV4RegistrationContext;

#[derive(Clone)]
pub(super) struct NativeHostApiV3RegistrationContext {
    pub(super) registry: usize,
    pub(super) owner: PluginModuleId,
    pub(super) lifetime: Arc<NativeHostRegistrationScopeState>,
}

pub(super) unsafe extern "C" fn native_host_register_system_v1(
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
    let result = unsafe { register_system_from_abi(&context, handle, registration) };
    match result {
        Ok(()) => ZrStatus::ok(),
        Err(_) => status(ZrStatusCode::Error),
    }
}

pub(super) unsafe extern "C" fn native_host_register_system_v2(
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
    let result = unsafe { register_system_from_abi_v4(&context, handle, registration) };
    match result {
        Ok(()) => ZrStatus::ok(),
        Err(_) => status(ZrStatusCode::Error),
    }
}

#[allow(clippy::needless_pass_by_value)]
pub(super) unsafe extern "C" fn native_host_register_component_v1(
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
    let result = unsafe { register_component_from_abi(&context, descriptor) };
    match result {
        Ok(()) => ZrStatus::ok(),
        Err(_) => status(ZrStatusCode::Error),
    }
}

pub(super) unsafe extern "C" fn native_host_spawn_command_v1(
    _handle: ZrRuntimePluginHandle,
    _payload: *const u8,
    _len: usize,
) -> ZrStatus {
    catch_native_host_api_panic(|| status(ZrStatusCode::UnsupportedVersion))
}

pub(super) unsafe extern "C" fn native_host_asset_request_v1(
    _handle: ZrRuntimePluginHandle,
    _request: ZrByteSlice,
    _output: *mut ZrOwnedByteBuffer,
) -> ZrStatus {
    catch_native_host_api_panic(|| status(ZrStatusCode::UnsupportedVersion))
}

pub(super) unsafe extern "C" fn native_host_event_emit_v1(
    _handle: ZrRuntimePluginHandle,
    _event_type: ZrEventTypeId,
    _payload: *const u8,
    _len: usize,
) -> ZrStatus {
    catch_native_host_api_panic(|| status(ZrStatusCode::UnsupportedVersion))
}

pub(super) unsafe extern "C" fn native_host_event_drain_v1(
    _handle: ZrRuntimePluginHandle,
    _event_type: ZrEventTypeId,
    _buffer: ZrByteBufferRef,
) -> ZrStatus {
    catch_native_host_api_panic(|| status(ZrStatusCode::UnsupportedVersion))
}

pub(super) unsafe extern "C" fn native_host_diagnostics_emit_v1(
    _handle: ZrRuntimePluginHandle,
    _target: ZrByteSlice,
    _message: ZrByteSlice,
) -> ZrStatus {
    catch_native_host_api_panic(ZrStatus::ok)
}

pub(super) unsafe extern "C" fn native_host_diagnostics_metric_v1(
    _handle: ZrRuntimePluginHandle,
    _path: ZrByteSlice,
    _value: f64,
    _unit: ZrByteSlice,
) -> ZrStatus {
    catch_native_host_api_panic(ZrStatus::ok)
}

unsafe fn register_system_from_abi(
    context: &NativeHostApiV3RegistrationContext,
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
    let registry = unsafe { registry_from_context(context) };
    let sets = set_names
        .into_iter()
        .map(|set_name| {
            registry
                .intern_system_set(set_name)
                .map_err(|source| NativeHostApiAdapterError::InvalidSystemSet { source })
        })
        .collect::<Result<Vec<_>, _>>()?;

    let mut builder = registry
        .register_native_system::<NativeDynamicAccess, _>(context.owner, id, stage, move || {
            move |()| {
                if let Some(invoke) = invoke {
                    let _ = unsafe { invoke(handle, user_data, ZrByteSlice::empty()) };
                }
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
    context: &NativeHostApiV4RegistrationContext,
    handle: ZrRuntimePluginHandle,
    registration: &ZrSystemRegistrationV2,
) -> NativeHostApiAdapterResult<()> {
    validate_v4_registration_header(registration)?;

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
    let v3_context = context.v3_context();
    let registry = unsafe { registry_from_context(&v3_context) };
    let authority = NativeSystemAccessAuthority::new(
        context.plugin_id.clone(),
        registry
            .components()
            .iter()
            .map(|component| component.type_id.clone()),
        context.policy.known_resource_ids.clone(),
        context.policy.granted_capabilities.clone(),
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
                move || {
                    if let Some(invoke) = invoke {
                        let _ = unsafe { invoke(handle, user_data, ZrByteSlice::empty()) };
                    }
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
    context: &NativeHostApiV3RegistrationContext,
    descriptor: &ZrComponentDescV1,
) -> NativeHostApiAdapterResult<()> {
    let type_id = read_utf8(descriptor.type_id)?;
    let display_name = read_utf8(descriptor.display_name)?;
    let registry = unsafe { registry_from_context(context) };
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
    context: &NativeHostApiV3RegistrationContext,
) -> &'a mut RuntimeExtensionRegistry {
    &mut *(context.registry as *mut RuntimeExtensionRegistry)
}

pub(super) fn plugin_id_from_runtime_module_name(module_name: &str) -> Option<&str> {
    module_name.strip_suffix(".runtime")
}

fn status(code: ZrStatusCode) -> ZrStatus {
    ZrStatus::new(code, ZrByteSlice::empty())
}
