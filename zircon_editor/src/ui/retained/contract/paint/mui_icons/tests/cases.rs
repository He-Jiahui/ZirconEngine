use super::names::module_name;
use super::parser::{path_elements, MuiIconPathElement};
use super::rendering::load_module_tree;

#[test]
fn path_parser_preserves_paths_and_opacity() {
    let source = r#"
            jsx(_Fragment.Fragment, {
                children: [jsx("path", { d: "M1 1h2v2H1z", opacity: ".3" }, "0"),
                    jsx("path", { d: "M4 4h2v2H4z" }, "1")]
            })
        "#;

    let elements = path_elements(source);

    assert_eq!(
        elements,
        vec![
            MuiIconPathElement {
                d: "M1 1h2v2H1z".to_string(),
                opacity: Some(".3".to_string())
            },
            MuiIconPathElement {
                d: "M4 4h2v2H4z".to_string(),
                opacity: None
            }
        ]
    );
}

#[test]
fn module_name_accepts_mui_icon_aliases() {
    for (source, expected) in [
        ("mui:Add", "Add"),
        ("mui/Add", "Add"),
        ("@mui/icons-material/Search.js", "Search"),
        ("icons-material/Menu", "Menu"),
        ("Delete", "Delete"),
        ("folder", "Folder"),
        ("add_circle", "AddCircle"),
    ] {
        assert_eq!(module_name(source), Some(expected.to_string()));
    }
    assert_eq!(module_name("folder-open-outline"), None);
}

#[test]
fn module_tree_is_shared_across_raster_variants() {
    let path = unique_module_source("shared-tree");
    std::fs::write(&path, r#"jsx("path", { d: "M1 1h2v2H1z" })"#)
        .expect("write MUI module fixture");

    let first = load_module_tree(&path).expect("parse first tree");
    let second = load_module_tree(&path).expect("reuse cached tree");

    assert!(std::sync::Arc::ptr_eq(&first, &second));
    let _ = std::fs::remove_file(path);
}

fn unique_module_source(label: &str) -> std::path::PathBuf {
    static NEXT_SOURCE: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);
    let sequence = NEXT_SOURCE.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    std::env::temp_dir().join(format!(
        "zircon-mui-icon-{label}-{}-{sequence}.js",
        std::process::id()
    ))
}
