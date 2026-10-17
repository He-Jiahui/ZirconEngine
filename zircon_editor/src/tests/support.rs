use std::fs;
use std::path::Path;
use std::sync::{Mutex, MutexGuard, OnceLock};

use std::collections::BTreeMap;

use toml::Value;
use zircon_runtime::core::CoreRuntime;
use zircon_runtime::ui::template::UiAssetLoader;
use zircon_runtime::ui::v2::UiZuiAssetLoader;
use zircon_runtime_interface::project::{
    ProjectActivationOperationIdGenerator, ProjectCreationProvenance, ProjectEngineVersion,
    ProjectLaunchInstanceId,
};
use zircon_runtime_interface::runtime_build_set::ZrRuntimeBuildSetId;
use zircon_runtime_interface::ui::accessibility::UiAccessibilityContract;
use zircon_runtime_interface::ui::focus::UiFocusContract;
use zircon_runtime_interface::ui::navigation::UiNavigationContract;
use zircon_runtime_interface::ui::picking::UiPickPolicy;
use zircon_runtime_interface::ui::template::{
    UiAssetDocument, UiAssetError, UiAssetHeader, UiAssetImports, UiBindingRef, UiChildMount,
    UiComponentDefinition, UiComponentParamSchema, UiNamedSlotSchema, UiNodeDefinition,
    UiNodeDefinitionKind, UiStyleDeclarationBlock, UiStyleScope, UiStyleSheet,
};
use zircon_runtime_interface::ui::widget::UiWidgetContract;

use crate::core::editor_authoring_extension::SceneModeDescriptor;
use crate::core::editor_message::SceneModeId;
use crate::scene::modes::{
    EditorSceneMode, InputOutcome, SceneModeCtx, SceneModeRegistration, ViewportOverlayBuilder,
};
use crate::scene::viewport::ViewportInput;
use crate::ui::host::module::EDITOR_MANAGER_NAME;
use crate::ui::host::EditorManager;

struct PassThroughTestSceneMode {
    id: SceneModeId,
}

impl EditorSceneMode for PassThroughTestSceneMode {
    fn id(&self) -> &SceneModeId {
        &self.id
    }

    fn enter(&mut self, _ctx: &mut SceneModeCtx<'_>) {}

    fn exit(&mut self, _ctx: &mut SceneModeCtx<'_>) {}

    fn handle_input(
        &mut self,
        _input: &ViewportInput,
        _ctx: &mut SceneModeCtx<'_>,
    ) -> InputOutcome {
        InputOutcome::PassThrough
    }

    fn build_overlay(&self, _out: &mut ViewportOverlayBuilder) {}
}

pub(crate) fn pass_through_scene_mode_registration(
    descriptor: SceneModeDescriptor,
) -> SceneModeRegistration {
    let mode_id = SceneModeId::new(descriptor.id());
    SceneModeRegistration::new(descriptor, move || pass_through_scene_mode(mode_id.clone()))
}

pub(crate) fn pass_through_scene_mode(mode_id: SceneModeId) -> Box<dyn EditorSceneMode> {
    Box::new(PassThroughTestSceneMode { id: mode_id })
}

/// Serializes process-global test configuration without cascading mutex poison.
pub(crate) struct TestEnvironmentLock {
    inner: Mutex<()>,
}

impl TestEnvironmentLock {
    fn new() -> Self {
        Self {
            inner: Mutex::new(()),
        }
    }

    pub(crate) fn lock(&self) -> std::sync::LockResult<MutexGuard<'_, ()>> {
        Ok(match self.inner.lock() {
            Ok(guard) => guard,
            Err(poisoned) => {
                self.inner.clear_poison();
                poisoned.into_inner()
            }
        })
    }
}

pub(crate) fn env_lock() -> &'static TestEnvironmentLock {
    static LOCK: OnceLock<TestEnvironmentLock> = OnceLock::new();
    LOCK.get_or_init(TestEnvironmentLock::new)
}

