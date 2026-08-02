use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::ZR_VM_PROJECT_BACKEND_SELECTOR;
use zircon_runtime::script::{
    VM_STATE_SCHEMA_VERSION_V2, VmStateBlob, VmStateSchema, VmStateTypeSchema,
};
use zircon_runtime_interface::reflect::{
    ReflectEditorHint, ReflectFieldInfo, ReflectSerializationStrategy, ReflectTypeInfo,
    ReflectTypePath, ReflectTypeRegistration,
};

const FIXTURE_STATE_TYPE_HASH: u32 = 0x5A56_0002;

pub(super) fn build_real_backend_host(
    manager: &Arc<zircon_runtime::script::VmPluginManager>,
    package: &zircon_runtime::script::DiscoveredVmPluginPackage,
) -> zircon_runtime::script::VmPluginHostContext {
    let source = package.source.clone();
    let package_root = source.package_root.clone().or_else(|| {
        source
            .manifest_path
            .as_ref()
            .and_then(|path| path.parent().map(Path::to_path_buf))
    });
    let mut plugin = manager.base_plugin_context().clone();
    plugin.package_root = package_root.clone();
    plugin.source_root = source
        .manifest_path
        .as_ref()
        .and_then(|path| path.parent().map(Path::to_path_buf))
        .or_else(|| package_root.clone());
    plugin.data_root = package_root.as_ref().map(|root| root.join("data"));

    zircon_runtime::script::VmPluginHostContext::new_for_tests(
        plugin,
        package.package.manifest.capabilities.clone(),
        ZR_VM_PROJECT_BACKEND_SELECTOR.to_string(),
        source,
        manager.host_registry(),
        manager.host_exports(),
        manager.host_interfaces(),
        manager.reflection_catalog(),
        Default::default(),
        Arc::new(NoopSlotLifecycle),
    )
}

struct NoopSlotLifecycle;

impl zircon_runtime::script::VmPluginSlotLifecycle for NoopSlotLifecycle {
    fn load_package(
        &self,
        _backend_selector: &str,
        _package: zircon_runtime::script::VmPluginPackage,
    ) -> Result<zircon_runtime::script::PluginSlotId, zircon_runtime::script::VmError> {
        Err(zircon_runtime::script::VmError::Operation(
            "test lifecycle facade does not load slots".to_string(),
        ))
    }

    fn hot_reload_slot(
        &self,
        _slot: zircon_runtime::script::PluginSlotId,
        _package: zircon_runtime::script::VmPluginPackage,
    ) -> Result<(), zircon_runtime::script::VmError> {
        Err(zircon_runtime::script::VmError::Operation(
            "test lifecycle facade does not hot reload slots".to_string(),
        ))
    }

    fn unload_slot(
        &self,
        _slot: zircon_runtime::script::PluginSlotId,
    ) -> Result<(), zircon_runtime::script::VmError> {
        Err(zircon_runtime::script::VmError::Operation(
            "test lifecycle facade does not unload slots".to_string(),
        ))
    }

    fn slot(
        &self,
        slot: zircon_runtime::script::PluginSlotId,
    ) -> Result<zircon_runtime::script::VmPluginSlotRecord, zircon_runtime::script::VmError> {
        Err(zircon_runtime::script::VmError::MissingSlot(slot.get()))
    }

    fn list_slots(&self) -> Vec<zircon_runtime::script::VmPluginSlotRecord> {
        Vec::new()
    }
}

pub(super) struct ZrVmProjectFixture {
    pub(super) root: PathBuf,
    pub(super) project_path: PathBuf,
}

pub(super) struct DocumentedZrVmExampleFixture {
    pub(super) root: PathBuf,
}

impl ZrVmProjectFixture {
    pub(super) fn new(name: &str, version: &str) -> Self {
        Self::new_with_host_interfaces(name, version, false)
    }

    pub(super) fn new_with_extension_channels(name: &str, version: &str) -> Self {
        Self::new_with_configuration(name, version, true, false)
    }

    pub(super) fn new_with_cooperative_gc(name: &str, version: &str) -> Self {
        Self::new_with_configuration(name, version, false, true)
    }

    fn new_with_host_interfaces(name: &str, version: &str, extension_channels: bool) -> Self {
        Self::new_with_configuration(name, version, extension_channels, false)
    }

