use super::super::ids::RuntimePluginId;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeRequiredPluginMissing {
    pub id: RuntimePluginId,
    pub reason: String,
}
