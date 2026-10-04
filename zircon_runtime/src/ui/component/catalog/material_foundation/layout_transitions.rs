//! 过渡目录提供角色、进入状态、时间和缓动默认值，供支持这些角色的消费方解释。auto 字符串与数值毫秒字段各自保留目录约定，实际计时与挂载由运行时拥有者负责。

use super::shared::*;

pub(super) fn descriptors() -> Vec<UiComponentDescriptor> {
    vec![
        transition("Collapse", "Collapse", "collapse"),
        transition("Fade", "Fade", "fade"),
        transition("Grow", "Grow", "grow"),
        transition("Slide", "Slide", "slide"),
        transition("Zoom", "Zoom", "zoom"),
    ]
}

fn transition(id: &str, display_name: &str, role: &str) -> UiComponentDescriptor {
    let descriptor = transition_props(
        composite(id, display_name, UiComponentCategory::Container, role),
        role,
        transition_timeout_ms(role),
        transition_easing(role),
    )
    .with_prop(bool_prop("appear", true))
    .with_prop(default_string_prop(
        "timeout",
        transition_timeout_prop(role),
    ))
    .with_prop(any_prop("addEndListener"))
    .with_prop(map_prop("style"))
    .slot(UiSlotSchema::new("content").multiple(true));

    match role {
        "collapse" => descriptor
            .with_prop(default_string_prop("component", "div"))
            .with_prop(enum_prop_with_options(
                "orientation",
                "vertical",
                ["horizontal", "vertical"]
                    .into_iter()
                    .map(enum_option_descriptor),
            ))
            .with_prop(default_string_prop("collapsed_size", "0px"))
            .with_prop(default_string_prop("collapsedSize", "0px"))
            .slot(UiSlotSchema::new("wrapper"))
            .slot(UiSlotSchema::new("wrapperInner")),
        "slide" => descriptor
            .with_prop(enum_prop_with_options(
                "direction",
                "down",
                ["down", "left", "right", "up"]
                    .into_iter()
                    .map(enum_option_descriptor),
            ))
            .with_prop(any_prop("container")),
        _ => descriptor,
    }
}

fn transition_timeout_ms(role: &str) -> i64 {
    match role {
        "collapse" => 300,
        "fade" | "slide" | "zoom" => 225,
        "grow" => 225,
        _ => 300,
    }
}

fn transition_timeout_prop(role: &str) -> &'static str {
    match role {
        "grow" => "auto",
        "collapse" => "300",
        "fade" | "slide" | "zoom" => "225",
        _ => "300",
    }
}

fn transition_easing(role: &str) -> &'static str {
    match role {
        "slide" => "cubic-bezier(0.0, 0, 0.2, 1)",
        _ => "cubic-bezier(0.4, 0, 0.2, 1)",
    }
}
