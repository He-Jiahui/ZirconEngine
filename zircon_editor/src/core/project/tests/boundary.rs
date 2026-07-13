use std::path::{Path, PathBuf};

#[test]
fn project_authority_core_has_no_ui_dependency_or_retired_template_generator() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("core")
        .join("project");
    let mut sources = Vec::new();
    collect_rs(&root, &mut sources);
    let mut violations = Vec::new();
    for source in sources {
        let text = std::fs::read_to_string(&source).unwrap();
        for forbidden in [
            "crate::ui",
            "super::ui",
            "DEFAULT_PBR_WGSL",
            "DEFAULT_CUBE_OBJ",
            "library_root(",
            "runtime_cache_root(",
        ] {
            if text.contains(forbidden) {
                violations.push(format!("{} contains {forbidden}", source.display()));
            }
        }
    }
    assert!(violations.is_empty(), "{}", violations.join("\n"));
}

fn collect_rs(path: &Path, sources: &mut Vec<PathBuf>) {
    for entry in std::fs::read_dir(path).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            collect_rs(&path, sources);
        } else if path.extension().is_some_and(|extension| extension == "rs") {
            sources.push(path);
        }
    }
}
