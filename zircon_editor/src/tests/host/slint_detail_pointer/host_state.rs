use crate::ui::slint_host::detail_pointer::inspector_scroll_layout;
use crate::ui::slint_host::scroll_surface_host::ScrollSurfaceHostState;
use zircon_runtime_interface::ui::layout::{UiPoint, UiSize};

#[test]
fn scroll_surface_host_state_tracks_size_and_shared_scroll_offset() {
    let mut host =
        ScrollSurfaceHostState::new("zircon.editor.inspector.pointer", "editor.inspector");
    host.set_size(UiSize::new(240.0, 96.0));
    host.sync(inspector_scroll_layout(host.size()));

    host.handle_scroll(UiPoint::new(108.0, 44.0), 120.0)
        .expect("host state should route inspector scroll through shared surface");

    assert!(host.scroll_offset() > 0.0);
}
