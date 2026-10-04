use std::any::Any;
use std::sync::Arc;
use std::time::{Duration, Instant};

use crate::core::asset::{
    DirtyExternalEffectId, DirtyRegistry, SaveDirtyViewCandidate, SaveDirtyViewCompletion,
    SaveDirtyViewExecutor, SaveDirtyViewsRequest,
};
use crate::core::editing::engine::{
    EditCommandError, EditContext, EditWorldRoute, EditorTransactionEngine, SelectionSnapshot,
};
use crate::core::editor_message::DocumentId;
use crate::core::extension::{
    DocumentAutosavePayload, DocumentToolkit, DocumentToolkitDescriptor, DocumentToolkitRegistry,
    SaveCtx, ToolkitInstanceId, ToolkitLayout, ToolkitSaveFailure,
};
use crate::core::jobs::{test_job_system, JobContext};
use crate::core::play::WorldDomain;

use super::{
    DirtyDocumentSaveOwner, DirtyDocumentSaveOwnership, DirtyDocumentSaveStart,
    EditorDirtySaveCoordinator, EditorDirtySaveError,
};

#[test]
fn dirty_save_owner_rejects_competitors_until_the_exact_owner_releases() {
    let mut ownership = DirtyDocumentSaveOwnership::default();

    assert_eq!(
        ownership.try_acquire(DirtyDocumentSaveOwner::ClosePrompt),
        DirtyDocumentSaveStart::Scheduled
    );
    assert_eq!(
        ownership.try_acquire(DirtyDocumentSaveOwner::SaveAll),
        DirtyDocumentSaveStart::Busy {
            owner: DirtyDocumentSaveOwner::ClosePrompt,
        }
    );
    assert!(matches!(
        ownership.release(DirtyDocumentSaveOwner::SaveAll),
        Err(EditorDirtySaveError::OwnerMismatch {
            expected: "the close prompt",
            received: "Save All",
        })
    ));
    assert_eq!(ownership.owner(), Some(DirtyDocumentSaveOwner::ClosePrompt));

    ownership
        .release(DirtyDocumentSaveOwner::ClosePrompt)
        .unwrap();
    assert_eq!(
        ownership.try_acquire(DirtyDocumentSaveOwner::SaveAll),
        DirtyDocumentSaveStart::Scheduled
    );
}

#[test]
fn wrong_owner_poll_cannot_consume_a_real_completed_save_batch() {
    let transactions = Arc::new(EditorTransactionEngine::new(TestEditContext::default()));
    let dirty = DirtyRegistry::new(Arc::clone(&transactions));
    let document = DocumentId::new(1);
    let instance = ToolkitInstanceId::parse("view.asset.owner_poll").unwrap();
    dirty.register_document(document).unwrap();
    dirty
        .mark_external_effect(
            document,
            DirtyExternalEffectId::parse("ui.owner_poll").unwrap(),
        )
        .unwrap();
    let toolkits = DocumentToolkitRegistry::<()>::default();
    toolkits
        .register(Arc::new(TestToolkit {
            descriptor: DocumentToolkitDescriptor::new(
                document,
                instance.clone(),
                "Owner poll fixture",
                ToolkitLayout::single_tab("layout.owner_poll", "tab.owner_poll").unwrap(),
            ),
        }))
        .unwrap();
    let request = SaveDirtyViewsRequest::prepare(
        &toolkits.snapshot(),
        [SaveDirtyViewCandidate::new(
            dirty.snapshot(document).unwrap(),
            instance,
            dirty.capture_save_token(document).unwrap(),
            "E:/ZirconEngineTests/owner-poll.zdoc",
            1,
        )],
    )
    .unwrap();
    let executor: Arc<dyn SaveDirtyViewExecutor> = Arc::new(
        |_: &crate::core::asset::SaveDirtyViewIntent, _: &JobContext| {
            SaveDirtyViewCompletion::Saved { written_bytes: 1 }
        },
    );
    let mut coordinator = EditorDirtySaveCoordinator::new(test_job_system());
    assert_eq!(
        coordinator
            .schedule(DirtyDocumentSaveOwner::ClosePrompt, request, executor)
            .unwrap(),
        DirtyDocumentSaveStart::Scheduled
    );

    assert!(matches!(
        coordinator.poll(
            DirtyDocumentSaveOwner::SaveAll,
            &dirty,
            transactions.as_ref(),
        ),
        Err(EditorDirtySaveError::OwnerMismatch {
            expected: "the close prompt",
            received: "Save All",
        })
    ));
    assert_eq!(
        coordinator.owner(),
        Some(DirtyDocumentSaveOwner::ClosePrompt)
    );

    let deadline = Instant::now() + Duration::from_secs(2);
    loop {
        match coordinator
            .poll(
                DirtyDocumentSaveOwner::ClosePrompt,
                &dirty,
                transactions.as_ref(),
            )
            .unwrap()
        {
            Some(result) => {
                assert!(result.all_saved());
                break;
            }
            None if Instant::now() < deadline => std::thread::yield_now(),
            None => panic!("owned save batch did not terminalize"),
        }
    }
    assert_eq!(coordinator.owner(), None);
}

#[derive(Default)]
struct TestEditContext;

impl EditContext for TestEditContext {
    fn capture_world_route(
        &self,
        world_domain: WorldDomain,
    ) -> Result<EditWorldRoute, EditCommandError> {
        Ok(EditWorldRoute::logical(world_domain))
    }

    fn activate_world_route(&mut self, _route: &EditWorldRoute) -> Result<(), EditCommandError> {
        Ok(())
    }

    fn retire_world_route(&mut self, _world_domain: WorldDomain) -> Result<(), EditCommandError> {
        Ok(())
    }

    fn selection_snapshot(&self) -> SelectionSnapshot {
        SelectionSnapshot::default()
    }

    fn restore_selection(&mut self, _snapshot: &SelectionSnapshot) -> Result<(), EditCommandError> {
        Ok(())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

struct TestToolkit {
    descriptor: DocumentToolkitDescriptor,
}

impl DocumentToolkit<()> for TestToolkit {
    fn descriptor(&self) -> &DocumentToolkitDescriptor {
        &self.descriptor
    }

    fn validate_references(&self, _host: &()) -> Result<(), ToolkitSaveFailure> {
        Ok(())
    }

    fn save(&self, _host: &(), _context: &mut SaveCtx) -> Result<(), ToolkitSaveFailure> {
        Ok(())
    }

    fn autosave_source_path(&self, _host: &()) -> Result<std::path::PathBuf, ToolkitSaveFailure> {
        Ok("E:/ZirconEngineTests/owner-poll.zdoc".into())
    }

    fn capture_autosave(&self, _host: &()) -> Result<DocumentAutosavePayload, ToolkitSaveFailure> {
        Ok(DocumentAutosavePayload::new(
            "E:/ZirconEngineTests/owner-poll.zdoc",
            Vec::new(),
        ))
    }
}
