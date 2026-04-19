use crate::scene::viewport::{RenderFrameExtract, RenderWorldSnapshotHandle};
use zircon_runtime::scene::world::World;

pub(super) fn test_extract() -> RenderFrameExtract {
    RenderFrameExtract::from_snapshot(
        RenderWorldSnapshotHandle::new(7),
        World::new().to_render_snapshot(),
    )
}
