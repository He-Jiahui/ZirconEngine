use super::support::collect_rust_files;

#[test]
fn ui_asset_editor_moves_into_a_folder_backed_ui_subsystem() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("ui")
        .join("asset_editor");

    for relative in ["mod.rs", "contract.rs"] {
        assert!(
            root.join(relative).exists(),
            "expected ui asset editor module {relative} under {:?}",
            root
        );
    }
}

#[test]
fn ui_asset_editor_folder_modules_do_not_backreference_flat_files() {
    let asset_editor_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("ui")
        .join("asset_editor");
    let rust_files = collect_rust_files(&asset_editor_root);

    let offenders = rust_files
        .into_iter()
        .filter_map(|path| {
            let source = std::fs::read_to_string(&path).ok()?;
            (source.contains("#[path = \"../") || source.contains("#[path = \"../../")).then(|| {
                path.strip_prefix(&asset_editor_root)
                    .unwrap()
                    .display()
                    .to_string()
            })
        })
        .collect::<Vec<_>>();

    assert!(
        offenders.is_empty(),
        "expected ui/asset_editor folder modules to own their code directly instead of path-reexporting flat files, found {:?}",
        offenders
    );
}
