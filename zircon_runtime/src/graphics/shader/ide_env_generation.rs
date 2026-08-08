use std::collections::BTreeSet;
use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

use serde::Serialize;

use crate::asset::assets::{ImportedAsset, ShaderAsset};
use crate::asset::project::ProjectManager;
use crate::asset::AssetUri;
use crate::core::framework::render::{
    shader_ide_generated_material_stub_relative_path, shader_ide_module_stub_relative_path,
    shader_ide_preview_relative_path, shader_ide_preview_segments_relative_path,
    shader_ide_relative_path_string, strip_wgsl_include_directives, wgsl_include_paths,
    RenderShaderDefinitionValue, ShaderAssetKind, ShaderIdeModuleMap, ShaderIdeModuleMapEntry,
    ShaderIdeModuleSource, ShaderIdePreviewMap, ShaderIdePreviewVariant,
    GENERATED_MATERIAL_MODULE_IMPORT_PATH, SHADER_IDE_ENV_CACHE_DIR, SHADER_IDE_MODULE_MAP_FILE,
};
use crate::core::resource::{ResourceKind, ResourceRecord, ResourceState};

use super::ide_preview::{assemble_shader_ide_surface_preview_with_index, shader_include_index};
use super::{
    builtin_shader_ide_module_sources, parse_shader_ide_wgsl_module,
    validate_shader_ide_wgsl_module, ShaderIdeSurfacePreview,
};

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct ShaderIdeEnvReport {
    pub output_dir: String,
    pub module_map: String,
    pub shader_count: usize,
    pub module_count: usize,
    pub generated_material_count: usize,
    pub preview_count: usize,
    pub naga_parsed_stub_count: usize,
    pub naga_validated_preview_count: usize,
    pub managed_file_count: usize,
    pub written_file_count: usize,
    pub removed_stale_file_count: usize,
}

struct ShaderIdeStub {
    relative_path: PathBuf,
    source: String,
    include_paths: Vec<String>,
    validation_defines: Vec<RenderShaderDefinitionValue>,
    entry: ShaderIdeModuleMapEntry,
}

struct ShaderIdePreviewFile {
    wgsl_relative_path: PathBuf,
    segment_relative_path: PathBuf,
    wgsl_source: String,
    segment_map: ShaderIdePreviewMap,
}

const SHADER_IDE_COMMON_BUILTIN_VALIDATION_MODULES: &[&str] = &[
    "zr_scene_runtime.wgsl",
    "zr_gpu_scene.wgsl",
    "zr_environment.wgsl",
    "zr_light_grid.wgsl",
    "zr_shadow.wgsl",
    "zr_surface_types.wgsl",
];

const SHADER_IDE_STUB_VALIDATION_DEFINES: &str = r#"
const ZR_GEOMETRY_SOURCE_TOKEN: u32 = 0u;
const ZR_BINDLESS_MATERIAL_SLOT_CAPACITY: u32 = 1u;
const ZR_FEATURE_ALPHA_TEST: bool = false;
const ZR_FEATURE_RECEIVE_SHADOWS: bool = false;
const ZR_FEATURE_DOUBLE_SIDED: bool = false;
const ZR_FEATURE_HAS_NORMAL_TEXTURE: bool = false;
const ZR_FEATURE_PBR_CLEARCOAT: bool = false;
const ZR_FEATURE_PBR_ANISOTROPY: bool = false;
const ZR_FEATURE_PBR_TRANSMISSION: bool = false;
"#;