    fn new_with_configuration(
        name: &str,
        version: &str,
        extension_channels: bool,
        cooperative_gc: bool,
    ) -> Self {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root = std::env::temp_dir().join(format!("zircon-zr-vm-real-fixture-{nonce}"));
        let package_root = root.join(name);
        let project_root = package_root.join("script");
        let source_root = project_root.join("src");
        fs::create_dir_all(&source_root).unwrap();
        fs::create_dir_all(project_root.join("bin")).unwrap();
        fs::create_dir_all(package_root.join("data")).unwrap();

        let project_path = project_root.join("plugin.zrp");
        fs::write(
            &project_path,
            concat!(
                "{\n",
                "  \"name\": \"native_host_roundtrip\",\n",
                "  \"source\": \"src\",\n",
                "  \"binary\": \"bin\",\n",
                "  \"entry\": \"main\"\n",
                "}\n",
            ),
        )
        .unwrap();
        fs::write(
            source_root.join("main.zr"),
            zr_vm_source(extension_channels),
        )
        .unwrap();
        let extension_capabilities = if extension_channels {
            concat!(
                ", \"runtime.script.extension.system\"",
                ", \"runtime.script.extension.bt_node\"",
                ", \"runtime.script.extension.rpc_handler\"",
                ", \"runtime.script.extension.editor_operation\"",
            )
        } else {
            ""
        };
        let garbage_collection = if cooperative_gc {
            concat!(
                "\n",
                "[management.garbage_collection]\n",
                "mode = \"cooperative\"\n",
                "interval_frames = 1\n",
            )
        } else {
            ""
        };
        fs::write(
            package_root.join("plugin.toml"),
            format!(
                concat!(
                    "name = \"{name}\"\n",
                    "version = \"{version}\"\n",
                    "entry = \"main\"\n",
                    "backend = \"zr_vm:project\"\n",
                    "\n",
                    "[capabilities]\n",
                    "capabilities = [\"foundation.time\", \"foundation.log\"{extension_capabilities}]\n",
                    "\n",
                    "[zr_vm]\n",
                    "project = \"script/plugin.zrp\"\n",
                    "entry_module = \"main\"\n",
                    "execution_mode = \"binary\"\n",
                    "{garbage_collection}",
                ),
                name = name,
                version = version,
                extension_capabilities = extension_capabilities,
                garbage_collection = garbage_collection,
            ),
        )
        .unwrap();

        Self { root, project_path }
    }
}

impl DocumentedZrVmExampleFixture {
    pub(super) fn copy_from_docs() -> Self {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root = std::env::temp_dir().join(format!("zircon-zr-vm-docs-example-{nonce}"));
        let package_root = root.join("zr_vm_minimal");
        fs::create_dir_all(&package_root).unwrap();

        let docs_example = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../../docs/zircon_runtime/script/vm/examples/zr_vm_minimal");
        for file_name in ["plugin.toml", "plugin.zrp", "main.zr"] {
            fs::copy(docs_example.join(file_name), package_root.join(file_name)).unwrap();
        }

        Self { root }
    }
}

impl Drop for ZrVmProjectFixture {
    fn drop(&mut self) {
        remove_dir_all_if_exists(&self.root);
    }
}

impl Drop for DocumentedZrVmExampleFixture {
    fn drop(&mut self) {
        remove_dir_all_if_exists(&self.root);
    }
}

