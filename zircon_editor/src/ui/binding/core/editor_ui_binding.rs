use serde::{Deserialize, Serialize};
use zircon_runtime::ui::binding::UiEventPath;

use super::EditorUiBindingPayload;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct EditorUiBinding {
    pub(crate) path: UiEventPath,
    pub(crate) payload: EditorUiBindingPayload,
}
