fn runtime_root() -> &'static std::path::Path {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
}

fn repo_root() -> std::path::PathBuf {
    runtime_root()
        .parent()
        .expect("runtime crate should have a workspace parent")
        .to_path_buf()
}

fn read_repo_file(path: &str) -> String {
    std::fs::read_to_string(repo_root().join(path)).unwrap_or_else(|error| {
        panic!("expected to read {path}: {error}");
    })
}

#[test]
fn runtime_tech_stack_doc_exists_and_is_linked_from_architecture_index() {
    let repo_root = repo_root();
    let tech_stack = repo_root.join("docs/engine-architecture/runtime-tech-stack.md");
    let index = read_repo_file("docs/engine-architecture/index.md");

    assert!(
        tech_stack.exists(),
        "docs/engine-architecture/runtime-tech-stack.md should be the runtime dependency authority"
    );
    assert!(
        index.contains("runtime-tech-stack.md")
            && index.contains("[Runtime Tech Stack](./runtime-tech-stack.md)"),
        "engine architecture index should link the runtime tech-stack authority document"
    );
}

#[test]
fn runtime_manifest_keeps_pinned_prerelease_versions_until_upgrade_gate() {
    let workspace_manifest = read_repo_file("Cargo.toml");
    let tech_stack = read_repo_file("docs/engine-architecture/runtime-tech-stack.md");

    assert!(
        workspace_manifest.contains("0.31.0-beta.2"),
        "winit should remain pinned until a dedicated ApplicationHandler upgrade gate changes the policy"
    );
    assert!(
        workspace_manifest.contains("9.0.0-rc.3"),
        "notify should remain pinned until a dedicated watcher event compatibility gate changes the policy"
    );
    assert!(
        tech_stack.contains("winit 0.31.0-beta.2"),
        "runtime tech-stack doc should record the current winit prerelease pin"
    );
    assert!(
        tech_stack.contains("notify 9.0.0-rc.3"),
        "runtime tech-stack doc should record the current notify prerelease pin"
    );
    assert!(
        tech_stack.contains("ApplicationHandler")
            && tech_stack.contains("watcher event compatibility"),
        "runtime tech-stack doc should spell out the prerelease upgrade gates"
    );
}

#[test]
fn wgpu_windows_dependency_family_stays_on_one_compatible_patch_line() {
    let lock = read_repo_file("Cargo.lock");
    let tech_stack = read_repo_file("docs/engine-architecture/runtime-tech-stack.md");

    for package in [
        "wgpu",
        "wgpu-core",
        "wgpu-core-deps-windows-linux-android",
        "wgpu-hal",
        "wgpu-naga-bridge",
        "wgpu-types",
    ] {
        let block = lock_package_block(&lock, package);
        assert!(
            block.contains("version = \"29.0.3\""),
            "{package} must remain on the validated 29.0.3 patch line so DX12 does not split Windows COM types"
        );
    }

    let allocator = lock_package_block(&lock, "gpu-allocator");
    assert!(
        allocator.contains("version = \"0.28.0\"") && allocator.contains("\"windows 0.62.2\""),
        "gpu-allocator 0.28.0 must resolve Windows 0.62.2 with wgpu-hal 29.0.3"
    );
    assert!(
        tech_stack.contains("WGPU 29.0.3 patch family")
            && tech_stack.contains("gpu-allocator 0.28.0")
            && tech_stack.contains("windows 0.62.2"),
        "runtime tech-stack authority must record the validated Windows dependency convergence"
    );
}

#[test]
fn zr_vm_path_dependency_gate_is_documented_with_version_pairing() {
    let plugin_manifest = read_repo_file("zircon_plugins/zr_vm_language/runtime/Cargo.toml");
    let tech_stack = read_repo_file("docs/engine-architecture/runtime-tech-stack.md");

    assert!(
        plugin_manifest.contains("backend-zr-vm"),
        "real ZrVM backend should stay explicitly feature gated by its plugin owner"
    );
    assert!(
        plugin_manifest.contains("../../../../zr_vm/zr_vm_rust_binding")
            && plugin_manifest.contains("optional = true")
            && !read_repo_file("zircon_runtime/Cargo.toml").contains("zr_vm_rust_binding"),
        "ZrVM path dependencies must stay optional and plugin-owned"
    );
    assert!(
        tech_stack.contains("../../zr_vm") && tech_stack.contains("external checkout"),
        "runtime tech-stack doc should document the external ZrVM checkout decision"
    );
    assert!(
        tech_stack.contains("empty export argument lists")
            && tech_stack.contains("non-null pointer with length `0`"),
        "runtime tech-stack doc should preserve the binding version-pairing gate"
    );
}

