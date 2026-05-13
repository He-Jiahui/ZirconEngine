use std::collections::BTreeMap;
use std::sync::OnceLock;

use crate::ui::component::UiComponentDescriptorRegistry;
use toml::Value;
use zircon_runtime_interface::ui::binding::UiEventKind;
use zircon_runtime_interface::ui::component::{UiComponentCategory, UiComponentEventKind};
use zircon_runtime_interface::ui::event_ui::UiStateFlags;
use zircon_runtime_interface::ui::tree::UiInputPolicy;
use zircon_runtime_interface::ui::v2::UiV2ArenaNode;

pub(super) fn infer_interaction(
    node: &UiV2ArenaNode,
    attributes: &BTreeMap<String, Value>,
) -> (UiStateFlags, UiInputPolicy) {
    let binding_capabilities = binding_capabilities(node);
    let explicit_interactive = bool_attr(attributes, "input_interactive");
    let explicit_clickable = bool_attr(attributes, "input_clickable");
    let explicit_hoverable = bool_attr(attributes, "input_hoverable");
    let explicit_focusable = bool_attr(attributes, "input_focusable");
    let has_explicit_input_metadata = explicit_interactive.is_some()
        || explicit_clickable.is_some()
        || explicit_hoverable.is_some()
        || explicit_focusable.is_some();
    let explicit_broad_interactive = explicit_interactive.unwrap_or(false);
    let component_capabilities = if has_explicit_input_metadata {
        InferredInputCapabilities::default()
    } else {
        catalog_component_capabilities(&node.component)
    };
    let is_interactive = binding_capabilities.receives_input
        || explicit_broad_interactive
        || component_capabilities.receives_input;
    let clickable = explicit_clickable.unwrap_or(
        binding_capabilities.clickable
            || component_capabilities.clickable
            || explicit_broad_interactive,
    );
    let hoverable = explicit_hoverable.unwrap_or(
        binding_capabilities.hoverable
            || component_capabilities.hoverable
            || explicit_broad_interactive,
    );
    let focusable = explicit_focusable.unwrap_or(
        binding_capabilities.focusable
            || component_capabilities.focusable
            || explicit_broad_interactive,
    );
    let receives_input = clickable || hoverable || focusable || is_interactive;
    let disabled = bool_attr(attributes, "disabled").unwrap_or(false);
    let enabled = bool_attr(attributes, "enabled").unwrap_or(true) && !disabled;
    let checked = bool_attr(attributes, "checked").unwrap_or(false);
    (
        UiStateFlags {
            visible: true,
            enabled,
            clickable,
            hoverable,
            focusable,
            pressed: false,
            checked,
            dirty: false,
        },
        if receives_input {
            UiInputPolicy::Receive
        } else {
            UiInputPolicy::Inherit
        },
    )
}

#[derive(Clone, Copy, Default)]
struct InferredInputCapabilities {
    receives_input: bool,
    clickable: bool,
    hoverable: bool,
    focusable: bool,
}

impl InferredInputCapabilities {
    fn merge(&mut self, other: Self) {
        self.receives_input |= other.receives_input;
        self.clickable |= other.clickable;
        self.hoverable |= other.hoverable;
        self.focusable |= other.focusable;
    }
}

fn binding_capabilities(node: &UiV2ArenaNode) -> InferredInputCapabilities {
    node.events
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

fn catalog_component_capabilities(component: &str) -> InferredInputCapabilities {
    let registry = component_descriptor_registry();
    let Some(descriptor) = registry.descriptor(component) else {
        return InferredInputCapabilities::default();
    };
    if descriptor.category == UiComponentCategory::Visual {
        return InferredInputCapabilities::default();
    }

    let mut capabilities = category_default_capabilities(descriptor.category);
    for event in &descriptor.events {
        capabilities.merge(capabilities_for_component_event(*event));
    }
    capabilities
}

fn component_descriptor_registry() -> &'static UiComponentDescriptorRegistry {
    static REGISTRY: OnceLock<UiComponentDescriptorRegistry> = OnceLock::new();
    REGISTRY.get_or_init(UiComponentDescriptorRegistry::editor_showcase)
}

fn category_default_capabilities(category: UiComponentCategory) -> InferredInputCapabilities {
    match category {
        UiComponentCategory::Input
        | UiComponentCategory::Numeric
        | UiComponentCategory::Selection
        | UiComponentCategory::Reference
        | UiComponentCategory::Collection => full_interaction_capabilities(),
        UiComponentCategory::Visual
        | UiComponentCategory::Container
        | UiComponentCategory::Feedback => InferredInputCapabilities::default(),
    }
}

fn capabilities_for_component_event(event: UiComponentEventKind) -> InferredInputCapabilities {
    match event {
        UiComponentEventKind::ValueChanged
        | UiComponentEventKind::Commit
        | UiComponentEventKind::Press
        | UiComponentEventKind::OpenPopup
        | UiComponentEventKind::OpenPopupAt
        | UiComponentEventKind::ClosePopup
        | UiComponentEventKind::SelectOption
        | UiComponentEventKind::ToggleExpanded
        | UiComponentEventKind::AddElement
        | UiComponentEventKind::SetElement
        | UiComponentEventKind::RemoveElement
        | UiComponentEventKind::MoveElement
        | UiComponentEventKind::AddMapEntry
        | UiComponentEventKind::SetMapEntry
        | UiComponentEventKind::RenameMapKey
        | UiComponentEventKind::RemoveMapEntry
        | UiComponentEventKind::ClearReference
        | UiComponentEventKind::LocateReference
        | UiComponentEventKind::OpenReference
        | UiComponentEventKind::SetPage => full_interaction_capabilities(),
        UiComponentEventKind::Focus => InferredInputCapabilities {
            receives_input: true,
            focusable: true,
            ..InferredInputCapabilities::default()
        },
        UiComponentEventKind::Hover => InferredInputCapabilities {
            receives_input: true,
            hoverable: true,
            ..InferredInputCapabilities::default()
        },
        UiComponentEventKind::BeginDrag
        | UiComponentEventKind::DragDelta
        | UiComponentEventKind::LargeDragDelta
        | UiComponentEventKind::EndDrag
        | UiComponentEventKind::DropHover
        | UiComponentEventKind::ActiveDragTarget
        | UiComponentEventKind::DropReference => InferredInputCapabilities {
            receives_input: true,
            hoverable: true,
            ..InferredInputCapabilities::default()
        },
        UiComponentEventKind::SetVisibleRange => InferredInputCapabilities {
            receives_input: true,
            ..InferredInputCapabilities::default()
        },
        UiComponentEventKind::SetWorldTransform | UiComponentEventKind::SetWorldSurface => {
            InferredInputCapabilities::default()
        }
    }
}

fn full_interaction_capabilities() -> InferredInputCapabilities {
    InferredInputCapabilities {
        receives_input: true,
        clickable: true,
        hoverable: true,
        focusable: true,
    }
}

fn bool_attr(attributes: &BTreeMap<String, Value>, key: &str) -> Option<bool> {
    attributes.get(key).and_then(Value::as_bool)
}