/// Registers the fixture's existing manager with its actual Core owner.
/// The caller keeps the Core alive and enters the returned handle per call.
pub(crate) fn register_test_editor_manager(
    core: &CoreRuntime,
    manager: std::sync::Arc<EditorManager>,
) -> zircon_runtime::core::runtime::ServiceHandle<EditorManager> {
    use zircon_runtime::core::runtime::ServiceObject;
    use zircon_runtime::core::{ManagerDescriptor, ModuleDescriptor, StartupMode};
    use zircon_runtime::engine_module::factory;

    core.register_module(
        ModuleDescriptor::new("EditorModule", "Editor manager test fixture").with_manager(
            ManagerDescriptor::new(
                zircon_runtime::engine_module::qualified_name(
                    "EditorModule",
                    zircon_runtime::core::ServiceKind::Manager,
                    "EditorManager",
                ),
                StartupMode::Immediate,
                Vec::new(),
                factory(move |_| Ok(manager.clone() as ServiceObject)),
            ),
        ),
    )
    .expect("fixture manager registration");
    core.activate_module("EditorModule")
        .expect("fixture manager activation");
    core.resolve_manager_handle::<EditorManager>(EDITOR_MANAGER_NAME)
        .expect("fixture manager handle")
}

/// Test runtime activation explicitly supplies the BuildSet that production receives from App.
pub(crate) fn configure_editor_test_runtime_build_set(runtime: &CoreRuntime) {
    let manager = runtime
        .resolve_manager::<EditorManager>(EDITOR_MANAGER_NAME)
        .expect("activated test runtime should resolve EditorManager");
    manager.configure_project_runtime_build_set(Some(test_runtime_build_set_id()));
}

pub(crate) fn test_project_creation_provenance() -> ProjectCreationProvenance {
    static OPERATION_IDS: OnceLock<ProjectActivationOperationIdGenerator> = OnceLock::new();

    let operation_id = OPERATION_IDS
        .get_or_init(|| ProjectActivationOperationIdGenerator::new(ProjectLaunchInstanceId::new()))
        .allocate()
        .expect("test project creation operation sequence must remain available");
    ProjectCreationProvenance::new(
        operation_id,
        ProjectEngineVersion::parse(env!("CARGO_PKG_VERSION"))
            .expect("editor package version must be a valid engine version"),
        test_runtime_build_set_id(),
    )
}

fn test_runtime_build_set_id() -> ZrRuntimeBuildSetId {
    ZrRuntimeBuildSetId::parse("0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef")
        .expect("test BuildSet fixture must be valid")
}

pub(crate) fn load_test_ui_asset(source: &str) -> Result<UiAssetDocument, UiAssetError> {
    UiAssetLoader::load_toml_str(source).or_else(|error| {
        if let Ok(document) = UiZuiAssetLoader::load_zui_str(source) {
            return crate::ui::asset_editor::project_v2_document_to_authoring(&document).map_err(
                |projection_error| UiAssetError::InvalidDocument {
                    asset_id: document.asset.id.clone(),
                    detail: projection_error.to_string(),
                },
            );
        }

        if looks_like_flat_ui_asset_source(source) {
            // Keep flat fixture migration isolated to editor tests.
            let migrated = migrate_flat_ui_asset_fixture_toml_str(source)?;
            return UiAssetLoader::load_toml_str(&migrated);
        }

        Err(error)
    })
}

pub(crate) fn write_test_ui_asset(
    path: impl AsRef<Path>,
    source: &str,
) -> Result<(), UiAssetError> {
    let document = load_test_ui_asset(source)?;
    let path = path.as_ref();
    let tree_source = if path.extension().and_then(|extension| extension.to_str()) == Some("zui") {
        let v2_document = crate::ui::asset_editor::project_authoring_document_to_v2(&document)
            .map_err(|error| UiAssetError::InvalidDocument {
                asset_id: document.asset.id.clone(),
                detail: error.to_string(),
            })?;
        toml::to_string_pretty(&v2_document)
    } else {
        toml::to_string_pretty(&document)
    }
    .map_err(|error| UiAssetError::ParseToml(error.to_string()))?;
    fs::write(path, tree_source).map_err(|error| UiAssetError::Io(error.to_string()))
}

fn looks_like_flat_ui_asset_source(source: &str) -> bool {
    source.contains("\n[nodes.")
        || source.contains("\n[root]\nnode =")
        || source.contains("\r\n[root]\r\nnode =")
        || source.contains("\n[components.")
            && (source.contains("\nroot = \"") || source.contains("\r\nroot = \""))
}

#[cfg(test)]
mod env_lock_tests {
    use super::env_lock;

