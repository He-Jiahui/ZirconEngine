use crate::diagnostic_log::write_log;
use crate::script::{
    VmError, VmPluginHostContext, VmPluginInstance, VmPluginPackage, ZrVmExecutionMode,
};
use zr_vm_rust_binding as zrvm;

use super::errors::map_zr_error;
use super::host_modules::register_host_modules;
use super::instance::ZrVmPluginInstance;
use super::lock::acquire_zr_vm_lock;

pub(in crate::script::vm::backend::zr_vm_project_backend) fn load_project_package(
    package: &VmPluginPackage,
    host: &VmPluginHostContext,
) -> Result<Box<dyn VmPluginInstance>, VmError> {
    let project = package
        .zr_vm_project
        .as_ref()
        .ok_or_else(|| VmError::Parse("zr_vm project metadata missing".to_string()))?;
    write_log(
        "zr_vm_project_backend",
        format!(
            "zr_vm_project_package_load_start package={} project={} mode={:?}",
            package.manifest.name,
            project.project_path.display(),
            project.execution_mode
        ),
    );
    let _guard = acquire_zr_vm_lock();
    let mut runtime = zrvm::RuntimeBuilder::standard()
        .build()
        .map_err(map_zr_error)?;
    write_log("zr_vm_project_backend", "zr_vm_project_runtime_built");
    let registrations = register_host_modules(&mut runtime, host)?;
    write_log(
        "zr_vm_project_backend",
        format!(
            "zr_vm_project_host_modules_registered count={}",
            registrations.len()
        ),
    );
    let workspace = zrvm::ProjectWorkspace::open(&project.project_path).map_err(map_zr_error)?;
    write_log("zr_vm_project_backend", "zr_vm_project_workspace_opened");
    workspace
        .compile(
            &mut runtime,
            &zrvm::CompileOptions {
                emit_intermediate: false,
                incremental: true,
            },
        )
        .map_err(map_zr_error)?;
    write_log("zr_vm_project_backend", "zr_vm_project_workspace_compiled");
    let run_options = zrvm::RunOptions {
        execution_mode: match project.execution_mode {
            ZrVmExecutionMode::Interp => zrvm::ExecutionMode::Interp,
            ZrVmExecutionMode::Binary => zrvm::ExecutionMode::Binary,
        },
        module_name: None,
        program_args: Vec::new(),
    };
    let session = workspace
        .start_session(&mut runtime, &run_options)
        .map_err(map_zr_error)?;
    write_log("zr_vm_project_backend", "zr_vm_project_session_started");

    let instance = ZrVmPluginInstance::new(
        package.manifest.clone(),
        session,
        registrations,
        runtime,
        project.entry_module.clone(),
    );
    write_log(
        "zr_vm_project_backend",
        format!(
            "zr_vm_project_package_load_done package={}",
            package.manifest.name
        ),
    );
    Ok(Box::new(instance))
}
