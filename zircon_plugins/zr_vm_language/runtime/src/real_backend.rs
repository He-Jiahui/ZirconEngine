#[path = "real_backend/errors.rs"]
mod errors;
#[path = "real_backend/extension_host.rs"]
mod extension_host;
#[path = "real_backend/host_modules.rs"]
mod host_modules;
#[path = "real_backend/instance.rs"]
mod instance;
#[path = "real_backend/lock.rs"]
mod lock;
#[path = "real_backend/package.rs"]
mod package;
#[path = "real_backend/reflection_host.rs"]
mod reflection_host;
#[path = "real_backend/runtime_owner.rs"]
mod runtime_owner;
#[path = "real_backend/values.rs"]
mod values;

type ZrVmRegistration = zr_vm_rust_binding::NativeModuleRegistration;

pub use host_modules::build_zr_vm_native_host_modules;
pub use package::load_project_package;

#[cfg(test)]
#[path = "real_backend/tests/cases.rs"]
mod tests;
