use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

use serde::Deserialize;

use crate::error::HubError;
use crate::projects::project_filesystem_path_key;

const PLUGINS_DIR: &str = "zircon_plugins";
const PLUGIN_MANIFEST_FILE: &str = "plugin.toml";
const PROJECT_PLUGIN_DIRS: &[&str] = &["Plugins", "plugins"];
const SKIPPED_DIRECTORIES: &[&str] = &[".git", "target"];
const EDITOR_CAPABILITY_PREFIX: &[u8] = b"editor.";
pub const PROJECT_PLUGIN_SCOPE: &str = "Project";
pub const ENGINE_PLUGIN_SCOPE: &str = "Engine";

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PluginCatalogEntry {
    pub id: String,
    pub display_name: String,
    pub description: String,
    pub category: String,
    pub maturity: String,
    pub editor_scoped: bool,
    pub default_packaging: Vec<String>,
    pub module_count: usize,
    pub scope: String,
    pub package_root: PathBuf,
    pub manifest_path: PathBuf,
}

#[derive(Debug, Deserialize)]
struct PluginManifest {
    id: Option<String>,
    display_name: Option<String>,
    description: Option<String>,
    category: Option<String>,
    maturity: Option<String>,
    #[serde(default)]
    supported_targets: Vec<String>,
    #[serde(default)]
    capabilities: Vec<String>,
    #[serde(default)]
    default_packaging: Vec<String>,
    #[serde(default)]
    modules: Vec<PluginManifestModule>,
}

#[derive(Debug, Deserialize)]
struct PluginManifestModule {
    name: Option<String>,
    kind: Option<String>,
    #[serde(default)]
    target_modes: Vec<String>,
    #[serde(default)]
    capabilities: Vec<String>,
}

pub fn discover_plugin_catalog<I>(repo_roots: I) -> Result<Vec<PluginCatalogEntry>, HubError>
where
    I: IntoIterator<Item = PathBuf>,
{
    discover_plugin_catalog_with_project_roots(Vec::<PathBuf>::new(), repo_roots)
}

pub fn discover_plugin_catalog_with_project_roots<P, R>(
    project_roots: P,
    repo_roots: R,
) -> Result<Vec<PluginCatalogEntry>, HubError>
where
    P: IntoIterator<Item = PathBuf>,
    R: IntoIterator<Item = PathBuf>,
{
    let mut entries = Vec::new();
    let mut visited_manifests = HashSet::new();

    for project_root in project_roots {
        if project_root.is_dir() {
            collect_project_plugin_manifests(&project_root, &mut visited_manifests, &mut entries)?;
        }
    }

    for repo_root in repo_roots {
        let plugins_root = repo_root.join(PLUGINS_DIR);
        if !plugins_root.is_dir() {
            continue;
        }
        collect_plugin_manifests(
            &plugins_root,
            ENGINE_PLUGIN_SCOPE,
            &mut visited_manifests,
            &mut entries,
        )?;
        break;
    }
    entries.sort_by(|left, right| {
        scope_rank(&left.scope)
            .cmp(&scope_rank(&right.scope))
            .then_with(|| left.id.cmp(&right.id))
            .then_with(|| left.package_root.cmp(&right.package_root))
    });
    Ok(entries)
}

fn scope_rank(scope: &str) -> usize {
    match scope {
        PROJECT_PLUGIN_SCOPE => 0,
        ENGINE_PLUGIN_SCOPE => 1,
        _ => 2,
    }
}

fn collect_project_plugin_manifests(
    project_root: &Path,
    visited_manifests: &mut HashSet<String>,
    entries: &mut Vec<PluginCatalogEntry>,
) -> Result<(), HubError> {
    let manifest_path = project_root.join(PLUGIN_MANIFEST_FILE);
    if manifest_path.is_file() {
        let manifest_key = project_filesystem_path_key(&manifest_path);
        if visited_manifests.insert(manifest_key) {
            entries.push(read_plugin_manifest(&manifest_path, PROJECT_PLUGIN_SCOPE)?);
        }
    }
    for plugin_dir in PROJECT_PLUGIN_DIRS {
        let root = project_root.join(plugin_dir);
        if root.is_dir() {
            collect_plugin_manifests(&root, PROJECT_PLUGIN_SCOPE, visited_manifests, entries)?;
        }
    }
    Ok(())
}

