use std::any::Any;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use zircon_editor::core::asset::{
    AssetTypeContribution, AssetTypeId, AssetTypePresentation, ThumbnailProviderDescriptor,
};
use zircon_editor::core::commands::EditorCommandDescriptor;
use zircon_editor::core::editing::engine::{
    CommandExecutionError, EditCommand, EditCommandError, EditContext, HistoryContextId,
};
use zircon_editor::core::editing::operation::{
    OperationCommand, OperationCommandFactory, OperationCommandFactoryError,
    OperationCommandFactoryRegistration,
};
use zircon_editor::core::editor_extension::{
    AssetImporterDescriptor, EditorExtensionRegistry, EditorExtensionRegistryError,
    EditorMenuItemDescriptor,
};
use zircon_editor::core::editor_operation::{EditorOperationInvocation, EditorOperationPath};
use zircon_editor::{EditorPlugin, EditorPluginDescriptor, EditorPluginRegistrationReport};
use zircon_plugin_sdk::EditorPluginDeclaration;
use zircon_runtime::asset::project::{ProjectManifest, ProjectPaths};
use zircon_runtime::plugin::PluginPackageManifest;

use crate::capability::{EDITOR_CAPABILITIES, EDITOR_CRATE_NAME, PLUGIN_ID};

const NEURAL_MODEL_ASSET_TYPE_ID: &str = "neural.model";
const NEURAL_MODEL_IMPORTER_ID: &str = "neural.model.onnx";
const NEURAL_MODEL_IMPORT_OPERATION: &str = "neural.model.import";
const NEURAL_MODEL_IMPORT_MENU_PATH: &str = "Assets/Neural/Import ONNX Model";

#[derive(Clone, Debug)]
pub struct NeuralEditorPlugin {
    declaration: EditorPluginDeclaration,
}

impl Default for NeuralEditorPlugin {
    fn default() -> Self {
        Self {
            declaration: EditorPluginDeclaration::new(
                PLUGIN_ID,
                zircon_plugin_neural_runtime::NEURAL_DECLARATION.display_name(),
                EDITOR_CRATE_NAME,
            )
            .with_category(zircon_plugin_neural_runtime::NEURAL_DECLARATION.category())
            .mirrors_runtime_manifest(zircon_plugin_neural_runtime::package_manifest())
            .with_capabilities(EDITOR_CAPABILITIES.iter().copied()),
        }
    }
}

impl NeuralEditorPlugin {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn declaration(&self) -> &EditorPluginDeclaration {
        &self.declaration
    }

    pub fn registration_report(&self) -> EditorPluginRegistrationReport {
        self.declaration.registration_report(self)
    }
}

impl EditorPlugin for NeuralEditorPlugin {
    fn descriptor(&self) -> &EditorPluginDescriptor {
        self.declaration.descriptor()
    }

    fn register_editor_extensions(
        &self,
        registry: &mut EditorExtensionRegistry,
    ) -> Result<(), EditorExtensionRegistryError> {
        register_neural_authoring_extensions(registry)
    }
}

fn register_neural_authoring_extensions(
    registry: &mut EditorExtensionRegistry,
) -> Result<(), EditorExtensionRegistryError> {
    let import_operation = EditorOperationPath::parse(NEURAL_MODEL_IMPORT_OPERATION)
        .map_err(EditorExtensionRegistryError::OperationPath)?;
    let neural_asset_type = AssetTypeId::parse(NEURAL_MODEL_ASSET_TYPE_ID)?;
    registry.register_operation_command(
        EditorCommandDescriptor::operation(import_operation.clone(), "Import ONNX Model")
            .with_menu_path(NEURAL_MODEL_IMPORT_MENU_PATH)
            .with_payload_schema_id("neural.model.import.v1")
            .with_callable_from_remote(false)
            .with_required_capabilities([crate::NEURAL_AUTHORING_CAPABILITY]),
        OperationCommandFactoryRegistration::new(
            import_operation.clone(),
            "Import ONNX Model",
            Arc::new(NeuralModelImportCommandFactory {
                operation: import_operation.clone(),
            }),
        ),
    )?;
    registry.register_menu_item(
        EditorMenuItemDescriptor::new(NEURAL_MODEL_IMPORT_MENU_PATH, import_operation.clone())
            .with_required_capabilities([crate::NEURAL_AUTHORING_CAPABILITY]),
    )?;
    registry.register_asset_importer(
        AssetImporterDescriptor::new(
            NEURAL_MODEL_IMPORTER_ID,
            "ONNX Neural Model",
            import_operation,
        )
        .with_source_extension("onnx")
        .with_output_type(neural_asset_type.clone())
        .with_required_capabilities([crate::NEURAL_AUTHORING_CAPABILITY]),
    )?;
    registry.register_asset_type_contribution(AssetTypeContribution::define(
        neural_asset_type,
        AssetTypePresentation::new("Neural Model", "NN", "asset-neural-model", "asset.neural"),
        ThumbnailProviderDescriptor::Icon("asset-neural-model".to_string()),
    ))
}

