use std::collections::BTreeMap;

use super::{
    plan_material_document_property_edit, project_authored_material_document,
    MaterialDocumentProjectionError, MaterialDocumentPropertyEdit as Edit,
    MaterialDocumentPropertyEditError as EditError,
};
use zircon_editor::ui::material_editor::MaterialEditorProjection;
use zircon_runtime::asset::{
    AssetReference, AssetUri, MaterialAsset, ShaderAsset, ShaderMaterialPropertyAsset,
    ShaderSourceLanguage, ShaderTextureSlotAsset, ZMaterialDocument,
};
use zircon_runtime::core::framework::render::{
    MaterialPropertyKind, RenderMaterialDiagnosticSource, RenderMaterialPropertyValue as Value,
    ShaderAssetKind,
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
