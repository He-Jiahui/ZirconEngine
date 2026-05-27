use std::fs;
use std::path::{Path, PathBuf};

use super::import_shader::shader_entry_points;
use super::validate_wgsl::validate_wgsl;
use crate::asset::assets::{
    validate_wgsl_captures, DataAsset, DataAssetFormat, ImportedAsset, ShaderAsset,
    ShaderEntryPointAsset, ShaderImportRedirectAsset, ShaderSourceFileAsset, ShaderSourceLanguage,
    ZShaderDocument,
};
use crate::asset::{
    AssetImportContext, AssetImportError, AssetImportOutcome, AssetUri, ImportedAssetEntry,
};
use crate::core::resource::ResourceKind;

pub(crate) fn import_shader_package(
    context: &AssetImportContext,
) -> Result<AssetImportOutcome, AssetImportError> {
    let package_dir = compound_dir_for_zmeta(&context.source_path)?;
    let zshader_path = primary_zshader_path(&package_dir)?;
    let zshader_source = fs::read_to_string(&zshader_path)?;
    let document = ZShaderDocument::from_toml_str(&zshader_source)
        .map_err(|error| AssetImportError::Parse(format!("parse zshader toml: {error}")))?;
    let shader_defs = document.shader_definition_values().map_err(|error| {
        AssetImportError::Parse(format!("parse zshader shader_def_values: {error}"))
    })?;
    let wgsl_files = wgsl_files_for_document(&package_dir, &document)?;
    let (wgsl_source, source_files) =
        read_wgsl_sources(&package_dir, &context.uri, wgsl_files.as_slice())?;
    let mut validation_diagnostics = Vec::new();
    let entry_points = if document.entry_points.is_empty() {
        match validate_wgsl(&context.uri, &wgsl_source) {
            Ok((module, _info)) => shader_entry_points(&module),
            Err(error) => {
                validation_diagnostics.push(error.to_string());
                Vec::new()
            }
        }
    } else {
        document
            .entry_points
            .iter()
            .map(|entry| ShaderEntryPointAsset {
                name: entry.name.clone(),
                stage: entry.stage.clone(),
            })
            .collect()
    };
    let imports = document
        .imports
        .iter()
        .map(|import| ShaderImportRedirectAsset {
            source: import.source.clone(),
            redirect: import.redirect.clone(),
        })
        .collect::<Vec<_>>();
    let dependency_locators = imports
        .iter()
        .filter_map(|import| {
            import
                .redirect
                .as_ref()
                .map(|redirect| redirect.locator.clone())
        })
        .collect::<Vec<_>>();

    let mut shader = ShaderAsset {
        uri: context.uri.clone(),
        source_language: ShaderSourceLanguage::Wgsl,
        source: wgsl_source.clone(),
        wgsl_source,
        import_path: document.import_path,
        entry_points,
        dependencies: imports
            .iter()
            .filter_map(|import| {
                import
                    .redirect
                    .clone()
                    .map(|reference| crate::asset::ShaderDependencyAsset {
                        kind: ResourceKind::Shader,
                        reference,
                    })
            })
            .collect(),
        source_files,
        imports,
        shader_defs,
        property_schema: document.properties,
        texture_slots: document
            .texture_slots
            .iter()
            .map(crate::asset::ShaderTextureSlotAsset::from)
            .collect(),
        editor: document.editor,
        pipeline_layout: document.pipeline_layout,
        validation_diagnostics,
    };
    shader
        .validation_diagnostics
        .extend(
            validate_wgsl_captures(&shader).into_iter().map(|error| {
                match error {
                crate::core::framework::render::RenderMaterialValidationError::MissingWgslCapture {
                    path,
                    name,
                    ..
                } if path.starts_with("properties.") => {
                    format!("wgsl_capture property `{name}` was not found at {path}")
                }
                crate::core::framework::render::RenderMaterialValidationError::MissingWgslCapture {
                    path,
                    name,
                    ..
                } => format!("wgsl_capture texture slot `{name}` was not found at {path}"),
                other => format!("{other:?}"),
            }
            }),
        );
    let mut outcome = AssetImportOutcome::new(context.uri.clone(), ImportedAsset::Shader(shader));
    for dependency in dependency_locators {
        outcome = outcome.with_dependency(dependency);
    }
    outcome = outcome.with_entry(data_entry_for_file(
        context,
        &zshader_path,
        "zshader",
        zshader_source,
    )?);
    for wgsl_file in wgsl_files {
        let path = package_dir.join(&wgsl_file);
        let source = fs::read_to_string(&path)?;
        outcome = outcome.with_entry(data_entry_for_file(context, &path, "wgsl", source)?);
    }
    Ok(outcome)
}

