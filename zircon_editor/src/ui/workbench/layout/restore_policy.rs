use serde::{Deserialize, Serialize};

use super::WorkbenchLayout;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
/// 已加载布局的选择次序；文件装载与视图实例恢复由宿主先后组织。
pub enum RestorePolicy {
    ProjectThenGlobal,
    PresetThenProjectThenGlobal { preset: Option<WorkbenchLayout> },
}
