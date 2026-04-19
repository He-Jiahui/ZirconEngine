use serde::{Deserialize, Serialize};
use zircon_ui::binding::UiBindingValue;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct InspectorFieldChange {
    pub field_id: String,
    pub value: UiBindingValue,
}
