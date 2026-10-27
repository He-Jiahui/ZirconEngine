use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

use zircon_runtime_interface::project::RelPath;
use zircon_runtime_interface::runtime_api::ZrRuntimeProjectSceneTransitionResultV1;

use crate::asset::assets::SceneAsset;
use crate::asset::project::{ProjectManifest, ProjectPaths};
use crate::asset::AssetUri;
use crate::core::framework::script::ScriptHostValue;
use crate::dynamic_api::session::{
    RuntimeDynamicSession, RuntimeDynamicSessionProfile, RuntimeProjectConfig,
};
use crate::script::{
    with_script_runtime_call_context, CapabilitySet, HostExportRegistry, ScriptRuntimeCallContext,
    VmBackend, VmBackendFamily, VmError, VmPluginHostContext, VmPluginInstance, VmPluginManager,
    VmPluginManifest, VmPluginPackage, VM_PLUGIN_MANAGER_NAME,
};

pub(super) const ENTRY: &str = "res://scenes/entry.scene.toml";
pub(super) const TARGET: &str = "res://scenes/target.scene.toml";

pub(super) struct TransitionProject {
    pub(super) root: PathBuf,
}

impl TransitionProject {
    pub(super) fn new() -> Self {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root = std::env::temp_dir().join(format!(
            "zircon-g15-transition-{}-{unique}",
            std::process::id()
        ));
        std::fs::create_dir_all(root.join("assets/scenes")).unwrap();
        ProjectManifest::new("G15Transition", AssetUri::parse(TARGET).unwrap(), 1)
            .save(root.join("zircon-project.toml"))
            .unwrap();
        ProjectPaths::from_root(&root)
            .unwrap()
            .ensure_derived_layout()
            .unwrap();
        let fixture = Self { root };
        fixture.write_scene("entry", 11);
        fixture.write_scene("target", 22);
        fixture
    }

    pub(super) fn config(&self) -> RuntimeProjectConfig {
        RuntimeProjectConfig::from_root(&self.root)
            .unwrap()
            .with_play_scene(RelPath::parse("assets/scenes/entry.scene.toml").unwrap())
    }

    pub(super) fn session(&self) -> RuntimeDynamicSession {
        RuntimeDynamicSession::new(RuntimeDynamicSessionProfile::Headless, Some(self.config()))
            .expect("headless Play scene should load")
    }

    fn write_scene(&self, name: &str, entity: u64) {
        let scene: SceneAsset = serde_json::from_value(serde_json::json!({
            "entities": [{
                "entity": entity, "name": name, "parent": null,
                "transform": {
                    "translation": [0.0, 0.0, 0.0],
                    "rotation": [0.0, 0.0, 0.0, 1.0], "scale": [1.0, 1.0, 1.0]
                },
                "camera": null, "mesh": null, "directional_light": null,
                "script_bindings": [{
                    "package": "g15_hooks", "module": name, "enabled": true,
                    "update": true, "fixed_update": false
                }]
            }]
        }))
        .unwrap();
        std::fs::write(
            self.root.join(format!("assets/scenes/{name}.scene.toml")),
            scene
                .to_project_toml_string(|_| unreachable!("fixture has no asset references"))
                .unwrap(),
        )
        .unwrap();
    }
}

pub(super) fn host_call(
    session: &RuntimeDynamicSession,
    name: &str,
    arguments: Vec<ScriptHostValue>,
) -> ScriptHostValue {
    let manager = session
        .runtime
        .handle()
        .resolve_manager::<VmPluginManager>(VM_PLUGIN_MANAGER_NAME)
        .unwrap();
    with_script_runtime_call_context(
        ScriptRuntimeCallContext {
            core: session.runtime.weak(),
            level: session.level.clone(),
            entity: 0,
            delta_seconds: 0.016,
        },
        || {
            manager.host_exports().call_with_capabilities(
                "zr.zircon.gameplay",
                name,
                arguments,
                &CapabilitySet::default().with("gameplay.scene_transition"),
            )
        },
    )
    .expect("gameplay scene transition host call")
}

