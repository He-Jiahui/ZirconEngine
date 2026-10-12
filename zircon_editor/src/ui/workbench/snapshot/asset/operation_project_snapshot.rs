#[derive(Clone, Debug, Default, PartialEq, Eq)]
/// 类型注册命令的展示与路由快照；类型内命令ID和执行operation路径分开，执行仍由操作中心验证。
pub struct AssetOperationProjectionSnapshot {
    pub asset_type_id: String,
    pub id: String,
    pub display_name: String,
    pub operation_id: String,
    pub icon_name: Option<String>,
    pub default_document: Option<String>,
}
