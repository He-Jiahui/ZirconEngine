use std::fs;
use std::path::{Path, PathBuf};

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
    wgsl_include_paths, ShaderAssetKind, ShaderImportPathDerivation,
    ShaderImportPathDerivationError, SHADER_IMPORT_PROJECT_NAMESPACE_SETTING,
};
use crate::core::resource::{ResourceDiagnostic, ResourceDiagnosticSeverity, ResourceKind};

pub(crate) fn import_shader_package(
    context: &AssetImportContext,
) -> Result<AssetImportOutcome, AssetImportError> {
    let package_dir = compound_dir_for_zmeta(&context.source_path)?;
    let zshader_path = primary_zshader_path(&package_dir)?;
    let zshader_source = fs::read_to_string(&zshader_path)?;
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
    let wgsl_files = wgsl_files_for_document(&package_dir, &document)?;
    let (wgsl_source, source_files) =
        read_wgsl_sources(&package_dir, &context.uri, wgsl_files.as_slice())?;
    let mut validation_diagnostics = Vec::new();
    let mut import_diagnostics = Vec::new();
    let import_path = document_import_path(&document, derived_import_path, &mut import_diagnostics)
        .map_err(|error| shader_import_path_derivation_error(&context.uri, &zshader_path, error))?;
    let entry_points = if document.entry_points().is_empty() {
        match validate_wgsl(&context.uri, &wgsl_source) {
            Ok((module, _info)) => shader_entry_points(&module),
            Err(error) => {
                validation_diagnostics.push(error.to_string());
                Vec::new()
            }
        }
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
    for wgsl_file in wgsl_files {
        let path = package_dir.join(&wgsl_file);
        let source = fs::read_to_string(&path)?;
        outcome = outcome.with_entry(data_entry_for_file(context, &path, "wgsl", source)?);
    }
    Ok(outcome)
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
        .import_settings
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
            "; legacy v1 .zshader must be migrated to schema v2 with kind = \"surface\", \"include\", \"compute\", or \"fullscreen\""
        }
        ZShaderV2Error::ForbiddenField { field, .. }
            if matches!(field.as_str(), "pipeline_layout" | "shader_defs" | "shader_def_values") =>
        {
            "; legacy user-authored pipeline layout and shader_defs fields are removed from .zshader v2 and must be migrated to generated ABI/options"
        }
        _ => ""
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
    document: &ZShaderDocumentV2,
) -> Result<Vec<PathBuf>, AssetImportError> {
    if !document.wgsl_files().is_empty() {
        return Ok(document.wgsl_files().iter().map(PathBuf::from).collect());
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

#[cfg(test)]
mod tests {
    use super::{
        append_shader_module_diagnostics, document_import_path, ShaderImportPathDerivationError,
        ShaderImportRedirectAsset, ZShaderDocumentV2,
    };

    #[test]
    fn zshader_import_diagnostics_report_undeclared_wgsl_include() {
        let document = ZShaderDocumentV2::from_toml_str(
            r#"
kind = "surface"
version = 2
shading_model = "standard_pbr"
wgsl_files = ["surface.wgsl"]
"#,
        )
        .expect("surface zshader should parse");
        let mut diagnostics = Vec::new();

        append_shader_module_diagnostics(
            &mut diagnostics,
            &document,
            "#include <project::math>\nfn zr_material_surface() {}",
            &[],
        );

        assert!(diagnostics
            .iter()
            .any(|diagnostic| diagnostic.contains("project::math")));
    }

    #[test]
    fn zshader_include_module_diagnostics_reject_entry_points_and_bindings() {
        let document = ZShaderDocumentV2::from_toml_str(
            r#"
kind = "include"
version = 2
import_path = "project::bad"
wgsl_files = ["bad.wgsl"]
"#,
        )
        .expect("include zshader should parse");
        let mut diagnostics = Vec::new();

        append_shader_module_diagnostics(
            &mut diagnostics,
            &document,
            "@group(2) @binding(0) var<uniform> bad: vec4<f32>;\n@fragment\nfn fs_main() {}",
            &[] as &[ShaderImportRedirectAsset],
        );

        assert!(diagnostics
            .iter()
            .any(|diagnostic| diagnostic.contains("@group binding")));
        assert!(diagnostics
            .iter()
            .any(|diagnostic| diagnostic.contains("entry point annotation")));
    }

    #[test]
    fn zshader_import_path_validation_rejects_self_namespace_overrides() {
        let document = ZShaderDocumentV2::from_toml_str(
            r#"
kind = "include"
version = 2
import_path = "self::material"
"#,
        )
        .expect("include zshader should parse before import path validation");
        let mut diagnostics = Vec::new();

        let error = document_import_path(&document, None, &mut diagnostics)
            .expect_err("self namespace must stay generated-local");

        assert_eq!(
            error,
            ShaderImportPathDerivationError::ReservedNamespace {
                namespace: "self".to_string()
            }
        );
    }
}
