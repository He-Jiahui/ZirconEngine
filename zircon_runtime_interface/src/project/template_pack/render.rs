use crate::project::{validate_project_name, ProjectGuid, ProjectManifestSummary, RelPath};

use super::embedded::{EmbeddedProjectTemplateEntry, RENDERABLE_EMPTY_ENTRIES};
use super::{
    ProjectTemplateId, ProjectTemplatePackError, RenderedProjectTemplate,
    RenderedProjectTemplateEntry,
};

const PROJECT_MANIFEST_PATH: &str = "zircon-project.toml";

/// Renders the versioned template pack without consulting a source checkout at runtime.
pub fn render_project_template(
    id: ProjectTemplateId,
    project_name: &str,
) -> Result<RenderedProjectTemplate, ProjectTemplatePackError> {
    validate_project_name(project_name)
        .map_err(|source| ProjectTemplatePackError::InvalidProjectName { source })?;
    let source = match id {
        ProjectTemplateId::RenderableEmpty => RENDERABLE_EMPTY_ENTRIES,
    };
    let mut entries = source
        .iter()
        .map(render_entry)
        .collect::<Result<Vec<_>, _>>()?;
    let manifest = entries
        .iter_mut()
        .find(|entry| entry.path.as_str() == PROJECT_MANIFEST_PATH)
        .ok_or(ProjectTemplatePackError::MissingManifest)?;
    rewrite_manifest_identity(&mut manifest.bytes, project_name, ProjectGuid::new())?;
    let summary = ProjectManifestSummary::parse_toml_bytes(&manifest.bytes)?.value;
    Ok(RenderedProjectTemplate {
        id,
        summary,
        entries,
    })
}

fn render_entry(
    entry: &EmbeddedProjectTemplateEntry,
) -> Result<RenderedProjectTemplateEntry, ProjectTemplatePackError> {
    Ok(RenderedProjectTemplateEntry {
        path: RelPath::parse(entry.path)?,
        bytes: entry.bytes.to_vec(),
    })
}

fn rewrite_manifest_identity(
    bytes: &mut Vec<u8>,
    project_name: &str,
    project_guid: ProjectGuid,
) -> Result<(), ProjectTemplatePackError> {
    let source = std::str::from_utf8(bytes)
        .map_err(|source| ProjectTemplatePackError::ManifestUtf8 { source })?;
    let mut manifest = toml::from_str::<toml::Table>(source)
        .map_err(|source| ProjectTemplatePackError::ManifestToml { source })?;
    manifest.insert(
        "name".to_string(),
        toml::Value::String(project_name.to_string()),
    );
    manifest.insert(
        "project_guid".to_string(),
        toml::Value::String(project_guid.to_string()),
    );
    *bytes = toml::to_string_pretty(&manifest)
        .map_err(|source| ProjectTemplatePackError::ManifestEncode { source })?
        .into_bytes();
    Ok(())
}
