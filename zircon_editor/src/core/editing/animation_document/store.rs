use std::collections::BTreeMap;
use std::sync::{Arc, RwLock, RwLockWriteGuard};

use crate::core::editor_message::DocumentId;

use super::{
    AnimationAuthoringAsset, AnimationAuthoringDocument, AnimationAuthoringDocumentError,
    AnimationAuthoringDocumentReadHandle, AnimationDocumentMutation,
    AnimationDocumentMutationError, AnimationDocumentRevision,
};

#[derive(Default)]
pub(crate) struct AnimationAuthoringDocumentStore {
    documents: BTreeMap<DocumentId, Arc<RwLock<AnimationAuthoringDocument>>>,
}

impl AnimationAuthoringDocumentStore {
    pub(crate) fn attach(
        &mut self,
        document: AnimationAuthoringDocument,
    ) -> Result<AnimationAuthoringDocumentReadHandle, AnimationAuthoringDocumentError> {
        let id = document.id();
        if self.documents.contains_key(&id) {
            return Err(AnimationAuthoringDocumentError::DuplicateDocument {
                document: id.value(),
            });
        }
        let document = Arc::new(RwLock::new(document));
        let handle = AnimationAuthoringDocumentReadHandle::from_document(Arc::clone(&document));
        self.documents.insert(id, document);
        Ok(handle)
    }

    pub(crate) fn detach(&mut self, id: DocumentId) -> bool {
        self.documents.remove(&id).is_some()
    }

    pub(crate) fn document_mut(
        &self,
        id: DocumentId,
    ) -> Result<RwLockWriteGuard<'_, AnimationAuthoringDocument>, AnimationAuthoringDocumentError>
    {
        let document =
            self.documents
                .get(&id)
                .ok_or(AnimationAuthoringDocumentError::MissingDocument {
                    document: id.value(),
                })?;
        Ok(document
            .write()
            .unwrap_or_else(|poisoned| poisoned.into_inner()))
    }

    pub(crate) fn handle(
        &self,
        id: DocumentId,
    ) -> Result<AnimationAuthoringDocumentReadHandle, AnimationAuthoringDocumentError> {
        let document = Arc::clone(self.documents.get(&id).ok_or(
            AnimationAuthoringDocumentError::MissingDocument {
                document: id.value(),
            },
        )?);
        Ok(AnimationAuthoringDocumentReadHandle::from_document(
            document,
        ))
    }

    pub(crate) fn prepare_mutation(
        &self,
        id: DocumentId,
        mutation: &AnimationDocumentMutation,
    ) -> Result<
        Option<(AnimationDocumentRevision, AnimationAuthoringAsset)>,
        AnimationDocumentMutationError,
    > {
        let handle = self.handle(id)?;
        let document = handle.read();
        let revision = document.revision();
        let mut replacement = document.asset().clone();
        drop(document);
        if !mutation.apply(&mut replacement)? {
            return Ok(None);
        }
        Ok(Some((revision, replacement)))
    }
}