fn zr_vm_source(extension_channels: bool) -> String {
    let extension_import = if extension_channels {
        "var extensions = %import(\"zr.zircon.extensions\");\n"
    } else {
        ""
    };
    let extension_registration = if extension_channels {
        concat!(
            "    extensions.register_system(\"game.script.update\", \"update\", \"main\", \"systemTick\");\n",
            "    extensions.register_bt_node(\"game.script.task\", \"Game Script Task\", \"main\", \"behaviorTick\");\n",
            "    extensions.register_rpc_handler(\"game.script.rpc\", \"game.script.rpc.v1\", \"main\", \"rpcHandle\");\n",
            "    extensions.register_editor_operation(\"Game.Script.Open\", \"main\", \"editorOpen\");\n",
        )
    } else {
        ""
    };
    let mut source = String::from(concat!(
        "var math = %import(\"zr.zircon.math\");\n",
        "var foundation = %import(\"zr.zircon.foundation\");\n",
    ));
    source.push_str(extension_import);
    source.push_str("var savedState = ");
    source.push_str(&zr_vm_string_literal(&fixture_state_json("created")));
    source.push_str(concat!(
        ";\n\n",
        "pub activate(): void {\n",
        "    var now = foundation.time_unix_millis();\n",
        "    var dot = math.vec3_dot(1.0, 2.0, 3.0, 4.0, 5.0, 6.0);\n",
        "    foundation.log_info(\"activated\");\n",
    ));
    source.push_str(extension_registration);
    source.push_str("    savedState = ");
    source.push_str(&zr_vm_string_literal(&fixture_state_json("activated")));
    source.push_str(concat!(
        ";\n",
        "}\n",
        "\n",
        "pub systemTick(deltaSeconds: float): void {\n",
        "}\n",
        "\n",
        "pub behaviorTick(): void {\n",
        "}\n",
        "\n",
        "pub rpcHandle(payload: string): void {\n",
        "}\n",
        "\n",
        "pub editorOpen(): void {\n",
        "}\n",
        "\n",
        "pub retainedValue(): string {\n",
        "    return \"adapter-owned-temporary\";\n",
        "}\n",
        "\n",
        "pub deactivate(): void {\n",
        "    savedState = savedState + \":deactivated\";\n",
        "}\n",
        "\n",
        "pub saveState(): string {\n",
        "    return savedState;\n",
        "}\n",
        "\n",
        "pub stateSchema(): string {\n",
        "    return ",
    ));
    source.push_str(&zr_vm_string_literal(&fixture_schema_json()));
    source.push_str(concat!(
        ";\n",
        "}\n",
        "\n",
        "pub restoreState(state: string): void {\n",
        "    savedState = state;\n",
        "}\n",
        "\n",
        "return 0;\n",
    ));
    source
}

pub(super) fn fixture_state_blob(value: &str) -> VmStateBlob {
    VmStateBlob::from_json(&fixture_state_json(value))
        .expect("fixture state should satisfy its reflected type table")
}

fn fixture_state_json(value: &str) -> String {
    let payload = format!(
        "[{{\"type_path\":{{\"type_path\":\"fixture.ZrVmState\",\"short_type_path\":\"ZrVmState\"}},\"fields\":[{{\"field_name\":\"value\",\"value\":{{\"kind\":\"String\",\"value\":\"{value}\"}}}}]}}]"
    );
    let payload_bytes = payload
        .as_bytes()
        .iter()
        .map(u8::to_string)
        .collect::<Vec<_>>()
        .join(",");
    format!(
        "{{\"schema_version\":{VM_STATE_SCHEMA_VERSION_V2},\"types\":[{{\"type_path\":{{\"type_path\":\"fixture.ZrVmState\",\"short_type_path\":\"ZrVmState\"}},\"type_hash\":{FIXTURE_STATE_TYPE_HASH}}}],\"payload\":[{payload_bytes}]}}"
    )
}

fn fixture_schema_json() -> String {
    let registration = ReflectTypeRegistration::new(
        ReflectTypePath::new("fixture.ZrVmState", "ZrVmState")
            .expect("fixture type path should be valid"),
        "Fixture State",
        ReflectTypeInfo::struct_with_fields(vec![ReflectFieldInfo::new(
            "value",
            "String",
            ReflectEditorHint::String,
        )]),
        ReflectSerializationStrategy::Value,
    );
    VmStateSchema {
        schema_version: VM_STATE_SCHEMA_VERSION_V2,
        types: vec![VmStateTypeSchema {
            registration,
            type_hash: FIXTURE_STATE_TYPE_HASH,
            renames: Vec::new(),
        }],
    }
    .to_json()
    .expect("fixture schema should serialize through the authoritative contract")
}

fn zr_vm_string_literal(value: &str) -> String {
    format!("\"{}\"", value.replace('\\', "\\\\").replace('"', "\\\""))
}

fn remove_dir_all_if_exists(path: &Path) {
    if path.exists() {
        let _ = fs::remove_dir_all(path);
    }
}
