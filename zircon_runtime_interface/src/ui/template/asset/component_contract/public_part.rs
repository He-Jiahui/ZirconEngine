use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
/// 显式导出的部件把外部名称映射到组件树节点或控件 ID，供样式、焦点和动作契约引用。
pub struct UiPublicPart {
    #[serde(default)]
    pub node_id: String,
    #[serde(default)]
    pub control_id: Option<String>,
}
