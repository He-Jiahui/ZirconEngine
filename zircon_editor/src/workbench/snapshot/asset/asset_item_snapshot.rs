use zircon_manager::ResourceStateRecord;
use zircon_resource::ResourceKind;

#[derive(Clone, Debug)]
pub struct AssetItemSnapshot {
    pub uuid: String,
    pub locator: String,
    pub display_name: String,
    pub file_name: String,
    pub extension: String,
    pub kind: ResourceKind,
    pub preview_artifact_path: String,
    pub dirty: bool,
    pub diagnostics: Vec<String>,
    pub selected: bool,
    pub resource_state: Option<ResourceStateRecord>,
    pub resource_revision: Option<u64>,
}
