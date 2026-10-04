use std::sync::Arc;

use super::{
    FieldEditorContainer, FieldEditorDefinition, FieldEditorInstance, FieldEditorKind,
    InspectTarget, InspectTargetType, InspectorCustomization, InspectorCustomizationChain,
    InspectorCustomizationDescriptor, InspectorCustomizationSurface, InspectorField,
    InspectorLayoutBuilder, InspectorLayoutRow, InspectorRegistrationError,
};

fn field(type_name: &str) -> InspectorField {
    InspectorField::new("field.value", "Value", type_name, "42", true).unwrap()
}

#[test]
fn builtins_select_six_editor_families_and_miss_falls_back_to_auto() {
    let editors = FieldEditorContainer::builtin();
    let expected = [
        ("u64", FieldEditorKind::Numeric),
        ("bool", FieldEditorKind::Boolean),
        ("LinearColor", FieldEditorKind::Color),
        ("EditorEnum", FieldEditorKind::Enum),
        ("TextureAsset", FieldEditorKind::AssetReference),
        ("AnimationCurve", FieldEditorKind::CurvePlaceholder),
        ("OpaqueCustomRecord", FieldEditorKind::Auto),
    ];
    for (type_name, kind) in expected {
        assert_eq!(editors.resolve(field(type_name)).kind(), kind);
    }
    let asset = editors.resolve(field("TextureAsset"));
    assert_eq!(asset.asset_reference_markers().len(), 21);
    assert!(asset.asset_reference_markers().contains(&"texture"));
}

#[test]
fn duplicate_field_editor_registration_preserves_the_original_definition() {
    let mut editors = FieldEditorContainer::builtin();
    assert!(matches!(
        editors.register(FieldEditorDefinition::new("bool", |_| {
            FieldEditorInstance::new(FieldEditorKind::Auto)
        })),
        Err(InspectorRegistrationError::DuplicateFieldEditor(type_name)) if type_name == "bool"
    ));
    assert_eq!(
        editors.resolve(field("bool")).kind(),
        FieldEditorKind::Boolean
    );
}

#[test]
fn field_editor_registration_rejects_builtin_aliases_but_keeps_qualified_types() {
    let mut editors = FieldEditorContainer::builtin();
    assert!(matches!(
        editors.register(FieldEditorDefinition::new("f32", |_| {
            FieldEditorInstance::new(FieldEditorKind::Color)
        })),
        Err(InspectorRegistrationError::NonCanonicalFieldEditorType(type_name))
            if type_name == "f32"
    ));
    assert_eq!(
        editors.resolve(field("f32")).kind(),
        FieldEditorKind::Numeric,
        "an invalid alias must not silently shadow the built-in numeric editor"
    );
    for type_name in [
        "plugin.sample.BrandColor",
        "plugin::sample::BrandColor",
        "plugin.sample.CloudAsset",
    ] {
        assert_eq!(
            editors.resolve(field(type_name)).kind(),
            FieldEditorKind::Auto,
            "an unregistered qualified plugin type must not fall through to a built-in editor"
        );
    }
    editors
        .register(FieldEditorDefinition::new(
            "plugin.sample.CloudColor",
            |_| FieldEditorInstance::new(FieldEditorKind::Enum),
        ))
        .unwrap();
    assert_eq!(
        editors.resolve(field("plugin.sample.CloudColor")).kind(),
        FieldEditorKind::Enum,
        "a qualified plugin type must win over the built-in color fallback"
    );
}

#[test]
fn descriptor_registration_rejects_an_invalid_target_type() {
    let descriptor = InspectorCustomizationDescriptor::new(
        "invalid target type",
        "plugins://weather/editor/cloud_layer_inspector.zui",
        "plugin.weather.CloudLayerController",
    )
    .with_id("plugin.weather.invalid_target");
    let mut chain = InspectorCustomizationChain::default();

    assert!(matches!(
        chain.register(Arc::new(descriptor)),
        Err(InspectorRegistrationError::InvalidTypeName(type_name))
            if type_name == "invalid target type"
    ));
}

#[test]
fn descriptor_registration_rejects_an_invalid_surface_before_layout() {
    let valid = InspectorCustomizationDescriptor::new(
        "plugin.weather.CloudLayer",
        "plugins://weather/editor/cloud_layer_inspector.zui",
        "plugin.weather.CloudLayerController",
    )
    .with_id("plugin.weather.valid_surface");
    InspectorCustomizationChain::default()
        .register(Arc::new(valid))
        .unwrap();

    let descriptor = InspectorCustomizationDescriptor::new(
        "plugin.weather.CloudLayer",
        "plugins://weather/editor/cloud_layer_inspector.txt",
        "plugin.weather.CloudLayerController",
    )
    .with_id("plugin.weather.invalid_surface");
    let mut chain = InspectorCustomizationChain::default();

    assert!(matches!(
        chain.register(Arc::new(descriptor)),
        Err(InspectorRegistrationError::InvalidCustomizationUiDocument(document))
            if document == "plugins://weather/editor/cloud_layer_inspector.txt"
    ));
}