pub(super) fn request(session: &RuntimeDynamicSession, uri: &str) -> u64 {
    match host_call(
        session,
        "request_scene_transition",
        vec![ScriptHostValue::String(uri.to_owned())],
    ) {
        ScriptHostValue::Int(id) => id as u64,
        value => panic!("unexpected request result: {value:?}"),
    }
}

pub(super) fn result(
    session: &RuntimeDynamicSession,
) -> Option<ZrRuntimeProjectSceneTransitionResultV1> {
    session.level.with_world(|world| {
        world
            .get_resource::<ZrRuntimeProjectSceneTransitionResultV1>()
            .cloned()
    })
}

pub(super) fn drain(session: &mut RuntimeDynamicSession) {
    session
        .prepare_host_request_output()
        .expect("drain runtime host requests");
    session.commit_host_request_output();
}

pub(super) fn record_script_hooks(
    session: &RuntimeDynamicSession,
    transition_on_update: bool,
) -> Arc<Mutex<Vec<String>>> {
    let events = Arc::new(Mutex::new(Vec::new()));
    let manager = session
        .runtime
        .handle()
        .resolve_manager::<VmPluginManager>(VM_PLUGIN_MANAGER_NAME)
        .unwrap();
    manager.register_family(Arc::new(HookBackend(
        Arc::clone(&events),
        transition_on_update,
    )));
    manager
        .load_package_with_backend(
            "g15:hooks",
            VmPluginPackage {
                manifest: VmPluginManifest {
                    name: "g15_hooks".to_owned(),
                    version: "0.1.0".to_owned(),
                    entry: "entry".to_owned(),
                    capabilities: CapabilitySet::default().with("gameplay.scene_transition"),
                    management: Default::default(),
                },
                zr_vm_project: None,
                bytecode: Vec::new(),
            },
        )
        .expect("recording script backend should load");
    events
}

#[derive(Clone)]
struct HookBackend(Arc<Mutex<Vec<String>>>, bool);

impl VmBackendFamily for HookBackend {
    fn family_name(&self) -> &str {
        "g15"
    }
    fn resolve(&self, selector: &str) -> Result<Arc<dyn VmBackend>, VmError> {
        match selector {
            "g15:hooks" => Ok(Arc::new(self.clone())),
            other => Err(VmError::UnknownBackend(other.to_owned())),
        }
    }
    fn visit_selectors(&self, visitor: &mut dyn FnMut(&str)) {
        visitor("g15:hooks");
    }
}

impl VmBackend for HookBackend {
    fn backend_name(&self) -> &str {
        "g15:hooks"
    }
    fn load_package(
        &self,
        package: &VmPluginPackage,
        host: &VmPluginHostContext,
    ) -> Result<Box<dyn VmPluginInstance>, VmError> {
        Ok(Box::new(HookInstance {
            manifest: package.manifest.clone(),
            events: Arc::clone(&self.0),
            exports: host.host_exports.clone(),
            transition_on_update: self.1,
        }))
    }
}

struct HookInstance {
    manifest: VmPluginManifest,
    events: Arc<Mutex<Vec<String>>>,
    exports: HostExportRegistry,
    transition_on_update: bool,
}

impl VmPluginInstance for HookInstance {
    fn manifest(&self) -> &VmPluginManifest {
        &self.manifest
    }
    fn call_export(
        &mut self,
        module: &str,
        export: &str,
        _: &[ScriptHostValue],
    ) -> Result<Option<ScriptHostValue>, VmError> {
        self.events
            .lock()
            .unwrap()
            .push(format!("{module}.{export}"));
        if self.transition_on_update && module == "entry" && export == "onUpdate" {
            self.transition_on_update = false;
            self.exports.call_with_capabilities(
                "zr.zircon.gameplay",
                "request_scene_transition",
                vec![ScriptHostValue::String(TARGET.to_owned())],
                &self.manifest.capabilities,
            )?;
        }
        Ok(Some(ScriptHostValue::Null))
    }
}
