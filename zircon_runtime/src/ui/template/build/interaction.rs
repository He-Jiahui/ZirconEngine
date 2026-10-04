//! 由模板的事件声明和显式输入元数据决定初始交互能力，供命中测试、焦点导航和事件分发消费。

use zircon_runtime_interface::ui::binding::UiEventKind;
use zircon_runtime_interface::ui::event_ui::UiStateFlags;
use zircon_runtime_interface::ui::template::UiTemplateNode;
use zircon_runtime_interface::ui::tree::UiInputPolicy;

/// 只推断初始能力和输入策略；状态变化仍由 surface 管理。
/// 任意显式输入元数据会关闭组件名称兜底，各能力仍可由绑定独立声明。
pub(super) fn infer_interaction(node: &UiTemplateNode) -> (UiStateFlags, UiInputPolicy) {
    let binding_capabilities = binding_capabilities(node);
    let explicit_interactive = bool_attr(node, "input_interactive");
    let explicit_clickable = bool_attr(node, "input_clickable");
    let explicit_hoverable = bool_attr(node, "input_hoverable");
    let explicit_focusable = bool_attr(node, "input_focusable");
    let has_explicit_input_metadata = explicit_interactive.is_some()
        || explicit_clickable.is_some()
        || explicit_hoverable.is_some()
        || explicit_focusable.is_some();
    let explicit_broad_interactive = explicit_interactive.unwrap_or(false);
    let component_name_interactive =
        !has_explicit_input_metadata && component_name_interaction_fallback(node);
    let is_interactive = binding_capabilities.receives_input
        || explicit_broad_interactive
        || component_name_interactive;
    let broad_capability = explicit_broad_interactive || component_name_interactive;
    let clickable =
        explicit_clickable.unwrap_or(binding_capabilities.clickable || broad_capability);
    let hoverable =
        explicit_hoverable.unwrap_or(binding_capabilities.hoverable || broad_capability);
    let focusable = explicit_focusable
        .unwrap_or(node.focus.focusable || binding_capabilities.focusable || broad_capability);
    let receives_input = clickable || hoverable || focusable || is_interactive;
    (
        UiStateFlags {
            visible: true,
            enabled: true,
            clickable,
            hoverable,
            focusable,
            pressed: false,
            checked: false,
            dirty: false,
        },
        if receives_input {
            UiInputPolicy::Receive
        } else {
            UiInputPolicy::Inherit
        },
    )
}

// 多条绑定分别声明的能力须合并，避免最后一条事件覆盖此前的输入需求。
#[derive(Default)]
struct InferredInputCapabilities {
    receives_input: bool,
    clickable: bool,
    hoverable: bool,
    focusable: bool,
}

fn binding_capabilities(node: &UiTemplateNode) -> InferredInputCapabilities {
    node.bindings
        .iter()
        .map(|binding| capabilities_for_event(binding.event))
        .fold(
            InferredInputCapabilities::default(),
            |mut accumulated, capability| {
                accumulated.receives_input |= capability.receives_input;
                accumulated.clickable |= capability.clickable;
                accumulated.hoverable |= capability.hoverable;
                accumulated.focusable |= capability.focusable;
                accumulated
            },
        )
}

// 声明滚动或拖放不能自动授予点击与焦点能力；尺寸通知也不应使装饰节点进入输入路由。
fn capabilities_for_event(event: UiEventKind) -> InferredInputCapabilities {
    match event {
        UiEventKind::Click
        | UiEventKind::DoubleClick
        | UiEventKind::Press
        | UiEventKind::Release
        | UiEventKind::Change
        | UiEventKind::Submit
        | UiEventKind::Toggle => InferredInputCapabilities {
            receives_input: true,
            clickable: true,
            hoverable: true,
            focusable: true,
        },
        UiEventKind::Hover => InferredInputCapabilities {
            receives_input: true,
            hoverable: true,
            ..InferredInputCapabilities::default()
        },
        UiEventKind::Focus | UiEventKind::Blur => InferredInputCapabilities {
            receives_input: true,
            focusable: true,
            ..InferredInputCapabilities::default()
        },
        UiEventKind::Scroll => InferredInputCapabilities {
            receives_input: true,
            ..InferredInputCapabilities::default()
        },
        UiEventKind::Resize => InferredInputCapabilities::default(),
        UiEventKind::DragBegin
        | UiEventKind::DragUpdate
        | UiEventKind::DragEnd
        | UiEventKind::Drop => InferredInputCapabilities {
            receives_input: true,
            hoverable: true,
            ..InferredInputCapabilities::default()
        },
    }
}

fn bool_attr(node: &UiTemplateNode, key: &str) -> Option<bool> {
    node.attributes.get(key).and_then(toml::Value::as_bool)
}

fn component_name_interaction_fallback(node: &UiTemplateNode) -> bool {
    // Temporary authored-asset fallback: future .zui controls should use bindings
    // or explicit input_* metadata instead of relying on component names.
    matches!(
        node.component.as_deref(),
        Some("Button" | "IconButton" | "TextField")
    )
}
