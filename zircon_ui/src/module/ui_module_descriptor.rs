use zircon_module::{stub_module_descriptor, ModuleDescriptor};

use super::UI_MODULE_NAME;

pub fn module_descriptor() -> ModuleDescriptor {
    stub_module_descriptor(
        UI_MODULE_NAME,
        "Runtime UI widgets and layout",
        "UiDriver",
        "UiManager",
    )
}