struct NeuralModelImportCommandFactory {
    operation: EditorOperationPath,
}

impl OperationCommandFactory for NeuralModelImportCommandFactory {
    fn create(
        &self,
        invocation: &EditorOperationInvocation,
    ) -> Result<OperationCommand, OperationCommandFactoryError> {
        if invocation.operation_id != self.operation {
            return Err(OperationCommandFactoryError::OperationMismatch {
                descriptor_operation: invocation.operation_id.clone(),
                factory_operation: self.operation.clone(),
            });
        }
        let (source_path, output_path) = resolve_import_paths(&self.operation, invocation)?;
        Ok(OperationCommand::new(
            Box::new(NeuralModelImportCommand::new(source_path, output_path)),
            HistoryContextId::Global,
        ))
    }
}

fn import_path_argument(
    invocation: &EditorOperationInvocation,
    name: &'static str,
) -> Result<PathBuf, OperationCommandFactoryError> {
    invocation
        .arguments
        .get(name)
        .and_then(serde_json::Value::as_str)
        .filter(|value| !value.trim().is_empty())
        .map(PathBuf::from)
        .ok_or_else(|| OperationCommandFactoryError::InvalidArguments {
            operation: invocation.operation_id.clone(),
            reason: format!("`{name}` must be a non-empty path string"),
        })
}

fn validate_import_paths(
    operation: &EditorOperationPath,
    source_path: &Path,
    output_path: &Path,
) -> Result<(), OperationCommandFactoryError> {
    let invalid = |reason| OperationCommandFactoryError::InvalidArguments {
        operation: operation.clone(),
        reason,
    };
    if source_path == output_path {
        return Err(invalid("source and output paths must differ".to_string()));
    }
    if !has_extension(source_path, "onnx") {
        return Err(invalid("source path must end in `.onnx`".to_string()));
    }
    if !has_extension(output_path, "znn") {
        return Err(invalid("output path must end in `.znn`".to_string()));
    }
    if output_path
        .parent()
        .is_none_or(|parent| parent.as_os_str().is_empty())
    {
        return Err(invalid(
            "output path must include a parent directory".to_string(),
        ));
    }
    Ok(())
}

fn resolve_import_paths(
    operation: &EditorOperationPath,
    invocation: &EditorOperationInvocation,
) -> Result<(PathBuf, PathBuf), OperationCommandFactoryError> {
    let invalid = |reason| OperationCommandFactoryError::InvalidArguments {
        operation: operation.clone(),
        reason,
    };
    let project_root = import_path_argument(invocation, "project_root")?;
    if !project_root.is_absolute() {
        return Err(invalid(
            "`project_root` must be an absolute path".to_string(),
        ));
    }
    let resolved_project_root = ProjectPaths::resolve_existing(&project_root)
        .map_err(|error| invalid(format!("failed to resolve `project_root`: {error}")))?;
    if !resolved_project_root.operation_path().is_dir() {
        return Err(invalid("`project_root` must name a directory".to_string()));
    }
    let project_paths = ProjectPaths::from_resolved_root(&resolved_project_root);
    let manifest = ProjectManifest::load(project_paths.manifest_path())
        .map_err(|error| invalid(format!("failed to load the project manifest: {error}")))?;
    manifest
        .validate()
        .map_err(|error| invalid(format!("project manifest is invalid: {error}")))?;

    let source_relative = project_relative_argument(invocation, "source_path")?;
    let output_relative = project_relative_argument(invocation, "output_path")?;
    let source_path = ProjectPaths::resolve_existing(
        resolved_project_root
            .operation_path()
            .join(&source_relative),
    )
    .map_err(|error| invalid(format!("failed to resolve `source_path`: {error}")))?
    .into_operation_path();
    let output_path = ProjectPaths::resolve_path(
        resolved_project_root
            .operation_path()
            .join(&output_relative),
    )
    .map_err(|error| invalid(format!("failed to resolve `output_path`: {error}")))?
    .into_operation_path();
    let asset_roots = manifest
        .asset_root_paths(&project_paths)
        .into_iter()
        .map(|root| ProjectPaths::resolve_existing(&root))
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| invalid(format!("failed to resolve project asset roots: {error}")))?;
    for (argument, path) in [("source_path", &source_path), ("output_path", &output_path)] {
        if !asset_roots
            .iter()
            .any(|root| path.strip_prefix(root.operation_path()).is_ok())
        {
            return Err(invalid(format!(
                "`{argument}` must resolve inside a configured project asset root"
            )));
        }
    }
    validate_import_paths(operation, &source_path, &output_path)?;
    Ok((source_path, output_path))
}