#[test]
fn chain_rejects_an_invalid_surface_from_a_customization_implementation() {
    let mut chain = InspectorCustomizationChain::default();
    assert!(matches!(
        chain.register(Arc::new(SurfaceCustomization {
            surface: InspectorCustomizationSurface::new(
                "plugins://weather/editor/cloud_layer_inspector.txt",
                "plugin.weather.CloudLayerController",
            ),
        })),
        Err(InspectorRegistrationError::InvalidCustomizationUiDocument(document))
            if document == "plugins://weather/editor/cloud_layer_inspector.txt"
    ));
}

#[test]
fn registration_rejects_invalid_surface_controllers_before_publication() {
    let descriptor = InspectorCustomizationDescriptor::new(
        "plugin.weather.CloudLayer",
        "plugins://weather/editor/cloud_layer_inspector.zui",
        "invalid controller",
    )
    .with_id("plugin.weather.invalid_controller");
    assert!(matches!(
        InspectorCustomizationChain::default().register(Arc::new(descriptor)),
        Err(InspectorRegistrationError::InvalidCustomizationController(controller))
            if controller == "invalid controller"
    ));

    let customization = SurfaceCustomization {
        surface: InspectorCustomizationSurface::new(
            "plugins://weather/editor/cloud_layer_inspector.zui",
            " ",
        ),
    };
    assert!(matches!(
        InspectorCustomizationChain::default().register(Arc::new(customization)),
        Err(InspectorRegistrationError::InvalidCustomizationController(controller))
            if controller == " "
    ));
}

#[test]
fn descriptor_registration_rejects_invalid_optional_surface_values() {
    let invalid_template = InspectorCustomizationDescriptor::new(
        "plugin.weather.CloudLayer",
        "plugins://weather/editor/cloud_layer_inspector.zui",
        "plugin.weather.CloudLayerController",
    )
    .with_id("plugin.weather.invalid_template")
    .with_template_id(" ");
    assert!(matches!(
        InspectorCustomizationChain::default().register(Arc::new(invalid_template)),
        Err(InspectorRegistrationError::InvalidCustomizationTemplateId(template_id))
            if template_id == " "
    ));

    let invalid_data_root = InspectorCustomizationDescriptor::new(
        "plugin.weather.CloudLayer",
        "plugins://weather/editor/cloud_layer_inspector.zui",
        "plugin.weather.CloudLayerController",
    )
    .with_id("plugin.weather.invalid_data_root")
    .with_data_root(" inspector.weather");
    assert!(matches!(
        InspectorCustomizationChain::default().register(Arc::new(invalid_data_root)),
        Err(InspectorRegistrationError::InvalidCustomizationDataRoot(data_root))
            if data_root == " inspector.weather"
    ));

    let invalid_binding = InspectorCustomizationDescriptor::new(
        "plugin.weather.CloudLayer",
        "plugins://weather/editor/cloud_layer_inspector.zui",
        "plugin.weather.CloudLayerController",
    )
    .with_id("plugin.weather.invalid_binding")
    .with_binding("invalid binding");
    assert!(matches!(
        InspectorCustomizationChain::default().register(Arc::new(invalid_binding)),
        Err(InspectorRegistrationError::InvalidCustomizationBinding(binding))
            if binding == "invalid binding"
    ));
}

#[test]
fn chain_rejects_invalid_optional_surface_values_from_a_customization_implementation() {
    let invalid_template = SurfaceCustomization {
        surface: InspectorCustomizationSurface::new(
            "plugins://weather/editor/cloud_layer_inspector.zui",
            "plugin.weather.CloudLayerController",
        )
        .with_template_id(" "),
    };
    assert!(matches!(
        InspectorCustomizationChain::default().register(Arc::new(invalid_template)),
        Err(InspectorRegistrationError::InvalidCustomizationTemplateId(template_id))
            if template_id == " "
    ));

    let invalid_data_root = SurfaceCustomization {
        surface: InspectorCustomizationSurface::new(
            "plugins://weather/editor/cloud_layer_inspector.zui",
            "plugin.weather.CloudLayerController",
        )
        .with_data_root(" inspector.weather"),
    };
    assert!(matches!(
        InspectorCustomizationChain::default().register(Arc::new(invalid_data_root)),
        Err(InspectorRegistrationError::InvalidCustomizationDataRoot(data_root))
            if data_root == " inspector.weather"
    ));

    let invalid_binding = SurfaceCustomization {
        surface: InspectorCustomizationSurface::new(
            "plugins://weather/editor/cloud_layer_inspector.zui",
            "plugin.weather.CloudLayerController",
        )
        .with_binding("invalid binding"),
    };
    assert!(matches!(
        InspectorCustomizationChain::default().register(Arc::new(invalid_binding)),
        Err(InspectorRegistrationError::InvalidCustomizationBinding(binding))
            if binding == "invalid binding"
    ));
}