pub fn write_shader_ide_env_for_project(
    project: &ProjectManager,
    output_dir: Option<&Path>,
    preview_variants: &[ShaderIdePreviewVariant],
) -> Result<ShaderIdeEnvReport, String> {
    validate_shader_ide_preview_variants(preview_variants)?;

    let output_dir = output_dir
        .map(Path::to_path_buf)
        .unwrap_or_else(|| project.paths().cache_root().join(SHADER_IDE_ENV_CACHE_DIR));
    fs::create_dir_all(&output_dir).map_err(|error| {
        format!(
            "create shader IDE output directory {}: {error}",
            output_dir.display()
        )
    })?;

    let shader_records = shader_records(project.registry().values());
    let shaders = load_shader_assets(project, &shader_records)?;
    let shader_count = shaders.len();
    let mut stubs = builtin_stubs();
    for shader in &shaders {
        stubs.extend(shader_stubs(shader));
    }
    let previews = if preview_variants.is_empty() {
        Vec::new()
    } else {
        shader_preview_files(&shaders, preview_variants)?
    };

    stubs.sort_by(|left, right| left.entry.stub_path.cmp(&right.entry.stub_path));
    let naga_parsed_stub_count = parse_shader_ide_stubs(&stubs)?;
    let naga_validated_preview_count = validate_shader_ide_previews(&previews)?;
    let module_map = ShaderIdeModuleMap::new(
        project.manifest().name.clone(),
        stubs.iter().map(|stub| stub.entry.clone()).collect(),
    );
    let module_map_json = serde_json::to_string_pretty(&module_map)
        .map_err(|error| format!("encode shader IDE module map: {error}"))?;
    let managed_paths = shader_ide_managed_paths(&stubs, &previews);
    let managed_file_count = managed_paths.len();

    let mut written_file_count = 0;
    for stub in &stubs {
        written_file_count += usize::from(write_stub(&output_dir, stub)?);
    }
    for preview in &previews {
        written_file_count += write_preview(&output_dir, preview)?;
    }

    let module_map_path = output_dir.join(SHADER_IDE_MODULE_MAP_FILE);
    written_file_count += usize::from(write_text_if_changed(
        &module_map_path,
        &module_map_json,
        "shader IDE module map",
    )?);
    let removed_stale_file_count = remove_stale_shader_ide_files(&output_dir, &managed_paths)?;

    let generated_material_count = stubs.iter().filter(|stub| stub.entry.generated).count();
    Ok(ShaderIdeEnvReport {
        output_dir: output_dir.display().to_string(),
        module_map: module_map_path.display().to_string(),
        shader_count,
        module_count: stubs.len(),
        generated_material_count,
        preview_count: previews.len(),
        naga_parsed_stub_count,
        naga_validated_preview_count,
        managed_file_count,
        written_file_count,
        removed_stale_file_count,
    })
}

fn validate_shader_ide_preview_variants(
    preview_variants: &[ShaderIdePreviewVariant],
) -> Result<(), String> {
    let mut names = BTreeSet::new();
    for variant in preview_variants {
        if variant.name.trim().is_empty() {
            return Err("shader IDE preview variant name cannot be empty".to_string());
        }
        if !names.insert(variant.name.as_str()) {
            return Err(format!(
                "duplicate shader IDE preview variant {}",
                variant.name
            ));
        }
    }
    Ok(())
}

fn shader_records<'a>(
    records: impl Iterator<Item = &'a ResourceRecord>,
) -> Vec<&'a ResourceRecord> {
    let mut records = records
        .filter(|record| {
            record.kind == ResourceKind::Shader && record.state == ResourceState::Ready
        })
        .collect::<Vec<_>>();
    records.sort_by(|left, right| left.primary_locator().cmp(right.primary_locator()));
    records
}

fn load_shader_assets(
    project: &ProjectManager,
    records: &[&ResourceRecord],
) -> Result<Vec<ShaderAsset>, String> {
    let mut shaders = Vec::new();
    for record in records {
        let uri = record.primary_locator().clone();
        let ImportedAsset::Shader(shader) = project
            .load_artifact(&uri)
            .map_err(|error| format!("load shader artifact {uri}: {error}"))?
        else {
            continue;
        };
        shaders.push(shader);
    }
    Ok(shaders)
}

fn builtin_stubs() -> Vec<ShaderIdeStub> {
    builtin_shader_ide_module_sources()
        .into_iter()
        .map(module_source_stub)
        .collect()
}

