use std::collections::BTreeMap;

use super::{
    check_material_document_asset_references, plan_material_document_property_edit,
    plan_material_document_property_reset, plan_material_document_texture_edit,
    project_authored_material_document, MaterialDocumentAssetReferenceError as ReferenceError,
    MaterialDocumentProjectionError, MaterialDocumentPropertyEdit as Edit,
    MaterialDocumentPropertyEditError as EditError,
    MaterialDocumentPropertyResetError as ResetError, MaterialDocumentTextureEdit as TextureEdit,
    MaterialDocumentTextureEditError as TextureEditError,
};
use zircon_editor::ui::material_editor::MaterialEditorProjection;
use zircon_runtime::asset::{
    AssetReference, AssetUri, MaterialAsset, MaterialTextureSlotValue, ShaderAsset,
    ShaderMaterialPropertyAsset, ShaderSourceLanguage, ShaderTextureSlotAsset, ZMaterialDocument,
};
use zircon_runtime::core::framework::render::{
    MaterialPropertyKind, RenderMaterialDiagnosticSource, RenderMaterialPropertyValue as Value,
    RenderMaterialTextureTransform, ShaderAssetKind,
};
use zircon_runtime_interface::project::PersistedAssetReference;

#[test]
fn material_graph_instance_projection_does_not_mark_inherited_values_as_local_overrides() {
    let mut child = document("", "");
    child.parent = Some(reference("builtin://material/base"));
    let before = child.clone();
    let parent = MaterialAsset::from_zmaterial_document(document(
        "roughness = 0.9",
        "base_color = { kind = 'builtin', locator = 'builtin://texture/white' }",
    ));
    let mut effective = MaterialAsset::from_zmaterial_document(child.clone());
    effective.inherit_parent_values_from(&parent);
    assert_eq!(effective.roughness, 0.9);
    assert!(effective.base_color_texture.is_some());
    assert!(effective.texture_slots.contains_key("base_color"));

    let schema = shader();
    let flattened = MaterialEditorProjection::from_material(&effective, Some(&schema));
    assert!(flattened.properties[1].is_overridden);
    assert!(flattened.texture_slots[0].is_overridden);

    let authored = project_authored_material_document(&child, Some(&schema)).unwrap();
    assert!(!authored.properties[1].is_overridden);
    assert!(authored.properties[1].override_value.is_none());
    assert!(!authored.texture_slots[0].is_overridden);
    assert!(authored.texture_slots[0].reference.is_none());
    // The shader default is not asserted to be the inherited effective 0.9.
    assert_eq!(
        authored.properties[1]
            .default_value
            .as_ref()
            .unwrap()
            .as_float(),
        Some(0.5)
    );
    assert_eq!(child, before);
}

#[test]
fn material_graph_instance_projection_preserves_default_override_and_empty_texture_presence() {
    let source = document("roughness = 0.5", "base_color = {}");
    let before = source.clone();
    let projected = project_authored_material_document(&source, Some(&shader())).unwrap();
    let roughness = &projected.properties[1];
    assert!(roughness.is_overridden);
    assert_eq!(roughness.override_value, roughness.default_value);
    let cleared = &projected.texture_slots[0];
    assert!(cleared.is_overridden);
    assert!(cleared.reference.is_none());
    assert!(cleared.fallback.is_none());
    assert_eq!(cleared.default_fallback.as_deref(), Some("white"));
    assert!(!projected.texture_slots[1].is_overridden);
    assert_eq!(source, before);
}

#[test]
fn material_graph_instance_projection_keeps_schema_order_groups_exact_keys_and_unknown_rows() {
    let source = document(
        "roughness = 0.75\n' roughness ' = 0.4\n'未知 参数' = 3",
        "extra = { fallback = 'black', uv_channel = 3 }",
    );
    let before = source.clone();
    let schema = shader();
    let schema_before = schema.clone();
    let projected = project_authored_material_document(&source, Some(&schema)).unwrap();
    assert_eq!(
        projected
            .properties
            .iter()
            .map(|row| row.name.as_str())
            .collect::<Vec<_>>(),
        ["color", "roughness", " roughness ", "未知 参数"]
    );
    assert_eq!(
        projected.properties[1].group.as_deref(),
        Some("表面参数 / Surface")
    );
    assert_eq!(
        projected.properties[1].label.as_deref(),
        Some("粗糙度：保留作者显示名")
    );
    assert!(projected.properties[2].kind.is_none());
    assert!(projected.properties[2].default_value.is_none());
    assert!(projected.properties[2].is_overridden);
    assert_eq!(projected.texture_slots[2].name, "extra");
    assert!(projected.texture_slots[2].kind.is_none());
    assert_eq!(source.textures["extra"].uv_channel, 3);
    assert_eq!(source, before);
    assert_eq!(schema, schema_before);
}

#[test]
fn material_graph_instance_projection_without_shader_does_not_invent_schema_or_values() {
    let source = document("roughness = 0.25", "base_color = {}");
    let projected = project_authored_material_document(&source, None).unwrap();
    assert_eq!(projected.shader_reference, source.shader);
    assert_eq!(projected.properties.len(), 1);
    assert!(projected.properties[0].kind.is_none());
    assert!(projected.properties[0].group.is_none());
    assert!(projected.properties[0].default_value.is_none());
    assert_eq!(
        projected.properties[0].override_value.as_ref(),
        source.overrides.get("roughness")
    );
    assert_eq!(projected.texture_slots.len(), 1);
    assert!(projected.texture_slots[0].is_overridden);
    assert!(projected.diagnostics.iter().any(|diagnostic| {
        diagnostic.path == "shader"
            && diagnostic.source == Some(RenderMaterialDiagnosticSource::DependencyResolution)
    }));
}

