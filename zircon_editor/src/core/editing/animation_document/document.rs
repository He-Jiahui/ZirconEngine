use std::sync::{Arc, RwLock, RwLockReadGuard};

use zircon_runtime::asset::AssetUri;

use crate::core::editor_message::DocumentId;

use super::{
    AnimationAuthoringAsset, AnimationAuthoringDocumentError, AnimationAuthoringDocumentKind,
    AnimationDocumentCompilation, AnimationDocumentRevision,
};

/// The sole mutable source for one animation authoring document.
///
/// Its read handle is intentionally separate so UI projections cannot mutate source outside an
/// edit command and its `HistoryContextId::Document` transaction.
#[derive(Debug)]
pub(crate) struct AnimationAuthoringDocument {
    id: DocumentId,
    asset_locator: AssetUri,
    revision: AnimationDocumentRevision,
    asset: AnimationAuthoringAsset,
    compilation: AnimationDocumentCompilation,
}

impl AnimationAuthoringDocument {
    pub(crate) fn new(
        id: DocumentId,
        asset_locator: AssetUri,
        asset: AnimationAuthoringAsset,
    ) -> Self {
        let revision = AnimationDocumentRevision::INITIAL;
        let compilation = AnimationDocumentCompilation::new(revision, &asset);
        Self {
            id,
            asset_locator,
            revision,
            asset,
            compilation,
        }
    }

    pub(crate) const fn id(&self) -> DocumentId {
        self.id
    }

    pub(crate) fn asset_locator(&self) -> &AssetUri {
        &self.asset_locator
    }

    pub(crate) const fn kind(&self) -> AnimationAuthoringDocumentKind {
        self.asset.kind()
    }

    pub(crate) const fn revision(&self) -> AnimationDocumentRevision {
        self.revision
    }

    pub(crate) fn asset(&self) -> &AnimationAuthoringAsset {
        &self.asset
    }

    pub(crate) fn compilation(&self) -> &AnimationDocumentCompilation {
        &self.compilation
    }

    pub(crate) fn document_bytes(&self) -> Result<Vec<u8>, AnimationAuthoringDocumentError> {
        self.asset.to_bytes()
    }

    pub(crate) fn swap_asset_if_revision(
        &mut self,
        expected_revision: AnimationDocumentRevision,
        replacement: &mut AnimationAuthoringAsset,
    ) -> Result<AnimationDocumentRevision, AnimationAuthoringDocumentError> {
        if self.revision != expected_revision {
            return Err(AnimationAuthoringDocumentError::stale_revision(
                self.id.value(),
                expected_revision,
                self.revision,
            ));
        }
        if replacement.kind() != self.kind() {
            return Err(AnimationAuthoringDocumentError::wrong_kind(
                self.kind(),
                replacement.kind(),
            ));
        }
        let next_revision = self.revision.next()?;
        std::mem::swap(&mut self.asset, replacement);
        self.revision = next_revision;
        self.compilation.recompile(self.revision, &self.asset);
        Ok(self.revision)
    }
}

#[derive(Clone, Debug)]
pub(crate) struct AnimationAuthoringDocumentReadHandle {
    id: DocumentId,
    kind: AnimationAuthoringDocumentKind,
    document: Arc<RwLock<AnimationAuthoringDocument>>,
}

impl AnimationAuthoringDocumentReadHandle {
    pub(crate) fn document_id(&self) -> DocumentId {
        self.id
    }

    pub(crate) const fn kind(&self) -> AnimationAuthoringDocumentKind {
        self.kind
    }

    pub(crate) fn read(&self) -> RwLockReadGuard<'_, AnimationAuthoringDocument> {
        self.document
            .read()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    pub(super) fn from_document(document: Arc<RwLock<AnimationAuthoringDocument>>) -> Self {
        let read = document
            .read()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let id = read.id();
        let kind = read.kind();
        drop(read);
        Self { id, kind, document }
    }

    #[cfg(test)]
    pub(crate) fn detached_for_test(document: AnimationAuthoringDocument) -> Self {
        Self::from_document(Arc::new(RwLock::new(document)))
    }
}
