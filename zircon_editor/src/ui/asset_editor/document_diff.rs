use std::sync::Arc;

use zircon_runtime_interface::ui::template::UiAssetDocument;

#[cfg(test)]
#[path = "document_diff/tests/shared_target_tests.rs"]
mod shared_target_tests;

#[derive(Clone, Debug, Default, PartialEq)]
pub(super) struct UiAssetDocumentDiff {
    target: Option<Arc<UiAssetDocument>>,
}

impl UiAssetDocumentDiff {
    pub fn between(current: &UiAssetDocument, target: &UiAssetDocument) -> Self {
        Self {
            target: (current != target).then(|| Arc::new(target.clone())),
        }
    }

    pub fn apply_to(&self, document: &mut UiAssetDocument) -> bool {
        let Some(target) = &self.target else {
            return false;
        };
        if &*document == target.as_ref() {
            return false;
        }
        *document = target.as_ref().clone();
        true
    }
}

#[cfg(test)]
#[path = "tests/document_diff.rs"]
mod tests;