#[test]
fn material_graph_instance_projection_rejects_an_unrelated_shader_without_source_mutation() {
    let source = document("roughness = 0.25", "");
    let before = source.clone();
    let mut wrong = shader();
    wrong.uri = AssetUri::parse("builtin://shader/unrelated").unwrap();
    let wrong_before = wrong.clone();
    assert_eq!(
        project_authored_material_document(&source, Some(&wrong)),
        Err(MaterialDocumentProjectionError::ShaderLocatorMismatch {
            expected: source.shader.clone(),
            actual: wrong.uri.clone(),
        })
    );
    assert_eq!(source, before);
    assert_eq!(wrong, wrong_before);
}

#[test]
fn material_graph_instance_projection_rejects_ambiguous_schema_keys_in_their_own_namespace() {
    let source = document("roughness = 0.25", "");
    let mut duplicate_property = shader();
    duplicate_property
        .property_schema
        .push(duplicate_property.property_schema[1].clone());
    assert_eq!(
        project_authored_material_document(&source, Some(&duplicate_property)),
        Err(MaterialDocumentProjectionError::DuplicatePropertyName(
            "roughness".into()
        ))
    );
    let mut duplicate_texture = shader();
    duplicate_texture
        .texture_slots
        .push(duplicate_texture.texture_slots[0].clone());
    assert_eq!(
        project_authored_material_document(&source, Some(&duplicate_texture)),
        Err(MaterialDocumentProjectionError::DuplicateTextureSlotName(
            "base_color".into()
        ))
    );
    let mut distinct_namespaces = shader();
    distinct_namespaces.texture_slots[0].name = "roughness".into();
    assert!(project_authored_material_document(&source, Some(&distinct_namespaces)).is_ok());
}

#[test]
fn material_graph_instance_projection_preserves_author_codec_metadata_and_repair_diagnostics() {
    let mut source = document("roughness = inf\ncolor = 'wrong-kind'", "base_color = {}");
    source.parent = Some(reference("builtin://material/base"));
    source.validation_diagnostics = vec!["原作者诊断，未由投影静默清除".into()];
    let slot = source.textures.get_mut("base_color").unwrap();
    slot.uv_channel = 2;
    slot.transform = Some(Default::default());
    let before = source.clone();
    let mut schema = shader();
    schema.validation_diagnostics = vec![
        "wgsl_capture 原着色器诊断".into(),
        "other shader import diagnostic".into(),
    ];
    let projected = project_authored_material_document(&source, Some(&schema)).unwrap();
    assert_eq!(
        projected.properties[0]
            .override_value
            .as_ref()
            .unwrap()
            .as_str(),
        Some("wrong-kind")
    );
    assert!(projected.properties[1]
        .override_value
        .as_ref()
        .unwrap()
        .as_float()
        .unwrap()
        .is_infinite());
    assert_eq!(projected.diagnostics.len(), 3);
    assert_eq!(
        projected.diagnostics[0].message,
        source.validation_diagnostics[0]
    );
    assert_eq!(
        projected.diagnostics[1].source,
        Some(RenderMaterialDiagnosticSource::WgslCapture)
    );
    assert_eq!(
        projected.diagnostics[2].message,
        schema.validation_diagnostics[1]
    );
    assert_eq!(source, before);

    let text = source
        .to_project_toml_string(|value| Ok(PersistedAssetReference::builtin(value.locator.clone())))
        .unwrap();
    let reopened = ZMaterialDocument::from_project_toml_str(&text, |value| {
        Ok(AssetReference::from_locator(
            value.builtin_locator().unwrap().clone(),
        ))
    })
    .unwrap();
    assert_eq!(reopened, source);
    assert_eq!(
        project_authored_material_document(&reopened, Some(&schema)).unwrap(),
        projected
    );
}

#[test]
fn material_graph_instance_property_set_keeps_explicit_default_and_borrows_original_sources() {
    let source = document("", "base_color = { uv_channel = 2 }");
    let before = source.clone();
    let schema = shader();
    let value = Value::Float { value: 0.5 };
    let plan = plan_material_document_property_edit(
        &source,
        Some(&schema),
        "roughness",
        Edit::Set(&value),
    )
    .unwrap();
    assert!(std::ptr::eq(plan.source_document(), &source));
    assert!(std::ptr::eq(
        plan.property_definition().unwrap(),
        &schema.property_schema[1]
    ));
    assert_eq!(plan.name(), "roughness");
    assert!(!plan.was_overridden());
    assert!(plan.will_be_overridden());
    let Edit::Set(proposed) = plan.edit() else {
        panic!("an explicit default remains a Set proposal");
    };
    assert!(std::ptr::eq(proposed, &value));
    assert_eq!(source, before);
}

#[test]
fn material_graph_instance_property_clear_preserves_raw_repair_values_and_required_diagnostics() {
    let mut source = document("roughness = nan\nunknown = 'repair me'", "");
    source.parent = Some(reference("builtin://material/unresolved-parent"));
    source.validation_diagnostics = vec!["missing required parent value".into()];
    let mut schema = shader();
    schema.property_schema[1].required = true;
    schema.property_schema[1].default = None;
    let plan = plan_material_document_property_edit(
        &source,
        Some(&schema),
        "roughness",
        Edit::ClearOverride,
    )
    .unwrap();
    assert!(plan.was_overridden());
    assert!(!plan.will_be_overridden());
    assert_eq!(plan.edit(), Edit::ClearOverride);
    assert!(plan.source_document().overrides["roughness"]
        .as_float()
        .unwrap()
        .is_nan());
    assert_eq!(
        plan.source_document().validation_diagnostics,
        source.validation_diagnostics
    );
    assert_eq!(
        source.parent.as_ref().unwrap().locator.to_string(),
        "builtin://material/unresolved-parent"
    );
    let repair =
        plan_material_document_property_edit(&source, None, "unknown", Edit::ClearOverride)
            .unwrap();
    assert!(repair.was_overridden());
    assert!(repair.property_definition().is_none());
    assert_eq!(source.overrides["unknown"].as_str(), Some("repair me"));
    let absent =
        plan_material_document_property_edit(&source, None, "absent", Edit::ClearOverride).unwrap();
    assert!(!absent.was_overridden());
    assert!(!absent.will_be_overridden());
    // The host skips an unchanged clear; this preflight creates no undo record.
}

