use std::fs;
use std::path::{Path, PathBuf};

use super::auxiliary_source::AuxiliarySourceResolver;
use super::import_shader::shader_entry_points;
use super::validate_wgsl::validate_wgsl;
use crate::asset::assets::{
    generate_material_artifact, validate_wgsl_captures, DataAsset, DataAssetFormat, ImportedAsset,
    ShaderAsset, ShaderEntryPointAsset, ShaderImportRedirectAsset, ShaderOptionAsset,
    ShaderSourceFileAsset, ShaderSourceLanguage, ZShaderDocumentV2, ZShaderV2Error,
};
use crate::asset::{
    AssetImportContext, AssetImportError, AssetImportOutcome, AssetUri, ImportedAssetEntry,
};
use crate::core::framework::render::{
    derive_shader_import_path, is_builtin_shader_module_token, is_generated_shader_module_token,
    strip_wgsl_include_directives, wgsl_include_paths, ShaderAssetKind, ShaderImportPathDerivation,
    ShaderImportPathDerivationError, SHADER_IMPORT_PROJECT_NAMESPACE_SETTING,
};
use crate::core::resource::{ResourceDiagnostic, ResourceDiagnosticSeverity, ResourceKind};

#[path = "import_shader_package/generated_material_anchor_hint.rs"]
mod generated_material_anchor_hint;
use generated_material_anchor_hint::append_generated_material_anchor_hint;