#[test]
fn interface_and_editor_dependency_boundaries_stay_documented_and_guarded() {
    let interface_manifest = read_repo_file("zircon_runtime_interface/Cargo.toml");
    let editor_manifest = read_repo_file("zircon_editor/Cargo.toml");
    let tech_stack = read_repo_file("docs/engine-architecture/runtime-tech-stack.md");

    assert!(
        !interface_manifest.contains("wgpu"),
        "zircon_runtime_interface must stay free of direct wgpu dependency"
    );
    assert!(
        !interface_manifest.contains("winit"),
        "zircon_runtime_interface must stay free of direct winit dependency"
    );
    assert!(
        !editor_manifest.contains("wgpu"),
        "zircon_editor should not add direct wgpu without an editor renderer ownership plan"
    );
    assert!(
        editor_manifest.contains("winit.workspace = true"),
        "zircon_editor currently owns a direct winit dependency for its retained host path"
    );
    assert!(
        tech_stack.contains("zircon_runtime_interface")
            && tech_stack.contains("free of `wgpu` and `winit`"),
        "runtime tech-stack doc should record the interface crate dependency boundary"
    );
    assert!(
        tech_stack.contains("zircon_editor") && tech_stack.contains("direct `winit` dependency"),
        "runtime tech-stack doc should record the corrected editor winit boundary"
    );
}

#[test]
fn removed_or_editor_only_dependencies_do_not_silently_enter_runtime_stack() {
    let runtime_manifest = read_repo_file("zircon_runtime/Cargo.toml");
    let tech_stack = read_repo_file("docs/engine-architecture/runtime-tech-stack.md");
    let manifests = all_manifest_sources();
    let zip_dependency_line =
        "zip = { version = \"9.0.0-pre2\", default-features = false, features = [\"deflate-flate2\"] }";
    const KIRA_CANONICAL_DEPENDENCY_LINE: &str = r#"kira = "0.12.2""#;

    for removed in ["cosmic-text", "rfd", "arboard"] {
        assert!(
            manifests.iter().all(|source| !source.contains(removed)),
            "{removed} should not appear in current Cargo manifests without updating the tech-stack decision"
        );
        assert!(
            tech_stack.contains(removed),
            "{removed} should be named in the corrected non-dependency decision table"
        );
    }

    let kira_owners = manifests
        .iter()
        .filter(|source| manifest_declares_package(source, "kira"))
        .map(|source| source.path.as_str())
        .collect::<Vec<_>>();
    assert_eq!(
        kira_owners,
        ["zircon_plugins/sound/runtime/Cargo.toml"],
        "Kira should have exactly one product manifest owner in the Sound runtime plugin"
    );
    let kira_owner = manifests
        .iter()
        .find(|source| source.path == "zircon_plugins/sound/runtime/Cargo.toml")
        .expect("Sound runtime manifest should be part of the current product scan");
    assert!(
        manifest_has_exact_single_runtime_dependency(kira_owner, "kira", "0.12.2"),
        "Sound runtime should have one production Kira 0.12.2 pin; the canonical form is `{KIRA_CANONICAL_DEPENDENCY_LINE}`"
    );

    let zip_dependency_count = manifests
        .iter()
        .filter(|source| manifest_declares_dependency(source, "zip"))
        .count();
    assert_eq!(
        zip_dependency_count, 1,
        "zip should only appear in zircon_runtime for export archive materialization"
    );
    assert!(
        runtime_manifest.contains(zip_dependency_line),
        "zip should stay pinned with minimal deflate-only features for the archive materializer"
    );
    assert!(
        tech_stack.contains("zip 9.0.0-pre2") && tech_stack.contains("export archive materializer"),
        "runtime tech-stack doc should record the single admitted zip dependency owner"
    );
    assert!(
        manifests
            .iter()
            .all(|source| !manifest_declares_dependency(source, "tar")),
        "tar should not appear as a manifest dependency until a server/CI artifact policy is implemented"
    );
    assert!(
        tech_stack.contains("`tar`"),
        "runtime tech-stack doc should record that tar is still not a current runtime dependency"
    );
}