#[test]
fn material_graph_instance_property_set_accepts_the_shader_abi_without_clamping_hdr_or_integers() {
    let cases = [
        (
            MaterialPropertyKind::Float,
            Value::Float { value: f32::MAX },
        ),
        (MaterialPropertyKind::Int, Value::Int { value: i32::MIN }),
        (MaterialPropertyKind::UInt, Value::UInt { value: u32::MAX }),
        (MaterialPropertyKind::Bool, Value::Bool { value: true }),
        (
            MaterialPropertyKind::Vec2,
            Value::Vec2 { value: [-2.0, 4.0] },
        ),
        (
            MaterialPropertyKind::Vec3,
            Value::Vec3 {
                value: [4.0, -2.0, 1.0],
            },
        ),
        (
            MaterialPropertyKind::Vec4,
            Value::Vec4 {
                value: [-2.0, 4.0, f32::MAX, -0.0],
            },
        ),
        (
            MaterialPropertyKind::Color,
            Value::Vec4 {
                value: [4.0, -2.0, 1.0, 0.5],
            },
        ),
    ];
    for (kind, value) in cases {
        let source = document("", "");
        let mut schema = shader();
        schema.property_schema[1].kind = kind;
        let plan = plan_material_document_property_edit(
            &source,
            Some(&schema),
            "roughness",
            Edit::Set(&value),
        )
        .unwrap();
        let Edit::Set(proposed) = plan.edit() else {
            panic!("Set must be retained")
        };
        assert!(std::ptr::eq(proposed, &value));
        assert_eq!(proposed, &value);
        if let Value::Vec4 { value } = proposed {
            assert_eq!(
                value[3].to_bits(),
                if kind == MaterialPropertyKind::Vec4 {
                    (-0.0_f32).to_bits()
                } else {
                    0.5_f32.to_bits()
                }
            );
        }
    }
}

#[test]
fn material_graph_instance_property_set_rejects_type_coercion_and_preserves_invalid_local_data() {
    let source = document("roughness = 'old invalid value'", "");
    let before = source.clone();
    let cases = [
        (MaterialPropertyKind::Float, Value::Int { value: 1 }),
        (MaterialPropertyKind::Int, Value::Float { value: 1.0 }),
        (MaterialPropertyKind::UInt, Value::Int { value: -1 }),
        (MaterialPropertyKind::Bool, Value::UInt { value: 1 }),
        (
            MaterialPropertyKind::Float,
            Value::String {
                value: "0.5".into(),
            },
        ),
        (
            MaterialPropertyKind::Vec3,
            Value::Vec2 { value: [1.0, 2.0] },
        ),
    ];
    for (kind, value) in cases {
        let mut schema = shader();
        schema.property_schema[1].kind = kind;
        let error = plan_material_document_property_edit(
            &source,
            Some(&schema),
            "roughness",
            Edit::Set(&value),
        )
        .unwrap_err();
        assert!(
            matches!(error, EditError::TypeMismatch { name, expected, .. } if name == "roughness" && expected == kind)
        );
        assert_eq!(source, before);
    }
}

#[test]
fn material_graph_instance_property_set_rejects_nonfinite_scalar_and_vector_components() {
    let cases = [
        (
            MaterialPropertyKind::Float,
            Value::Float { value: f32::NAN },
        ),
        (
            MaterialPropertyKind::Float,
            Value::Float {
                value: f32::INFINITY,
            },
        ),
        (
            MaterialPropertyKind::Float,
            Value::Float {
                value: f32::NEG_INFINITY,
            },
        ),
        (
            MaterialPropertyKind::Vec2,
            Value::Vec2 {
                value: [0.0, f32::NAN],
            },
        ),
        (
            MaterialPropertyKind::Vec3,
            Value::Vec3 {
                value: [f32::INFINITY, 0.0, 0.0],
            },
        ),
        (
            MaterialPropertyKind::Color,
            Value::Vec4 {
                value: [0.0, 0.0, f32::NEG_INFINITY, 1.0],
            },
        ),
    ];
    let source = document("roughness = 0.25", "");
    for (kind, value) in cases {
        let mut schema = shader();
        schema.property_schema[1].kind = kind;
        assert_eq!(
            plan_material_document_property_edit(
                &source,
                Some(&schema),
                "roughness",
                Edit::Set(&value),
            )
            .unwrap_err(),
            EditError::NonFiniteValue("roughness".into())
        );
    }
    assert_eq!(source.overrides["roughness"].as_float(), Some(0.25));
}

#[test]
fn material_graph_instance_property_edit_requires_the_real_shader_and_an_unambiguous_target() {
    let source = document("", "");
    let value = Value::Float { value: 0.5 };
    assert_eq!(
        plan_material_document_property_edit(&source, None, "roughness", Edit::Set(&value))
            .unwrap_err(),
        EditError::SchemaUnavailable
    );
    let mut wrong = shader();
    wrong.uri = AssetUri::parse("builtin://shader/unrelated").unwrap();
    for edit in [Edit::Set(&value), Edit::ClearOverride] {
        assert!(matches!(
            plan_material_document_property_edit(&source, Some(&wrong), "roughness", edit)
                .unwrap_err(),
            EditError::Source(MaterialDocumentProjectionError::ShaderLocatorMismatch { .. })
        ));
    }
    let mut duplicate = shader();
    duplicate
        .property_schema
        .push(duplicate.property_schema[1].clone());
    for edit in [Edit::Set(&value), Edit::ClearOverride] {
        assert_eq!(
            plan_material_document_property_edit(&source, Some(&duplicate), "roughness", edit)
                .unwrap_err(),
            EditError::Source(MaterialDocumentProjectionError::DuplicatePropertyName(
                "roughness".into()
            ))
        );
    }
    let mut unrelated_duplicate = shader();
    unrelated_duplicate
        .property_schema
        .push(unrelated_duplicate.property_schema[0].clone());
    assert!(plan_material_document_property_edit(
        &source,
        Some(&unrelated_duplicate),
        "roughness",
        Edit::Set(&value),
    )
    .is_ok());
    // A single-key proposal is not a full shader validation or a ready UI projection.
    assert!(project_authored_material_document(&source, Some(&unrelated_duplicate)).is_err());
}