fn project_relative_argument(
    invocation: &EditorOperationInvocation,
    name: &'static str,
) -> Result<PathBuf, OperationCommandFactoryError> {
    let path = import_path_argument(invocation, name)?;
    if path.is_absolute()
        || path
            .components()
            .any(|component| !matches!(component, std::path::Component::Normal(_)))
    {
        return Err(OperationCommandFactoryError::InvalidArguments {
            operation: invocation.operation_id.clone(),
            reason: format!("`{name}` must be a normalized project-relative path"),
        });
    }
    Ok(path)
}

fn has_extension(path: &Path, expected: &str) -> bool {
    path.extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| extension.eq_ignore_ascii_case(expected))
}

pub(crate) struct NeuralModelImportCommand {
    source_path: PathBuf,
    output_path: PathBuf,
    before_output: Option<Option<Vec<u8>>>,
}

impl NeuralModelImportCommand {
    pub(crate) fn new(source_path: PathBuf, output_path: PathBuf) -> Self {
        Self {
            source_path,
            output_path,
            before_output: None,
        }
    }

    fn convert(&self) -> Result<Vec<u8>, String> {
        let source = fs::read(&self.source_path).map_err(|error| {
            format!(
                "failed to read ONNX input {}: {error}",
                self.source_path.display()
            )
        })?;
        let graph = crate::onnx::read_onnx_graph(&source)
            .map_err(|error| format!("failed to decode ONNX graph: {error}"))?;
        let model = crate::onnx::convert_graph(&graph).map_err(|diagnostics| {
            diagnostics
                .into_iter()
                .map(|diagnostic| diagnostic.to_json_line())
                .collect::<Vec<_>>()
                .join("\n")
        })?;
        model.to_znn_bytes().map_err(|error| error.to_string())
    }

    fn execution_error(reason: impl Into<String>, applied: bool) -> CommandExecutionError {
        let source = EditCommandError::ExternalEffect {
            source: Box::new(std::io::Error::other(reason.into())),
        };
        if applied {
            CommandExecutionError::applied(source)
        } else {
            CommandExecutionError::unchanged(source)
        }
    }

    pub(crate) fn apply_to_filesystem(&mut self) -> Result<(), CommandExecutionError> {
        let converted = self
            .convert()
            .map_err(|error| Self::execution_error(error, false))?;
        if self.before_output.is_none() {
            self.before_output = Some(match fs::read(&self.output_path) {
                Ok(bytes) => Some(bytes),
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => None,
                Err(error) => {
                    return Err(Self::execution_error(
                        format!(
                            "failed to inspect neural model output {}: {error}",
                            self.output_path.display()
                        ),
                        false,
                    ));
                }
            });
        }
        fs::write(&self.output_path, converted).map_err(|error| {
            Self::execution_error(
                format!(
                    "failed to write neural model output {}: {error}",
                    self.output_path.display()
                ),
                true,
            )
        })
    }

    pub(crate) fn revert_filesystem(&mut self) -> Result<(), CommandExecutionError> {
        match self.before_output.as_ref() {
            Some(Some(bytes)) => fs::write(&self.output_path, bytes).map_err(|error| {
                Self::execution_error(
                    format!(
                        "failed to restore neural model output {}: {error}",
                        self.output_path.display()
                    ),
                    true,
                )
            }),
            Some(None) => match fs::remove_file(&self.output_path) {
                Ok(()) => Ok(()),
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
                Err(error) => Err(Self::execution_error(
                    format!(
                        "failed to remove neural model output {}: {error}",
                        self.output_path.display()
                    ),
                    true,
                )),
            },
            None => Err(CommandExecutionError::unchanged(
                EditCommandError::InvariantViolation {
                    invariant: "neural model import command has no captured output state",
                },
            )),
        }
    }
}

impl EditCommand for NeuralModelImportCommand {
    fn label(&self) -> &str {
        "Import ONNX Model"
    }

    fn apply(&mut self, _context: &mut dyn EditContext) -> Result<(), CommandExecutionError> {
        self.apply_to_filesystem()
    }

    fn revert(&mut self, _context: &mut dyn EditContext) -> Result<(), CommandExecutionError> {
        self.revert_filesystem()
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

pub fn editor_plugin_declaration() -> EditorPluginDeclaration {
    editor_plugin().declaration().clone()
}

pub fn editor_plugin_descriptor() -> EditorPluginDescriptor {
    editor_plugin_declaration().descriptor().clone()
}

pub fn editor_plugin() -> NeuralEditorPlugin {
    NeuralEditorPlugin::new()
}

pub fn package_manifest() -> PluginPackageManifest {
    editor_plugin().declaration().package_manifest()
}

pub fn editor_capabilities() -> Vec<String> {
    editor_plugin().declaration().capabilities().to_vec()
}

pub fn plugin_registration() -> EditorPluginRegistrationReport {
    editor_plugin().registration_report()
}
