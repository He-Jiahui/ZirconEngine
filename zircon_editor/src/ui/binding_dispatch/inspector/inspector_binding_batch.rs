use crate::ui::InspectorFieldChange;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct InspectorBindingBatch {
    pub subject_path: String,
    pub changes: Vec<InspectorFieldChange>,
}
