use crate::plugin::RuntimeExtensionRegistry;

#[derive(Clone, Debug)]
pub struct RuntimeExtensionCatalogReport {
    pub registry: RuntimeExtensionRegistry,
    pub diagnostics: Vec<String>,
    pub fatal_diagnostics: Vec<String>,
}
