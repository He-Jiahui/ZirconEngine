use serde::{Deserialize, Serialize};

use crate::ui::workbench::view::ViewInstanceId;

use super::TabInsertionAnchor;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
/// 单个叶空间的tab顺序和活动项；跨工作区实例唯一性由布局命令协调。
pub struct TabStackLayout {
    pub tabs: Vec<ViewInstanceId>,
    pub active_tab: Option<ViewInstanceId>,
}

impl TabStackLayout {
    /// 移动同栈成员时先去重，再按邻接身份插入并激活；不存在的锚点回退末尾。
    pub(crate) fn insert(
        &mut self,
        instance_id: ViewInstanceId,
        anchor: Option<&TabInsertionAnchor>,
    ) {
        self.tabs.retain(|current| current != &instance_id);

        if let Some(anchor) = anchor {
            if let Some(anchor_index) = self
                .tabs
                .iter()
                .position(|current| current == &anchor.target_id)
            {
                let insert_index = match anchor.side {
                    super::TabInsertionSide::Before => anchor_index,
                    super::TabInsertionSide::After => anchor_index + 1,
                };
                self.tabs
                    .insert(insert_index.min(self.tabs.len()), instance_id.clone());
                self.active_tab = Some(instance_id);
                return;
            }
        }

        self.tabs.push(instance_id.clone());
        self.active_tab = Some(instance_id);
    }

    /// 移除成员并修复本栈活动项；返回成员列表是否变化。
    pub(crate) fn remove(&mut self, instance_id: &ViewInstanceId) -> bool {
        let before = self.tabs.len();
        self.tabs.retain(|current| current != instance_id);
        if self.active_tab.as_ref() == Some(instance_id) {
            self.active_tab = self.tabs.last().cloned();
        }
        before != self.tabs.len()
    }
}
