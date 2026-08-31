use std::sync::Arc;

use crate::core::editing::engine::HistoryContextId;
use crate::core::editor_extension::EditorMenuItemDescriptor;
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
    menu_items: Arc<[EditorMenuItemDescriptor]>,
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
            menu_items: Arc::from([]),
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

    pub fn with_menu_items(
        mut self,
        menu_items: impl IntoIterator<Item = EditorMenuItemDescriptor>,
    ) -> Self {
        self.menu_items = menu_items.into_iter().collect::<Vec<_>>().into();
        self
    }

    pub fn menu_items(&self) -> &[EditorMenuItemDescriptor] {
        &self.menu_items
    }

    pub const fn history_context(&self) -> HistoryContextId {
        HistoryContextId::Document(self.document)
    }
}
