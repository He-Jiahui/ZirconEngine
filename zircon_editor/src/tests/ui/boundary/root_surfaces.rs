#[test]
fn editor_crate_root_stops_flattening_asset_editor_and_workbench_specialists() {
    let crate_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("lib.rs");
    let source = std::fs::read_to_string(crate_root).expect("editor crate root");

    for forbidden in [
        "pub struct EditorModule",
        "impl EngineModule for EditorModule",
        "pub use ui::asset_editor::{",
        "pub use ui::workbench::autolayout::{",
        "pub use ui::workbench::event::{",
        "pub use ui::workbench::fixture::{",
        "pub use ui::workbench::layout::{",
        "pub use ui::workbench::model::{",
        "pub use ui::workbench::project::{",
        "pub use ui::workbench::reflection::{",
        "pub use ui::workbench::snapshot::{",
        "pub use ui::workbench::startup::{",
        "pub use ui::workbench::view::{",
    ] {
        assert!(
            !source.contains(forbidden),
            "expected zircon_editor crate root to stop flattening specialist surface `{forbidden}`"
        );
    }
}

#[test]
fn editor_ui_root_stops_flattening_binding_asset_editor_control_and_template_specialists() {
    let ui_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("ui")
        .join("mod.rs");
    let source = std::fs::read_to_string(ui_root).expect("editor ui root");

    for forbidden in [
        "pub use asset_editor::{",
        "pub use binding::{",
        "pub use control::{",
        "pub use template::{",
    ] {
        assert!(
            !source.contains(forbidden),
            "expected zircon_editor ui root to stop flattening specialist surface `{forbidden}`"
        );
    }
}