struct FirstMatchingCustomization;

struct SurfaceCustomization {
    surface: InspectorCustomizationSurface,
}

impl InspectorCustomization for SurfaceCustomization {
    fn id(&self) -> &str {
        "fixture.surface"
    }

    fn can_handle(&self, _target: &InspectTargetType) -> bool {
        false
    }

    fn build(&self, _target: &InspectTarget, _layout: &mut InspectorLayoutBuilder) {}

    fn surface(&self) -> Option<&InspectorCustomizationSurface> {
        Some(&self.surface)
    }
}

impl InspectorCustomization for FirstMatchingCustomization {
    fn id(&self) -> &str {
        "fixture.first"
    }

    fn can_handle(&self, target: &InspectTargetType) -> bool {
        target.type_name() == "fixture::target"
    }

    fn build(&self, _target: &InspectTarget, layout: &mut InspectorLayoutBuilder) {
        layout.add_custom_row("fixture.row", "First customization");
    }
}

struct LaterMatchingCustomization;

impl InspectorCustomization for LaterMatchingCustomization {
    fn id(&self) -> &str {
        "fixture.later"
    }

    fn can_handle(&self, _target: &InspectTargetType) -> bool {
        true
    }

    fn build(&self, _target: &InspectTarget, layout: &mut InspectorLayoutBuilder) {
        layout.add_custom_row("fixture.later.row", "Later customization");
    }
}

#[test]
fn customization_chain_intercepts_once_and_auto_layout_handles_all_misses() {
    let editors = FieldEditorContainer::builtin();
    let mut chain = InspectorCustomizationChain::default();
    chain
        .register(Arc::new(FirstMatchingCustomization))
        .unwrap();
    chain
        .register(Arc::new(LaterMatchingCustomization))
        .unwrap();

    let target = InspectTarget::new(
        InspectTargetType::new("fixture::target").unwrap(),
        "entity:7",
    )
    .unwrap();
    let custom = chain.build(&target, [field("bool")], &editors);
    assert_eq!(custom.customization_id(), Some("fixture.first"));
    assert_eq!(custom.rows().len(), 1);
    assert!(matches!(
        custom.rows()[0],
        InspectorLayoutRow::Custom { ref id, .. } if id == "fixture.row"
    ));

    let fallback_target = InspectTarget::new(
        InspectTargetType::new("fixture::other").unwrap(),
        "entity:8",
    )
    .unwrap();
    let fallback =
        InspectorCustomizationChain::default().build(&fallback_target, [field("bool")], &editors);
    assert_eq!(fallback.customization_id(), None);
    assert!(matches!(
        fallback.rows()[0],
        InspectorLayoutRow::Field { ref editor, .. } if editor.kind() == FieldEditorKind::Boolean
    ));
}

#[test]
fn descriptor_customization_exposes_one_target_scoped_surface() {
    let descriptor = InspectorCustomizationDescriptor::new(
        "plugin.weather.CloudLayer",
        "plugins://weather/editor/cloud_layer_inspector.zui",
        "plugin.weather.CloudLayerController",
    )
    .with_id("plugin.weather.cloud_layer")
    .with_template_id("plugin.weather.cloud_layer.template")
    .with_data_root("inspector.plugin.weather.cloud_layer")
    .with_binding("plugin.weather.cloud_layer.refresh");
    descriptor.validate().unwrap();

    let target = InspectTarget::new(
        InspectTargetType::new("plugin.weather.CloudLayer").unwrap(),
        "entity:7:plugin.weather.CloudLayer",
    )
    .unwrap();
    let mut chain = InspectorCustomizationChain::default();
    chain.register(Arc::new(descriptor)).unwrap();

    let customization = chain.matching(&target).expect("matching customization");
    assert_eq!(customization.id(), "plugin.weather.cloud_layer");
    let surface = customization.surface().expect("retained UI surface");
    assert_eq!(
        surface.ui_document(),
        "plugins://weather/editor/cloud_layer_inspector.zui"
    );
    assert_eq!(
        surface.template_id(),
        Some("plugin.weather.cloud_layer.template")
    );
    assert_eq!(surface.bindings(), ["plugin.weather.cloud_layer.refresh"]);
}
