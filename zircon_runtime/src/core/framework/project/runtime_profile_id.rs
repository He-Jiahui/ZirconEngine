use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
/// 项目与导出配置中的产品组合标识；入口层据此选择实际模块集合，不能仅凭名称推断已启用插件。
pub enum RuntimeProfileId {
    Minimal,
    Client2d,
    Client3d,
    Editor,
    Dev,
    Server,
}
