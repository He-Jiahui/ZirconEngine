use zircon_scene::{RenderFrameExtract, RenderWorldSnapshotHandle, World};

pub(super) fn test_extract() -> RenderFrameExtract {
    RenderFrameExtract::from_snapshot(
        RenderWorldSnapshotHandle::new(7),
        World::new().to_render_snapshot(),
    )
}
