mod errors;
mod host_modules;
mod instance;
mod lock;
mod package;
mod values;

type ZrVmRegistration = zr_vm_rust_binding::NativeModuleRegistration;

pub use package::load_project_package;

#[cfg(test)]
mod tests;