fn module_source_stub(source: ShaderIdeModuleSource) -> ShaderIdeStub {
    let relative_path = shader_ide_module_stub_relative_path(&source.import_path);
    let include_paths = wgsl_include_paths(&source.source);
    let source_text = shader_stub_source_header(&source.import_path, None)
        + strip_wgsl_include_directives(&source.source).as_str();
    let content_hash = shader_source_hash(&source_text);
    let stub_path = shader_ide_relative_path_string(&relative_path);
    ShaderIdeStub {
        relative_path,
        source: source_text,
        include_paths,
        validation_defines: Vec::new(),
        entry: ShaderIdeModuleMapEntry {
            import_path: source.import_path,
            scope_uri: None,
            kind: source.kind,
            stub_path,
            source_uri: None,
            source_files: Vec::new(),
            content_hash,
            generated: false,
        },
    }
}

fn shader_stubs(shader: &ShaderAsset) -> Vec<ShaderIdeStub> {
    let mut stubs = Vec::new();
    if let Some(import_path) = shader
        .import_path
        .as_ref()
        .filter(|path| !path.trim().is_empty())
    {
        let runtime_source = shader
            .runtime_wgsl_source()
            .unwrap_or(shader.wgsl_source.as_str());
        let relative_path = shader_ide_module_stub_relative_path(import_path);
        let source = shader_stub_source_header(import_path, Some(&shader.uri))
            + strip_wgsl_include_directives(runtime_source).as_str();
        let content_hash = shader_source_hash(&source);
        let stub_path = shader_ide_relative_path_string(&relative_path);
        stubs.push(ShaderIdeStub {
            relative_path,
            source,
            include_paths: wgsl_include_paths(runtime_source),
            validation_defines: shader
                .material_option_table
                .definition_values_for_bits(shader.material_option_table.default_bits()),
            entry: ShaderIdeModuleMapEntry {
                import_path: import_path.clone(),
                scope_uri: None,
                kind: shader.kind,
                stub_path,
                source_uri: Some(shader.uri.clone()),
                source_files: shader
                    .source_files
                    .iter()
                    .map(|source_file| source_file.path.clone())
                    .collect(),
                content_hash,
                generated: false,
            },
        });
    }
    if shader.kind == ShaderAssetKind::Surface && !shader.generated_material_wgsl.trim().is_empty()
    {
        let import_path = GENERATED_MATERIAL_MODULE_IMPORT_PATH.to_string();
        let relative_path = shader_ide_generated_material_stub_relative_path(&shader.uri);
        let source = shader_stub_source_header(&import_path, Some(&shader.uri))
            + shader.generated_material_wgsl.as_str();
        let content_hash = shader_source_hash(&source);
        let stub_path = shader_ide_relative_path_string(&relative_path);
        stubs.push(ShaderIdeStub {
            relative_path,
            source,
            include_paths: Vec::new(),
            validation_defines: Vec::new(),
            entry: ShaderIdeModuleMapEntry {
                import_path,
                scope_uri: Some(shader.uri.clone()),
                kind: shader.kind,
                stub_path,
                source_uri: Some(shader.uri.clone()),
                source_files: Vec::new(),
                content_hash,
                generated: true,
            },
        });
    }
    stubs
}

fn shader_stub_source_header(import_path: &str, source_uri: Option<&AssetUri>) -> String {
    let mut header = format!("// Zircon shader IDE stub: {import_path}\n");
    if let Some(source_uri) = source_uri {
        header.push_str(&format!("// Source asset: {source_uri}\n"));
    }
    header.push('\n');
    header
}

fn shader_source_hash(source: &str) -> String {
    blake3::hash(source.as_bytes()).to_hex().to_string()
}

fn shader_preview_files(
    shaders: &[ShaderAsset],
    preview_variants: &[ShaderIdePreviewVariant],
) -> Result<Vec<ShaderIdePreviewFile>, String> {
    let mut previews = Vec::new();
    let shader_index = shader_include_index(shaders.iter());
    for shader in shaders
        .iter()
        .filter(|shader| shader.kind.participates_in_material_variants())
    {
        for variant in preview_variants {
            let preview =
                assemble_shader_ide_surface_preview_with_index(shader, &shader_index, variant)
                    .map_err(|error| error.to_string())?;
            previews.push(shader_preview_file(shader, variant, preview));
        }
    }
    previews.sort_by(|left, right| left.wgsl_relative_path.cmp(&right.wgsl_relative_path));
    Ok(previews)
}

