use zircon_runtime::script::{
    VmError, VmPluginHostContext, VmPluginInstance, VmPluginPackage, ZrVmExecutionMode,
};
use zr_vm_rust_binding as zrvm;

use super::errors::map_zr_error;
use super::host_modules::register_host_modules;
use super::instance::ZrVmPluginInstance;
use super::lock::acquire_zr_vm_lock;
use super::runtime_owner::ZrVmRuntimeOwner;

pub fn load_project_package(
    package: &VmPluginPackage,
    host: &VmPluginHostContext,
) -> Result<Box<dyn VmPluginInstance>, VmError> {
    let project = package
        .zr_vm_project
        .as_ref()
        .ok_or_else(|| VmError::Parse("zr_vm project metadata missing".to_string()))?;
    let _guard = acquire_zr_vm_lock();
    let mut runtime = zrvm::RuntimeBuilder::standard()
        .build()
        .map_err(map_zr_error)?;
    let host_modules = register_host_modules(&mut runtime, host)?;
    let reflection_host = host_modules.reflection_host.clone();
    let reflection_catalog = host.reflection_catalog.clone();
    host.reflection_schema_installer.register(move |snapshot| {
        reflection_host
            .install_registry_snapshot(snapshot, &reflection_catalog)
            .map_err(|error| {
                VmError::Operation(format!(
                    "failed to compile canonical VM reflection call table: {error}"
                ))
            })
    })?;
    let workspace = zrvm::ProjectWorkspace::open(&project.project_path).map_err(map_zr_error)?;
    workspace
        .compile(
            &mut runtime,
            &zrvm::CompileOptions {
                emit_intermediate: false,
                incremental: true,
            },
        )
        .map_err(map_zr_error)?;
    let run_options = zrvm::RunOptions {
        execution_mode: match project.execution_mode {
            ZrVmExecutionMode::Interp => zrvm::ExecutionMode::Interp,
            ZrVmExecutionMode::Binary => zrvm::ExecutionMode::Binary,
        },
        // Lifecycle export calls name the target module separately; keeping
        // this empty makes ZrVM load the project entry before resolving it.
        module_name: None,
        program_args: Vec::new(),
    };
    let session = workspace
        .start_session(&mut runtime, &run_options)
        .map_err(map_zr_error)?;

    Ok(Box::new(ZrVmPluginInstance::new(
        package.manifest.clone(),
        ZrVmRuntimeOwner::new(session, host_modules.registrations, runtime),
        project.entry_module.clone(),
    )))
}
