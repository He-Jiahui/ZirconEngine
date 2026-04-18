use super::ViewportOverlayPointerBridge;

impl Clone for ViewportOverlayPointerBridge {
    fn clone(&self) -> Self {
        let mut clone = Self::new();
        clone.sync(self.layout.clone());
        clone
    }
}