#[test]
fn material_graph_instance_property_edit_preserves_exact_names_and_separates_clear_from_set_default(
) {
    let source = document(
        "' 粗糙度 ' = 'repair'\n'' = nan",
        "base_color = { uv_channel = 2 }",
    );
    let mut schema = shader();
    schema.property_schema[1].name = " 粗糙度 ".into();
    let value = Value::Float { value: 0.5 };
    let plan =
        plan_material_document_property_edit(&source, Some(&schema), " 粗糙度 ", Edit::Set(&value))
            .unwrap();
    assert_eq!(plan.name(), " 粗糙度 ");
    assert!(plan.was_overridden());
    assert!(plan.will_be_overridden());
    assert_eq!(
        plan_material_document_property_edit(&source, Some(&schema), "粗糙度", Edit::Set(&value),)
            .unwrap_err(),
        EditError::UndeclaredProperty("粗糙度".into())
    );
    assert_eq!(
        plan_material_document_property_edit(&source, Some(&schema), "", Edit::Set(&value))
            .unwrap_err(),
        EditError::EmptyPropertyName
    );
    let clear =
        plan_material_document_property_edit(&source, None, "", Edit::ClearOverride).unwrap();
    assert!(clear.was_overridden());
    assert!(!clear.will_be_overridden());
    assert!(source.overrides[""].as_float().unwrap().is_nan());
    assert_eq!(source.overrides[" 粗糙度 "].as_str(), Some("repair"));
    assert_eq!(source.textures["base_color"].uv_channel, 2);
    assert_eq!(
        source.editor["note"].as_str(),
        Some("保留中文和长作者元数据")
    );
}

// Unit inputs for the real authoring codec/projection, never a preview producer,
// published ResourceManager handle or substitute for an Engine-rendered asset.
#[test]
fn material_graph_instance_texture_empty_set_keeps_local_presence_and_required_diagnostics() {
    let mut source = document("roughness = 0.5", "");
    source.parent = Some(reference("builtin://material/base"));
    source.validation_diagnostics = vec!["required texture is unresolved".into()];
    let before = source.clone();
    let mut schema = shader();
    schema.texture_slots[0].required = true;
    let empty = MaterialTextureSlotValue {
        reference: None,
        fallback: None,
        transform: None,
        uv_channel: 0,
    };
    let plan = plan_material_document_texture_edit(
        &source,
        Some(&schema),
        "base_color",
        TextureEdit::Set(&empty),
    )
    .unwrap();
    assert!(std::ptr::eq(plan.source_document(), &source));
    assert!(std::ptr::eq(
        plan.slot_definition().unwrap(),
        &schema.texture_slots[0]
    ));
    assert!(!plan.was_overridden());
    assert!(plan.will_be_overridden());
    let TextureEdit::Set(value) = plan.edit() else {
        panic!("Set must remain Set")
    };
    assert!(std::ptr::eq(value, &empty));
    assert!(value.reference.is_none() && value.fallback.is_none());
    assert_eq!(schema.texture_slots[0].default.as_deref(), Some("white"));
    assert_eq!(source, before);
}

#[test]
fn material_graph_instance_texture_set_borrows_full_reference_fallback_and_unclamped_uv_payload() {
    let source = document(
        "roughness = 0.25",
        "normal = { fallback = 'normal', uv_channel = 1 }",
    );
    let before = source.clone();
    let schema = shader();
    let schema_before = schema.clone();
    let mut value = MaterialTextureSlotValue::new(reference("builtin://texture/white"));
    value.fallback = Some(" raw fallback value ".into());
    value.uv_channel = u32::MAX;
    value.transform = Some(RenderMaterialTextureTransform {
        scale: [-2.0, 0.0],
        offset: [10.0, -100.0],
        rotation: f32::MAX,
    });
    let plan = plan_material_document_texture_edit(
        &source,
        Some(&schema),
        "normal",
        TextureEdit::Set(&value),
    )
    .unwrap();
    assert_eq!(plan.name(), "normal");
    assert!(plan.was_overridden() && plan.will_be_overridden());
    assert_eq!(plan.edit(), TextureEdit::Set(&value));
    let TextureEdit::Set(borrowed) = plan.edit() else {
        panic!("Set must remain Set")
    };
    assert!(std::ptr::eq(borrowed, &value));
    assert_eq!(borrowed.uv_channel, u32::MAX);
    assert_eq!(borrowed.transform.unwrap().scale, [-2.0, 0.0]);
    assert_eq!(borrowed.fallback.as_deref(), Some(" raw fallback value "));
    assert_eq!(source, before);
    assert_eq!(schema, schema_before);
}