fn shader_preview_file(
    shader: &ShaderAsset,
    variant: &ShaderIdePreviewVariant,
    preview: ShaderIdeSurfacePreview,
) -> ShaderIdePreviewFile {
    let wgsl_relative_path = shader_ide_preview_relative_path(&shader.uri, &variant.name);
    let segment_relative_path =
        shader_ide_preview_segments_relative_path(&shader.uri, &variant.name);
    let wgsl_path = shader_ide_relative_path_string(&wgsl_relative_path);
    let segment_map = ShaderIdePreviewMap::new(
        shader.uri.clone(),
        variant.name.clone(),
        wgsl_path,
        preview.segments,
    );
    ShaderIdePreviewFile {
        wgsl_relative_path,
        segment_relative_path,
        wgsl_source: preview.wgsl_source,
        segment_map,
    }
}

fn parse_shader_ide_stubs(stubs: &[ShaderIdeStub]) -> Result<usize, String> {
    for stub in stubs {
        let validation_source =
            shader_ide_stub_validation_source(stub, stubs).map_err(|error| {
                format!(
                    "resolve shader IDE stub dependencies for {} ({}): {error}",
                    stub.entry.import_path,
                    shader_ide_relative_path_string(&stub.relative_path)
                )
            })?;
        parse_shader_ide_wgsl_module(&stub.entry.import_path, &validation_source).map_err(
            |error| {
                format!(
                    "parse shader IDE stub {} ({}): {error}",
                    stub.entry.import_path,
                    shader_ide_relative_path_string(&stub.relative_path)
                )
            },
        )?;
    }
    Ok(stubs.len())
}

fn shader_ide_stub_validation_source(
    stub: &ShaderIdeStub,
    stubs: &[ShaderIdeStub],
) -> Result<String, String> {
    let mut source = stub.source.clone();
    source.push_str("\n\n// Zircon shader IDE validation defines\n");
    source.push_str(SHADER_IDE_STUB_VALIDATION_DEFINES);
    for define in &stub.validation_defines {
        source.push('\n');
        source.push_str(&shader_ide_validation_define_source(define));
    }
    let mut appended_paths = BTreeSet::new();
    for dependency in shader_ide_stub_validation_dependencies(stub, stubs)? {
        if dependency.entry.stub_path == stub.entry.stub_path
            || !appended_paths.insert(dependency.entry.stub_path.clone())
        {
            continue;
        }
        source.push_str("\n\n// Zircon shader IDE validation dependency: ");
        source.push_str(&dependency.entry.import_path);
        source.push('\n');
        source.push_str(&dependency.source);
    }
    Ok(source)
}

fn shader_ide_validation_define_source(define: &RenderShaderDefinitionValue) -> String {
    let name = define.normalized_name();
    match define {
        RenderShaderDefinitionValue::Bool { value, .. } => {
            format!("const {name}: bool = {value};")
        }
        RenderShaderDefinitionValue::Int { value, .. } => {
            format!("const {name}: i32 = {value};")
        }
        RenderShaderDefinitionValue::UInt { value, .. } => {
            format!("const {name}: u32 = {value}u;")
        }
    }
}

