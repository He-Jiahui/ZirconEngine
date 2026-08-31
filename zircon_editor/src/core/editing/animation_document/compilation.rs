//! Compiler currentness and last-known-good ownership for animation documents.

use zircon_runtime::core::framework::animation::compiler::AnimationCompileProduct;

use super::{AnimationAuthoringAsset, AnimationDocumentRevision};

/// Core-owned compiler state for one source document revision.
///
/// Invalid intermediate authoring source remains editable and undoable. Consumers that require an
/// executable topology must use `last_good_product` until the current revision compiles again.
#[derive(Clone, Debug)]
pub(crate) struct AnimationDocumentCompilation {
    current_revision: AnimationDocumentRevision,
    current_product: AnimationCompileProduct,
    last_good: Option<AnimationDocumentLastGoodCompilation>,
}

#[derive(Clone, Debug)]
struct AnimationDocumentLastGoodCompilation {
    revision: AnimationDocumentRevision,
    product: AnimationCompileProduct,
}

impl AnimationDocumentCompilation {
    pub(crate) fn new(
        revision: AnimationDocumentRevision,
        asset: &AnimationAuthoringAsset,
    ) -> Self {
        let product = asset.compile();
        let last_good = product
            .is_successful()
            .then(|| AnimationDocumentLastGoodCompilation {
                revision,
                product: product.clone(),
            });
        Self {
            current_revision: revision,
            current_product: product,
            last_good,
        }
    }

    pub(crate) fn recompile(
        &mut self,
        revision: AnimationDocumentRevision,
        asset: &AnimationAuthoringAsset,
    ) {
        let product = asset.compile();
        if product.is_successful() {
            self.last_good = Some(AnimationDocumentLastGoodCompilation {
                revision,
                product: product.clone(),
            });
        }
        self.current_revision = revision;
        self.current_product = product;
    }

    pub(crate) const fn current_revision(&self) -> AnimationDocumentRevision {
        self.current_revision
    }

    pub(crate) fn current_product(&self) -> &AnimationCompileProduct {
        &self.current_product
    }

    pub(crate) fn last_good_product(&self) -> Option<&AnimationCompileProduct> {
        self.last_good.as_ref().map(|last_good| &last_good.product)
    }

    pub(crate) fn last_good_revision(&self) -> Option<AnimationDocumentRevision> {
        self.last_good.as_ref().map(|last_good| last_good.revision)
    }
}
