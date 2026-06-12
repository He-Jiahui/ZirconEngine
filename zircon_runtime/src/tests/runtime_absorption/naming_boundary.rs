use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug)]
struct NamingReference {
    path: String,
    line: usize,
    snippet: String,
}

#[test]
fn runtime_editor_and_legacy_naming_is_classified_by_owner() {
    let manifest_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let source_root = manifest_root.join("src");
    let files = rust_source_files(&source_root);

    assert_no_unclassified_naming(
        "editor",
        &collect_naming_references(manifest_root, &files, "editor"),
        classify_editor_reference,
    );
    assert_no_unclassified_naming(
        "legacy",
        &collect_naming_references(manifest_root, &files, "legacy"),
        classify_legacy_reference,
    );
}

fn rust_source_files(root: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    collect_rust_source_files(root, &mut files);
    files.sort();
    files
}

fn collect_rust_source_files(root: &Path, files: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(root).expect("runtime source directory should be readable") {
        let entry = entry.expect("runtime source entry should be readable");
        let path = entry.path();
        if path.is_dir() {
            collect_rust_source_files(&path, files);
        } else if path.extension().and_then(|extension| extension.to_str()) == Some("rs") {
            files.push(path);
        }
    }
}

fn collect_naming_references(
    manifest_root: &Path,
    files: &[PathBuf],
    term: &str,
) -> Vec<NamingReference> {
    let mut references = Vec::new();
    for path in files {
        let relative = relative_path(manifest_root, path);
        let source = fs::read_to_string(path)
            .unwrap_or_else(|error| panic!("failed to read {relative}: {error}"));
        for (line_index, line) in source.lines().enumerate() {
            if line_has_term(line, term) {
                references.push(NamingReference {
                    path: relative.clone(),
                    line: line_index + 1,
                    snippet: line.trim().to_string(),
                });
            }
        }
    }
    references
}

fn line_has_term(line: &str, term: &str) -> bool {
    line.split(|character: char| !(character.is_ascii_alphanumeric() || character == '_'))
        .any(|token| token.to_ascii_lowercase().contains(term))
}

fn assert_no_unclassified_naming(
    term: &str,
    references: &[NamingReference],
    classifier: fn(&str) -> Option<&'static str>,
) {
    let mut classifications = BTreeSet::new();
    let unclassified = references
        .iter()
        .filter(|reference| match classifier(&reference.path) {
            Some(classification) => {
                classifications.insert(classification);
                false
            }
            None => true,
        })
        .take(20)
        .map(|reference| {
            format!(
                "{}:{}: {}",
                reference.path, reference.line, reference.snippet
            )
        })
        .collect::<Vec<_>>();

    assert!(
        !classifications.is_empty(),
        "runtime {term} naming guard should classify at least one owner bucket"
    );
    assert!(
        unclassified.is_empty(),
        "runtime {term} naming contains unclassified owner references:\n{}",
        unclassified.join("\n")
    );
}

fn classify_editor_reference(relative_path: &str) -> Option<&'static str> {
    if is_test_path(relative_path) {
        return Some("test-fixture");
    }
    if relative_path.starts_with("src/plugin/")
        || relative_path.starts_with("src/builtin/runtime_modules/")
    {
        return Some("runtime-profile-editor-host-target");
    }
    if relative_path.starts_with("src/dynamic_api/") {
        return Some("dynamic-api-editor-host-mode");
    }
    if relative_path.starts_with("src/ui/component/catalog/")
        || relative_path.starts_with("src/ui/v2/surface_tree/")
    {
        return Some("runtime-ui-component-catalog-editor-controls");
    }
    if relative_path.starts_with("src/ui/template/") {
        return Some("runtime-ui-template-editor-profile");
    }
    if relative_path.starts_with("src/asset/") {
        return Some("runtime-asset-editor-metadata");
    }
    if relative_path.starts_with("src/core/framework/")
        || relative_path.starts_with("src/core/runtime/diagnostics/")
    {
        return Some("framework-editor-facing-descriptor");
    }
    if relative_path.starts_with("src/graphics/") {
        return Some("graphics-editor-facing-metadata");
    }
    if relative_path.starts_with("src/platform/") {
        return Some("platform-editor-target-diagnostic");
    }
    if relative_path.starts_with("src/rhi") {
        return Some("rhi-editor-surface-label");
    }
    if relative_path.starts_with("src/scene/reflect/")
        || relative_path.starts_with("src/scene/inspection/")
    {
        return Some("scene-reflection-editor-visible-metadata");
    }
    if matches!(
        relative_path,
        "src/diagnostic_log/sink.rs" | "src/prelude.rs"
    ) {
        return Some("curated-runtime-facade-editor-reference");
    }
    None
}

fn classify_legacy_reference(relative_path: &str) -> Option<&'static str> {
    if is_test_path(relative_path) {
        return Some("test-fixture");
    }
    if relative_path.starts_with("src/ui/surface/input/")
        || relative_path == "src/ui/surface/property_mutation.rs"
        || relative_path == "src/ui/surface/surface/default_interactions.rs"
    {
        return Some("legacy-runtime-ui-input-debt");
    }
    if relative_path == "src/ui/surface/render/collection_rows/table.rs" {
        return Some("legacy-runtime-ui-render-table-debt");
    }
    if relative_path.starts_with("src/graphics/")
        || relative_path.starts_with("src/core/framework/render/")
    {
        return Some("legacy-runtime-graphics-debt");
    }
    if relative_path == "src/asset/assets/texture/upload_support/dds.rs" {
        return Some("legacy-runtime-dds-container-policy");
    }
    if relative_path.starts_with("src/ui/template/") {
        return Some("legacy-runtime-ui-template-schema-debt");
    }
    if relative_path.starts_with("src/ui/layout/") {
        return Some("legacy-runtime-ui-layout-debt");
    }
    if relative_path.starts_with("src/input/")
        || relative_path.starts_with("src/core/framework/input/")
    {
        return Some("legacy-runtime-input-event-debt");
    }
    if relative_path.starts_with("src/asset/") {
        return Some("legacy-runtime-asset-schema-debt");
    }
    if relative_path.starts_with("src/dynamic_api/") {
        return Some("legacy-dynamic-api-migration-debt");
    }
    if relative_path.starts_with("src/scene/") {
        return Some("legacy-scene-schema-render-debt");
    }
    if matches!(
        relative_path,
        "src/prelude.rs" | "src/ui/accessibility/extract.rs"
    ) {
        return Some("curated-runtime-facade-legacy-reference");
    }
    None
}

fn is_test_path(relative_path: &str) -> bool {
    let file_name = relative_path.rsplit('/').next().unwrap_or(relative_path);
    relative_path.split('/').any(|part| part == "tests")
        || file_name == "tests.rs"
        || file_name.ends_with("_tests.rs")
}

fn relative_path(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .expect("runtime source path should live under manifest root")
        .to_string_lossy()
        .replace('\\', "/")
}