#[test]
fn material_graph_instance_texture_set_rejects_missing_stale_or_ambiguous_target_schema() {
    let source = document("roughness = 0.5", "base_color = {}");
    let before = source.clone();
    let value = source.textures.get("base_color").unwrap();
    let schema = shader();
    assert_eq!(
        plan_material_document_texture_edit(&source, None, "base_color", TextureEdit::Set(value))
            .unwrap_err(),
        TextureEditError::SchemaUnavailable
    );
    assert_eq!(
        plan_material_document_texture_edit(&source, Some(&schema), "", TextureEdit::Set(value))
            .unwrap_err(),
        TextureEditError::EmptySlotName
    );
    assert_eq!(
        plan_material_document_texture_edit(
            &source,
            Some(&schema),
            "unknown",
            TextureEdit::Set(value)
        )
        .unwrap_err(),
        TextureEditError::UndeclaredSlot("unknown".into())
    );
    let mut wrong = shader();
    wrong.uri = AssetUri::parse("builtin://shader/unrelated").unwrap();
    assert!(matches!(
        plan_material_document_texture_edit(
            &source,
            Some(&wrong),
            "base_color",
            TextureEdit::ClearOverride
        ),
        Err(TextureEditError::Source(
            MaterialDocumentProjectionError::ShaderLocatorMismatch { .. }
        ))
    ));
    let mut ambiguous = shader();
    ambiguous
        .texture_slots
        .push(ambiguous.texture_slots[0].clone());
    for edit in [TextureEdit::Set(value), TextureEdit::ClearOverride] {
        assert_eq!(
            plan_material_document_texture_edit(&source, Some(&ambiguous), "base_color", edit)
                .unwrap_err(),
            TextureEditError::Source(MaterialDocumentProjectionError::DuplicateTextureSlotName(
                "base_color".into()
            ))
        );
    }
    // A single-target proposal is not whole-schema validation.
    let mut unrelated = shader();
    unrelated
        .texture_slots
        .push(unrelated.texture_slots[1].clone());
    unrelated
        .property_schema
        .push(unrelated.property_schema[0].clone());
    assert!(plan_material_document_texture_edit(
        &source,
        Some(&unrelated),
        "base_color",
        TextureEdit::Set(value)
    )
    .is_ok());
    assert_eq!(source, before);
}

#[test]
fn material_graph_instance_texture_clear_can_repair_exact_unknown_empty_and_nonfinite_entries() {
    let mut source = document("roughness = 0.5", "");
    for key in ["", "未知 纹理", "base_color"] {
        source.textures.insert(
            key.into(),
            MaterialTextureSlotValue {
                reference: None,
                fallback: Some("unknown fallback".into()),
                transform: Some(RenderMaterialTextureTransform {
                    rotation: f32::NAN,
                    ..Default::default()
                }),
                uv_channel: u32::MAX,
            },
        );
    }
    let editor_before = source.editor.clone();
    for key in ["", "未知 纹理", "base_color", "absent"] {
        let plan =
            plan_material_document_texture_edit(&source, None, key, TextureEdit::ClearOverride)
                .unwrap();
        assert_eq!(plan.name(), key);
        assert_eq!(plan.was_overridden(), key != "absent");
        assert!(!plan.will_be_overridden());
        assert!(plan.slot_definition().is_none());
        assert_eq!(plan.edit(), TextureEdit::ClearOverride);
        assert!(std::ptr::eq(plan.source_document(), &source));
    }
    assert_eq!(source.textures.len(), 3);
    assert!(source.textures["base_color"]
        .transform
        .unwrap()
        .rotation
        .is_nan());
    assert_eq!(source.editor, editor_before);
}

#[test]
fn material_graph_instance_texture_set_rejects_each_nonfinite_uv_component_without_sanitizing() {
    let source = document("roughness = 0.5", "base_color = {}");
    let before = source.clone();
    let schema = shader();
    for bad in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
        for component in 0..5 {
            let mut transform = RenderMaterialTextureTransform::IDENTITY;
            match component {
                0..=1 => transform.scale[component] = bad,
                2..=3 => transform.offset[component - 2] = bad,
                _ => transform.rotation = bad,
            }
            let mut value = source.textures["base_color"].clone();
            value.transform = Some(transform);
            assert_eq!(
                plan_material_document_texture_edit(
                    &source,
                    Some(&schema),
                    "base_color",
                    TextureEdit::Set(&value)
                )
                .unwrap_err(),
                TextureEditError::NonFiniteTransform("base_color".into())
            );
            let preserved = value.transform.unwrap();
            let actual = match component {
                0..=1 => preserved.scale[component],
                2..=3 => preserved.offset[component - 2],
                _ => preserved.rotation,
            };
            assert!(actual.is_nan() && bad.is_nan() || actual == bad);
        }
    }
    assert_eq!(source, before);
}

#[test]
fn material_graph_instance_texture_keys_keep_spaces_unicode_aliases_and_property_namespaces() {
    let source = document("roughness = 0.5", "base_color = {}");
    let before = source.clone();
    let mut schema = shader();
    schema.texture_slots[0].name = " 法线 参数 ".into();
    schema.texture_slots[1].name = "roughness".into();
    let value = &source.textures["base_color"];
    for name in [" 法线 参数 ", "roughness"] {
        let plan = plan_material_document_texture_edit(
            &source,
            Some(&schema),
            name,
            TextureEdit::Set(value),
        )
        .unwrap();
        assert_eq!(plan.name(), name);
        assert!(!plan.was_overridden());
    }
    for name in ["法线 参数", "base_color_texture", "normal"] {
        assert_eq!(
            plan_material_document_texture_edit(
                &source,
                Some(&schema),
                name,
                TextureEdit::Set(value)
            )
            .unwrap_err(),
            TextureEditError::UndeclaredSlot(name.into())
        );
    }
    let alias_clear = plan_material_document_texture_edit(
        &source,
        None,
        "base_color_texture",
        TextureEdit::ClearOverride,
    )
    .unwrap();
    assert!(!alias_clear.was_overridden());
    assert_eq!(alias_clear.name(), "base_color_texture");
    assert!(source.textures.contains_key("base_color"));
    assert_eq!(source, before);
}

#[test]
fn material_graph_instance_source_references_preserve_original_root_and_instance_payloads() {
    for parent in [None, Some(reference("builtin://material/base"))] {
        let mut source = document("roughness = 0.5", "base_color = {}");
        source.parent = parent;
        let source_before = source.clone();
        let material = MaterialAsset::from_zmaterial_document(source.clone());
        let material_before = material.clone();

        check_material_document_asset_references(&source, &material).unwrap();
        assert_eq!(source, source_before);
        assert_eq!(material, material_before);
        assert_eq!(material.parent, source.parent);
        assert_eq!(material.shader, source.shader);
        assert!(!source.editor.is_empty());
    }
}

