use crate::ui::template_runtime::builtin::{
    builtin_component_descriptors, builtin_template_bindings, builtin_template_documents,
};
use std::collections::{BTreeMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};
use std::time::UNIX_EPOCH;
use zircon_runtime::asset::runtime_asset_path_with_dev_asset_root;
use zircon_runtime::diagnostic_log::{
    diagnostic_log_allows, write_diagnostic_log, DiagnosticLogLevel,
};
use zircon_runtime::ui::template::UiCompiledDocument;
use zircon_runtime_interface::ui::template::{
    UiAssetDocument, UiAssetError, UiComponentDefinition,
};

use crate::ui::template::EditorTemplateRuntimeService;

use super::runtime_host::{EditorUiHostRuntime, EditorUiHostRuntimeError};

pub(super) fn load_builtin_host_templates(
    runtime: &mut EditorUiHostRuntime,
) -> Result<(), EditorUiHostRuntimeError> {
    load_builtin_host_templates_for_documents(runtime, None)
}

pub(super) fn load_builtin_host_templates_for_document_ids(
    runtime: &mut EditorUiHostRuntime,
    document_ids: &[&str],
) -> Result<(), EditorUiHostRuntimeError> {
    let document_ids = document_ids.iter().copied().collect::<HashSet<_>>();
    load_builtin_host_templates_for_documents(runtime, Some(&document_ids))
}

fn load_builtin_host_templates_for_documents(
    runtime: &mut EditorUiHostRuntime,
    document_ids: Option<&HashSet<&str>>,
) -> Result<(), EditorUiHostRuntimeError> {
    if runtime.builtin_host_templates_loaded {
        return Ok(());
    }

    for descriptor in builtin_component_descriptors()? {
        runtime.register_component(descriptor)?;
    }

    register_builtin_template_documents(runtime, document_ids)?;

    for (binding_id, binding) in builtin_template_bindings() {
        runtime.register_binding(binding_id.as_str(), binding.clone())?;
    }

    runtime.builtin_host_templates_loaded = true;
    Ok(())
}

fn register_builtin_template_documents(
    runtime: &mut EditorUiHostRuntime,
    document_ids: Option<&HashSet<&str>>,
) -> Result<(), EditorUiHostRuntimeError> {
    for (document_id, path) in builtin_template_documents() {
        if document_ids.is_some_and(|document_ids| !document_ids.contains(document_id)) {
            continue;
        }
        if editor_template_verbose_enabled() {
            write_diagnostic_log(
                "editor_builtin_templates",
                format!(
                    "register_document id={} path={} exists={}",
                    document_id,
                    path.display(),
                    path.exists()
                ),
            );
        }
        runtime.register_document_file(document_id, path)?;
    }

    Ok(())
}

pub(super) fn compile_template_document_file(
    template_service: &EditorTemplateRuntimeService,
    path: &Path,
) -> Result<UiCompiledDocument, EditorUiHostRuntimeError> {
    let cache_key = BuiltinTemplateCompileCacheKey::from_path(path);
    if let Some(compiled) = builtin_template_compile_cache()
        .lock()
        .expect("builtin template compile cache mutex should not be poisoned")
        .get(&cache_key)
        .cloned()
    {
        if editor_template_verbose_enabled() {
            write_diagnostic_log(
                "editor_template_compile_cache",
                format!(
                    "hit path={} modified_unix_ns={}",
                    cache_key.path.display(),
                    cache_key.modified_unix_ns.unwrap_or(0)
                ),
            );
        }
        return Ok(compiled);
    }

    if editor_template_verbose_enabled() {
        write_diagnostic_log(
            "editor_template_compile",
            format!(
                "load_document path={} exists={}",
                path.display(),
                path.exists()
            ),
        );
    }
    let document = load_builtin_template_document_file(template_service, path)
        .map_err(EditorUiHostRuntimeError::from)?;
    let compiled = compile_template_document_with_builtin_imports(template_service, &document)?;
    builtin_template_compile_cache()
        .lock()
        .expect("builtin template compile cache mutex should not be poisoned")
        .insert(cache_key, compiled.clone());
    Ok(compiled)
}

pub(crate) fn compile_template_document_with_builtin_imports(
    template_service: &EditorTemplateRuntimeService,
    document: &UiAssetDocument,
) -> Result<UiCompiledDocument, EditorUiHostRuntimeError> {
    let (widget_imports, style_imports) =
        collect_builtin_template_imports(template_service, document)?;
    template_service
        .compile_document_with_import_maps(document, &widget_imports, &style_imports)
        .map_err(EditorUiHostRuntimeError::from)
}