#[test]
fn dependency_package_identity_cannot_hide_kira_behind_alias_or_metadata() {
    assert!(!manifest_declares_package(
        "[package.metadata]\naudio = { package = \"kira\", version = \"0.12.2\" }\n",
        "kira"
    ));
    assert!(manifest_declares_package(
        "[target.'cfg(windows)'.dependencies]\naudio = { package = \"kira\", version = \"0.12.2\" }\n",
        "kira"
    ));
    assert!(manifest_has_exact_single_runtime_dependency(
        "[target.'cfg(windows)'.dependencies]\naudio = { package = \"kira\", version = \"0.12.2\" }\n",
        "kira",
        "0.12.2"
    ));
    assert!(manifest_has_exact_single_runtime_dependency(
        "[dependencies]\naudio = { package = \"kira\", version = \"0.12.2\" }\n",
        "kira",
        "0.12.2"
    ));
    assert!(manifest_has_exact_single_runtime_dependency(
        "[dependencies]\nkira = \"0.12.2\"\n",
        "kira",
        "0.12.2"
    ));
    assert!(!manifest_has_exact_single_runtime_dependency(
        "[dependencies]\nkira = \"0.12.2\"\naudio = { package = \"kira\", version = \"0.12.2\" }\n",
        "kira",
        "0.12.2"
    ));
    for non_runtime in [
        "[dev-dependencies]\nkira = \"0.12.2\"\n",
        "[build-dependencies]\nkira = \"0.12.2\"\n",
        "[workspace.dependencies]\nkira = \"0.12.2\"\n",
        "[target.'cfg(windows)'.dev-dependencies]\nkira = \"0.12.2\"\n",
    ] {
        assert!(!manifest_has_exact_single_runtime_dependency(
            non_runtime,
            "kira",
            "0.12.2"
        ));
    }
}

#[test]
fn export_archive_policy_allows_zip_only_for_archive_materializer() {
    let export_profile = read_repo_file("zircon_runtime/src/plugin/export_profile.rs");
    let runtime_manifest = read_repo_file("zircon_runtime/Cargo.toml");
    let tech_stack = read_repo_file("docs/engine-architecture/runtime-tech-stack.md");
    let manifests = all_manifest_sources();
    let zip_dependency_line =
        "zip = { version = \"9.0.0-pre2\", default-features = false, features = [\"deflate-flate2\"] }";

    assert!(
        export_profile.contains("pub enum ExportPackagingStrategy")
            && export_profile.contains("SourceTemplate")
            && export_profile.contains("LibraryEmbed")
            && export_profile.contains("NativeDynamic"),
        "current export packaging strategy should remain code-materialization oriented"
    );
    assert!(
        tech_stack.contains("## Export Archive Decision")
            && tech_stack.contains("ZIP archive materialization is implemented")
            && tech_stack.contains("directory-first")
            && tech_stack.contains("materialize_zip_archive")
            && tech_stack.contains("preview_zip_archive"),
        "runtime tech-stack doc should record the implemented ZIP archive materialization API"
    );
    let zip_dependency_count = manifests
        .iter()
        .filter(|source| manifest_declares_dependency(source, "zip"))
        .count();
    assert_eq!(
        zip_dependency_count, 1,
        "zip should only be declared by the runtime archive materializer"
    );
    assert!(
        runtime_manifest.contains(zip_dependency_line),
        "zip dependency should remain pinned with no default features"
    );
    assert!(
        manifests
            .iter()
            .all(|source| !manifest_declares_dependency(source, "tar")),
        "tar should stay out of manifests because the primary archive decision is ZIP"
    );
}

