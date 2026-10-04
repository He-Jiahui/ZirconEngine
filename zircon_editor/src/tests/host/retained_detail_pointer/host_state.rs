// 从共享滚动宿主状态读取尺寸与偏移，约束多个详情表面使用同一滚动权威。
use crate::ui::retained_host::detail_pointer::inspector_scroll_layout;
use crate::ui::retained_host::scroll_surface_host::ScrollSurfaceHostState;
use zircon_runtime_interface::ui::layout::{UiPoint, UiSize};

#[test]
fn scroll_surface_host_state_tracks_size_and_shared_scroll_offset() {
    let mut host = ScrollSurfaceHostState::new();
    host.set_size(UiSize::new(240.0, 96.0));
    host.sync(inspector_scroll_layout(host.size()));

    assert!(host.handle_scroll(UiPoint::new(108.0, 44.0), 120.0));

    assert!(host.scroll_offset() > 0.0);
}