pub(crate) fn collect_builtin_template_imports(
    template_service: &EditorTemplateRuntimeService,
    document: &UiAssetDocument,
) -> Result<
    (
        BTreeMap<String, UiAssetDocument>,
        BTreeMap<String, UiAssetDocument>,
    ),
    UiAssetError,
> {
    let mut widget_imports = BTreeMap::new();
    let mut style_imports = BTreeMap::new();
    let mut seen_imports = HashSet::new();
    register_document_imports(
        template_service,
        &mut widget_imports,
        &mut style_imports,
        document,
        &mut seen_imports,
    )?;
    Ok((widget_imports, style_imports))
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct BuiltinTemplateCompileCacheKey {
    path: PathBuf,
    modified_unix_ns: Option<u128>,
    len: Option<u64>,
}

impl BuiltinTemplateCompileCacheKey {
    fn from_path(path: &Path) -> Self {
        let path = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
        let metadata = std::fs::metadata(&path).ok();
        let modified_unix_ns = metadata
            .as_ref()
            .and_then(|metadata| metadata.modified().ok())
            .and_then(|modified| modified.duration_since(UNIX_EPOCH).ok())
            .map(|duration| duration.as_nanos());
        let len = metadata.as_ref().map(std::fs::Metadata::len);
        Self {
            path,
            modified_unix_ns,
            len,
        }
    }
}

fn builtin_template_compile_cache(
) -> &'static Mutex<BTreeMap<BuiltinTemplateCompileCacheKey, UiCompiledDocument>> {
    static CACHE: OnceLock<Mutex<BTreeMap<BuiltinTemplateCompileCacheKey, UiCompiledDocument>>> =
        OnceLock::new();
    CACHE.get_or_init(|| Mutex::new(BTreeMap::new()))
}

fn load_builtin_template_document_file(
    template_service: &EditorTemplateRuntimeService,
    path: &Path,
) -> Result<UiAssetDocument, UiAssetError> {
    let cache_key = BuiltinTemplateCompileCacheKey::from_path(path);
    if let Some(document) = builtin_template_document_cache()
        .lock()
        .expect("builtin template document cache mutex should not be poisoned")
        .get(&cache_key)
        .cloned()
    {
        if editor_template_verbose_enabled() {
            write_diagnostic_log(
                "editor_template_document_cache",
                format!(
                    "hit path={} modified_unix_ns={}",
                    cache_key.path.display(),
                    cache_key.modified_unix_ns.unwrap_or(0)
                ),
            );
        }
        return Ok(document);
    }

    let document = template_service.load_document_file(path)?;
    builtin_template_document_cache()
        .lock()
        .expect("builtin template document cache mutex should not be poisoned")
        .insert(cache_key, document.clone());
    Ok(document)
}

fn editor_template_verbose_enabled() -> bool {
    diagnostic_log_allows(DiagnosticLogLevel::Verbose)
}

fn builtin_template_document_cache(
) -> &'static Mutex<BTreeMap<BuiltinTemplateCompileCacheKey, UiAssetDocument>> {
    static CACHE: OnceLock<Mutex<BTreeMap<BuiltinTemplateCompileCacheKey, UiAssetDocument>>> =
        OnceLock::new();
    CACHE.get_or_init(|| Mutex::new(BTreeMap::new()))
}

fn register_document_imports(
    template_service: &EditorTemplateRuntimeService,
    widget_imports: &mut BTreeMap<String, UiAssetDocument>,
    style_imports: &mut BTreeMap<String, UiAssetDocument>,
    document: &UiAssetDocument,
    seen_imports: &mut HashSet<String>,
) -> Result<(), UiAssetError> {
    for reference in &document.imports.widgets {
        if !admit_import_reference(seen_imports, reference) {
            continue;
        }
        let Some(imported) = resolve_builtin_import(template_service, reference)? else {
            continue;
        };
        widget_imports.insert(reference.clone(), imported.clone());
        if !reference.contains('#') {
            for component_name in imported.components.keys() {
                widget_imports.insert(format!("{reference}#{component_name}"), imported.clone());
            }
            for alias in root_component_aliases(&imported) {
                widget_imports.insert(
                    format!("{reference}#{alias}"),
                    document_with_root_component_alias(imported.clone(), alias),
                );
            }
        }
        register_document_imports(
            template_service,
            widget_imports,
            style_imports,
            &imported,
            seen_imports,
        )?;
    }

    for reference in &document.imports.styles {
        if !admit_import_reference(seen_imports, reference) {
            continue;
        }
        let Some(imported) = resolve_builtin_import(template_service, reference)? else {
            continue;
        };
        style_imports.insert(reference.clone(), imported.clone());
    }

    Ok(())
}

