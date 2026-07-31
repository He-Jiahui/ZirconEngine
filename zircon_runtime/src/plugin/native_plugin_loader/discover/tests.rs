use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::thread;
use std::time::{SystemTime, UNIX_EPOCH};

use super::super::collect_manifests::{collect_plugin_manifests, MAX_DISCOVERY_DEPTH};
use super::super::NativePluginLoader;

#[test]
fn cold_discovery_publishes_the_authority_snapshot() {
    let root = TempDiscoveryRoot::new("cold-publication");
    root.write_manifest("weather", "weather");

    let report = NativePluginLoader.discover(root.path());

    assert_eq!(plugin_ids(&report), vec!["weather"]);
    assert_eq!(
        NativePluginLoader.discovery_generation(root.path()),
        Some(1)
    );
}

#[test]
fn unchanged_discovery_projects_the_published_generation_without_a_second_scanner() {
    let root = TempDiscoveryRoot::new("unchanged-publication");
    root.write_manifest("weather", "weather");

    let first = NativePluginLoader.discover(root.path());
    let first_generation = NativePluginLoader
        .discovery_generation(root.path())
        .expect("published generation");
    let second = NativePluginLoader.discover(root.path());

    assert_eq!(plugin_ids(&first), plugin_ids(&second));
    assert_eq!(
        NativePluginLoader.discovery_generation(root.path()),
        Some(first_generation)
    );

    let authority = include_str!("authority.rs");
    assert!(!authority.contains("DiscoveryRootState"));
    assert!(!authority.contains("collect_plugin_manifests"));
    assert!(authority.contains("in_flight"));
}

#[test]
fn forced_refresh_submits_once_then_reuses_a_superseding_winner() {
    let authority = include_str!("authority.rs");

    assert!(authority.contains("let mut force_refresh = force_refresh;"));
    assert!(authority.contains("force_refresh = false;"));
    assert!(authority.contains("Reuse that winner instead of submitting another"));
}

#[test]
fn terminal_tickets_are_cleared_and_collector_workers_never_wait() {
    let authority = include_str!("authority.rs");

    assert!(authority.contains("fn clear_terminal_ticket"));
    assert!(authority.contains("ticket.wait_terminal()"));
    assert!(authority.contains("is_native_plugin_discovery_io_lane()"));
    assert!(!authority.contains("thread::yield_now"));
}

#[test]
fn authority_caches_root_identities_before_any_blocking_discovery_work() {
    let authority = include_str!("authority.rs");

    assert!(authority.contains("root_identities: Mutex<BTreeMap<PathBuf"));
    assert!(authority.contains("const MAX_ROOT_IDENTITIES: usize = 32"));
    assert!(authority.contains("fn cached_root_identity"));
    let lane_branch = authority
        .find("if is_native_plugin_discovery_io_lane()")
        .expect("collector lane guard");
    let root_identity = authority
        .find("let root = self.root_identity(path);")
        .expect("root cache lookup");
    assert!(
        lane_branch < root_identity,
        "collector-lane re-entry must not canonicalize an uncached root"
    );
}

#[test]
fn relative_root_cache_keys_bind_to_the_current_working_directory() {
    let authority = include_str!("authority.rs");
    let cache_key = authority
        .split_once("fn lexical_root_path")
        .expect("root cache key helper")
        .1
        .split_once("fn lock_recover")
        .expect("root cache key helper boundary")
        .0;

    assert!(cache_key.contains("std::env::current_dir()"));
    assert!(cache_key.contains("current.join(path)"));
}

#[test]
fn generation_projection_requires_a_cached_root_identity() {
    let authority = include_str!("authority.rs");
    let generation = authority
        .split_once("pub(super) fn generation")
        .expect("generation projection")
        .1
        .split_once("fn project_root")
        .expect("generation projection boundary")
        .0;

    assert!(generation.contains("self.cached_root_identity(root)?"));
    assert!(!generation.contains("self.root_identity(root)"));
}

#[test]
fn deadline_terminal_is_reported_without_waiting_for_worker_retirement() {
    let authority = include_str!("authority.rs");

    assert!(authority.contains("self.failure_report(&root, ticket.generation(), terminal)"));
    assert!(authority.contains("NativePluginDiscoveryRefreshTerminal::DeadlineExceeded"));
    assert!(authority.contains("exceeded its deadline before publication"));
}

#[test]
fn manifest_notifications_refresh_the_same_authority_generation() {
    let root = TempDiscoveryRoot::new("notification-refresh");
    let weather = root.write_manifest("weather", "weather");
    let first = NativePluginLoader.discover(root.path());
    let first_generation = NativePluginLoader
        .discovery_generation(root.path())
        .expect("first generation");
    assert_eq!(plugin_ids(&first), vec!["weather"]);

    fs::write(&weather, plugin_manifest("storm")).expect("replace weather manifest");
    let changed = NativePluginLoader.refresh_discovery_manifest(root.path(), &weather);
    assert_eq!(plugin_ids(&changed), vec!["storm"]);
    assert_eq!(
        NativePluginLoader.discovery_generation(root.path()),
        Some(first_generation + 1)
    );

    fs::remove_dir_all(weather.parent().expect("package root")).expect("remove package root");
    let removed = NativePluginLoader
        .remove_discovered_path(root.path(), weather.parent().expect("package root"));
    assert!(removed.discovered.is_empty());
    assert_eq!(
        NativePluginLoader.discovery_generation(root.path()),
        Some(first_generation + 2)
    );
}

