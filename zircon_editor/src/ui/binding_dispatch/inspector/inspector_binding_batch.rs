use serde::{Deserialize, Serialize};
use crate::ui::InspectorFieldChange;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct InspectorBindingBatch {
    pub subject_path: String,
    pub changes: Vec<InspectorFieldChange>,
}
