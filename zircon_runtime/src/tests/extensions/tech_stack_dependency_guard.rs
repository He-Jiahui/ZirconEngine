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
fn zr_vm_path_dependency_gate_is_documented_with_version_pairing() {
    let runtime_manifest = read_repo_file("zircon_runtime/Cargo.toml");
    let tech_stack = read_repo_file("docs/engine-architecture/runtime-tech-stack.md");

    assert!(
        runtime_manifest.contains("zr-vm-real-backend"),
        "real ZrVM backend should stay explicitly feature gated"
    );
    assert!(
        runtime_manifest.contains("../../zr_vm/zr_vm_rust_binding")
            && runtime_manifest.contains("optional = true"),
        "ZrVM path dependencies should stay optional external-checkout dependencies"
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

    for removed in ["cosmic-text", "kira", "rfd", "arboard"] {
        assert!(
            manifests.iter().all(|source| !source.contains(removed)),
            "{removed} should not appear in current Cargo manifests without updating the tech-stack decision"
        );
        assert!(
            tech_stack.contains(removed),
            "{removed} should be named in the corrected non-dependency decision table"
        );
    }

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
fn physics_backend_option_decision_keeps_jolt_unavailable_and_plugin_owned() {
    let runtime_manifest = read_repo_file("zircon_runtime/Cargo.toml");
    let physics_manifest = read_repo_file("zircon_plugins/physics/runtime/Cargo.toml");
    let physics_backend = read_repo_file("zircon_plugins/physics/runtime/src/backend.rs");
    let physics_options = read_repo_file("docs/zircon_plugins/physics-plugin-options.md");
    let physics_runtime_doc = read_repo_file("docs/zircon_plugins/physics/runtime.md");
    let manifests = all_manifest_sources();

    let jolt_feature_slots = manifests
        .iter()
        .map(|source| source.matches("jolt = []").count())
        .sum::<usize>();
    assert_eq!(
        jolt_feature_slots, 2,
        "Runtime 01 M3 expects exactly two visible-but-unavailable jolt feature slots"
    );
    assert!(
        runtime_manifest.contains("jolt = []")
            && physics_manifest.contains("jolt = []"),
        "the jolt feature slots should stay in the runtime profile and physics plugin manifests until the plugin-owned bridge lands"
    );
    assert!(
        physics_backend.contains("const JOLT_BACKEND_AVAILABLE: bool = false")
            && physics_backend.contains("PhysicsRuntimeBackend::Unavailable")
            && physics_backend
                .contains("feature `jolt` is enabled, but no runtime Jolt backend is linked"),
        "physics backend source should keep jolt unavailable instead of downgrading it to builtin"
    );
    assert!(
        physics_options.contains("only executable V1 backend")
            && physics_options.contains("Jolt as the future native backend direction")
            && physics_options.contains("No Rapier dependency is introduced")
            && physics_options.contains("never become a `zircon_runtime` dependency")
            && physics_options.contains("JOLT_BACKEND_AVAILABLE = false"),
        "physics option decision should keep builtin executable, jolt future/plugin-owned, and rapier out of the primary path"
    );
    assert!(
        physics_runtime_doc.contains("builtin remains the only executable V1 backend")
            && physics_runtime_doc.contains("Jolt is the future native backend direction")
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
        shaper.contains("active_layout_backend_for_intent")
            && shaper.contains("UiTextBackendIntent::NativeGlyphon")
            && shaper.contains("UiTextBackendIntent::SdfAtlas")
            && shaper.contains("UiTextBackendIntent::SharedTextService"),
        "NativeGlyphon and SdfAtlas layout intents should remain explicit while current layout uses the shared text service"
    );
    assert!(
        shaper.contains("fallback_reason_for_backend") && shaper.contains("None"),
        "current text-stack selection should keep fallback reasons absent while SharedTextService is the active layout backend"
    );
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

fn all_manifest_sources() -> Vec<String> {
    collect_manifest_sources(&repo_root())
}

fn collect_manifest_sources(root: &std::path::Path) -> Vec<String> {
    let mut pending = vec![root.to_path_buf()];
    let mut sources = Vec::new();

    while let Some(path) = pending.pop() {
        let entries = match std::fs::read_dir(&path) {
            Ok(entries) => entries,
            Err(_) => continue,
        };

        for entry in entries.filter_map(Result::ok) {
            let path = entry.path();
            let file_name = entry.file_name();
            let file_name = file_name.to_string_lossy();

            if path.is_dir() {
                if matches!(
                    file_name.as_ref(),
                    ".git" | "target" | "dev" | "node_modules"
                ) {
                    continue;
                }
                pending.push(path);
            } else if file_name == "Cargo.toml" {
                sources.push(std::fs::read_to_string(path).unwrap_or_default());
            }
        }
    }

    sources
}

fn manifest_declares_dependency(source: &str, crate_name: &str) -> bool {
    source.lines().any(|line| {
        let line = line.trim();
        line == format!("[dependencies.{crate_name}]")
            || line == format!("[workspace.dependencies.{crate_name}]")
            || line.starts_with(&format!("{crate_name} ="))
            || line.starts_with(&format!("{crate_name}.workspace"))
    })
}