#[test]
fn physics_backend_option_decision_keeps_jolt_feature_gated_and_plugin_owned() {
    let runtime_manifest = read_repo_file("zircon_runtime/Cargo.toml");
    let physics_manifest = read_repo_file("zircon_plugins/physics/runtime/Cargo.toml");
    let physics_backend = format!(
        "{}\n{}",
        read_repo_file("zircon_plugins/physics/runtime/src/backend/mod.rs"),
        read_repo_file("zircon_plugins/physics/runtime/src/backend/selection.rs")
    );
    let physics_options = read_repo_file("docs/zircon_plugins/physics-plugin-options.md");
    let physics_runtime_doc = read_repo_file("docs/zircon_plugins/physics/runtime.md");
    let manifests = all_manifest_sources();

    let jolt_feature_slots = runtime_manifest.matches("backend-jolt = []").count()
        + physics_manifest
            .matches("backend-jolt = [\"dep:joltc-sys\"]")
            .count();
    assert_eq!(
        jolt_feature_slots, 2,
        "Runtime 01 M3 expects one runtime profile hook and one dependency-backed plugin feature"
    );
    assert!(
        runtime_manifest.contains("backend-jolt = []")
            && physics_manifest.contains("backend-jolt = [\"dep:joltc-sys\"]")
            && physics_manifest.contains("joltc-sys = { version = \"=0.3.1\", optional = true }"),
        "runtime should keep profile vocabulary while the physics plugin owns the optional Jolt dependency"
    );
    assert!(
        physics_backend
            .contains("JOLT_BACKEND_AVAILABLE: bool = cfg!(feature = \"backend-jolt\")")
            && physics_backend.contains("#[cfg(feature = \"backend-jolt\")]\nmod jolt;")
            && physics_backend.contains("PhysicsRuntimeBackend::Unavailable")
            && physics_backend.contains("feature `backend-jolt` is not enabled"),
        "physics backend source should enable Jolt only through the plugin feature and keep feature-off unavailable without builtin fallback"
    );
    assert!(
        physics_options.contains("selected native backend")
            && physics_options.contains("optional `joltc-sys`")
            && physics_options.contains("Neither path silently downgrades to builtin stepping")
            && physics_options.contains("No Rapier dependency is introduced")
            && physics_options.contains("no concrete physics library is added to `zircon_runtime`"),
        "physics option decision should keep Jolt feature-gated and plugin-owned while Rapier stays off the primary path"
    );
    assert!(
        physics_runtime_doc
            .contains("Jolt is executable when the plugin's `backend-jolt` feature is enabled")
            && physics_runtime_doc.contains("feature-off Jolt remains explicitly unavailable")
            && physics_runtime_doc.contains("Rapier is not introduced on the primary path"),
        "physics runtime doc should cross-reference the Runtime 01 backend option ruling"
    );

    for dependency in ["rapier2d", "rapier3d", "avian2d", "avian3d"] {
        assert!(
            manifests
                .iter()
                .all(|source| !manifest_declares_dependency(source, dependency)),
            "{dependency} should not enter manifests without a new physics backend decision"
        );
    }
}

#[test]
fn editor_only_dependency_candidates_have_editor_backlog_owner() {
    let tech_stack = read_repo_file("docs/engine-architecture/runtime-tech-stack.md");
    let backlog =
        read_repo_file("docs/editor-and-tooling/runtime-editor-only-dependency-backlog.md");
    let editor_index = read_repo_file("docs/editor-and-tooling/index.md");
    let manifests = all_manifest_sources();

    assert!(
        tech_stack.contains("runtime-editor-only-dependency-backlog.md"),
        "runtime tech-stack doc should link editor-only candidates to the editor backlog"
    );
    assert!(
        editor_index.contains("runtime-editor-only-dependency-backlog.md"),
        "editor-and-tooling index should expose the editor-only dependency backlog"
    );

    for candidate in ["rfd", "arboard"] {
        assert!(
            backlog.contains(candidate),
            "{candidate} should be recorded as an editor-only candidate"
        );
        assert!(
            manifests
                .iter()
                .all(|source| !manifest_declares_dependency(source, candidate)),
            "{candidate} should not appear in Cargo manifests before the editor plan admits it"
        );
    }

    assert!(
        backlog.contains("zircon_editor/src/ui/host")
            && backlog.contains("file and folder dialogs")
            && backlog.contains("Clipboard integration"),
        "editor backlog should name the expected owner paths and demand surfaces"
    );
    assert!(
        backlog.contains("Do not add to `zircon_runtime`")
            && backlog.contains("zircon_runtime_interface"),
        "editor backlog should keep these candidates out of runtime and interface crates"
    );
}