    #[test]
    fn shared_test_environment_lock_recovers_after_poison() {
        let lock = env_lock();
        let poisoned = std::panic::catch_unwind(|| {
            let _guard = lock.lock().expect("initial environment lock");
            panic!("poison shared test environment lock");
        });
        assert!(poisoned.is_err());

        let recovered = lock.lock();
        assert!(recovered.is_ok(), "environment lock must recover poison");
    }
}

fn migrate_flat_ui_asset_fixture_toml_str(input: &str) -> Result<String, UiAssetError> {
    let flat_fixture: FlatUiAssetDocument =
        toml::from_str(input).map_err(|error| UiAssetError::ParseToml(error.to_string()))?;
    let migrated = flat_fixture.into_tree_document()?;
    toml::to_string_pretty(&migrated).map_err(|error| UiAssetError::ParseToml(error.to_string()))
}

impl FlatUiAssetDocument {
    fn into_tree_document(self) -> Result<UiAssetDocument, UiAssetError> {
        let root = self
            .root
            .map(|root| build_tree_node(&self.asset.id, &self.nodes, &root.node, &mut Vec::new()))
            .transpose()?;
        let components = self
            .components
            .into_iter()
            .map(|(name, component)| {
                Ok((
                    name,
                    UiComponentDefinition {
                        root: build_tree_node(
                            &self.asset.id,
                            &self.nodes,
                            &component.root,
                            &mut Vec::new(),
                        )?,
                        style_scope: component.style_scope,
                        contract: Default::default(),
                        params: component.params,
                        slots: component.slots,
                    },
                ))
            })
            .collect::<Result<_, UiAssetError>>()?;

        Ok(UiAssetDocument {
            asset: self.asset,
            imports: self.imports,
            tokens: self.tokens,
            root,
            components,
            stylesheets: self.stylesheets,
        })
    }
}

fn build_tree_node(
    asset_id: &str,
    nodes: &BTreeMap<String, FlatUiNodeDefinition>,
    node_id: &str,
    visiting: &mut Vec<String>,
) -> Result<UiNodeDefinition, UiAssetError> {
    if visiting.iter().any(|current| current == node_id) {
        return Err(UiAssetError::InvalidDocument {
            asset_id: asset_id.to_string(),
            detail: format!("ui asset tree contains a cycle at {node_id}"),
        });
    }
    let node = nodes
        .get(node_id)
        .ok_or_else(|| UiAssetError::MissingNode {
            asset_id: asset_id.to_string(),
            node_id: node_id.to_string(),
        })?;

    visiting.push(node_id.to_string());
    let children = node
        .children
        .iter()
        .map(|child| {
            Ok(UiChildMount {
                mount: child.mount.clone(),
                slot: child.slot.clone(),
                node: build_tree_node(asset_id, nodes, &child.child, visiting)?,
            })
        })
        .collect::<Result<Vec<_>, UiAssetError>>()?;
    let _ = visiting.pop();

    Ok(UiNodeDefinition {
        node_id: node_id.to_string(),
        kind: node.kind,
        widget_type: node.widget_type.clone(),
        component: node.component.clone(),
        component_ref: node.component_ref.clone(),
        component_api_version: None,
        slot_name: node.slot_name.clone(),
        control_id: node.control_id.clone(),
        classes: node.classes.clone(),
        params: node.params.clone(),
        props: node.props.clone(),
        layout: node.layout.clone(),
        bindings: node.bindings.clone(),
        style_overrides: node.style_overrides.clone(),
        children,
        focus: node.focus.clone(),
        navigation: node.navigation.clone(),
        picking: node.picking,
        a11y: node.a11y.clone(),
        widget: node.widget.clone(),
    })
}

#[derive(Clone, Debug, PartialEq, serde::Deserialize)]
struct FlatUiAssetDocument {
    asset: UiAssetHeader,
    #[serde(default)]
    imports: UiAssetImports,
    #[serde(default)]
    tokens: BTreeMap<String, Value>,
    #[serde(default)]
    root: Option<FlatUiAssetRoot>,
    #[serde(default)]
    nodes: BTreeMap<String, FlatUiNodeDefinition>,
    #[serde(default)]
    components: BTreeMap<String, FlatUiComponentDefinition>,
    #[serde(default)]
    stylesheets: Vec<UiStyleSheet>,
}

#[derive(Clone, Debug, PartialEq, serde::Deserialize)]
struct FlatUiAssetRoot {
    node: String,
}

