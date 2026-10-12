use serde::{Deserialize, Serialize};

use super::ProjectPluginSelection;

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
/// 项目保存的插件选择清单；启动和导出会按目标模式解析它，列表本身仍保留禁用项供编辑器修改。
pub struct ProjectPluginManifest {
    #[serde(default)]
    pub selections: Vec<ProjectPluginSelection>,
}