#[test]
fn fontdue_editor_retained_host_dependency_has_migration_owner() {
    let runtime_manifest = read_repo_file("zircon_runtime/Cargo.toml");
    let editor_manifest = read_repo_file("zircon_editor/Cargo.toml");
    let retained_text = [
        "zircon_editor/src/ui/retained_host/host_contract/paint_text/font.rs",
        "zircon_editor/src/ui/retained_host/host_contract/paint_text/raster.rs",
        "zircon_editor/src/ui/retained_host/host_contract/paint_text/draw.rs",
    ]
    .into_iter()
    .map(read_repo_file)
    .collect::<Vec<_>>()
    .join("\n");
    let tech_stack = read_repo_file("docs/engine-architecture/runtime-tech-stack.md");
    let backlog =
        read_repo_file("docs/editor-and-tooling/runtime-editor-only-dependency-backlog.md");

    assert!(
        !runtime_manifest.contains("fontdue"),
        "fontdue must stay out of zircon_runtime until the runtime text stack explicitly admits it"
    );
    assert!(
        editor_manifest.contains("fontdue"),
        "zircon_editor currently owns fontdue for the retained-host text fallback"
    );
    assert!(
        retained_text.contains("fontdue::"),
        "fontdue use should stay visible in the retained-host painter owner until migration"
    );
    assert!(
        tech_stack.contains("temporary `zircon_editor` retained-host text fallback")
            && tech_stack.contains("runtime UI text/glyphon/SDF"),
        "runtime tech-stack doc should record fontdue as temporary editor-only text debt"
    );
    assert!(
        backlog.contains("fontdue")
            && backlog.contains("retained-host text")
            && backlog.contains("runtime UI text boundary"),
        "editor backlog should define the fontdue migration owner and replacement boundary"
    );
}

#[test]
fn complex_text_backends_can_only_enter_through_ui_text_shaper() {
    let shaper = read_repo_file("zircon_runtime/src/ui/text/shaper.rs");
    let shaper_tests = read_repo_file("zircon_runtime/src/ui/tests/text_shaper.rs");
    let tech_stack = read_repo_file("docs/engine-architecture/runtime-tech-stack.md");

    assert!(
        shaper.contains("trait UiTextShaper")
            && shaper.contains("fn shape_text")
            && shaper.contains("fn measure_text"),
        "runtime text backend replacements should go through the UiTextShaper boundary"
    );
    assert!(
        shaper.contains("struct UiTextShaperStack")
            && shaper.contains("shared: UiSharedTextShaper")
            && shaper.contains("self.shared.shape_text(request)")
            && shaper.contains("self.shared.measure_text(text, style)"),
        "UiTextShaperStack must be a direct adapter over the shared text service"
    );
    for removed in [
        "UiTextBackendIntent",
        "UiTextShaperSelection",
        "active_layout_backend_for_intent",
        "fallback_reason_for_backend",
    ] {
        assert!(
            !shaper.contains(removed),
            "text layout must not retain the removed pseudo-backend route: {removed}"
        );
    }
    assert!(
        shaper_tests.contains("shared_text_shaper_matches_public_layout_entrypoint")
            && shaper_tests.contains(
                "text_shaper_stack_uses_shared_text_service_for_font_backends"
            ),
        "runtime text tests should lock public layout parity and current SharedTextService backend behavior"
    );
    assert!(
        tech_stack.contains("cosmic-text")
            && tech_stack.contains("Parley")
            && tech_stack.contains("Swash")
            && tech_stack.contains("HarfBuzz")
            && tech_stack.contains("replacement implementation of `UiTextShaper`"),
        "runtime tech-stack doc should route complex text candidates through UiTextShaper only"
    );
}

