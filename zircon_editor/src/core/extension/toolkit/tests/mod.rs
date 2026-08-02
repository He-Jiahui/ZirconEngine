mod lifecycle;
mod saving;

use std::sync::Arc;

use crate::core::editing::engine::HistoryContextId;
use crate::core::editor_message::DocumentId;

use super::{
    DocumentToolkit, DocumentToolkitDescriptor, SaveCtx, ToolkitInstanceId, ToolkitLayout,
    ToolkitSaveFailure,
};

struct FixtureToolkit {
    descriptor: DocumentToolkitDescriptor,
    save: Arc<dyn Fn(&mut SaveCtx) -> Result<(), ToolkitSaveFailure> + Send + Sync>,
}

impl FixtureToolkit {
    fn new(
        document: u64,
        instance: &str,
        save: impl Fn(&mut SaveCtx) -> Result<(), ToolkitSaveFailure> + Send + Sync + 'static,
    ) -> Self {
        let document = DocumentId::new(document);
        Self {
            descriptor: DocumentToolkitDescriptor::new(
                document,
                ToolkitInstanceId::parse(instance).unwrap(),
                format!("Document {document:?}"),
                ToolkitLayout::single_tab(
                    format!("layout.{document:?}"),
                    format!("tab.{document:?}"),
                )
                .unwrap(),
            ),
            save: Arc::new(save),
        }
    }
}

impl DocumentToolkit<()> for FixtureToolkit {
    fn descriptor(&self) -> &DocumentToolkitDescriptor {
        &self.descriptor
    }

    fn save(&self, _host: &(), context: &mut SaveCtx) -> Result<(), ToolkitSaveFailure> {
        (self.save)(context)
    }
}

fn assert_document_history(descriptor: &DocumentToolkitDescriptor) {
    assert_eq!(
        descriptor.history_context(),
        HistoryContextId::Document(descriptor.document_id())
    );
}