pub(crate) fn import_shader_package(
    context: &AssetImportContext,
) -> Result<AssetImportOutcome, AssetImportError> {
    let package_dir = compound_dir_for_zmeta(&context.source_path)?;
    let zshader_path = primary_zshader_path(context, &package_dir)?;
    let zshader_bytes = context
        .source_file_snapshot(&zshader_path)
        .map(<[u8]>::to_vec)
        .map(Ok)
        .unwrap_or_else(|| {
            let resolver = AuxiliarySourceResolver::new(context, &package_dir)?;
            let relative = zshader_path.strip_prefix(&package_dir).map_err(|_| {
                AssetImportError::Parse("shader descriptor is outside package".into())
            })?;
            resolver
                .read_path_snapshot(relative, super::gltf_decode::MAX_GLTF_AUXILIARY_BYTES)
                .map(|(_, bytes, _)| bytes)
        })?;
    let zshader_source =
        String::from_utf8(zshader_bytes).map_err(|source| AssetImportError::SourceTextDecode {
            path: zshader_path.clone(),
            source,
        })?;
    let document = ZShaderDocumentV2::from_toml_str(&zshader_source)
        .map_err(|error| zshader_v2_import_error(&context.uri, &zshader_path, error))?;
    let derived_import_path = match derive_document_import_path(context, &zshader_path, &document) {
        Ok(derived) => derived,
        Err(_) if document.import_path().is_some() => None,
        Err(error) => {
            return Err(shader_import_path_derivation_error(
                &context.uri,
                &zshader_path,
                error,
            ));
        }
    };
    let wgsl_files = wgsl_files_for_document(context, &package_dir, &document)?;
    let (wgsl_source, source_files, wgsl_data_sources) =
        read_wgsl_sources(context, &package_dir, wgsl_files.as_slice())?;
    let mut validation_diagnostics = Vec::new();
    let mut import_diagnostics = Vec::new();
    let import_path = document_import_path(&document, derived_import_path, &mut import_diagnostics)
        .map_err(|error| shader_import_path_derivation_error(&context.uri, &zshader_path, error))?;
    let entry_points = if document.entry_points().is_empty() {
        package_shader_entry_points(&context.uri, &wgsl_source, &mut validation_diagnostics)
    } else {
        document
            .entry_points()
            .iter()
            .map(|entry| ShaderEntryPointAsset {
                name: entry.name.clone(),
                stage: entry.stage.clone(),
            })
            .collect()
    };
    let imports = document
        .imports()
        .iter()
        .map(|import| ShaderImportRedirectAsset {
            source: import.source.clone(),
            redirect: import.redirect.clone(),
        })
        .collect::<Vec<_>>();
    append_shader_module_diagnostics(
        &mut validation_diagnostics,
        &document,
        &wgsl_source,
        &imports,
    );
    append_generated_material_anchor_hint(&mut import_diagnostics, &document, &wgsl_source);
    let dependency_locators = imports
        .iter()
        .filter_map(|import| {
            import
                .redirect
                .as_ref()
                .map(|redirect| redirect.locator.clone())
        })
        .collect::<Vec<_>>();
    let property_schema = document.properties().to_vec();
    let options = document
        .options()
        .iter()
        .map(ShaderOptionAsset::from)
        .collect::<Vec<_>>();
    let texture_slots = document
        .texture_slots()
        .iter()
        .map(crate::asset::ShaderTextureSlotAsset::from)
        .collect::<Vec<_>>();
    let generated_material = generate_material_artifact(&property_schema, &options, &texture_slots);

    let mut shader = ShaderAsset {
        uri: context.uri.clone(),
        kind: document.kind(),
        source_language: ShaderSourceLanguage::Wgsl,
        source: wgsl_source.clone(),
        wgsl_source,
        import_path,
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
        shader_defs: Vec::new(),
        property_schema,
        options,
        texture_slots,
        shading_model: document.shading_model().map(str::to_string),
        render_state: document.render_state(),
        queue: document.queue(),
        disabled_passes: document.disabled_passes().to_vec(),
        resources: document.resources().to_vec(),
        material_property_layout: generated_material.property_layout,
        material_option_table: generated_material.option_table,
        generated_material_wgsl: generated_material.wgsl_source,
        editor: document.editor().clone(),
        pipeline_layout: Default::default(),
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
    for diagnostic in import_diagnostics {
        outcome = outcome.with_diagnostic(diagnostic);
    }
    for dependency in dependency_locators {
        outcome = outcome.with_dependency(dependency);
    }
    outcome = outcome.with_entry(data_entry_for_file(
        context,
        &zshader_path,
        "zshader",
        zshader_source,
    )?);
    for (path, source) in wgsl_data_sources {
        outcome = outcome.with_entry(data_entry_for_file(context, &path, "wgsl", source)?);
    }
    Ok(outcome)
}

fn package_shader_entry_points(
    uri: &AssetUri,
    wgsl_source: &str,
    validation_diagnostics: &mut Vec<String>,
) -> Vec<ShaderEntryPointAsset> {
    let entry_point_source = strip_wgsl_include_directives(wgsl_source);
    match validate_wgsl(uri, &entry_point_source) {
        Ok((module, _info)) => shader_entry_points(&module),
        Err(error) => {
            validation_diagnostics.push(error.to_string());
            Vec::new()
        }
    }
}

fn derive_document_import_path(
    context: &AssetImportContext,
    zshader_path: &Path,
    document: &ZShaderDocumentV2,
) -> Result<Option<ShaderImportPathDerivation>, ShaderImportPathDerivationError> {
    if !matches!(
        document.kind(),
        ShaderAssetKind::Surface | ShaderAssetKind::Include
    ) {
        return Ok(None);
    }
    let project_namespace = shader_project_namespace(context);
    derive_shader_import_path(
        project_namespace.as_str(),
        logical_zshader_asset_path(context, zshader_path).as_str(),
    )
    .map(Some)
}

fn document_import_path(
    document: &ZShaderDocumentV2,
    derived: Option<ShaderImportPathDerivation>,
    diagnostics: &mut Vec<ResourceDiagnostic>,
) -> Result<Option<String>, ShaderImportPathDerivationError> {
    let Some(derived) = derived else {
        if let Some(explicit) = document
            .import_path()
            .map(str::trim)
            .filter(|path| !path.is_empty())
        {
            validate_explicit_shader_import_path(explicit)?;
            return Ok(Some(explicit.to_string()));
        }
        return Ok(None);
    };
    if let Some(explicit) = document
        .import_path()
        .map(str::trim)
        .filter(|path| !path.is_empty())
    {
        validate_explicit_shader_import_path(explicit)?;
        if explicit == derived.import_path {
            diagnostics.push(ResourceDiagnostic {
                severity: ResourceDiagnosticSeverity::Warning,
                message: format!(
                    "zshader import_path `{explicit}` duplicates the derived shader import path; remove the redundant declaration"
                ),
            });
        }
        Ok(Some(explicit.to_string()))
    } else if matches!(
        document.kind(),
        ShaderAssetKind::Surface | ShaderAssetKind::Include
    ) {
        Ok(Some(derived.import_path))
    } else {
        Ok(None)
    }
}

fn validate_explicit_shader_import_path(
    import_path: &str,
) -> Result<(), ShaderImportPathDerivationError> {
    let namespace = import_path.split("::").next().unwrap_or_default().trim();
    if namespace == "self" || namespace.starts_with("zr_") {
        return Err(ShaderImportPathDerivationError::ReservedNamespace {
            namespace: namespace.to_string(),
        });
    }
    Ok(())
}

fn shader_project_namespace(context: &AssetImportContext) -> String {
    context
        .import_settings()
        .get(SHADER_IMPORT_PROJECT_NAMESPACE_SETTING)
        .and_then(toml::Value::as_str)
        .unwrap_or("project")
        .to_string()
}

fn logical_zshader_asset_path(context: &AssetImportContext, zshader_path: &Path) -> String {
    let mut path = context.uri.path().trim_matches('/').to_string();
    if let Some(file_name) = zshader_path
        .file_name()
        .and_then(|file_name| file_name.to_str())
    {
        if !path.is_empty() {
            path.push('/');
        }
        path.push_str(file_name);
    }
    path
}

fn append_shader_module_diagnostics(
    diagnostics: &mut Vec<String>,
    document: &ZShaderDocumentV2,
    wgsl_source: &str,
    imports: &[ShaderImportRedirectAsset],
) {
    let include_paths = wgsl_include_paths(wgsl_source);
    for include_path in &include_paths {
        if is_builtin_shader_module_token(include_path)
            || is_generated_shader_module_token(include_path)
            || imports.iter().any(|import| import.source == *include_path)
        {
            continue;
        }
        diagnostics.push(format!(
            "wgsl include `{include_path}` is not declared in zshader imports"
        ));
    }
    for import in imports {
        if !include_paths
            .iter()
            .any(|include_path| include_path == &import.source)
        {
            diagnostics.push(format!(
                "zshader import `{}` has no matching WGSL #include directive",
                import.source
            ));
        }
    }
    if document.kind().is_include() {
        append_include_module_lexical_diagnostics(diagnostics, wgsl_source);
    }
}

fn append_include_module_lexical_diagnostics(diagnostics: &mut Vec<String>, wgsl_source: &str) {
    for (line_index, line) in wgsl_source.lines().enumerate() {
        let line_number = line_index + 1;
        let trimmed = line.trim_start();
        if trimmed.contains("@group(") {
            diagnostics.push(format!(
                "include shader module declares @group binding at line {line_number}; module bindings must be generated by the engine ABI"
            ));
        }
        if trimmed.contains("@vertex")
            || trimmed.contains("@fragment")
            || trimmed.contains("@compute")
        {
            diagnostics.push(format!(
                "include shader module declares an entry point annotation at line {line_number}; include modules must not own entry points"
            ));
        }
        if let Some(symbol) = declared_module_symbol(trimmed) {
            if let Some(prefix) = reserved_shader_module_prefix(symbol) {
                diagnostics.push(format!(
                    "include shader module symbol `{symbol}` uses reserved prefix `{prefix}`"
                ));
            }
        }
    }
}

fn declared_module_symbol(line: &str) -> Option<&str> {
    let rest = line
        .strip_prefix("fn ")
        .or_else(|| line.strip_prefix("struct "))
        .or_else(|| line.strip_prefix("const "))?;
    rest.split(['(', '{', ':', '='])
        .next()
        .map(str::trim)
        .filter(|symbol| !symbol.is_empty())
}

fn reserved_shader_module_prefix(symbol: &str) -> Option<&'static str> {
    ["zr_", "ZR_OPT_", "ZrMaterial", "ZrCompute"]
        .into_iter()
        .find(|prefix| symbol.starts_with(prefix))
}

