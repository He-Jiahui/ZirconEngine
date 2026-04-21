use super::ViewportOverlayPointerRouter;

impl Clone for ViewportOverlayPointerRouter {
    fn clone(&self) -> Self {
        let mut clone = Self::new();
        clone.sync(self.layout.clone());
        clone
    }
}
