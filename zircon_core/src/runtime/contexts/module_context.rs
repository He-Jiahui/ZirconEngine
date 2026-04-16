use super::super::weak::CoreWeak;

#[derive(Clone, Debug)]
pub struct ModuleContext {
    pub module_name: String,
    pub core: CoreWeak,
}