#[test]
fn runtime_text_doc_records_three_layer_stack_and_cross_reference() {
    let text_doc = read_repo_file("docs/zircon_runtime/ui/text.md");
    let tech_stack = read_repo_file("docs/engine-architecture/runtime-tech-stack.md");

    assert!(
        text_doc.contains("## Backend Responsibility Matrix"),
        "runtime UI text doc should contain the Runtime 01 backend responsibility matrix"
    );
    for expected in [
        "Shaping, segmentation, layout, and measurement",
        "Font registry, raster, and SDF policy",
        "GPU/native text submission",
    ] {
        assert!(
            text_doc.contains(expected),
            "runtime UI text matrix should record the {expected} layer"
        );
    }
    assert!(
        text_doc.contains("../../engine-architecture/runtime-tech-stack.md#text-stack-boundary"),
        "runtime UI text doc should cross-reference the runtime tech-stack text boundary"
    );
    assert!(
        text_doc.contains("SharedTextService") && text_doc.contains("Native and SDF render modes"),
        "glyphon row should preserve the current SharedTextService layout-metrics status required by Runtime 01 M2.1"
    );
    assert!(
        text_doc.contains("shared_text_shaper_matches_public_layout_entrypoint")
            && text_doc.contains("text_shaper_stack_uses_shared_text_service_for_font_backends"),
        "runtime UI text doc should cite the tests that lock the current SharedTextService backend"
    );
    assert!(
        tech_stack.contains("## Text Stack Boundary")
            && tech_stack.contains("replacement implementation of `UiTextShaper`"),
        "runtime tech-stack text boundary should remain the authority for complex text backend entry"
    );
}

#[derive(Debug)]
struct ManifestSource {
    path: String,
    source: String,
}

impl std::ops::Deref for ManifestSource {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.source
    }
}

fn all_manifest_sources() -> &'static [ManifestSource] {
    static MANIFEST_SOURCES: std::sync::OnceLock<Vec<ManifestSource>> = std::sync::OnceLock::new();
    MANIFEST_SOURCES
        .get_or_init(|| collect_manifest_sources(&repo_root()))
        .as_slice()
}

fn collect_manifest_sources(root: &std::path::Path) -> Vec<ManifestSource> {
    let mut sources = Vec::new();
    let mut pending = vec![root.to_path_buf()];

    while let Some(path) = pending.pop() {
        let entries = std::fs::read_dir(&path).unwrap_or_else(|error| {
            panic!(
                "expected to scan current product path {}: {error}",
                path.display()
            )
        });

        for entry in entries {
            let entry = entry.unwrap_or_else(|error| {
                panic!(
                    "expected to enumerate current product path {}: {error}",
                    path.display()
                )
            });
            let path = entry.path();
            let file_name = entry.file_name().into_string().unwrap_or_else(|_| {
                panic!(
                    "current product path should be valid UTF-8: {}",
                    path.display()
                )
            });
            let file_type = entry.file_type().unwrap_or_else(|error| {
                panic!(
                    "expected to inspect current product path {}: {error}",
                    path.display()
                )
            });

            if file_type.is_dir() && !file_type.is_symlink() {
                if matches!(
                    file_name.as_str(),
                    ".git"
                        | ".zircon"
                        | "target"
                        | "dev"
                        | "docs"
                        | "node_modules"
                        | "tests"
                        | "tools"
                ) {
                    continue;
                }
                if path == root.join("zircon_plugins") {
                    pending.push(path);
                    continue;
                }
                if path.parent() == Some(root) && !file_name.starts_with("zircon_") {
                    continue;
                }
                pending.push(path);
            } else if file_name == "Cargo.toml" {
                let relative_path = path
                    .strip_prefix(root)
                    .expect("manifest scan should remain within the repository root")
                    .to_string_lossy()
                    .replace('\\', "/");
                let source = std::fs::read_to_string(&path).unwrap_or_else(|error| {
                    panic!("expected to read current manifest {relative_path}: {error}")
                });
                sources.push(ManifestSource {
                    path: relative_path,
                    source,
                });
            }
        }
    }

    sources.sort_unstable_by(|left, right| left.path.cmp(&right.path));
    sources
}

fn manifest_declares_dependency(source: &str, crate_name: &str) -> bool {
    manifest_declares_package(source, crate_name)
}