#[test]
fn material_graph_instance_source_references_reject_parent_cleared_after_real_inheritance() {
    let mut source = document("", "");
    source.parent = Some(reference("builtin://material/base"));
    let source_before = source.clone();
    let parent = MaterialAsset::from_zmaterial_document(document("roughness = 0.9", ""));
    let mut effective = MaterialAsset::from_zmaterial_document(source.clone());
    effective.inherit_parent_values_from(&parent);
    assert_eq!(effective.roughness, 0.9);
    // Mirrors the final parent-clear in load_effective_material.rs; this test
    // does not call its private loader or claim registry/renderer execution.
    effective.parent = None;
    let effective_before = effective.clone();

    let ReferenceError::ParentReferenceMismatch { authored, actual } =
        check_material_document_asset_references(&source, &effective).unwrap_err()
    else {
        panic!("flattened material must not report an authored empty parent chain");
    };
    assert!(std::ptr::eq(
        authored.unwrap(),
        source.parent.as_ref().unwrap()
    ));
    assert!(actual.is_none());
    assert_eq!(source, source_before);
    assert_eq!(effective, effective_before);
}

#[test]
fn material_graph_instance_source_references_reject_unexpected_parent_before_shader_drift() {
    let root = document("", "");
    let mut unexpected = MaterialAsset::from_zmaterial_document(root.clone());
    unexpected.parent = Some(reference("builtin://material/unexpected"));
    unexpected.shader = reference("builtin://shader/other");
    let before = unexpected.clone();

    let ReferenceError::ParentReferenceMismatch { authored, actual } =
        check_material_document_asset_references(&root, &unexpected).unwrap_err()
    else {
        panic!("parent mismatch must remain actionable when shader also differs");
    };
    assert!(authored.is_none());
    assert!(std::ptr::eq(
        actual.unwrap(),
        unexpected.parent.as_ref().unwrap()
    ));
    assert_eq!(unexpected, before);
}

#[test]
fn material_graph_instance_source_references_keep_both_declared_parent_identity_fields() {
    let mut source = document("", "");
    source.parent = Some(reference("builtin://material/base"));
    let expected = source.parent.as_ref().unwrap();
    let alternatives = [
        AssetReference::new(
            reference("builtin://material/other").uuid,
            expected.locator.clone(),
        ),
        AssetReference::new(
            expected.uuid,
            AssetUri::parse("builtin://material/moved").unwrap(),
        ),
    ];
    let source_before = source.clone();
    for alternative in alternatives {
        let mut material = MaterialAsset::from_zmaterial_document(source.clone());
        material.parent = Some(alternative);
        let before = material.clone();
        let ReferenceError::ParentReferenceMismatch { authored, actual } =
            check_material_document_asset_references(&source, &material).unwrap_err()
        else {
            panic!("matching only the parent UUID or locator is insufficient");
        };
        assert!(std::ptr::eq(authored.unwrap(), expected));
        assert!(std::ptr::eq(
            actual.unwrap(),
            material.parent.as_ref().unwrap()
        ));
        assert_eq!(material, before);
    }
    assert_eq!(source, source_before);
}

#[test]
fn material_graph_instance_source_references_keep_both_declared_shader_identity_fields() {
    let mut source = document("", "");
    source.parent = Some(reference("builtin://material/base"));
    let alternatives = [
        AssetReference::new(
            reference("builtin://shader/other").uuid,
            source.shader.locator.clone(),
        ),
        AssetReference::new(
            source.shader.uuid,
            AssetUri::parse("builtin://shader/moved").unwrap(),
        ),
    ];
    let source_before = source.clone();
    for alternative in alternatives {
        let mut material = MaterialAsset::from_zmaterial_document(source.clone());
        material.shader = alternative;
        let before = material.clone();
        let ReferenceError::ShaderReferenceMismatch { authored, actual } =
            check_material_document_asset_references(&source, &material).unwrap_err()
        else {
            panic!("matching only the shader UUID or locator is insufficient");
        };
        assert!(std::ptr::eq(authored, &source.shader));
        assert!(std::ptr::eq(actual, &material.shader));
        let diagnostic = ReferenceError::ShaderReferenceMismatch { authored, actual }.to_string();
        assert!(diagnostic.contains(&authored.uuid.to_string()));
        assert!(diagnostic.contains(&actual.uuid.to_string()));
        assert_eq!(material, before);
    }
    assert_eq!(source, source_before);
}

#[test]
fn material_graph_instance_source_reference_match_is_not_a_payload_or_revision_proof() {
    let mut source = document("", "");
    source.parent = Some(reference("builtin://material/base"));
    let before = source.clone();
    let parent = MaterialAsset::from_zmaterial_document(document("roughness = 0.9", ""));
    let mut inherited = MaterialAsset::from_zmaterial_document(source.clone());
    inherited.inherit_parent_values_from(&parent);
    assert_eq!(inherited.roughness, 0.9);
    assert!(source.overrides.is_empty());
    assert!(!inherited.property_values.is_empty());

    // Retaining references can pass this necessary metadata check even though
    // values were inherited. The host still needs original coherent sources.
    check_material_document_asset_references(&source, &inherited).unwrap();
    assert_eq!(source, before);
}

