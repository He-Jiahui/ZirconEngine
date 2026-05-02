use crate::ui::host::EditorManager;
use crate::ui::template_runtime::component_adapter::{asset_editor, inspector, reflection};
use crate::ui::workbench::state::EditorState;
use zircon_runtime::ui::component::inspector_selected_entity_data_source;
use zircon_runtime_interface::ui::component::{
    UiComponentAdapterError, UiComponentAdapterResult, UiComponentDataSourceDescriptor,
    UiComponentDataSourceFieldDescriptor, UiComponentDataSourceFieldOption,
    UiComponentDataSourceKind, UiComponentEventEnvelope, UiValueKind,
};

pub(crate) struct EditorUiComponentAdapterRegistry;

impl EditorUiComponentAdapterRegistry {
    #[allow(dead_code)]
    pub(crate) fn data_sources() -> Vec<UiComponentDataSourceDescriptor> {
        vec![
            inspector_selected_entity_data_source(),
            reflection_source(
                "component",
                "Selected Component Reflection",
                "component://selected",
            ),
            reflection_source("asset", "Selected Asset Reflection", "asset://selected"),
            asset_editor_source("widget", "UI Asset Widget Fields", "widget"),
            asset_editor_source("layout", "UI Asset Layout Fields", "layout"),
            asset_editor_source("slot", "UI Asset Slot Fields", "slot"),
            asset_editor_source("binding", "UI Asset Binding Fields", "binding"),
            asset_editor_source("style", "UI Asset Style Fields", "style"),
        ]
    }

    pub(crate) fn apply_envelope(
        state: &mut EditorState,
        manager: &EditorManager,
        envelope: &UiComponentEventEnvelope,
    ) -> Result<UiComponentAdapterResult, UiComponentAdapterError> {
        match envelope.target.domain.as_str() {
            "inspector" => inspector::apply_inspector_component_envelope(state, envelope),
            "reflection" => reflection::apply_reflection_component_envelope(state, envelope),
            "asset_editor" => {
                asset_editor::apply_asset_editor_component_envelope(manager, envelope)
            }
            domain => Err(UiComponentAdapterError::UnsupportedTargetDomain {
                domain: domain.to_string(),
            }),
        }
    }
}

#[allow(dead_code)]
fn reflection_source(
    source_name: &'static str,
    display_name: &'static str,
    subject: &'static str,
) -> UiComponentDataSourceDescriptor {
    UiComponentDataSourceDescriptor::new(
        "reflection",
        source_name,
        display_name,
        UiComponentDataSourceKind::Reflection,
    )
    .with_subject(subject)
    .writable(true)
    .with_value_kinds([
        UiValueKind::Bool,
        UiValueKind::Int,
        UiValueKind::Float,
        UiValueKind::String,
        UiValueKind::Color,
        UiValueKind::Vec2,
        UiValueKind::Vec3,
        UiValueKind::Vec4,
        UiValueKind::AssetRef,
        UiValueKind::InstanceRef,
        UiValueKind::Array,
        UiValueKind::Map,
        UiValueKind::Enum,
        UiValueKind::Flags,
    ])
    .with_fields(reflection_fields(source_name))
}

#[allow(dead_code)]
fn asset_editor_source(
    source_name: &'static str,
    display_name: &'static str,
    path_prefix: &'static str,
) -> UiComponentDataSourceDescriptor {
    UiComponentDataSourceDescriptor::new(
        "asset_editor",
        source_name,
        display_name,
        UiComponentDataSourceKind::AssetEditor,
    )
    .with_subject("asset://selected")
    .with_path_prefix(path_prefix)
    .writable(true)
    .with_value_kinds([
        UiValueKind::Bool,
        UiValueKind::Int,
        UiValueKind::Float,
        UiValueKind::String,
        UiValueKind::Color,
        UiValueKind::Vec2,
        UiValueKind::Vec3,
        UiValueKind::Vec4,
        UiValueKind::AssetRef,
        UiValueKind::Array,
        UiValueKind::Map,
        UiValueKind::Enum,
        UiValueKind::Flags,
    ])
    .with_fields(asset_editor_fields(path_prefix))
}

#[allow(dead_code)]
fn reflection_fields(source_name: &str) -> Vec<UiComponentDataSourceFieldDescriptor> {
    match source_name {
        "component" => vec![
            UiComponentDataSourceFieldDescriptor::new("name", "Name", UiValueKind::String)
                .writable(true)
                .group("Entity"),
            UiComponentDataSourceFieldDescriptor::new("parent", "Parent", UiValueKind::String)
                .writable(true)
                .group("Entity")
                .reference_kind("scene-entity"),
            UiComponentDataSourceFieldDescriptor::new(
                "component.enabled",
                "Enabled",
                UiValueKind::Bool,
            )
            .group("Component"),
            UiComponentDataSourceFieldDescriptor::new(
                "transform.translation",
                "Translation",
                UiValueKind::Vec3,
            )
            .writable(true)
            .group("Transform")
            .step(0.1),
            UiComponentDataSourceFieldDescriptor::new(
                "transform.translation.x",
                "Translation X",
                UiValueKind::Float,
            )
            .writable(true)
            .group("Transform")
            .range(-100000.0, 100000.0)
            .step(0.1),
            UiComponentDataSourceFieldDescriptor::new(
                "transform.translation.y",
                "Translation Y",
                UiValueKind::Float,
            )
            .writable(true)
            .group("Transform")
            .range(-100000.0, 100000.0)
            .step(0.1),
            UiComponentDataSourceFieldDescriptor::new(
                "transform.translation.z",
                "Translation Z",
                UiValueKind::Float,
            )
            .writable(true)
            .group("Transform")
            .range(-100000.0, 100000.0)
            .step(0.1),
            UiComponentDataSourceFieldDescriptor::new(
                "transform.rotation",
                "Rotation",
                UiValueKind::Vec3,
            )
            .group("Transform")
            .step(1.0),
        ],
        "asset" => vec![
            UiComponentDataSourceFieldDescriptor::new("asset.path", "Path", UiValueKind::String)
                .group("Asset"),
            UiComponentDataSourceFieldDescriptor::new("asset.kind", "Kind", UiValueKind::Enum)
                .group("Asset")
                .options([
                    UiComponentDataSourceFieldOption::new("ui", "UI"),
                    UiComponentDataSourceFieldOption::new("scene", "Scene"),
                    UiComponentDataSourceFieldOption::new("material", "Material"),
                ]),
            UiComponentDataSourceFieldDescriptor::new(
                "asset.dependencies",
                "Dependencies",
                UiValueKind::Array,
            )
            .group("Asset")
            .array_element_kind(UiValueKind::AssetRef),
        ],
        _ => Vec::new(),
    }
}

