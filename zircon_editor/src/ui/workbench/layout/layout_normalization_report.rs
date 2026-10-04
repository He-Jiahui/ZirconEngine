use serde::{Deserialize, Serialize};

use crate::ui::workbench::view::ViewInstanceId;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
/// 布局规范化报告；当前占位列表为空，不能将其当作缺失视图已验证的证明。
pub struct LayoutNormalizationReport {
    pub placeholders: Vec<ViewInstanceId>,
    pub removed_missing_active_tabs: usize,
}
