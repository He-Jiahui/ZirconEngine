use std::path::PathBuf;
use std::sync::Arc;

use crate::asset::AssetUri;
use crate::core::framework::ui::UI_MODULE_NAME;
use crate::core::{CoreError, CoreRuntime, InitLevel, ModuleDescriptor};
use crate::text::TextRuntimeContext;
use crate::ui::{UiConfig, UI_CONFIG_KEY, UI_RUNTIME_DRIVER_NAME};

use super::{RuntimeLoadedProjectManifest, RuntimePreparedProject, RuntimeProjectError};

fn project_with_roots(roots: &[&str]) -> RuntimePreparedProject {
    RuntimePreparedProject {
        root: PathBuf::from("ui-admission-unopened-project"),
        manifest: RuntimeLoadedProjectManifest {
            default_scene: "res://main.scene.toml".to_owned(),
            ui_roots: roots
                .iter()
                .map(|root| AssetUri::parse(root).unwrap())
                .collect(),
            plugins: Default::default(),
            scripts: Default::default(),
        },
        project: None,
        play_scene: None,
    }
}

fn text_context() -> Arc<TextRuntimeContext> {
    TextRuntimeContext::new().unwrap()
}

fn runtime_with_ui(enabled: bool) -> CoreRuntime {
    let runtime = CoreRuntime::new();
    runtime
        .handle()
        .store_config(UI_CONFIG_KEY, &UiConfig { enabled })
        .unwrap();
    let descriptor = crate::ui::module_descriptor();
    for dependency in &descriptor.module_dependencies {
        runtime
            .register_module(
                ModuleDescriptor::new(&dependency.module_name, "UI test dependency")
                    .with_init_level(InitLevel::Kernel),
            )
            .unwrap();
    }
    runtime.register_module(descriptor).unwrap();
    runtime.activate_module(UI_MODULE_NAME).unwrap();
    runtime
}

#[test]
fn project_ui_disabled_skips_asset_admission_but_enabled_reaches_assets() {
    let project = project_with_roots(&["res://unavailable.zui"]);
    let disabled = runtime_with_ui(false);
    assert!(project
        .load_runtime_ui_surfaces(&disabled.handle(), text_context())
        .unwrap()
        .is_empty());
    let enabled = runtime_with_ui(true);
    assert!(matches!(
        project.load_runtime_ui_surfaces(&enabled.handle(), text_context()),
        Err(RuntimeProjectError::ResolveProjectAssetManager { .. })
    ));
}

#[test]
fn project_ui_requires_driver_for_declared_roots_and_no_driver_for_empty_roots() {
    let runtime = CoreRuntime::new();
    assert!(project_with_roots(&[])
        .load_runtime_ui_surfaces(&runtime.handle(), text_context())
        .unwrap()
        .is_empty());
    assert!(matches!(
        project_with_roots(&["res://unavailable.zui"])
            .load_runtime_ui_surfaces(&runtime.handle(), text_context()),
        Err(RuntimeProjectError::AdmitRuntimeUi {
            source: CoreError::MissingService(name)
        }) if name == UI_RUNTIME_DRIVER_NAME
    ));
}

#[test]
fn project_ui_rejects_admission_after_module_cleanup() {
    let runtime = runtime_with_ui(true);
    runtime.deactivate_module(UI_MODULE_NAME).unwrap();
    assert!(matches!(
        project_with_roots(&["res://unavailable.zui"])
            .load_runtime_ui_surfaces(&runtime.handle(), text_context()),
        Err(RuntimeProjectError::AdmitRuntimeUi { .. })
    ));
}
