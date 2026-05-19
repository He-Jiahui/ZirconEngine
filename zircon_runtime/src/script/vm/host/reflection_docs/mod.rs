mod markdown;
mod options;
mod writer;

pub use markdown::render_script_host_modules_markdown;
pub use options::ScriptHostInterfaceMarkdownOptions;
pub use writer::write_script_host_modules_markdown;

use super::{register_builtin_host_modules, HostExportRegistry, HostRegistry};
use crate::core::framework::script::ScriptHostModuleDescriptor;
use crate::script::VmError;

pub fn builtin_host_module_descriptors() -> Result<Vec<ScriptHostModuleDescriptor>, VmError> {
    let registry = HostRegistry::default();
    let exports = HostExportRegistry::new(registry.clone());
    register_builtin_host_modules(&exports, &registry)?;
    Ok(exports
        .modules()
        .into_iter()
        .map(|record| record.descriptor)
        .collect())
}
