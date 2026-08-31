use super::*;
use zircon_runtime_interface::ui::style::{ButtonColor, UiStyleColor};

const STARSHIP_SECONDARY_SURFACE: [u8; 4] = [29, 35, 40, 255];
const STARSHIP_PRIMARY_SURFACE: [u8; 4] = [18, 57, 65, 255];
const TRANSPARENT: [u8; 4] = [0, 0, 0, 0];

#[test]
fn toolbar_command_buttons_resolve_shared_starship_semantics() {
    let _guard = match env_lock().lock() {
        Ok(guard) => guard,
        Err(error) => panic!("test environment lock is poisoned: {error}"),
    };
    let bridge = BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(
        FULL_WORKBENCH_WIDTH as f32,
        FULL_WORKBENCH_HEIGHT as f32,
    ))
    .expect("full workbench should build");

    for control_id in ["WorkbenchModuleDiff", "WorkbenchModuleSimulate"] {
        let node = workbench_window_node(&bridge, control_id);
        assert_eq!(node.button_variant.as_str(), "outlined");
        assert_eq!(
            style_color_u8(node.button_style.element.background_color.as_ref()),
            Some(STARSHIP_SECONDARY_SURFACE),
            "{control_id} should use the shared secondary button surface"
        );
    }

    for control_id in ["WorkbenchModuleSave", "WorkbenchModuleBrowse"] {
        let node = workbench_window_node(&bridge, control_id);
        assert_eq!(node.button_variant.as_str(), "text");
        assert_eq!(
            style_color_u8(node.button_style.element.background_color.as_ref()),
            Some(TRANSPARENT),
            "{control_id} should remain a quiet icon action"
        );
    }

    let compile = workbench_window_node(&bridge, "WorkbenchModuleCompile");
    assert_eq!(compile.button_variant.as_str(), "filled");
    assert_eq!(
        style_color_u8(compile.button_style.element.background_color.as_ref()),
        Some(STARSHIP_PRIMARY_SURFACE),
        "Compile should use the shared primary button surface"
    );

    let select = workbench_window_node(&bridge, "WorkbenchToolSelect");
    assert!(select.selected, "Select should keep active tool identity");
    assert!(
        select.checked,
        "Select should project checked toggle identity"
    );
    assert!(
        !select.pressed,
        "an idle selected tool must not project the transient pressed state"
    );

    let play = workbench_window_node(&bridge, "WorkbenchRunPlay");
    assert_eq!(play.button_style.color, ButtonColor::Success);
}

fn style_color_u8(color: Option<&UiStyleColor>) -> Option<[u8; 4]> {
    match color? {
        UiStyleColor::Rgba(color) => Some(color.to_u8()),
        UiStyleColor::Transparent => Some([0, 0, 0, 0]),
        UiStyleColor::Role(_) | UiStyleColor::Inherit => None,
    }
}
