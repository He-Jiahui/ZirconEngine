use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use zircon_runtime::script::{
    register_builtin_host_modules, CapabilitySet, HostExportRegistry, HostRegistry,
};
use zr_vm_rust_binding::{ProjectWorkspace, RunOptions, RuntimeBuilder};

use crate::build_zr_vm_native_host_modules;
use crate::real_backend::lock::acquire_zr_vm_lock;

struct HostModuleProject(PathBuf);

impl HostModuleProject {
    fn new() -> Self {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        Self(std::env::temp_dir().join(format!(
            "zircon-native-host-modules-{}-{nonce}",
            std::process::id()
        )))
    }

    fn workspace(&self) -> ProjectWorkspace {
        let workspace = ProjectWorkspace::scaffold(&self.0, "native_host_modules").unwrap();
        fs::write(
            self.0.join("src/main.zr"),
            concat!(
                "module main;\n",
                "var math = %import(\"zr.zircon.math\");\n",
                "pub fn root(): float { return math.sqrt(81.0); }\n",
                "pub fn rounded(): float { return math.round(-1.5); }\n",
                "return 0;\n",
            ),
        )
        .unwrap();
        workspace
    }
}

impl Drop for HostModuleProject {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[test]
fn standalone_project_calls_runtime_math_exports_with_admitted_capability() {
    let _guard = acquire_zr_vm_lock();
    let fixture = HostModuleProject::new();
    let workspace = fixture.workspace();
    let registry = HostRegistry::default();
    let exports = HostExportRegistry::new(registry.clone());
    register_builtin_host_modules(&exports, &registry).unwrap();
    let mut runtime = RuntimeBuilder::standard().build().unwrap();
    let registrations =
        build_zr_vm_native_host_modules(&exports, &CapabilitySet::default().with("math.scalar"))
            .unwrap()
            .into_iter()
            .map(|module| runtime.register_native_module(module).unwrap())
            .collect::<Vec<_>>();
    let mut session = workspace
        .start_session(&mut runtime, &RunOptions::default())
        .unwrap();

    assert_eq!(
        session
            .call_module_export("main", "root", &[])
            .unwrap()
            .as_float()
            .unwrap(),
        9.0
    );
    assert_eq!(
        session
            .call_module_export("main", "rounded", &[])
            .unwrap()
            .as_float()
            .unwrap(),
        -1.0
    );

    drop(session);
    drop(registrations);
}

#[test]
fn standalone_project_rejects_runtime_math_without_admitted_capability() {
    let _guard = acquire_zr_vm_lock();
    let fixture = HostModuleProject::new();
    let workspace = fixture.workspace();
    let registry = HostRegistry::default();
    let exports = HostExportRegistry::new(registry.clone());
    register_builtin_host_modules(&exports, &registry).unwrap();
    let mut runtime = RuntimeBuilder::standard().build().unwrap();
    let registrations = build_zr_vm_native_host_modules(&exports, &CapabilitySet::default())
        .unwrap()
        .into_iter()
        .map(|module| runtime.register_native_module(module).unwrap())
        .collect::<Vec<_>>();
    let mut session = workspace
        .start_session(&mut runtime, &RunOptions::default())
        .unwrap();

    let error = session.call_module_export("main", "root", &[]).unwrap_err();
    assert!(error.to_string().contains("math.scalar"), "{error}");

    drop(session);
    drop(registrations);
}