fn compound_dir_for_zmeta(zmeta_path: &Path) -> Result<PathBuf, AssetImportError> {
    let file_name = zmeta_path
        .file_name()
        .and_then(|file_name| file_name.to_str())
        .ok_or_else(|| {
            AssetImportError::Parse(format!(
                "compound shader meta path {} has no file name",
                zmeta_path.display()
            ))
        })?;
    let dir_name = file_name.strip_suffix(".zmeta").ok_or_else(|| {
        AssetImportError::Parse(format!(
            "compound shader source {} is not a .zmeta file",
            zmeta_path.display()
        ))
    })?;
    Ok(zmeta_path.with_file_name(dir_name))
}

fn primary_zshader_path(package_dir: &Path) -> Result<PathBuf, AssetImportError> {
    let mut zshader_files = Vec::new();
    collect_files_with_extension(package_dir, "zshader", &mut zshader_files)?;
    zshader_files.sort();
    zshader_files.into_iter().next().ok_or_else(|| {
        AssetImportError::Parse(format!(
            "compound shader package {} does not contain a .zshader descriptor",
            package_dir.display()
        ))
    })
}

fn wgsl_files_for_document(
    package_dir: &Path,
    document: &ZShaderDocument,
) -> Result<Vec<PathBuf>, AssetImportError> {
    if !document.wgsl_files.is_empty() {
        return Ok(document.wgsl_files.iter().map(PathBuf::from).collect());
    }
    let mut wgsl_files = Vec::new();
    collect_files_with_extension(package_dir, "wgsl", &mut wgsl_files)?;
    wgsl_files.sort();
    wgsl_files
        .into_iter()
        .map(|path| {
            path.strip_prefix(package_dir)
                .map(PathBuf::from)
                .map_err(|error| {
                    AssetImportError::Parse(format!(
                        "shader source {} is outside package dir {}: {error}",
                        path.display(),
                        package_dir.display()
                    ))
                })
        })
        .collect()
}

fn read_wgsl_sources(
    package_dir: &Path,
    root_uri: &AssetUri,
    files: &[PathBuf],
) -> Result<(String, Vec<ShaderSourceFileAsset>), AssetImportError> {
    let mut combined = String::new();
    let mut source_files = Vec::new();
    for file in files {
        let source_path = package_dir.join(file);
        let source = fs::read_to_string(&source_path)?;
        if !combined.is_empty() {
            combined.push('\n');
        }
        combined.push_str(&source);
        source_files.push(ShaderSourceFileAsset {
            path: normalized_relative_path(file),
            url: included_file_uri(root_uri, file)?,
        });
    }
    Ok((combined, source_files))
}

fn data_entry_for_file(
    context: &AssetImportContext,
    path: &Path,
    prefix: &str,
    text: String,
) -> Result<ImportedAssetEntry, AssetImportError> {
    let label = path
        .file_name()
        .and_then(|file_name| file_name.to_str())
        .map(|file_name| format!("{prefix}:{file_name}"))
        .ok_or_else(|| {
            AssetImportError::Parse(format!(
                "compound shader file {} has no file name",
                path.display()
            ))
        })?;
    let uri = AssetUri::new(
        context.uri.scheme(),
        context.uri.path().to_string(),
        Some(label),
    )?;
    Ok(ImportedAssetEntry::new(
        uri.clone(),
        ImportedAsset::Data(DataAsset {
            uri,
            format: DataAssetFormat::Text,
            text,
            canonical_json: serde_json::Value::Null,
        }),
    ))
}

fn collect_files_with_extension(
    root: &Path,
    extension: &str,
    files: &mut Vec<PathBuf>,
) -> Result<(), std::io::Error> {
    if !root.exists() {
        return Ok(());
    }
    for entry in fs::read_dir(root)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            collect_files_with_extension(&path, extension, files)?;
        } else if path
            .extension()
            .and_then(|value| value.to_str())
            .is_some_and(|value| value.eq_ignore_ascii_case(extension))
        {
            files.push(path);
        }
    }
    Ok(())
}

fn included_file_uri(root_uri: &AssetUri, relative: &Path) -> Result<AssetUri, AssetImportError> {
    AssetUri::new(
        root_uri.scheme(),
        format!("{}/{}", root_uri.path(), normalized_relative_path(relative)),
        None,
    )
    .map_err(AssetImportError::from)
}

fn normalized_relative_path(path: &Path) -> String {
    path.components()
        .map(|component| component.as_os_str().to_string_lossy())
        .collect::<Vec<_>>()
        .join("/")
}
