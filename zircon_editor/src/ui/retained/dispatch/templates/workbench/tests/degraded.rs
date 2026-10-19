use super::*;
use crate::ui::template_runtime::RetainedUiHostValue;
use zircon_runtime_interface::ui::layout::UiSize;

#[test]
fn software_degradation_is_persistent_visible_chrome_with_the_actual_reason() {
    let mut bridge = BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(1280.0, 800.0)).unwrap();
    let reason = "Software presenter (backend=softbuffer): test GPU unavailable; PIE may be slow.";
    bridge.prepare_graphics_degradation(Some(reason)).unwrap();
    bridge.refresh_prepared_state_change().unwrap();
    assert!(bridge.control_frame(GRAPHICS_BANNER).is_some());
    let banner = bridge.host_projection().node_by_control_id(GRAPHICS_BANNER).unwrap();
    assert_eq!(banner.properties.get("text"), Some(&RetainedUiHostValue::String(reason.to_owned())));

    // Normal business refreshes cannot dismiss a persistent graphics diagnostic.
    bridge.sync_from_chrome(&crate::ui::workbench::fixture::default_preview_fixture().build_chrome()).unwrap();
    assert!(bridge.control_frame(GRAPHICS_BANNER).is_some());
    bridge.prepare_graphics_degradation(None).unwrap();
    bridge.refresh_prepared_state_change().unwrap();
    assert!(bridge.control_frame(GRAPHICS_BANNER).is_none());
}