#[allow(dead_code)]
fn asset_editor_fields(path_prefix: &str) -> Vec<UiComponentDataSourceFieldDescriptor> {
    match path_prefix {
        "widget" => vec![
            UiComponentDataSourceFieldDescriptor::new(
                "widget.control_id",
                "Control ID",
                UiValueKind::String,
            )
            .writable(true)
            .group("Widget"),
            UiComponentDataSourceFieldDescriptor::new("widget.text", "Text", UiValueKind::String)
                .writable(true)
                .group("Widget"),
            UiComponentDataSourceFieldDescriptor::new(
                "component.root_class_policy",
                "Root Class Policy",
                UiValueKind::Enum,
            )
            .writable(true)
            .group("Component Contract")
            .options([
                UiComponentDataSourceFieldOption::new("append_only", "Append Only"),
                UiComponentDataSourceFieldOption::new("closed", "Closed"),
            ]),
        ],
        "slot" => vec![
            UiComponentDataSourceFieldDescriptor::new("slot.mount", "Mount", UiValueKind::String)
                .writable(true)
                .group("Slot"),
            UiComponentDataSourceFieldDescriptor::new(
                "slot.padding",
                "Padding",
                UiValueKind::String,
            )
            .writable(true)
            .group("Slot"),
            UiComponentDataSourceFieldDescriptor::new(
                "slot.width_preferred",
                "Preferred Width",
                UiValueKind::String,
            )
            .writable(true)
            .group("Slot Layout"),
            UiComponentDataSourceFieldDescriptor::new(
                "slot.height_preferred",
                "Preferred Height",
                UiValueKind::String,
            )
            .writable(true)
            .group("Slot Layout"),
            UiComponentDataSourceFieldDescriptor::new(
                "slot.semantic.value",
                "Semantic Value",
                UiValueKind::String,
            )
            .writable(true)
            .group("Slot Semantics"),
        ],
        "layout" => vec![
            UiComponentDataSourceFieldDescriptor::new(
                "layout.width_preferred",
                "Preferred Width",
                UiValueKind::String,
            )
            .writable(true)
            .group("Layout"),
            UiComponentDataSourceFieldDescriptor::new(
                "layout.height_preferred",
                "Preferred Height",
                UiValueKind::String,
            )
            .writable(true)
            .group("Layout"),
            UiComponentDataSourceFieldDescriptor::new(
                "layout.semantic.value",
                "Semantic Value",
                UiValueKind::String,
            )
            .writable(true)
            .group("Layout Semantics"),
        ],
        "binding" => vec![
            UiComponentDataSourceFieldDescriptor::new(
                "binding.id",
                "Binding ID",
                UiValueKind::String,
            )
            .writable(true)
            .group("Binding"),
            UiComponentDataSourceFieldDescriptor::new("binding.event", "Event", UiValueKind::Enum)
                .writable(true)
                .group("Binding")
                .options([
                    UiComponentDataSourceFieldOption::new("Click", "Click"),
                    UiComponentDataSourceFieldOption::new("Change", "Change"),
                    UiComponentDataSourceFieldOption::new("Submit", "Submit"),
                ]),
            UiComponentDataSourceFieldDescriptor::new(
                "binding.route",
                "Route",
                UiValueKind::String,
            )
            .writable(true)
            .group("Binding"),
            UiComponentDataSourceFieldDescriptor::new(
                "binding.route_target",
                "Route Target",
                UiValueKind::String,
            )
            .writable(true)
            .group("Binding Route"),
            UiComponentDataSourceFieldDescriptor::new(
                "binding.action_target",
                "Action Target",
                UiValueKind::String,
            )
            .writable(true)
            .group("Binding Route"),
        ],
        "style" => vec![
            UiComponentDataSourceFieldDescriptor::new(
                "style.classes",
                "Classes",
                UiValueKind::Array,
            )
            .writable(true)
            .group("Style")
            .array_element_kind(UiValueKind::String),
            UiComponentDataSourceFieldDescriptor::new(
                "style.pseudo_states",
                "Pseudo States",
                UiValueKind::Flags,
            )
            .writable(true)
            .group("Style")
            .options([
                UiComponentDataSourceFieldOption::new("hover", "Hover"),
                UiComponentDataSourceFieldOption::new("focus", "Focus"),
                UiComponentDataSourceFieldOption::new("pressed", "Pressed"),
                UiComponentDataSourceFieldOption::new("disabled", "Disabled"),
                UiComponentDataSourceFieldOption::new("selected", "Selected"),
            ]),
        ],
        _ => Vec::new(),
    }
}