#[derive(Clone, Debug, Default, PartialEq, serde::Deserialize)]
struct FlatUiNodeDefinition {
    #[serde(default)]
    kind: UiNodeDefinitionKind,
    #[serde(default, rename = "type")]
    widget_type: Option<String>,
    #[serde(default)]
    component: Option<String>,
    #[serde(default)]
    component_ref: Option<String>,
    #[serde(default)]
    slot_name: Option<String>,
    #[serde(default)]
    control_id: Option<String>,
    #[serde(default)]
    classes: Vec<String>,
    #[serde(default)]
    params: BTreeMap<String, Value>,
    #[serde(default)]
    props: BTreeMap<String, Value>,
    #[serde(default)]
    layout: Option<BTreeMap<String, Value>>,
    #[serde(default)]
    bindings: Vec<UiBindingRef>,
    #[serde(default)]
    style_overrides: UiStyleDeclarationBlock,
    #[serde(default)]
    focus: Option<UiFocusContract>,
    #[serde(default)]
    navigation: Option<UiNavigationContract>,
    #[serde(default)]
    picking: Option<UiPickPolicy>,
    #[serde(default)]
    a11y: Option<UiAccessibilityContract>,
    #[serde(default)]
    widget: Option<UiWidgetContract>,
    #[serde(default)]
    children: Vec<FlatUiChildMount>,
}

#[derive(Clone, Debug, Default, PartialEq, serde::Deserialize)]
struct FlatUiChildMount {
    child: String,
    #[serde(default)]
    mount: Option<String>,
    #[serde(default)]
    slot: BTreeMap<String, Value>,
}

#[derive(Clone, Debug, Default, PartialEq, serde::Deserialize)]
struct FlatUiComponentDefinition {
    root: String,
    #[serde(default)]
    style_scope: UiStyleScope,
    #[serde(default)]
    params: BTreeMap<String, UiComponentParamSchema>,
    #[serde(default)]
    slots: BTreeMap<String, UiNamedSlotSchema>,
}

/// Owns one exclusively created source-write test leaf under the authorized physical TEMP.
pub(crate) struct OwnedSourceWriteFixture {
    root: std::path::PathBuf,
    created: std::time::SystemTime,
    directory_handle: Option<std::fs::File>,
    files: Vec<OwnedSourceWriteFixtureFile>,
}

struct OwnedSourceWriteFixtureFile {
    path: std::path::PathBuf,
    created: std::time::SystemTime,
    bytes: Vec<u8>,
}

impl OwnedSourceWriteFixture {
    pub(crate) fn new(prefix: &str) -> std::io::Result<Self> {
        if prefix.is_empty()
            || !prefix
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
        {
            return Err(std::io::Error::other(
                "fixture prefix must be a single plain component",
            ));
        }
        let temp = std::fs::canonicalize(std::env::temp_dir())?;
        Self::require_authorized_physical_path(&temp)?;
        let root = temp.join(format!(
            "{prefix}_{}_{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map_err(std::io::Error::other)?
                .as_nanos(),
        ));
        // An existing leaf is foreign. Never merge into it or retry after AlreadyExists.
        std::fs::create_dir(&root)?;
        let metadata = std::fs::symlink_metadata(&root)?;
        let mut fixture = Self {
            root,
            created: metadata.created()?,
            directory_handle: None,
            files: Vec::new(),
        };
        fixture.verify_root()?;
        #[cfg(windows)]
        {
            use std::os::windows::fs::OpenOptionsExt;
            // Hold the real directory without delete sharing until fixed-file cleanup ends.
            fixture.directory_handle = Some(
                std::fs::OpenOptions::new()
                    .read(true)
                    .share_mode(0x0001 | 0x0002)
                    .custom_flags(0x0200_0000 | 0x0020_0000)
                    .open(&fixture.root)?,
            );
        }
        Ok(fixture)
    }

    pub(crate) fn root(&self) -> &std::path::Path {
        &self.root
    }

