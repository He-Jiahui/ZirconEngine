use std::sync::Arc;

use crate::core::editing::engine::HistoryContextId;
use crate::core::editor_message::DocumentId;
use crate::core::extension::DefaultWorkbenchPreset;

use super::{ToolkitInstanceId, ToolkitLayout};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DocumentToolkitDescriptor {
    document: DocumentId,
    instance: ToolkitInstanceId,
    title: String,
    layout: ToolkitLayout,
    default_presets: Arc<[DefaultWorkbenchPreset]>,
}

impl DocumentToolkitDescriptor {
    pub fn new(
        document: DocumentId,
        instance: ToolkitInstanceId,
        title: impl Into<String>,
        layout: ToolkitLayout,
    ) -> Self {
        Self {
            document,
            instance,
            title: title.into(),
            layout,
            default_presets: Arc::from([DefaultWorkbenchPreset::Authoring]),
        }
    }

    pub const fn document_id(&self) -> DocumentId {
        self.document
    }

    pub fn instance_id(&self) -> &ToolkitInstanceId {
        &self.instance
    }

    pub fn title(&self) -> &str {
        &self.title
    }

    pub fn layout(&self) -> &ToolkitLayout {
        &self.layout
    }

    pub fn with_default_presets(
        mut self,
        presets: impl IntoIterator<Item = DefaultWorkbenchPreset>,
    ) -> Self {
        self.default_presets = DefaultWorkbenchPreset::normalize(presets).into();
        self
    }

    pub fn default_presets(&self) -> &[DefaultWorkbenchPreset] {
        &self.default_presets
    }

    pub const fn history_context(&self) -> HistoryContextId {
        HistoryContextId::Document(self.document)
    }
}
