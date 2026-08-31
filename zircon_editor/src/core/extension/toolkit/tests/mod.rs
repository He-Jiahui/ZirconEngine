mod lifecycle;
mod saving;

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use crate::core::editing::engine::HistoryContextId;
use crate::core::editor_message::DocumentId;

use super::{
    DocumentAutosavePayload, DocumentToolkit, DocumentToolkitDescriptor, SaveCtx,
    ToolkitInstanceId, ToolkitLayout, ToolkitSaveFailure,
};

struct FixtureToolkit {
    descriptor: DocumentToolkitDescriptor,
    validate_references: Arc<dyn Fn() -> Result<(), ToolkitSaveFailure> + Send + Sync>,
    save: Arc<dyn Fn(&mut SaveCtx) -> Result<(), ToolkitSaveFailure> + Send + Sync>,
    descriptor_calls: Option<Arc<AtomicUsize>>,
    drop_callback: Option<Arc<dyn Fn() + Send + Sync>>,
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
            validate_references: Arc::new(|| Ok(())),
            save: Arc::new(save),
            descriptor_calls: None,
            drop_callback: None,
        }
    }

    fn with_descriptor_counter(mut self, descriptor_calls: Arc<AtomicUsize>) -> Self {
        self.descriptor_calls = Some(descriptor_calls);
        self
    }

    fn with_reference_validation(
        mut self,
        validate: impl Fn() -> Result<(), ToolkitSaveFailure> + Send + Sync + 'static,
    ) -> Self {
        self.validate_references = Arc::new(validate);
        self
    }

    fn with_drop_callback(mut self, drop_callback: impl Fn() + Send + Sync + 'static) -> Self {
        self.drop_callback = Some(Arc::new(drop_callback));
        self
    }
}

impl Drop for FixtureToolkit {
    fn drop(&mut self) {
        if let Some(callback) = &self.drop_callback {
            callback();
        }
    }
}

impl DocumentToolkit<()> for FixtureToolkit {
    fn descriptor(&self) -> &DocumentToolkitDescriptor {
        if let Some(descriptor_calls) = &self.descriptor_calls {
            descriptor_calls.fetch_add(1, Ordering::Relaxed);
        }
        &self.descriptor
    }

    fn validate_references(&self, _host: &()) -> Result<(), ToolkitSaveFailure> {
        (self.validate_references)()
    }

    fn save(&self, _host: &(), context: &mut SaveCtx) -> Result<(), ToolkitSaveFailure> {
        (self.save)(context)
    }

    fn autosave_source_path(&self, _host: &()) -> Result<std::path::PathBuf, ToolkitSaveFailure> {
        Ok("fixture.zdoc".into())
    }

    fn capture_autosave(&self, _host: &()) -> Result<DocumentAutosavePayload, ToolkitSaveFailure> {
        Ok(DocumentAutosavePayload::new("fixture.zdoc", Vec::new()))
    }
}

fn assert_document_history(descriptor: &DocumentToolkitDescriptor) {
    assert_eq!(
        descriptor.history_context(),
        HistoryContextId::Document(descriptor.document_id())
    );
}
