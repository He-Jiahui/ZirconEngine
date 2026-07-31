use std::sync::Arc;

use super::ViewportOverlayPointerRouter;

impl Clone for ViewportOverlayPointerRouter {
    fn clone(&self) -> Self {
        let mut clone = Self::new();
        clone.sync(self.layout.clone());
        clone.interaction_extract = self.interaction_extract.as_ref().map(Arc::clone);
        clone
    }
}
