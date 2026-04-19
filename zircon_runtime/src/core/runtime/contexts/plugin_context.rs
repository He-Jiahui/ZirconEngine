use super::super::weak::CoreWeak;

#[derive(Clone, Debug)]
pub struct PluginContext {
    pub plugin_name: String,
    pub core: CoreWeak,
}