fn admit_import_reference(seen_imports: &mut HashSet<String>, reference: &str) -> bool {
    if seen_imports.contains(reference) {
        return false;
    }
    seen_imports.insert(reference.to_owned());
    true
}

fn root_component_aliases(document: &UiAssetDocument) -> Vec<String> {
    let Some(root) = &document.root else {
        return Vec::new();
    };
    [root.control_id.as_ref(), Some(&root.node_id)]
        .into_iter()
        .flatten()
        .filter(|alias| !alias.is_empty() && !document.components.contains_key(alias.as_str()))
        .cloned()
        .collect()
}

fn document_with_root_component_alias(
    mut document: UiAssetDocument,
    alias: String,
) -> UiAssetDocument {
    let Some(root) = document.root.clone() else {
        return document;
    };
    document.components.insert(
        alias,
        UiComponentDefinition {
            root,
            ..Default::default()
        },
    );
    document
}

fn resolve_builtin_import(
    template_service: &EditorTemplateRuntimeService,
    reference: &str,
) -> Result<Option<UiAssetDocument>, UiAssetError> {
    let Some(path) = reference
        .strip_prefix("res://")
        .and_then(|value| value.split('#').next())
    else {
        return Ok(None);
    };
    let source_path = editor_runtime_asset_path(path);
    if editor_template_verbose_enabled() {
        write_diagnostic_log(
            "editor_template_import",
            format!(
                "reference={} resolved_path={} exists={}",
                reference,
                source_path.display(),
                source_path.exists()
            ),
        );
    }
    if !source_path.exists() {
        return Ok(None);
    }
    Ok(Some(load_builtin_template_document_file(
        template_service,
        &source_path,
    )?))
}

fn editor_runtime_asset_path(relative: &str) -> std::path::PathBuf {
    runtime_asset_path_with_dev_asset_root(relative, editor_dev_asset_root())
}

fn editor_dev_asset_root() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets")
}

#[cfg(test)]
mod tests {
    use std::collections::{BTreeSet, HashSet};
    use std::hint::black_box;
    use std::time::{Duration, Instant};

    use super::super::runtime_host::{
        clear_v2_template_file_cache_for_tests, v2_template_file_cache_len_for_tests,
    };
    use super::*;

    const IMPORT_ADMISSION_COUNT: usize = 65_536;
    const UNIQUE_IMPORT_COUNT: usize = 8_192;
    const SAMPLE_COUNT: usize = 17;

    fn percentile_95(samples: &mut [Duration]) -> Duration {
        samples.sort_unstable();
        samples[(samples.len() - 1) * 95 / 100]
    }

    fn import_references() -> Vec<String> {
        (0..IMPORT_ADMISSION_COUNT)
            .map(|index| {
                format!(
                    "res://ui/imports/{:04}.zui",
                    (index * 4_099) % UNIQUE_IMPORT_COUNT
                )
            })
            .collect()
    }

    fn legacy_import_admission_count(references: &[String]) -> usize {
        let mut seen = BTreeSet::new();
        references
            .iter()
            .filter(|reference| seen.insert((*reference).clone()))
            .count()
    }

    fn optimized_import_admission_count(references: &[String]) -> usize {
        let mut seen = HashSet::new();
        references
            .iter()
            .filter(|reference| admit_import_reference(&mut seen, reference))
            .count()
    }

