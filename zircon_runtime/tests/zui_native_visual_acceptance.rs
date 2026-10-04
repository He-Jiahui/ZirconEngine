#![cfg(feature = "ui")]

//! Runtime GPU evidence; editor host parity remains a separate acceptance gate.

#[path = "zui_native_visual_acceptance/assets.rs"]
mod assets;
#[path = "zui_native_visual_acceptance/batch.rs"]
mod batch;
#[path = "zui_native_visual_acceptance/catalog.rs"]
mod catalog;
#[path = "zui_native_visual_acceptance/data.rs"]
mod data;
#[path = "zui_native_visual_acceptance/dpi.rs"]
mod dpi;
#[path = "zui_native_visual_acceptance/evidence.rs"]
mod evidence;
#[path = "zui_native_visual_acceptance/preview.rs"]
mod preview;
#[path = "runtime_text_multilingual_product_framebuffer/product_renderer.rs"]
mod product_renderer;
#[path = "zui_native_visual_acceptance/semantic.rs"]
mod semantic;
#[path = "zui_native_visual_acceptance/state.rs"]
mod state;
#[path = "support/project_asset_runtime.rs"]
mod support;
#[path = "zui_native_visual_acceptance/tests.rs"]
mod tests;

#[test]
#[ignore = "exports native WGPU evidence for the generated ZUI layout catalog"]
fn export_all_zui_native_visual_acceptance() {
    batch::export().expect("write native ZUI engine report");
}