#[test]
fn duplicate_package_selection_is_path_deterministic() {
    let root = TempDiscoveryRoot::new("duplicates");
    let first_path = root.write_manifest("a-first", "weather");
    root.write_manifest("z-second", "weather");

    let report = NativePluginLoader.discover(root.path());
    let canonical_first_path = first_path
        .canonicalize()
        .expect("canonicalize first duplicate manifest path");

    assert_eq!(plugin_ids(&report), vec!["weather"]);
    assert_eq!(report.discovered()[0].manifest_path, canonical_first_path);
    assert!(report.diagnostics().iter().any(|diagnostic| {
        diagnostic.contains("duplicate native plugin package id `weather`")
            && diagnostic.contains("a-first")
            && diagnostic.contains("z-second")
    }));
}

#[test]
fn concurrent_cold_discovery_waits_for_one_authority_ticket() {
    const THREADS: usize = 8;

    let root = Arc::new(TempDiscoveryRoot::new("concurrent-cold"));
    root.write_manifest("weather", "weather");
    let mut workers = Vec::with_capacity(THREADS);
    for _ in 0..THREADS {
        let root = Arc::clone(&root);
        workers.push(thread::spawn(move || {
            NativePluginLoader.discover(root.path())
        }));
    }

    for worker in workers {
        assert_eq!(
            plugin_ids(&worker.join().expect("discovery worker")),
            vec!["weather"]
        );
    }
    assert_eq!(
        NativePluginLoader.discovery_generation(root.path()),
        Some(1)
    );
}

#[test]
fn load_discovered_uses_the_authority_publication_before_dynamic_library_loading() {
    let root = TempDiscoveryRoot::new("load-parity");
    root.write_manifest("weather", "weather");

    let discovery = NativePluginLoader.discover(root.path());
    let generation = NativePluginLoader
        .discovery_generation(root.path())
        .expect("published generation");
    let loading = NativePluginLoader.load_discovered_runtime(root.path());

    assert_eq!(plugin_ids(&discovery), plugin_ids(&loading));
    assert_eq!(
        NativePluginLoader.discovery_generation(root.path()),
        Some(generation)
    );

    let loading_source = include_str!("../load_discovered.rs");
    assert!(!loading_source.contains("collect_plugin_manifests"));
    assert!(!loading_source.contains("candidate_from_manifest_path"));
}

#[test]
fn load_manifest_discovery_is_an_authority_owned_refresh_input() {
    let load_manifest = include_str!("../discover_load_manifest.rs");
    assert!(load_manifest.contains("discovery_authority().discover_load_manifest"));
    assert!(!load_manifest.contains("fs::read_to_string"));
    assert!(!load_manifest.contains("push_candidate_from_manifest_path"));

    let authority = include_str!("authority.rs");
    assert!(authority.contains("NativePluginDiscoveryRefreshInput::LoadManifest"));
    assert!(authority.contains("collect_load_manifest"));

    let refresh = include_str!("../discovery_refresh/service.rs");
    assert!(refresh.contains("submit_with_input"));
}

#[test]
fn recursive_discovery_stops_at_the_depth_bound() {
    let root = TempDiscoveryRoot::new("depth-bound");
    let mut nested = root.path().to_path_buf();
    for index in 0..=MAX_DISCOVERY_DEPTH {
        nested = nested.join(format!("level-{index}"));
        fs::create_dir_all(&nested).expect("create nested discovery directory");
    }
    fs::write(nested.join("plugin.toml"), plugin_manifest("too-deep"))
        .expect("write over-depth manifest");

    let collection = collect_plugin_manifests(root.path()).expect("bounded collection");

    assert!(collection.manifest_paths.is_empty());
    assert!(collection
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.contains("maximum depth")));
}

#[cfg(unix)]
#[test]
fn recursive_discovery_does_not_follow_directory_symlinks() {
    use std::os::unix::fs::symlink;

    let root = TempDiscoveryRoot::new("symlink-cycle");
    let package = root.path().join("package");
    fs::create_dir_all(&package).expect("create package root");
    symlink(root.path(), package.join("cycle")).expect("create discovery cycle symlink");
    fs::write(package.join("plugin.toml"), plugin_manifest("weather"))
        .expect("write package manifest");

    let collection = collect_plugin_manifests(root.path()).expect("cycle-safe collection");

    assert_eq!(collection.manifest_paths.len(), 1);
    assert!(collection
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.contains("symbolic link")));
}

fn plugin_ids(report: &super::super::NativePluginLoadReport) -> Vec<&str> {
    report
        .discovered()
        .iter()
        .map(|candidate| candidate.plugin_id.as_str())
        .collect()
}

fn plugin_manifest(plugin_id: &str) -> String {
    format!(
        r#"
id = "{plugin_id}"
version = "0.1.0"
display_name = "{plugin_id}"

[[modules]]
name = "{plugin_id}.runtime"
kind = "runtime"
crate_name = "zircon_plugin_{plugin_id}_runtime"
"#
    )
}

struct TempDiscoveryRoot {
    path: PathBuf,
}

impl TempDiscoveryRoot {
    fn new(label: &str) -> Self {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time should be after unix epoch")
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "zircon-native-discovery-{label}-{}-{stamp}",
            std::process::id()
        ));
        fs::create_dir_all(&path).expect("create native discovery root");
        Self { path }
    }

    fn path(&self) -> &Path {
        &self.path
    }

    fn write_manifest(&self, package: &str, plugin_id: &str) -> PathBuf {
        let package_root = self.path.join(package);
        fs::create_dir_all(&package_root).expect("create native package root");
        let manifest_path = package_root.join("plugin.toml");
        fs::write(&manifest_path, plugin_manifest(plugin_id)).expect("write plugin manifest");
        manifest_path
    }
}

impl Drop for TempDiscoveryRoot {
    fn drop(&mut self) {
        debug_assert!(self.path.starts_with(std::env::temp_dir()));
        let _ = fs::remove_dir_all(&self.path);
    }
}