fn zshader_v2_import_error(uri: &AssetUri, path: &Path, error: ZShaderV2Error) -> AssetImportError {
    let migration_note = match &error {
        ZShaderV2Error::MissingDocumentField { field } if field == "kind" => {
            "; schema v1 .zshader must be migrated to schema v2 with kind = \"surface\", \"include\", \"compute\", or \"fullscreen\""
        }
        ZShaderV2Error::ForbiddenField { field, .. }
            if matches!(
                field.as_str(),
                "pipeline_layout" | "shader_defs" | "shader_def_values"
            ) =>
        {
            "; removed user-authored pipeline layout and shader_defs fields must be migrated to generated ABI/options for .zshader v2"
        }
        _ => "",
    };
    AssetImportError::Parse(format!(
        "parse zshader v2 toml for {uri} at {}: {error}{migration_note}",
        path.display()
    ))
}

fn shader_import_path_derivation_error(
    uri: &AssetUri,
    path: &Path,
    error: ShaderImportPathDerivationError,
) -> AssetImportError {
    AssetImportError::Parse(format!(
        "derive zshader import_path for {uri} at {}: {error}",
        path.display()
    ))
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

fn primary_zshader_path(
    context: &AssetImportContext,
    package_dir: &Path,
) -> Result<PathBuf, AssetImportError> {
    let mut zshader_files = Vec::new();
    if context.has_source_file_snapshots() {
        zshader_files.extend(
            context
                .source_file_snapshot_paths()
                .filter(|path| {
                    path.starts_with(package_dir)
                        && path
                            .extension()
                            .and_then(|value| value.to_str())
                            .is_some_and(|value| value.eq_ignore_ascii_case("zshader"))
                })
                .map(Path::to_path_buf),
        );
    } else {
        collect_files_with_extension(package_dir, "zshader", &mut zshader_files)?;
    }
    zshader_files.sort();
    zshader_files.into_iter().next().ok_or_else(|| {
        AssetImportError::Parse(format!(
            "compound shader package {} does not contain a .zshader descriptor",
            package_dir.display()
        ))
    })
}

fn wgsl_files_for_document(
    context: &AssetImportContext,
    package_dir: &Path,
    document: &ZShaderDocumentV2,
) -> Result<Vec<PathBuf>, AssetImportError> {
    if !document.wgsl_files().is_empty() {
        return Ok(document.wgsl_files().iter().map(PathBuf::from).collect());
    }
    let mut wgsl_files = Vec::new();
    if context.has_source_file_snapshots() {
        for path in context.source_file_snapshot_paths().filter(|path| {
            path.starts_with(package_dir)
                && path
                    .extension()
                    .and_then(|value| value.to_str())
                    .is_some_and(|value| value.eq_ignore_ascii_case("wgsl"))
        }) {
            wgsl_files.push(path.to_path_buf());
        }
    } else {
        collect_files_with_extension(package_dir, "wgsl", &mut wgsl_files)?;
    }
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
    context: &AssetImportContext,
    package_dir: &Path,
    files: &[PathBuf],
) -> Result<(String, Vec<ShaderSourceFileAsset>, Vec<(PathBuf, String)>), AssetImportError> {
    let resolver = AuxiliarySourceResolver::new(context, package_dir)?;
    let mut combined = String::new();
    let mut source_files = Vec::with_capacity(files.len());
    let mut data_sources = Vec::with_capacity(files.len());
    for file in files {
        let lexical_path = resolver.resolve_lexical_path(file)?;
        let (source_path, bytes) = if let Some(bytes) = context.source_file_snapshot(&lexical_path)
        {
            (lexical_path, bytes.to_vec())
        } else if context.has_source_file_snapshots() {
            return Err(AssetImportError::Parse(format!(
                "shader auxiliary snapshot does not contain {}",
                lexical_path.display()
            )));
        } else {
            let (source_path, bytes, _) =
                resolver.read_path_snapshot(file, super::gltf_decode::MAX_GLTF_AUXILIARY_BYTES)?;
            (source_path, bytes)
        };
        let source =
            String::from_utf8(bytes).map_err(|source| AssetImportError::SourceTextDecode {
                path: source_path.clone(),
                source,
            })?;
        let root_relative = resolver.root_relative_path(&source_path)?;
        let package_relative = source_path
            .strip_prefix(package_dir)
            .unwrap_or(root_relative);
        if !combined.is_empty() {
            combined.push('\n');
        }
        combined.push_str(&source);
        source_files.push(ShaderSourceFileAsset {
            path: normalized_relative_path(package_relative),
            url: included_file_uri(&context.uri, root_relative)?,
        });
        data_sources.push((source_path, source));
    }
    Ok((combined, source_files, data_sources))
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
    let relative = normalized_relative_path(relative);
    let path = root_uri
        .package_id()
        .map(|package_id| format!("{package_id}/{relative}"))
        .unwrap_or(relative);
    AssetUri::new(root_uri.scheme(), path, None).map_err(AssetImportError::from)
}

fn normalized_relative_path(path: &Path) -> String {
    let mut normalized = String::with_capacity(path.as_os_str().len());
    for component in path.components() {
        if !normalized.is_empty() {
            normalized.push('/');
        }
        normalized.push_str(&component.as_os_str().to_string_lossy());
    }
    normalized
}

pub(crate) fn snapshot_external_shader_sources(
    asset_root: &Path,
    source_path: &Path,
    source_uri: &AssetUri,
    existing: &std::collections::BTreeMap<PathBuf, Vec<u8>>,
    limit: u64,
) -> Result<(std::collections::BTreeMap<PathBuf, Vec<u8>>, u64), AssetImportError> {
    let package_dir = compound_dir_for_zmeta(source_path)?;
    let mut zshader_paths = existing
        .keys()
        .filter(|path| {
            path.starts_with(&package_dir)
                && path
                    .extension()
                    .and_then(|value| value.to_str())
                    .is_some_and(|value| value.eq_ignore_ascii_case("zshader"))
        })
        .cloned()
        .collect::<Vec<_>>();
    zshader_paths.sort();
    let zshader_path = zshader_paths.into_iter().next().ok_or_else(|| {
        AssetImportError::Parse(format!(
            "shader snapshot for {} contains no .zshader descriptor",
            package_dir.display()
        ))
    })?;
    let zshader_bytes = existing.get(&zshader_path).ok_or_else(|| {
        AssetImportError::Parse(format!(
            "shader snapshot is missing descriptor {}",
            zshader_path.display()
        ))
    })?;
    let zshader_source = std::str::from_utf8(zshader_bytes).map_err(|error| {
        AssetImportError::Parse(format!(
            "shader descriptor {} is not valid UTF-8: {error}",
            zshader_path.display()
        ))
    })?;
    let document = ZShaderDocumentV2::from_toml_str(zshader_source)
        .map_err(|error| zshader_v2_import_error(source_uri, &zshader_path, error))?;
    let wgsl_files = if document.wgsl_files().is_empty() {
        existing
            .keys()
            .filter(|path| {
                path.starts_with(&package_dir)
                    && path
                        .extension()
                        .and_then(|value| value.to_str())
                        .is_some_and(|value| value.eq_ignore_ascii_case("wgsl"))
            })
            .filter_map(|path| path.strip_prefix(&package_dir).ok().map(Path::to_path_buf))
            .collect::<Vec<_>>()
    } else {
        document.wgsl_files().iter().map(PathBuf::from).collect()
    };
    let resolver = AuxiliarySourceResolver::for_asset_root(asset_root, &package_dir)?;
    let mut snapshots = std::collections::BTreeMap::new();
    let mut remaining = limit;
    let mut latest_mtime_unix_ms = 0;
    for file in wgsl_files {
        let lexical = resolver.resolve_lexical_path(&file)?;
        if existing.contains_key(&lexical) || snapshots.contains_key(&lexical) {
            continue;
        }
        if existing.len().saturating_add(snapshots.len())
            >= AuxiliarySourceResolver::MAX_SNAPSHOT_FILES
        {
            return Err(AssetImportError::Parse(format!(
                "shader source snapshot exceeds the {}-file cumulative limit",
                AuxiliarySourceResolver::MAX_SNAPSHOT_FILES
            )));
        }
        let (_, bytes, mtime_unix_ms) = resolver.read_path_snapshot(&file, remaining)?;
        remaining = remaining.checked_sub(bytes.len() as u64).ok_or_else(|| {
            AssetImportError::Parse(format!(
                "shader auxiliary sources exceed the {limit}-byte cumulative limit"
            ))
        })?;
        latest_mtime_unix_ms = latest_mtime_unix_ms.max(mtime_unix_ms);
        snapshots.insert(lexical, bytes);
    }
    Ok((snapshots, latest_mtime_unix_ms))
}

#[cfg(test)]
#[path = "tests/import_shader_package.rs"]
mod tests;
