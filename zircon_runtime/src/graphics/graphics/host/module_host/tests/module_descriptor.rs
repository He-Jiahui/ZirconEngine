use crate::core::framework::render::GRAPHICS_MODULE_NAME;
use crate::text::TEXT_MODULE_NAME;

use super::module_descriptor;

#[test]
fn graphics_render_framework_depends_on_core_owned_text_font_services() {
    let descriptor = module_descriptor();

    assert!(descriptor
        .module_dependencies
        .iter()
        .any(|dependency| dependency.module_name == TEXT_MODULE_NAME));
    let render_framework = descriptor
        .managers
        .iter()
        .find(|manager| manager.name.to_string() == "GraphicsModule.Manager.RenderFramework")
        .expect("graphics render framework manager");
    assert!(render_framework
        .dependencies
        .iter()
        .any(|dependency| { dependency.name.to_string() == "TextModule.Manager.FontServices" }));
    assert_eq!(descriptor.name, GRAPHICS_MODULE_NAME);
}