    #[test]
    fn builtin_v2_template_file_cache_is_reused_across_runtime_instances() {
        clear_v2_template_file_cache_for_tests();
        builtin_template_compile_cache()
            .lock()
            .expect("tree-template compile cache mutex should not be poisoned")
            .clear();
        builtin_template_document_cache()
            .lock()
            .expect("tree-template document cache mutex should not be poisoned")
            .clear();

        let mut first = EditorUiHostRuntime::default();
        first
            .load_builtin_host_templates()
            .expect("first runtime should load builtin templates");
        let v2_entries_after_first = v2_template_file_cache_len_for_tests();

        let mut second = EditorUiHostRuntime::default();
        second
            .load_builtin_host_templates()
            .expect("second runtime should reuse builtin template cache");

        assert!(v2_entries_after_first > 0);
        assert_eq!(
            v2_template_file_cache_len_for_tests(),
            v2_entries_after_first,
            "second runtime should not reload or recompile additional v2 builtin documents"
        );
        assert_eq!(
            builtin_template_compile_cache()
                .lock()
                .expect("tree-template compile cache mutex should not be poisoned")
                .len(),
            0,
            "v2 builtin host templates should bypass the tree-template compiler cache"
        );
        assert_eq!(
            builtin_template_document_cache()
                .lock()
                .expect("tree-template document cache mutex should not be poisoned")
                .len(),
            0,
            "v2 builtin host templates should bypass the tree-template document cache"
        );
    }

    #[test]
    fn optimization_batch_20260826r_editor01_hash_membership_preserves_first_import_admission() {
        let mut seen = HashSet::new();

        assert!(admit_import_reference(&mut seen, "res://ui/shared.zui"));
        assert!(!admit_import_reference(&mut seen, "res://ui/shared.zui"));
        assert!(admit_import_reference(&mut seen, "res://ui/other.zui"));
        assert_eq!(seen.len(), 2);
    }

    #[test]
    fn optimization_batch_20260826r_editor01_builtin_templates_use_hash_membership() {
        let source = include_str!("build_session.rs");
        let production = source.split("#[cfg(test)]").next().unwrap();

        assert!(production.contains("use std::collections::{BTreeMap, HashSet};"));
        assert!(production.contains("collect::<HashSet<_>>()"));
        assert_eq!(production.matches("Option<&HashSet<&str>>").count(), 2);
        assert_eq!(production.matches("&mut HashSet<String>").count(), 2);
        assert!(!production.contains("BTreeSet"));
        assert!(
            production.find("seen_imports.contains(reference)").unwrap()
                < production
                    .find("seen_imports.insert(reference.to_owned())")
                    .unwrap()
        );
    }

    #[test]
    #[ignore = "release performance evidence"]
    fn optimization_batch_20260826r_editor01_builtin_template_hash_membership_performance_evidence()
    {
        let references = import_references();
        assert_eq!(
            legacy_import_admission_count(&references),
            UNIQUE_IMPORT_COUNT
        );
        assert_eq!(
            optimized_import_admission_count(&references),
            UNIQUE_IMPORT_COUNT
        );

        let mut legacy_samples = Vec::with_capacity(SAMPLE_COUNT);
        let mut optimized_samples = Vec::with_capacity(SAMPLE_COUNT);
        for sample in 0..SAMPLE_COUNT {
            if sample % 2 == 0 {
                let started = Instant::now();
                black_box(legacy_import_admission_count(black_box(&references)));
                legacy_samples.push(started.elapsed());

                let started = Instant::now();
                black_box(optimized_import_admission_count(black_box(&references)));
                optimized_samples.push(started.elapsed());
            } else {
                let started = Instant::now();
                black_box(optimized_import_admission_count(black_box(&references)));
                optimized_samples.push(started.elapsed());

                let started = Instant::now();
                black_box(legacy_import_admission_count(black_box(&references)));
                legacy_samples.push(started.elapsed());
            }
        }

        let legacy_p95 = percentile_95(&mut legacy_samples);
        let optimized_p95 = percentile_95(&mut optimized_samples);
        println!(
            "EDITOR01_BUILTIN_TEMPLATE_HASH_MEMBERSHIP_BENCH_V1 admissions={IMPORT_ADMISSION_COUNT} \
             unique_imports={UNIQUE_IMPORT_COUNT} legacy_string_allocations={IMPORT_ADMISSION_COUNT} \
             optimized_string_allocations={UNIQUE_IMPORT_COUNT} legacy_p95_ns={} optimized_p95_ns={}",
            legacy_p95.as_nanos(),
            optimized_p95.as_nanos(),
        );
        assert!(
            optimized_p95.as_nanos() * 100 <= legacy_p95.as_nanos() * 60,
            "hash-membership P95 {:?} exceeded 60% of tree-membership P95 {:?}",
            optimized_p95,
            legacy_p95,
        );
    }
}
