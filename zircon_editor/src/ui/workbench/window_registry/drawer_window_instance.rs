use serde::{Deserialize, Serialize};

use crate::ui::workbench::layout::MainPageId;
use crate::ui::workbench::view::ViewInstanceId;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DrawerWindowInstance {
    pub window_id: MainPageId,
    pub drawer_view: ViewInstanceId,
    pub title: String,
}

impl DrawerWindowInstance {
    pub fn new(
        window_id: MainPageId,
        drawer_view: ViewInstanceId,
        title: impl Into<String>,
    ) -> Self {
        Self {
            window_id,
            drawer_view,
            title: title.into(),
        }
    }
}
