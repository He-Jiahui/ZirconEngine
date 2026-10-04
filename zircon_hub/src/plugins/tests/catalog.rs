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
