use std::fs;
use std::path::Path;

fn source(relative: &str) -> String {
    fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join(relative))
        .unwrap_or_else(|error| panic!("read `{relative}`: {error}"))
}

#[test]
fn editor_crate_root_keeps_editor_module_out_of_lib_rs() {
    let source = source("src/lib.rs");

    for forbidden in [
        "pub struct EditorModule",
        "impl EngineModule for EditorModule",
    ] {
        assert!(
            !source.contains(forbidden),
            "expected zircon_editor/src/lib.rs to stay structural, found `{forbidden}`"
        );
    }
}

#[test]
fn slint_host_root_stays_structural_after_rust_contract_cutover() {
    let root = source("src/ui/slint_host/mod.rs");
    let host_contract = source("src/ui/slint_host/host_contract/mod.rs");
    let data_root = source("src/ui/slint_host/host_contract/data/mod.rs");

    for required in ["mod host_contract;", "pub(crate) use host_contract::*;"] {
        assert!(
            root.contains(required),
            "slint_host root missing `{required}`"
        );
    }
    for required in ["mod data;", "mod globals;", "mod window;"] {
        assert!(
            host_contract.contains(required),
            "host contract root missing `{required}`"
        );
    }
    for required in [
        "mod assets;",
        "mod host_components;",
        "mod host_interaction;",
        "mod host_root;",
        "mod panes;",
        "mod template_nodes;",
        "mod ui_asset;",
        "mod welcome;",
    ] {
        assert!(
            data_root.contains(required),
            "host data root missing `{required}`"
        );
    }
}
