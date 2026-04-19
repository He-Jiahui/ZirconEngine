use zircon_framework::render::{RenderFrameExtract, RenderWorldSnapshotHandle};
use zircon_scene::world::World;

pub(super) fn test_extract() -> RenderFrameExtract {
    RenderFrameExtract::from_snapshot(
        RenderWorldSnapshotHandle::new(7),
        World::new().to_render_snapshot(),
    )
}