fn shader_ide_stub_validation_dependencies<'a>(
    stub: &'a ShaderIdeStub,
    stubs: &'a [ShaderIdeStub],
) -> Result<Vec<&'a ShaderIdeStub>, String> {
    let mut dependencies = Vec::new();
    let mut visited_paths = BTreeSet::new();
    let mut visiting = vec![stub];
    for candidate in stubs
        .iter()
        .filter(|candidate| shader_ide_common_builtin_validation_dependency(stub, candidate))
    {
        append_shader_ide_stub_validation_dependency(
            stub,
            candidate,
            stubs,
            &mut visited_paths,
            &mut visiting,
            &mut dependencies,
        )?;
    }
    if let Some(source_uri) = stub.entry.source_uri.as_ref() {
        for candidate in stubs.iter().filter(|candidate| {
            candidate.entry.stub_path != stub.entry.stub_path
                && candidate.entry.generated
                && candidate.entry.scope_uri.as_ref() == Some(source_uri)
        }) {
            append_shader_ide_stub_validation_dependency(
                stub,
                candidate,
                stubs,
                &mut visited_paths,
                &mut visiting,
                &mut dependencies,
            )?;
        }
    }
    for include_path in &stub.include_paths {
        for candidate in stubs.iter().filter(|candidate| {
            candidate.entry.import_path == *include_path
                && shader_ide_include_validation_dependency_matches_scope(stub, candidate)
        }) {
            append_shader_ide_stub_validation_dependency(
                stub,
                candidate,
                stubs,
                &mut visited_paths,
                &mut visiting,
                &mut dependencies,
            )?;
        }
    }
    Ok(dependencies)
}

fn append_shader_ide_stub_validation_dependency<'a>(
    root: &'a ShaderIdeStub,
    candidate: &'a ShaderIdeStub,
    stubs: &'a [ShaderIdeStub],
    visited_paths: &mut BTreeSet<String>,
    visiting: &mut Vec<&'a ShaderIdeStub>,
    dependencies: &mut Vec<&'a ShaderIdeStub>,
) -> Result<(), String> {
    if visited_paths.contains(&candidate.entry.stub_path) {
        return Ok(());
    }
    if let Some(cycle_start) = visiting
        .iter()
        .position(|active| active.entry.stub_path == candidate.entry.stub_path)
    {
        let mut cycle = visiting[cycle_start..]
            .iter()
            .map(|active| active.entry.import_path.as_str())
            .collect::<Vec<_>>();
        cycle.push(candidate.entry.import_path.as_str());
        return Err(format!(
            "circular shader IDE dependency: {}",
            cycle.join(" -> ")
        ));
    }
    visiting.push(candidate);
    for include_path in &candidate.include_paths {
        for transitive in stubs.iter().filter(|transitive| {
            transitive.entry.import_path == *include_path
                && shader_ide_include_validation_dependency_matches_scope(root, transitive)
        }) {
            append_shader_ide_stub_validation_dependency(
                root,
                transitive,
                stubs,
                visited_paths,
                visiting,
                dependencies,
            )?;
        }
    }
    visiting.pop();
    visited_paths.insert(candidate.entry.stub_path.clone());
    dependencies.push(candidate);
    Ok(())
}

fn shader_ide_include_validation_dependency_matches_scope(
    stub: &ShaderIdeStub,
    candidate: &ShaderIdeStub,
) -> bool {
    if !candidate.entry.generated {
        return true;
    }
    stub.entry
        .source_uri
        .as_ref()
        .is_some_and(|source_uri| candidate.entry.scope_uri.as_ref() == Some(source_uri))
}

fn shader_ide_common_builtin_validation_dependency(
    stub: &ShaderIdeStub,
    candidate: &ShaderIdeStub,
) -> bool {
    candidate.entry.stub_path != stub.entry.stub_path
        && candidate.entry.source_uri.is_none()
        && !candidate.entry.generated
        && SHADER_IDE_COMMON_BUILTIN_VALIDATION_MODULES
            .contains(&candidate.entry.import_path.as_str())
}

fn validate_shader_ide_previews(previews: &[ShaderIdePreviewFile]) -> Result<usize, String> {
    for preview in previews {
        validate_shader_ide_wgsl_module(
            &shader_ide_relative_path_string(&preview.wgsl_relative_path),
            &preview.wgsl_source,
        )
        .map_err(|error| {
            format!(
                "validate shader IDE preview {}: {error}",
                shader_ide_relative_path_string(&preview.wgsl_relative_path)
            )
        })?;
    }
    Ok(previews.len())
}

fn write_stub(output_dir: &Path, stub: &ShaderIdeStub) -> Result<bool, String> {
    let path = output_dir.join(&stub.relative_path);
    write_text_if_changed(&path, &stub.source, "shader IDE stub")
}

