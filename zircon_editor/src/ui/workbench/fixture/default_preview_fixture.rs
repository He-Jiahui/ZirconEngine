use crate::{ViewDescriptor, ViewInstance, WorkbenchLayout};

use super::constants::{
    DEFAULT_DESCRIPTORS_JSON, DEFAULT_EDITOR_DATA_JSON, DEFAULT_INSTANCES_JSON, DEFAULT_LAYOUT_JSON,
};
use super::{
    ensure_ui_asset_descriptor::ensure_ui_asset_descriptor, PreviewEditorData, PreviewFixture,
};

pub fn default_preview_fixture() -> PreviewFixture {
    let mut descriptors: Vec<ViewDescriptor> =
        serde_json::from_str(DEFAULT_DESCRIPTORS_JSON).expect("preview view descriptors fixture");
    ensure_ui_asset_descriptor(&mut descriptors);

    PreviewFixture {
        layout: serde_json::from_str::<WorkbenchLayout>(DEFAULT_LAYOUT_JSON)
            .expect("preview layout fixture"),
        descriptors,
        instances: serde_json::from_str::<Vec<ViewInstance>>(DEFAULT_INSTANCES_JSON)
            .expect("preview view instances fixture"),
        editor: serde_json::from_str::<PreviewEditorData>(DEFAULT_EDITOR_DATA_JSON)
            .expect("preview editor data fixture"),
    }
}
