use serde::{Deserialize, Serialize};

use crate::ViewInstanceId;

use super::TabInsertionAnchor;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct TabStackLayout {
    pub tabs: Vec<ViewInstanceId>,
    pub active_tab: Option<ViewInstanceId>,
}

impl TabStackLayout {
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

    pub(crate) fn remove(&mut self, instance_id: &ViewInstanceId) -> bool {
        let before = self.tabs.len();
        self.tabs.retain(|current| current != instance_id);
        if self.active_tab.as_ref() == Some(instance_id) {
            self.active_tab = self.tabs.last().cloned();
        }
        before != self.tabs.len()
    }
}
