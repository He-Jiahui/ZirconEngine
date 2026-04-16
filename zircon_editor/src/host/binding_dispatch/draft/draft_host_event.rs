use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum DraftHostEvent {
    SetInspectorField {
        subject_path: String,
        field_id: String,
        value: String,
    },
    SetMeshImportPath {
        value: String,
    },
}