fn collect_plugin_manifests(
    directory: &Path,
    scope: &str,
    visited_manifests: &mut HashSet<String>,
    entries: &mut Vec<PluginCatalogEntry>,
) -> Result<(), HubError> {
    for entry in fs::read_dir(directory)? {
        let entry = entry?;
        let path = entry.path();
        let file_type = entry.file_type()?;
        if file_type.is_dir() {
            if should_skip_directory(&entry.file_name().to_string_lossy()) {
                continue;
            }
            collect_plugin_manifests(&path, scope, visited_manifests, entries)?;
        } else if file_type.is_file()
            && path.file_name().and_then(|name| name.to_str()) == Some(PLUGIN_MANIFEST_FILE)
        {
            let manifest_key = project_filesystem_path_key(&path);
            if visited_manifests.insert(manifest_key) {
                entries.push(read_plugin_manifest(&path, scope)?);
            }
        }
    }
    Ok(())
}

fn read_plugin_manifest(manifest_path: &Path, scope: &str) -> Result<PluginCatalogEntry, HubError> {
    let text = fs::read_to_string(manifest_path)?;
    let manifest = toml::from_str::<PluginManifest>(&text)?;
    let package_root = manifest_path
        .parent()
        .unwrap_or(Path::new(""))
        .to_path_buf();
    let editor_scoped = plugin_manifest_is_editor_scoped(&manifest);
    let id = non_empty_or_else(manifest.id, || {
        package_root
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("plugin")
            .to_string()
    });
    let display_name = non_empty_or_else(manifest.display_name, || id.clone());
    Ok(PluginCatalogEntry {
        id,
        display_name,
        description: manifest.description.unwrap_or_default(),
        category: manifest
            .category
            .unwrap_or_else(|| "uncategorized".to_string()),
        maturity: manifest.maturity.unwrap_or_else(|| "unknown".to_string()),
        editor_scoped,
        default_packaging: manifest.default_packaging,
        module_count: manifest
            .modules
            .iter()
            .filter(|module| {
                module
                    .name
                    .as_deref()
                    .is_some_and(|name| !name.trim().is_empty())
            })
            .count(),
        scope: scope.to_string(),
        package_root,
        manifest_path: manifest_path.to_path_buf(),
    })
}

fn plugin_manifest_is_editor_scoped(manifest: &PluginManifest) -> bool {
    manifest
        .supported_targets
        .iter()
        .any(|target| is_editor_target(target))
        || manifest
            .capabilities
            .iter()
            .any(|capability| is_editor_capability(capability))
        || manifest.modules.iter().any(|module| {
            module
                .kind
                .as_deref()
                .is_some_and(|kind| kind.eq_ignore_ascii_case("editor"))
                || module
                    .target_modes
                    .iter()
                    .any(|mode| is_editor_target(mode))
                || module
                    .capabilities
                    .iter()
                    .any(|capability| is_editor_capability(capability))
        })
}

fn is_editor_target(value: &str) -> bool {
    let value = value.trim();
    value.eq_ignore_ascii_case("editor") || value.eq_ignore_ascii_case("editor_host")
}

fn is_editor_capability(value: &str) -> bool {
    value
        .trim()
        .as_bytes()
        .get(..EDITOR_CAPABILITY_PREFIX.len())
        .is_some_and(|prefix| prefix.eq_ignore_ascii_case(EDITOR_CAPABILITY_PREFIX))
}

fn should_skip_directory(name: &str) -> bool {
    SKIPPED_DIRECTORIES
        .iter()
        .any(|skipped| skipped.eq_ignore_ascii_case(name))
}

