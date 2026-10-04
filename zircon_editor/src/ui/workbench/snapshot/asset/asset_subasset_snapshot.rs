use zircon_runtime_interface::resource::ResourceKind;

use super::AssetTypeProjectionSnapshot;

#[derive(Clone, Debug)]
/// source资产包含的子资源身份和导入产物信息；产物locator可缺席，不能据此推断资源已加载。
pub struct AssetSubassetSnapshot {
    pub uuid: String,
    pub locator: String,
    pub kind: ResourceKind,
    pub asset_type: AssetTypeProjectionSnapshot,
    pub artifact_locator: Option<String>,
    pub dependency_locators: Vec<String>,
}
