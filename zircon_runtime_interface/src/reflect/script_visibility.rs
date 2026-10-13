use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
/// 类型是否可投影给脚本宿主；该标记独立于 Editor 与远程 schema 可见性。
pub enum ReflectScriptVisibility {
    /// 不加入脚本可见的类型投影。
    #[default]
    Private,
    /// 可由脚本类型注册路径投影；仍需满足脚本宿主的其他类型/角色条件。
    Public,
}
