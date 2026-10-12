use zircon_runtime_interface::ui::{binding::UiEventKind, component::UiValue, layout::UiSize};

use super::*;

#[test]
fn pass_graph_and_compile_keep_distinct_state_domains() {
    let mut bridge = BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(900.0, 620.0))
        .expect("workbench bridge should build");

    assert!(bridge.control_bool("WorkbenchRenderLightingPassRow", "selected"));
    assert!(bridge.control_bool("WorkbenchRenderFrameStartRow", "selected"));

    assert!(bridge
        .select_dropdown_option(RENDER_PLATFORM_DROPDOWN, "vulkan")
        .expect("render platform should select"));
    for (control_id, value) in [
        ("WorkbenchRenderPipelineField", "Cinematic.rp"),
        ("WorkbenchRenderFrameField", "2048"),
    ] {
        bridge
            .mutate_control_property(control_id, "value", UiValue::String(value.to_string()))
            .expect("render property should edit");
    }

    bridge
        .dispatch_control_state("WorkbenchRenderBloomPassRow", UiEventKind::Click)
        .expect("bloom pass should dispatch")
        .expect("bloom pass should bind");
    bridge
        .dispatch_control_state("WorkbenchRenderLightingNodeRow", UiEventKind::Click)
        .expect("lighting graph node should dispatch")
        .expect("lighting graph node should bind");
    assert!(bridge.control_bool("WorkbenchRenderBloomPassRow", "selected"));
    assert!(bridge.control_bool("WorkbenchRenderLightingNodeRow", "selected"));

    bridge
        .dispatch_control_state("WorkbenchRenderCompileButton", UiEventKind::Click)
        .expect("render compile should dispatch")
        .expect("render compile should bind");
    assert_eq!(
        Some("Vulkan   frame 2048   Lighting compiled".to_string()),
        bridge.control_string("WorkbenchRenderCaptureRow", "value_text")
    );
    assert_eq!(
        Some("Cinematic.rp / Bloom Pass".to_string()),
        bridge.control_string("WorkbenchRenderCenterTitle", "text")
    );
}
