use zircon_runtime::script::{
    VmError, VmPluginHostContext, VmPluginInstance, VmPluginPackage, ZrVmExecutionMode,
};
use zr_vm_rust_binding as zrvm;

use super::errors::map_zr_error;
use super::host_modules::register_host_modules;
use super::instance::ZrVmPluginInstance;
use super::lock::acquire_zr_vm_lock;

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
    let registrations = register_host_modules(&mut runtime, host)?;
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
        session,
        registrations,
        runtime,
        project.entry_module.clone(),
    )))
}
