use serde::{Deserialize, Serialize};

use super::UiRenderList;
use crate::ui::event_ui::UiTreeId;

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct UiRenderExtract {
    pub tree_id: UiTreeId,
    pub list: UiRenderList,
}