fn manifest_declares_package(source: &str, crate_name: &str) -> bool {
    manifest_package_declaration_count(source, crate_name) > 0
}

fn manifest_has_exact_single_runtime_dependency(
    source: &str,
    crate_name: &str,
    version: &str,
) -> bool {
    let Ok(manifest) = toml::from_str::<toml::Value>(source) else {
        return false;
    };
    if manifest_package_declaration_count_in_value(&manifest, crate_name) != 1 {
        return false;
    }
    let runtime_versions = runtime_dependency_versions(&manifest, crate_name);
    runtime_versions.len() == 1 && runtime_versions[0] == Some(version)
}

fn manifest_package_declaration_count(source: &str, crate_name: &str) -> usize {
    toml::from_str::<toml::Value>(source)
        .map(|manifest| manifest_package_declaration_count_in_value(&manifest, crate_name))
        .unwrap_or_else(|error| panic!("current Cargo manifest should parse as TOML: {error}"))
}

fn manifest_package_declaration_count_in_value(manifest: &toml::Value, crate_name: &str) -> usize {
    let Some(root) = manifest.as_table() else {
        return 0;
    };
    let mut count = dependency_group_package_count(root, crate_name);
    count += root
        .get("workspace")
        .and_then(toml::Value::as_table)
        .map_or(0, |workspace| {
            dependency_table_package_count(workspace.get("dependencies"), crate_name)
        });
    count
        + root
            .get("target")
            .and_then(toml::Value::as_table)
            .map_or(0, |targets| {
                targets
                    .values()
                    .filter_map(toml::Value::as_table)
                    .map(|target| dependency_group_package_count(target, crate_name))
                    .sum()
            })
}

fn dependency_group_package_count(table: &toml::value::Table, crate_name: &str) -> usize {
    ["dependencies", "dev-dependencies", "build-dependencies"]
        .into_iter()
        .map(|group| dependency_table_package_count(table.get(group), crate_name))
        .sum()
}

fn dependency_table_package_count(table: Option<&toml::Value>, crate_name: &str) -> usize {
    table
        .and_then(toml::Value::as_table)
        .map_or(0, |dependencies| {
            dependencies
                .iter()
                .filter(|(alias, specification)| {
                    specification
                        .as_table()
                        .and_then(|table| table.get("package"))
                        .and_then(toml::Value::as_str)
                        .unwrap_or(alias)
                        == crate_name
                })
                .count()
        })
}

fn runtime_dependency_versions<'a>(
    manifest: &'a toml::Value,
    crate_name: &str,
) -> Vec<Option<&'a str>> {
    let Some(root) = manifest.as_table() else {
        return Vec::new();
    };
    let mut versions = dependency_table_package_versions(root.get("dependencies"), crate_name);
    if let Some(targets) = root.get("target").and_then(toml::Value::as_table) {
        for target in targets.values().filter_map(toml::Value::as_table) {
            versions.extend(dependency_table_package_versions(
                target.get("dependencies"),
                crate_name,
            ));
        }
    }
    versions
}

fn dependency_table_package_versions<'a>(
    table: Option<&'a toml::Value>,
    crate_name: &str,
) -> Vec<Option<&'a str>> {
    table
        .and_then(toml::Value::as_table)
        .into_iter()
        .flat_map(|dependencies| dependencies.iter())
        .filter(|(alias, specification)| {
            specification
                .as_table()
                .and_then(|table| table.get("package"))
                .and_then(toml::Value::as_str)
                .unwrap_or(alias)
                == crate_name
        })
        .map(|(_, specification)| {
            specification.as_str().or_else(|| {
                specification
                    .as_table()
                    .and_then(|table| table.get("version"))
                    .and_then(toml::Value::as_str)
            })
        })
        .collect()
}

fn lock_package_block<'a>(lock: &'a str, package_name: &str) -> &'a str {
    lock.split("[[package]]")
        .find(|block| {
            block
                .lines()
                .any(|line| line.trim() == format!("name = \"{package_name}\""))
        })
        .unwrap_or_else(|| panic!("Cargo.lock should contain package {package_name}"))
}
