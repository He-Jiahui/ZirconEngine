#[test]
fn prepared_surface_target_discards_unpresented_leases() {
    let source = include_str!("../native_surface_recording.rs");
    let source = source.split("mod tests {").next().unwrap();

    assert!(source.contains("pub struct WgpuNativeSurfaceFrameTarget"));
    assert!(source.contains("impl Drop for WgpuNativeSurfaceFrameTarget"));
    assert!(source.contains("std::ptr::eq(Arc::as_ptr(&self.owner), owner)"));
    assert!(source.contains("self.validate_owner(owner).map_err(E::from)?"));
    assert!(source.contains("record(&owner.device, &self.target_view, encoder)"));
    assert!(!source.contains("pub fn target_view"));
    assert!(source.contains("self.owner.discard_surface_frame(frame)"));
    assert!(source.contains("self.owner.present_surface_frame(frame, submission)?"));
    assert!(source.contains("self.owner.discard_surface_frame(frame)?"));
    assert!(source.contains("self.frame.take();"));
    assert!(source.contains("RhiError::SurfaceFrameCleanupFailed"));
}
