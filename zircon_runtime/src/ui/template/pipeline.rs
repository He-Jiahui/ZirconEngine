use std::path::Path;

use thiserror::Error;
use zircon_runtime_interface::ui::{
    event_ui::UiTreeId,
    template::{UiTemplateDocument, UiTemplateError},
};

use crate::ui::{
    surface::UiSurface,
    template::{
        UiTemplateBuildError, UiTemplateInstance, UiTemplateLoader, UiTemplateSurfaceBuilder,
        UiTemplateValidator,
    },
};

pub const UI_TEMPLATE_RUNTIME_PIPELINE_STAGES: [&str; 4] =
    ["load", "validate", "instance", "build"];

#[derive(Clone, Debug, Error, PartialEq)]
pub enum UiTemplateRuntimePipelineError {
    #[error("failed to load ui template document: {0}")]
    Load(#[source] UiTemplateError),
    #[error("failed to validate ui template document: {0}")]
    Validate(#[source] UiTemplateError),
    #[error("failed to instantiate ui template document: {0}")]
    Instance(#[source] UiTemplateError),
    #[error("failed to build ui template surface: {0}")]
    Build(#[source] UiTemplateBuildError),
}

#[derive(Default)]
pub struct UiTemplateRuntimePipeline;

impl UiTemplateRuntimePipeline {
    pub fn load_document_from_toml_str(
        input: &str,
    ) -> Result<UiTemplateDocument, UiTemplateRuntimePipelineError> {
        UiTemplateLoader::load_toml_str(input).map_err(UiTemplateRuntimePipelineError::Load)
    }

    pub fn load_document_from_toml_file(
        path: impl AsRef<Path>,
    ) -> Result<UiTemplateDocument, UiTemplateRuntimePipelineError> {
        UiTemplateLoader::load_toml_file(path).map_err(UiTemplateRuntimePipelineError::Load)
    }

    pub fn validate_document(
        document: &UiTemplateDocument,
    ) -> Result<(), UiTemplateRuntimePipelineError> {
        UiTemplateValidator::validate_document(document)
            .map_err(UiTemplateRuntimePipelineError::Validate)
    }

    pub fn instantiate_document(
        document: &UiTemplateDocument,
    ) -> Result<UiTemplateInstance, UiTemplateRuntimePipelineError> {
        Self::validate_document(document)?;
        UiTemplateInstance::from_validated_document(document)
            .map_err(UiTemplateRuntimePipelineError::Instance)
    }

    pub fn build_surface(
        tree_id: UiTreeId,
        instance: &UiTemplateInstance,
    ) -> Result<UiSurface, UiTemplateRuntimePipelineError> {
        UiTemplateSurfaceBuilder::build_surface(tree_id, instance)
            .map_err(UiTemplateRuntimePipelineError::Build)
    }

    pub fn build_surface_from_document(
        tree_id: UiTreeId,
        document: &UiTemplateDocument,
    ) -> Result<UiSurface, UiTemplateRuntimePipelineError> {
        let instance = Self::instantiate_document(document)?;
        Self::build_surface(tree_id, &instance)
    }

    pub fn build_surface_from_toml_str(
        tree_id: UiTreeId,
        input: &str,
    ) -> Result<UiSurface, UiTemplateRuntimePipelineError> {
        let document = Self::load_document_from_toml_str(input)?;
        Self::build_surface_from_document(tree_id, &document)
    }

    pub fn build_surface_from_toml_file(
        tree_id: UiTreeId,
        path: impl AsRef<Path>,
    ) -> Result<UiSurface, UiTemplateRuntimePipelineError> {
        let document = Self::load_document_from_toml_file(path)?;
        Self::build_surface_from_document(tree_id, &document)
    }
}