fn write_preview(output_dir: &Path, preview: &ShaderIdePreviewFile) -> Result<usize, String> {
    let wgsl_path = output_dir.join(&preview.wgsl_relative_path);
    let segment_path = output_dir.join(&preview.segment_relative_path);
    let segment_json = serde_json::to_string_pretty(&preview.segment_map)
        .map_err(|error| format!("encode shader IDE preview segment map: {error}"))?;
    let mut written = 0;
    written += usize::from(write_text_if_changed(
        &wgsl_path,
        &preview.wgsl_source,
        "shader IDE preview",
    )?);
    written += usize::from(write_text_if_changed(
        &segment_path,
        &segment_json,
        "shader IDE preview segments",
    )?);
    Ok(written)
}

fn shader_ide_managed_paths(
    stubs: &[ShaderIdeStub],
    previews: &[ShaderIdePreviewFile],
) -> BTreeSet<PathBuf> {
    let mut paths = stubs
        .iter()
        .map(|stub| stub.relative_path.clone())
        .collect::<BTreeSet<_>>();
    for preview in previews {
        paths.insert(preview.wgsl_relative_path.clone());
        paths.insert(preview.segment_relative_path.clone());
    }
    paths.insert(PathBuf::from(SHADER_IDE_MODULE_MAP_FILE));
    paths
}

fn write_text_if_changed(path: &Path, content: &str, label: &str) -> Result<bool, String> {
    match fs::read_to_string(path) {
        Ok(existing) if existing == content => return Ok(false),
        Ok(_) => {}
        Err(error) if error.kind() == ErrorKind::NotFound => {}
        Err(error) if error.kind() == ErrorKind::InvalidData => {}
        Err(error) => return Err(format!("read existing {label} {}: {error}", path.display())),
    }
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|error| format!("create {label} dir {}: {error}", parent.display()))?;
    }
    fs::write(path, content)
        .map_err(|error| format!("write {label} {}: {error}", path.display()))?;
    Ok(true)
}

fn remove_stale_shader_ide_files(
    output_dir: &Path,
    managed_paths: &BTreeSet<PathBuf>,
) -> Result<usize, String> {
    let mut removed = 0;
    for directory in ["modules", "generated", "preview"] {
        let relative_dir = PathBuf::from(directory);
        let absolute_dir = output_dir.join(&relative_dir);
        if absolute_dir.exists() {
            removed +=
                remove_stale_shader_ide_files_in_dir(&absolute_dir, &relative_dir, managed_paths)?;
        }
    }
    Ok(removed)
}

fn remove_stale_shader_ide_files_in_dir(
    directory: &Path,
    relative_dir: &Path,
    managed_paths: &BTreeSet<PathBuf>,
) -> Result<usize, String> {
    let mut removed = 0;
    for entry in fs::read_dir(directory).map_err(|error| {
        format!(
            "read shader IDE output dir {}: {error}",
            directory.display()
        )
    })? {
        let entry = entry.map_err(|error| {
            format!(
                "read shader IDE output entry under {}: {error}",
                directory.display()
            )
        })?;
        let path = entry.path();
        let relative_path = relative_dir.join(entry.file_name());
        let file_type = entry.file_type().map_err(|error| {
            format!(
                "read shader IDE output entry type {}: {error}",
                path.display()
            )
        })?;
        if file_type.is_dir() {
            removed += remove_stale_shader_ide_files_in_dir(&path, &relative_path, managed_paths)?;
            if fs::read_dir(&path)
                .map_err(|error| format!("read shader IDE output dir {}: {error}", path.display()))?
                .next()
                .is_none()
            {
                fs::remove_dir(&path).map_err(|error| {
                    format!(
                        "remove empty shader IDE output dir {}: {error}",
                        path.display()
                    )
                })?;
            }
        } else if file_type.is_file() && !managed_paths.contains(&relative_path) {
            fs::remove_file(&path).map_err(|error| {
                format!("remove stale shader IDE file {}: {error}", path.display())
            })?;
            removed += 1;
        }
    }
    Ok(removed)
}

#[cfg(test)]
mod tests;