    pub(crate) fn create_file(
        &mut self,
        name: &str,
        bytes: &[u8],
    ) -> std::io::Result<std::path::PathBuf> {
        use std::io::Write;
        self.verify_root()?;
        let mut components = std::path::Path::new(name).components();
        if !matches!(components.next(), Some(std::path::Component::Normal(_)))
            || components.next().is_some()
        {
            return Err(std::io::Error::other(
                "fixture file must be one fixed component",
            ));
        }
        let path = self.root.join(name);
        let mut file = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path)?;
        let created = file.metadata()?.created()?;
        self.files.push(OwnedSourceWriteFixtureFile {
            path: path.clone(),
            created,
            bytes: bytes.to_vec(),
        });
        file.write_all(bytes)?;
        Ok(path)
    }

    /// Accepts only a known file's explicitly asserted publication bytes, never arbitrary contents.
    pub(crate) fn expect_written_bytes(&mut self, name: &str, bytes: &[u8]) -> std::io::Result<()> {
        self.verify_root()?;
        let path = self.root.join(name);
        let file = self
            .files
            .iter_mut()
            .find(|file| file.path == path)
            .ok_or_else(|| std::io::Error::other("unowned fixture file"))?;
        let metadata = std::fs::symlink_metadata(&path)?;
        Self::require_plain_path(&metadata)?;
        if metadata.len() != bytes.len() as u64 || std::fs::read(&path)? != bytes {
            return Err(std::io::Error::other(
                "fixture publication differs from asserted bytes",
            ));
        }
        if std::fs::canonicalize(&path)? != path {
            return Err(std::io::Error::other(
                "fixture publication path was replaced",
            ));
        }
        file.created = metadata.created()?;
        file.bytes = bytes.to_vec();
        Ok(())
    }

    fn require_authorized_physical_path(path: &std::path::Path) -> std::io::Result<()> {
        let display = zircon_runtime::asset::project::ProjectPaths::display_path(path);
        let allowed = [
            std::path::Path::new(r"D:\cargo-targets\zircon-local"),
            std::path::Path::new(r"E:\cargo-targets\zircon-local"),
            std::path::Path::new(r"F:\cargo-targets\zircon-local"),
        ];
        if !allowed.iter().any(|root| display.starts_with(root)) {
            return Err(std::io::Error::other(format!(
                "fixture TEMP is outside authorized physical output: {}",
                display.display()
            )));
        }
        Ok(())
    }

    fn require_plain_path(metadata: &std::fs::Metadata) -> std::io::Result<()> {
        if metadata.file_type().is_symlink() {
            return Err(std::io::Error::other("fixture path became a link"));
        }
        #[cfg(windows)]
        {
            use std::os::windows::fs::MetadataExt;
            if metadata.file_attributes() & 0x0400 != 0 {
                return Err(std::io::Error::other("fixture path became a reparse point"));
            }
        }
        Ok(())
    }

    fn verify_root(&self) -> std::io::Result<()> {
        let metadata = std::fs::symlink_metadata(&self.root)?;
        Self::require_plain_path(&metadata)?;
        if !metadata.is_dir() || metadata.created()? != self.created {
            return Err(std::io::Error::other("owned fixture leaf was replaced"));
        }
        if std::fs::canonicalize(&self.root)? != self.root {
            return Err(std::io::Error::other(
                "owned fixture leaf no longer has its captured physical path",
            ));
        }
        Self::require_authorized_physical_path(&self.root)
    }
}

impl Drop for OwnedSourceWriteFixture {
    fn drop(&mut self) {
        if let Err(error) = self.verify_root() {
            eprintln!(
                "retained source-write fixture {}: {error}",
                self.root.display()
            );
            return;
        }
        for file in &self.files {
            let cleanup = (|| -> std::io::Result<()> {
                let metadata = match std::fs::symlink_metadata(&file.path) {
                    Ok(metadata) => metadata,
                    Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
                    Err(error) => return Err(error),
                };
                Self::require_plain_path(&metadata)?;
                if !metadata.is_file()
                    || metadata.created()? != file.created
                    || metadata.len() != file.bytes.len() as u64
                    || std::fs::canonicalize(&file.path)? != file.path
                    || std::fs::read(&file.path)? != file.bytes
                {
                    return Err(std::io::Error::other(
                        "fixture file ownership or contents changed",
                    ));
                }
                std::fs::remove_file(&file.path)
            })();
            if let Err(error) = cleanup {
                eprintln!(
                    "retained source-write fixture file {}: {error}",
                    file.path.display()
                );
            }
        }
        // Remove only an empty leaf; unknown files and changed contents remain for diagnosis.
        drop(self.directory_handle.take());
        if let Err(error) = std::fs::remove_dir(&self.root) {
            eprintln!(
                "retained source-write fixture leaf {}: {error}",
                self.root.display()
            );
        }
    }
}