#[test]
fn material_graph_instance_property_reset_borrows_real_default_and_retains_override_presence() {
    let source = document("roughness = 0.5", "");
    let before = source.clone();
    let schema = shader();
    for value in [0.5, 0.9] {
        let resolved = Value::Float { value };
        let calls = std::cell::Cell::new(0);
        let plan =
            plan_material_document_property_reset(&source, Some(&schema), "roughness", |target| {
                calls.set(calls.get() + 1);
                assert!(std::ptr::eq(target, &schema.property_schema[1]));
                Ok::<_, &'static str>(Some(&resolved))
            })
            .unwrap();
        assert_eq!(calls.get(), 1);
        assert!(std::ptr::eq(plan.source_document(), &source));
        assert_eq!(plan.name(), "roughness");
        assert!(std::ptr::eq(plan.resolved_default(), &resolved));
        assert!(plan.was_overridden());
        assert!(plan.will_be_overridden());
        let edit = plan.property_edit().unwrap();
        assert!(edit.was_overridden());
        assert!(edit.will_be_overridden());
        match edit.edit() {
            Edit::Set(value) => assert!(std::ptr::eq(value, &resolved)),
            Edit::ClearOverride => panic!("Reset must retain the local override"),
        }
        // Equal defaults remain a qualified candidate; the host owns exact no-op comparison.
        assert_eq!(
            schema.property_schema[1]
                .default
                .as_ref()
                .unwrap()
                .as_float(),
            Some(0.5)
        );
        assert_eq!(source, before);
    }
}

#[test]
fn material_graph_instance_property_reset_unchecked_row_returns_value_without_authoring_edit() {
    let mut source = document("", "");
    source.parent = Some(reference("builtin://material/base"));
    source
        .validation_diagnostics
        .push("required parent value remains unresolved".into());
    let before = source.clone();
    let mut schema = shader();
    schema.property_schema[1].required = true;
    let resolved = Value::Float { value: 0.75 };
    let calls = std::cell::Cell::new(0);
    let plan = plan_material_document_property_reset(&source, Some(&schema), "roughness", |_| {
        calls.set(calls.get() + 1);
        Ok::<_, &'static str>(Some(&resolved))
    })
    .unwrap();
    assert_eq!(calls.get(), 1);
    assert!(std::ptr::eq(plan.resolved_default(), &resolved));
    assert!(!plan.was_overridden());
    assert!(!plan.will_be_overridden());
    assert!(plan.property_edit().is_none());
    assert!(source.overrides.is_empty());
    assert_eq!(source, before);
}

#[test]
fn material_graph_instance_property_reset_preflights_schema_target_before_any_lookup() {
    let source = document("roughness = 0.2", "");
    let before = source.clone();
    let schema = shader();
    let mut wrong = shader();
    wrong.uri = AssetUri::parse("builtin://shader/unrelated").unwrap();
    let mut duplicate = shader();
    duplicate
        .property_schema
        .push(duplicate.property_schema[1].clone());
    let resolved = Value::Float { value: 0.5 };
    let calls = std::cell::Cell::new(0);
    for (candidate, name, expected) in [
        (None, "roughness", EditError::SchemaUnavailable),
        (Some(&schema), "", EditError::EmptyPropertyName),
        (
            Some(&schema),
            " roughness ",
            EditError::UndeclaredProperty(" roughness ".into()),
        ),
        (
            Some(&wrong),
            "roughness",
            EditError::Source(MaterialDocumentProjectionError::ShaderLocatorMismatch {
                expected: source.shader.clone(),
                actual: wrong.uri.clone(),
            }),
        ),
        (
            Some(&duplicate),
            "roughness",
            EditError::Source(MaterialDocumentProjectionError::DuplicatePropertyName(
                "roughness".into(),
            )),
        ),
    ] {
        let error = plan_material_document_property_reset(&source, candidate, name, |_| {
            calls.set(calls.get() + 1);
            Ok::<_, &'static str>(Some(&resolved))
        })
        .unwrap_err();
        assert_eq!(error, ResetError::Edit(expected));
    }
    assert_eq!(calls.get(), 0);
    assert_eq!(source, before);
}

#[test]
fn material_graph_instance_property_reset_missing_default_and_resolver_failure_publish_nothing() {
    let mut source = document("roughness = 'old invalid author value'", "");
    source.validation_diagnostics.push("保留原诊断".into());
    let before = source.clone();
    let schema = shader();
    let calls = std::cell::Cell::new(0);
    let missing =
        plan_material_document_property_reset(&source, Some(&schema), "roughness", |_| {
            calls.set(calls.get() + 1);
            Ok::<_, &'static str>(None)
        })
        .unwrap_err();
    assert_eq!(missing, ResetError::DefaultUnavailable("roughness".into()));
    assert_eq!(calls.get(), 1);
    let failed = plan_material_document_property_reset(&source, Some(&schema), "roughness", |_| {
        calls.set(calls.get() + 1);
        Err::<Option<&Value>, _>(std::io::Error::other(
            "current default dependency unavailable",
        ))
    })
    .unwrap_err();
    assert_eq!(calls.get(), 2);
    assert_eq!(
        std::error::Error::source(&failed).unwrap().to_string(),
        "current default dependency unavailable"
    );
    assert!(failed
        .to_string()
        .contains("current default dependency unavailable"));
    assert_eq!(source, before);
}

#[test]
fn material_graph_instance_property_reset_rejects_wrong_kind_and_nonfinite_resolved_values() {
    let source = document("roughness = 'invalid local data kept for repair'", "");
    let before = source.clone();
    let schema = shader();
    for value in [
        Value::Float { value: f32::NAN },
        Value::Float {
            value: f32::INFINITY,
        },
        Value::Float {
            value: f32::NEG_INFINITY,
        },
        Value::Vec4 { value: [0.5; 4] },
        Value::Int { value: 1 },
    ] {
        let calls = std::cell::Cell::new(0);
        let error =
            plan_material_document_property_reset(&source, Some(&schema), "roughness", |_| {
                calls.set(calls.get() + 1);
                Ok::<_, &'static str>(Some(&value))
            })
            .unwrap_err();
        assert_eq!(calls.get(), 1);
        match value {
            Value::Float { .. } => assert_eq!(
                error,
                ResetError::Edit(EditError::NonFiniteValue("roughness".into()))
            ),
            _ => assert!(matches!(
                error,
                ResetError::Edit(EditError::TypeMismatch { .. })
            )),
        }
    }
    assert_eq!(source, before);
}