fn non_empty_or_else(value: Option<String>, fallback: impl FnOnce() -> String) -> String {
    match value {
        Some(value) => {
            let trimmed = value.trim();
            if trimmed.is_empty() {
                fallback()
            } else if trimmed.len() == value.len() {
                value
            } else {
                trimmed.to_owned()
            }
        }
        None => fallback(),
    }
}

#[cfg(test)]
mod tests {
    use std::alloc::{GlobalAlloc, Layout, System};
    use std::hint::black_box;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::time::{Instant, SystemTime, UNIX_EPOCH};

    use super::*;

    struct CountingAllocator;

    static ALLOCATIONS: AtomicUsize = AtomicUsize::new(0);
    static ALLOCATED_BYTES: AtomicUsize = AtomicUsize::new(0);

    #[global_allocator]
    static COUNTING_ALLOCATOR: CountingAllocator = CountingAllocator;

    unsafe impl GlobalAlloc for CountingAllocator {
        unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
            ALLOCATIONS.fetch_add(1, Ordering::Relaxed);
            ALLOCATED_BYTES.fetch_add(layout.size(), Ordering::Relaxed);
            unsafe { System.alloc(layout) }
        }

        unsafe fn dealloc(&self, pointer: *mut u8, layout: Layout) {
            unsafe { System.dealloc(pointer, layout) }
        }

        unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
            ALLOCATIONS.fetch_add(1, Ordering::Relaxed);
            ALLOCATED_BYTES.fetch_add(layout.size(), Ordering::Relaxed);
            unsafe { System.alloc_zeroed(layout) }
        }

        unsafe fn realloc(&self, pointer: *mut u8, layout: Layout, size: usize) -> *mut u8 {
            ALLOCATIONS.fetch_add(1, Ordering::Relaxed);
            ALLOCATED_BYTES.fetch_add(size, Ordering::Relaxed);
            unsafe { System.realloc(pointer, layout, size) }
        }
    }

    #[test]
    fn hub03_discover_plugin_catalog_reads_manifest_metadata() {
        let repo_root = temp_repo_root("catalog");
        let plugin_root = repo_root.join(PLUGINS_DIR).join("demo");
        fs::create_dir_all(&plugin_root).unwrap();
        fs::write(
            plugin_root.join(PLUGIN_MANIFEST_FILE),
            r#"
id = "demo"
display_name = "Demo Plugin"
description = "Demo plugin description."
category = "runtime"
maturity = "beta"
default_packaging = ["native_dynamic", "library_embed"]

[[modules]]
name = "demo.runtime"

[[modules]]
name = "demo.editor"
"#,
        )
        .unwrap();

        let entries = discover_plugin_catalog([repo_root.clone()]).unwrap();
        fs::remove_dir_all(repo_root).unwrap();

        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].id, "demo");
        assert_eq!(entries[0].scope, ENGINE_PLUGIN_SCOPE);
        assert_eq!(entries[0].display_name, "Demo Plugin");
        assert_eq!(
            entries[0].default_packaging,
            vec!["native_dynamic".to_string(), "library_embed".to_string()]
        );
        assert_eq!(entries[0].module_count, 2);
    }

    #[test]
    fn hub03_editor_scoped_manifest_does_not_depend_on_description_copy() {
        let repo_root = temp_repo_root("catalog-editor-scope");
        let editor_plugin_root = repo_root.join(PLUGINS_DIR).join("authoring");
        let runtime_plugin_root = repo_root.join(PLUGINS_DIR).join("runtime");
        fs::create_dir_all(&editor_plugin_root).unwrap();
        fs::create_dir_all(&runtime_plugin_root).unwrap();
        fs::write(
            editor_plugin_root.join(PLUGIN_MANIFEST_FILE),
            r#"
id = "authoring"
display_name = "Authoring"
description = "Graph tools."
supported_targets = ["editor_host"]
capabilities = ["editor.extension.graph"]

[[modules]]
name = "authoring.tools"
kind = "editor"
target_modes = ["editor_host"]
capabilities = ["editor.extension.graph"]
"#,
        )
        .unwrap();
        fs::write(
            runtime_plugin_root.join(PLUGIN_MANIFEST_FILE),
            r#"
id = "runtime"
display_name = "Runtime"
description = "Runtime tools."
supported_targets = ["client_runtime"]

[[modules]]
name = "runtime.core"
target_modes = ["client_runtime"]
capabilities = ["runtime.plugin.core"]
"#,
        )
        .unwrap();

        let entries = discover_plugin_catalog([repo_root.clone()]).unwrap();
        fs::remove_dir_all(repo_root).unwrap();

        assert!(entries
            .iter()
            .any(|entry| entry.id == "authoring" && entry.editor_scoped));
        assert!(entries
            .iter()
            .any(|entry| entry.id == "runtime" && !entry.editor_scoped));
    }

    #[test]
    fn hub03_editor_scope_classifiers_reuse_canonical_manifest_strings() {
        let id = String::from("demo");
        let original_id_pointer = id.as_ptr();
        let mut fallback_calls = 0;
        let normalized_id = non_empty_or_else(Some(id), || {
            fallback_calls += 1;
            String::from("fallback")
        });

        assert_eq!(normalized_id, "demo");
        assert_eq!(normalized_id.as_ptr(), original_id_pointer);
        assert_eq!(fallback_calls, 0);
        assert_eq!(
            non_empty_or_else(Some(String::from("  trimmed  ")), || {
                String::from("fallback")
            }),
            "trimmed"
        );
        assert!(is_editor_target(" EDITOR_HOST "));
        assert!(is_editor_capability(" Editor.Extension.Graph "));
        assert!(!is_editor_target("runtime"));
        assert!(!is_editor_capability("runtime.editor.extension"));
    }

    #[test]
    fn hub03_discover_plugin_catalog_falls_back_to_next_root() {
        let missing_root = temp_repo_root("catalog-missing");
        let repo_root = temp_repo_root("catalog-fallback");
        let plugin_root = repo_root.join(PLUGINS_DIR).join("fallback");
        fs::create_dir_all(&plugin_root).unwrap();
        fs::write(plugin_root.join(PLUGIN_MANIFEST_FILE), "id = \"fallback\"").unwrap();

        let entries = discover_plugin_catalog([missing_root.clone(), repo_root.clone()]).unwrap();
        fs::remove_dir_all(missing_root).unwrap();
        fs::remove_dir_all(repo_root).unwrap();

        assert_eq!(entries[0].id, "fallback");
        assert_eq!(entries[0].scope, ENGINE_PLUGIN_SCOPE);
    }

    #[test]
    fn hub03_discover_plugin_catalog_reads_project_and_engine_scopes() {
        let project_root = temp_repo_root("catalog-project-scope");
        let repo_root = temp_repo_root("catalog-engine-scope");
        let project_plugin_root = project_root.join("Plugins").join("project_runtime");
        let engine_plugin_root = repo_root.join(PLUGINS_DIR).join("engine_runtime");
        fs::create_dir_all(&project_plugin_root).unwrap();
        fs::create_dir_all(&engine_plugin_root).unwrap();
        fs::write(
            project_plugin_root.join(PLUGIN_MANIFEST_FILE),
            "id = \"project_runtime\"\ndisplay_name = \"Project Runtime\"",
        )
        .unwrap();
        fs::write(
            engine_plugin_root.join(PLUGIN_MANIFEST_FILE),
            "id = \"engine_runtime\"\ndisplay_name = \"Engine Runtime\"",
        )
        .unwrap();

        let entries =
            discover_plugin_catalog_with_project_roots([project_root.clone()], [repo_root.clone()])
                .unwrap();
        fs::remove_dir_all(project_root).unwrap();
        fs::remove_dir_all(repo_root).unwrap();

        assert!(entries
            .iter()
            .any(|entry| entry.id == "project_runtime" && entry.scope == PROJECT_PLUGIN_SCOPE));
        assert!(entries
            .iter()
            .any(|entry| entry.id == "engine_runtime" && entry.scope == ENGINE_PLUGIN_SCOPE));
    }

    #[derive(Clone)]
    struct BenchmarkManifest {
        id: String,
        display_name: String,
    }

    #[derive(Clone, Copy)]
    struct Measurement {
        elapsed_ns: u128,
        allocations: usize,
        allocated_bytes: usize,
        checksum: usize,
    }

    #[test]
    #[ignore = "release-only plugin scope matching benchmark"]
    fn hub03_plugin_scope_matching_release_benchmark_evidence() {
        const MANIFESTS: usize = 32_768;
        const SAMPLE_PAIRS: usize = 21;

        let source = benchmark_manifests(MANIFESTS);
        for _ in 0..4 {
            black_box(measure_legacy(&source));
            black_box(measure_optimized(&source));
        }

        let mut legacy = Vec::with_capacity(SAMPLE_PAIRS);
        let mut optimized = Vec::with_capacity(SAMPLE_PAIRS);
        for pair in 0..SAMPLE_PAIRS {
            if pair % 2 == 0 {
                legacy.push(measure_legacy(&source));
                optimized.push(measure_optimized(&source));
            } else {
                optimized.push(measure_optimized(&source));
                legacy.push(measure_legacy(&source));
            }
        }

        let checksum = legacy[0].checksum;
        assert!(legacy.iter().all(|sample| sample.checksum == checksum));
        assert!(optimized.iter().all(|sample| sample.checksum == checksum));

        let legacy_allocations = legacy[0].allocations;
        let optimized_allocations = optimized[0].allocations;
        assert!(legacy
            .iter()
            .all(|sample| sample.allocations == legacy_allocations));
        assert!(optimized
            .iter()
            .all(|sample| sample.allocations == optimized_allocations));
        assert_eq!(legacy_allocations, MANIFESTS * 10);
        assert_eq!(optimized_allocations, 0);

        let legacy_ns = legacy
            .iter()
            .map(|sample| sample.elapsed_ns)
            .collect::<Vec<_>>();
        let optimized_ns = optimized
            .iter()
            .map(|sample| sample.elapsed_ns)
            .collect::<Vec<_>>();
        let legacy_p50_ns = percentile(&legacy_ns, 50);
        let optimized_p50_ns = percentile(&optimized_ns, 50);
        let legacy_p95_ns = percentile(&legacy_ns, 95);
        let optimized_p95_ns = percentile(&optimized_ns, 95);

        println!(
            "HUB03_PLUGIN_SCOPE_MATCHING_BENCH_V1 manifests={MANIFESTS} \
classifier_calls_per_manifest=6 sample_pairs={SAMPLE_PAIRS} checksum={checksum} \
legacy_allocations={legacy_allocations} optimized_allocations={optimized_allocations} \
legacy_allocated_bytes={} optimized_allocated_bytes={} \
legacy_p50_ns={legacy_p50_ns} optimized_p50_ns={optimized_p50_ns} \
legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} \
legacy_raw_ns={} optimized_raw_ns={}",
            legacy[0].allocated_bytes,
            optimized[0].allocated_bytes,
            raw(&legacy_ns),
            raw(&optimized_ns),
        );

        assert!(
            optimized_p50_ns.saturating_mul(100) <= legacy_p50_ns.saturating_mul(25),
            "borrowed plugin scope matching must improve P50 by at least 75%: \
legacy={legacy_p50_ns}ns optimized={optimized_p50_ns}ns"
        );
        assert!(
            optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(25),
            "borrowed plugin scope matching must improve P95 by at least 75%: \
legacy={legacy_p95_ns}ns optimized={optimized_p95_ns}ns"
        );
    }

    fn benchmark_manifests(count: usize) -> Vec<BenchmarkManifest> {
        (0..count)
            .map(|index| BenchmarkManifest {
                id: format!("plugin-{index:05}"),
                display_name: format!("Plugin {index:05}"),
            })
            .collect()
    }

    fn measure_legacy(source: &[BenchmarkManifest]) -> Measurement {
        let inputs = source.to_vec();
        reset_allocation_counters();
        let started = Instant::now();
        let checksum = run_legacy(inputs);
        let elapsed_ns = started.elapsed().as_nanos().max(1);
        Measurement {
            elapsed_ns,
            allocations: ALLOCATIONS.load(Ordering::Relaxed),
            allocated_bytes: ALLOCATED_BYTES.load(Ordering::Relaxed),
            checksum,
        }
    }

    fn measure_optimized(source: &[BenchmarkManifest]) -> Measurement {
        let inputs = source.to_vec();
        reset_allocation_counters();
        let started = Instant::now();
        let checksum = run_optimized(inputs);
        let elapsed_ns = started.elapsed().as_nanos().max(1);
        Measurement {
            elapsed_ns,
            allocations: ALLOCATIONS.load(Ordering::Relaxed),
            allocated_bytes: ALLOCATED_BYTES.load(Ordering::Relaxed),
            checksum,
        }
    }

    fn run_legacy(inputs: Vec<BenchmarkManifest>) -> usize {
        let mut checksum = 0usize;
        for (index, input) in inputs.into_iter().enumerate() {
            for value in ["editor", " EDITOR_HOST ", "runtime"] {
                checksum = checksum.wrapping_add(legacy_is_editor_target(value) as usize);
            }
            for value in [
                "editor.extension.graph",
                " Editor.Window ",
                "runtime.plugin.core",
            ] {
                checksum = checksum.wrapping_add(legacy_is_editor_capability(value) as usize);
            }

            let fallback_id = black_box(format!("plugin-{index:05}"));
            let id = legacy_non_empty_or(Some(input.id), fallback_id);
            let fallback_display = black_box(id.clone());
            let display_name = legacy_non_empty_or(Some(input.display_name), fallback_display);
            checksum = checksum
                .wrapping_mul(31)
                .wrapping_add(black_box(id.len()))
                .wrapping_add(black_box(display_name.len()));
        }
        checksum
    }

    fn run_optimized(inputs: Vec<BenchmarkManifest>) -> usize {
        let mut checksum = 0usize;
        for input in inputs {
            for value in ["editor", " EDITOR_HOST ", "runtime"] {
                checksum = checksum.wrapping_add(is_editor_target(value) as usize);
            }
            for value in [
                "editor.extension.graph",
                " Editor.Window ",
                "runtime.plugin.core",
            ] {
                checksum = checksum.wrapping_add(is_editor_capability(value) as usize);
            }

            let id = non_empty_or_else(Some(input.id), || String::from("fallback"));
            let display_name = non_empty_or_else(Some(input.display_name), || id.clone());
            checksum = checksum
                .wrapping_mul(31)
                .wrapping_add(black_box(id.len()))
                .wrapping_add(black_box(display_name.len()));
        }
        checksum
    }

    fn legacy_is_editor_target(value: &str) -> bool {
        let normalized = black_box(value.trim().to_ascii_lowercase());
        normalized == "editor" || normalized == "editor_host"
    }

    fn legacy_is_editor_capability(value: &str) -> bool {
        let normalized = black_box(value.trim().to_ascii_lowercase());
        normalized.starts_with("editor.")
    }

    fn legacy_non_empty_or(value: Option<String>, fallback: String) -> String {
        value
            .map(|value| black_box(value.trim().to_string()))
            .filter(|value| !value.is_empty())
            .unwrap_or(fallback)
    }

    fn reset_allocation_counters() {
        ALLOCATIONS.store(0, Ordering::Relaxed);
        ALLOCATED_BYTES.store(0, Ordering::Relaxed);
    }

    fn percentile(samples: &[u128], percentile: usize) -> u128 {
        let mut sorted = samples.to_vec();
        sorted.sort_unstable();
        let rank = (sorted.len() * percentile).div_ceil(100);
        sorted[rank.saturating_sub(1)]
    }

    fn raw(samples: &[u128]) -> String {
        samples
            .iter()
            .map(u128::to_string)
            .collect::<Vec<_>>()
            .join(",")
    }

    fn temp_repo_root(label: &str) -> PathBuf {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis();
        let root = std::env::temp_dir().join(format!("zircon-hub-{label}-{now}"));
        fs::create_dir_all(&root).unwrap();
        root
    }
}