#[test]
fn material_graph_instance_property_reset_preserves_exact_key_and_resolved_signed_zero() {
    let source = document("' 粗糙度 ' = 0.75", "");
    let before = source.clone();
    let mut schema = shader();
    schema.property_schema[1].name = " 粗糙度 ".into();
    // Metadata is deliberately not the current resolved default.
    schema.property_schema[1].default = None;
    let schema_before = schema.clone();
    let resolved = Value::Float { value: -0.0 };
    let plan =
        plan_material_document_property_reset(&source, Some(&schema), " 粗糙度 ", |target| {
            assert_eq!(target.name, " 粗糙度 ");
            Ok::<_, &'static str>(Some(&resolved))
        })
        .unwrap();
    assert_eq!(plan.name(), " 粗糙度 ");
    let Edit::Set(Value::Float { value }) = plan.property_edit().unwrap().edit() else {
        panic!("expected a borrowed finite scalar reset");
    };
    assert_eq!(value.to_bits(), (-0.0f32).to_bits());
    assert!(plan.will_be_overridden());
    assert_eq!(source, before);
    assert_eq!(schema, schema_before);
}

#[test]
fn material_graph_instance_property_reset_roundtrip_retains_parent_metadata_and_full_inverse() {
    let mut source = document("roughness = 0.2", "");
    source.parent = Some(reference("builtin://material/base"));
    let before = source.clone();
    let parent = MaterialAsset::from_zmaterial_document(document("roughness = 0.9", ""));
    let schema = shader();
    let resolved = Value::Float {
        value: parent.roughness,
    };
    let plan = plan_material_document_property_reset(&source, Some(&schema), "roughness", |_| {
        Ok::<_, &'static str>(Some(&resolved))
    })
    .unwrap();
    let edit = plan.property_edit().unwrap();
    let Edit::Set(Value::Float { value }) = edit.edit() else {
        panic!("expected resolved parent scalar");
    };
    // Only this test installs a proposal and inverse. Production history is not installed here.
    let mut changed = source.clone();
    let previous = changed
        .overrides
        .insert(edit.name().into(), f64::from(*value).into());
    let text = changed
        .to_project_toml_string(|value| Ok(PersistedAssetReference::builtin(value.locator.clone())))
        .unwrap();
    let roundtrip = ZMaterialDocument::from_project_toml_str(&text, |value| {
        Ok(AssetReference::from_locator(
            value.builtin_locator().unwrap().clone(),
        ))
    })
    .unwrap();
    assert_eq!(roundtrip.parent, source.parent);
    assert_eq!(roundtrip.shader, source.shader);
    assert_eq!(roundtrip.editor, source.editor);
    assert!(roundtrip.overrides.contains_key("roughness"));
    let mut effective = MaterialAsset::from_zmaterial_document(roundtrip);
    effective.inherit_parent_values_from(&parent);
    assert_eq!(effective.roughness, parent.roughness);
    changed
        .overrides
        .insert("roughness".into(), previous.unwrap());
    assert_eq!(changed, source);
    assert_eq!(source, before);
}

fn document(overrides: &str, textures: &str) -> ZMaterialDocument {
    let text = format!(
        r#"version = 2
name = "实例作者参数"
[shader]
kind = "builtin"
locator = "builtin://shader/pbr"
[overrides]
{overrides}
[textures]
{textures}
[options]
quality = 2
[editor]
note = "保留中文和长作者元数据"
expanded = ["表面参数 / Surface"]
"#
    );
    ZMaterialDocument::from_project_toml_str(&text, |value| {
        Ok(AssetReference::from_locator(
            value.builtin_locator().unwrap().clone(),
        ))
    })
    .unwrap()
}

fn reference(locator: &str) -> AssetReference {
    AssetReference::from_locator(AssetUri::parse(locator).unwrap())
}

fn shader() -> ShaderAsset {
    let defaults = document("roughness = 0.5\ncolor = [1.0, 1.0, 1.0, 1.0]", "");
    ShaderAsset {
        uri: AssetUri::parse("builtin://shader/pbr").unwrap(),
        kind: ShaderAssetKind::Surface,
        source_language: ShaderSourceLanguage::Wgsl,
        source: String::new(),
        wgsl_source: String::new(),
        import_path: None,
        entry_points: Vec::new(),
        dependencies: Vec::new(),
        source_files: Vec::new(),
        imports: Vec::new(),
        shader_defs: Vec::new(),
        property_schema: [
            ("color", MaterialPropertyKind::Vec4, "颜色"),
            (
                "roughness",
                MaterialPropertyKind::Float,
                "粗糙度：保留作者显示名",
            ),
        ]
        .into_iter()
        .map(|(name, kind, label)| ShaderMaterialPropertyAsset {
            name: name.into(),
            kind,
            required: false,
            default: defaults.overrides.get(name).cloned(),
            editor: BTreeMap::from([
                ("group".into(), "表面参数 / Surface".into()),
                ("label".into(), label.into()),
            ]),
        })
        .collect(),
        options: Vec::new(),
        texture_slots: ["base_color", "normal"]
            .into_iter()
            .map(|name| ShaderTextureSlotAsset {
                name: name.into(),
                kind: "texture2d".into(),
                required: false,
                default: Some(
                    if name == "base_color" {
                        "white"
                    } else {
                        "normal"
                    }
                    .into(),
                ),
                sampler: None,
                group: Some("纹理".into()),
                label: Some(name.into()),
                option: None,
                st: false,
                editor: BTreeMap::new(),
            })
            .collect(),
        shading_model: None,
        render_state: Default::default(),
        queue: None,
        disabled_passes: Vec::new(),
        resources: Vec::new(),
        material_property_layout: Default::default(),
        material_option_table: Default::default(),
        generated_material_wgsl: String::new(),
        editor: Default::default(),
        pipeline_layout: Default::default(),
        validation_diagnostics: Vec::new(),
    }
}
