use super::*;

use crate::core::framework::script::{ScriptHostFieldProjection, ScriptHostTypeProjection};
use zircon_runtime_interface::reflect::{
    ReflectEditorHint, ReflectError, ReflectFieldInfo, ReflectScriptVisibility,
    ReflectSerializationStrategy, ReflectTypeInfo, ReflectTypePath, ReflectTypeRegistration,
};

fn indexed_projection_registration(field_names: &[String]) -> ReflectTypeRegistration {
    let fields = field_names
        .iter()
        .enumerate()
        .map(|(index, name)| {
            ReflectFieldInfo::from_stable_keys(
                "test.IndexedProjection",
                name,
                name,
                indexed_projection_type_name(index),
                ReflectEditorHint::None,
            )
        })
        .collect();

    ReflectTypeRegistration::new(
        ReflectTypePath::new("test.IndexedProjection", "IndexedProjection")
            .expect("test reflection path should be valid"),
        "IndexedProjection",
        ReflectTypeInfo::struct_with_fields(fields),
        ReflectSerializationStrategy::Value,
    )
    .with_script_visibility(ReflectScriptVisibility::Public)
}

fn indexed_projection_type_name(field_index: usize) -> String {
    format!("test.ProjectedField{field_index:05}")
}

fn indexed_projection_kind(field_index: usize) -> ScriptHostValueKind {
    if field_index % 2 == 0 {
        ScriptHostValueKind::Float
    } else {
        ScriptHostValueKind::Int
    }
}

#[test]
fn reflect_registration_builds_script_field_projection_in_one_indexed_pass() {
    for field_count in [1_usize, 100, 10_000] {
        let field_names = (0..field_count)
            .map(|index| format!("field_{index:05}"))
            .collect::<Vec<_>>();
        let registration = indexed_projection_registration(&field_names);
        let projection = field_names.iter().enumerate().rev().fold(
            ScriptHostTypeProjection::new(ScriptHostValueKind::Float),
            |projection, (index, name)| {
                projection.with_field(ScriptHostFieldProjection::new(
                    name,
                    indexed_projection_kind(index),
                ))
            },
        );

        let descriptor =
            ScriptHostTypeDescriptor::from_reflect_registration(&registration, &projection)
                .expect("complete shuffled projection should compile");

        assert_eq!(descriptor.fields.len(), field_count);
        for (index, field) in descriptor.fields.iter().enumerate() {
            assert_eq!(field.name, field_names[index]);
            assert_eq!(field.value_kind, indexed_projection_kind(index));
            assert_eq!(
                field.type_ref.type_name,
                indexed_projection_type_name(index)
            );
        }
    }

    let source = include_str!("../../../core/framework/script/descriptors.rs");
    assert!(source.contains("use std::collections::HashMap;"));
    assert!(source.contains("HashMap::with_capacity(projection.fields.len())"));
    assert!(source.contains("projection_fields.remove(field.name.as_str())"));
    assert!(!source.contains("fn field_value_kind("));
}

#[test]
fn reflect_registration_rejects_duplicate_and_unknown_projection_fields() {
    let alpha_names = vec!["alpha".to_string()];
    let alpha_registration = indexed_projection_registration(&alpha_names);
    let duplicate_projection = ScriptHostTypeProjection::new(ScriptHostValueKind::Float)
        .with_field(ScriptHostFieldProjection::new(
            "alpha",
            ScriptHostValueKind::Float,
        ))
        .with_field(ScriptHostFieldProjection::new(
            "alpha",
            ScriptHostValueKind::Int,
        ));

    assert_eq!(
        ScriptHostTypeDescriptor::from_reflect_registration(
            &alpha_registration,
            &duplicate_projection,
        ),
        Err(ReflectError::InvalidRegistration {
            type_path: "test.IndexedProjection".to_string(),
            reason: "script field projection `alpha` is duplicated".to_string(),
        })
    );

    let reflected_only_names = vec!["reflected_only".to_string()];
    let reflected_only_registration = indexed_projection_registration(&reflected_only_names);
    let projected_only = ScriptHostTypeProjection::new(ScriptHostValueKind::Float).with_field(
        ScriptHostFieldProjection::new("projected_only", ScriptHostValueKind::Float),
    );
    assert_eq!(
        ScriptHostTypeDescriptor::from_reflect_registration(
            &reflected_only_registration,
            &projected_only,
        ),
        Err(ReflectError::InvalidRegistration {
            type_path: "test.IndexedProjection".to_string(),
            reason: "script field projection `projected_only` has no reflected field".to_string(),
        })
    );

    let missing_names = vec![
        "first_missing".to_string(),
        "projected".to_string(),
        "second_missing".to_string(),
    ];
    let missing_registration = indexed_projection_registration(&missing_names);
    let missing_projection = ScriptHostTypeProjection::new(ScriptHostValueKind::Float).with_field(
        ScriptHostFieldProjection::new("projected", ScriptHostValueKind::Float),
    );
    assert_eq!(
        ScriptHostTypeDescriptor::from_reflect_registration(
            &missing_registration,
            &missing_projection,
        ),
        Err(ReflectError::InvalidRegistration {
            type_path: "test.IndexedProjection".to_string(),
            reason: "reflected field `first_missing` has no script ABI value-kind projection"
                .to_string(),
        })
    );

    let unknown_before_missing = ScriptHostTypeProjection::new(ScriptHostValueKind::Float)
        .with_field(ScriptHostFieldProjection::new(
            "projected",
            ScriptHostValueKind::Float,
        ))
        .with_field(ScriptHostFieldProjection::new(
            "unknown",
            ScriptHostValueKind::Int,
        ));
    assert_eq!(
        ScriptHostTypeDescriptor::from_reflect_registration(
            &missing_registration,
            &unknown_before_missing,
        ),
        Err(ReflectError::InvalidRegistration {
            type_path: "test.IndexedProjection".to_string(),
            reason: "script field projection `unknown` has no reflected field".to_string(),
        })
    );
}

#[test]
fn host_reflection_docs_render_synthetic_descriptor_deterministically() {
    let alpha = ScriptHostModuleDescriptor::new("example.alpha", "0.1.0")
        .with_capability("alpha.write")
        .with_capability("alpha.read")
        .with_type(
            ScriptHostTypeDescriptor::new("Zeta", ScriptHostValueKind::Int)
                .with_prototype_kind(ScriptHostPrototypeKind::Enum)
                .with_documentation("Zeta docs."),
        )
        .with_type(
            ScriptHostTypeDescriptor::new("Vec3", ScriptHostValueKind::Float)
                .with_prototype_kind(ScriptHostPrototypeKind::Struct)
                .allow_value_construction(true)
                .with_field(
                    ScriptHostFieldDescriptor::new("z", ScriptHostValueKind::Float)
                        .with_type_ref(ScriptHostTypeRef::new(ScriptHostValueKind::Float, "float"))
                        .with_documentation("Z axis."),
                )
                .with_field(
                    ScriptHostFieldDescriptor::new("x", ScriptHostValueKind::Float)
                        .with_type_ref(ScriptHostTypeRef::new(ScriptHostValueKind::Float, "float"))
                        .with_documentation("X axis."),
                )
                .with_documentation("Vec3 docs."),
        )
        .with_function(
            ScriptHostFunctionDescriptor::new("length", 1, 1, ScriptHostValueKind::Float)
                .with_return_type(ScriptHostTypeRef::new(ScriptHostValueKind::Float, "float"))
                .with_required_capability("alpha.read")
                .with_parameter(
                    ScriptHostParameterDescriptor::new("y", ScriptHostValueKind::Float)
                        .with_type_ref(ScriptHostTypeRef::new(ScriptHostValueKind::Float, "float")),
                )
                .with_parameter(
                    ScriptHostParameterDescriptor::new("x", ScriptHostValueKind::Float)
                        .with_type_ref(ScriptHostTypeRef::new(ScriptHostValueKind::Float, "float")),
                )
                .with_documentation("Return vector length."),
        )
        .with_function(
            ScriptHostFunctionDescriptor::new("normalize", 1, 1, ScriptHostValueKind::Float)
                .with_return_type(ScriptHostTypeRef::new(ScriptHostValueKind::Float, "Vec3"))
                .with_required_capability("alpha.write")
                .with_parameter(
                    ScriptHostParameterDescriptor::new("value", ScriptHostValueKind::Float)
                        .with_type_ref(ScriptHostTypeRef::new(ScriptHostValueKind::Float, "Vec3")),
                )
                .with_documentation("Normalize vector."),
        )
        .with_documentation("Example module docs.");
    let beta = ScriptHostModuleDescriptor::new("example.beta", "0.1.0")
        .with_documentation("Beta module docs.");

    let markdown = render_script_host_modules_markdown(
        &[beta, alpha],
        &ScriptHostInterfaceMarkdownOptions::default(),
    );

    assert_eq!(
        markdown,
        concat!(
            "# ZrVM Host Interface\n",
            "\n",
            "## Module `example.alpha`\n",
            "\n",
            "- Version: `0.1.0`\n",
            "- Documentation: Example module docs.\n",
            "- Capabilities: `alpha.read`, `alpha.write`\n",
            "\n",
            "### Types\n",
            "\n",
            "#### Type `Vec3`\n",
            "\n",
            "- Type ref: `Vec3` (`float`)\n",
            "- Prototype: `struct`\n",
            "- Value construction: `true`\n",
            "- Documentation: Vec3 docs.\n",
            "\n",
            "Fields:\n",
            "\n",
            "- `z`: `float` (`float`) - Z axis.\n",
            "- `x`: `float` (`float`) - X axis.\n",
            "\n",
            "#### Type `Zeta`\n",
            "\n",
            "- Type ref: `Zeta` (`int`)\n",
            "- Prototype: `enum`\n",
            "- Value construction: `false`\n",
            "- Documentation: Zeta docs.\n",
            "\n",
            "### Functions\n",
            "\n",
            "#### Function `length`\n",
            "\n",
            "- Return: `float` (`float`)\n",
            "- Required capabilities: `alpha.read`\n",
            "- Documentation: Return vector length.\n",
            "\n",
            "Parameters:\n",
            "\n",
            "- `y`: `float` (`float`)\n",
            "- `x`: `float` (`float`)\n",
            "\n",
            "#### Function `normalize`\n",
            "\n",
            "- Return: `Vec3` (`float`)\n",
            "- Required capabilities: `alpha.write`\n",
            "- Documentation: Normalize vector.\n",
            "\n",
            "Parameters:\n",
            "\n",
            "- `value`: `Vec3` (`float`)\n",
            "\n",
            "## Module `example.beta`\n",
            "\n",
            "- Version: `0.1.0`\n",
            "- Documentation: Beta module docs.\n",
            "\n",
        )
    );
}

#[test]
fn host_reflection_docs_clamp_heading_levels_without_overflow() {
    let descriptor = ScriptHostModuleDescriptor::new("heading.example", "0.1.0");
    let mut options = ScriptHostInterfaceMarkdownOptions {
        title: "Heading Edge".to_string(),
        heading_level: 0,
        include_capabilities: true,
        include_empty_sections: false,
    };

    assert_eq!(
        render_script_host_modules_markdown(&[descriptor.clone()], &options),
        concat!(
            "# Heading Edge\n",
            "\n",
            "## Module `heading.example`\n",
            "\n",
            "- Version: `0.1.0`\n",
            "\n",
        )
    );

    options.heading_level = 99;
    assert_eq!(
        render_script_host_modules_markdown(&[descriptor.clone()], &options),
        concat!(
            "###### Heading Edge\n",
            "\n",
            "###### Module `heading.example`\n",
            "\n",
            "- Version: `0.1.0`\n",
            "\n",
        )
    );

    options.heading_level = usize::MAX;
    assert_eq!(
        render_script_host_modules_markdown(&[descriptor], &options),
        concat!(
            "###### Heading Edge\n",
            "\n",
            "###### Module `heading.example`\n",
            "\n",
            "- Version: `0.1.0`\n",
            "\n",
        )
    );
}

#[test]
fn host_reflection_docs_writer_creates_parent_directory_and_file() {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let root = std::env::temp_dir().join(format!("zircon-host-reflection-docs-{nonce}"));
    let output_path = root.join("nested").join("host-interface.md");
    let descriptor = ScriptHostModuleDescriptor::new("writer.example", "0.1.0")
        .with_documentation("Writer example.");

    write_script_host_modules_markdown(
        &output_path,
        &[descriptor],
        &ScriptHostInterfaceMarkdownOptions::default(),
    )
    .unwrap();

    let contents = fs::read_to_string(&output_path).unwrap();
    assert!(contents.contains("## Module `writer.example`"));
    assert!(contents.contains("Writer example."));

    fs::remove_dir_all(root).unwrap();
}

#[test]
fn host_reflection_docs_include_macro_generated_builtin_math_module() {
    let modules = builtin_host_module_descriptors().unwrap();
    let markdown = render_script_host_modules_markdown(
        &modules,
        &ScriptHostInterfaceMarkdownOptions::default(),
    );

    assert!(markdown.contains("## Module `zr.zircon.math`"));
    assert!(markdown.contains("#### Type `Vec3`"));
    assert!(markdown.contains("#### Type `ColorRgba`"));
    assert!(markdown.contains("#### Function `vec3_length`"));
    assert!(markdown.contains("#### Function `vec3_dot`"));
    for name in [
        "abs", "atan2", "ceil", "cos", "exp", "floor", "sin", "sqrt", "pow",
    ] {
        assert!(
            markdown.contains(&format!("#### Function `{name}`")),
            "generated math reflection must publish {name}"
        );
    }
    assert!(markdown.contains("Deterministic scalar ABI backed by libm 0.2.16"));
    assert!(markdown.contains("- Return: `float` (`float`)"));
    assert!(markdown.contains("- `x`: `float` (`float`)"));
}

#[test]
fn rust_reflection_macros_generate_type_function_and_module_descriptors() {
    #[derive(crate::ZirconScriptType)]
    #[zircon_script(
        name = "TestVec3",
        value_kind = ScriptHostValueKind::Float,
        prototype = ScriptHostPrototypeKind::Struct,
        allow_value_construction = true,
        documentation = "test vector"
    )]
    struct TestVec3 {
        #[zircon_script(type_name = "float", documentation = "x axis")]
        x: f64,
        #[zircon_script(type_name = "float")]
        y: f64,
        #[zircon_script(type_name = "float")]
        z: f64,
    }

    let registration = TestVec3::reflect_type_registration().unwrap();
    let type_descriptor = TestVec3::script_host_type_descriptor().unwrap();
    let test_vec3 = TestVec3 {
        x: 1.0,
        y: 2.0,
        z: 3.0,
    };
    assert_eq!(test_vec3.x + test_vec3.y + test_vec3.z, 6.0);
    assert_eq!(type_descriptor.name, "TestVec3");
    assert_eq!(registration.display_name, type_descriptor.name);
    assert_eq!(registration.documentation.as_deref(), Some("test vector"));
    assert_eq!(type_descriptor.documentation, registration.documentation);
    assert_eq!(
        registration.script_visibility,
        zircon_runtime_interface::reflect::ReflectScriptVisibility::Public
    );
    assert_eq!(
        registration.type_info.fields,
        vec![
            zircon_runtime_interface::reflect::ReflectFieldInfo::from_stable_keys(
                "TestVec3",
                "x",
                "x",
                "float",
                zircon_runtime_interface::reflect::ReflectEditorHint::None,
            )
            .with_serializable(false)
            .with_editor_visible(false)
            .with_documentation("x axis"),
            zircon_runtime_interface::reflect::ReflectFieldInfo::from_stable_keys(
                "TestVec3",
                "y",
                "y",
                "float",
                zircon_runtime_interface::reflect::ReflectEditorHint::None,
            )
            .with_serializable(false)
            .with_editor_visible(false),
            zircon_runtime_interface::reflect::ReflectFieldInfo::from_stable_keys(
                "TestVec3",
                "z",
                "z",
                "float",
                zircon_runtime_interface::reflect::ReflectEditorHint::None,
            )
            .with_serializable(false)
            .with_editor_visible(false),
        ]
    );
    assert_eq!(type_descriptor.type_ref.type_name, "TestVec3");
    assert_eq!(type_descriptor.fields[0].type_ref.type_name, "float");
    assert_eq!(
        type_descriptor.fields[0].documentation.as_deref(),
        Some("x axis")
    );
    assert!(type_descriptor.allow_value_construction);

    #[derive(crate::ZirconScriptType)]
    #[zircon_script(name = "TestEnum", value_kind = ScriptHostValueKind::Int)]
    enum TestEnum {
        A,
    }

    assert!(matches!(TestEnum::A, TestEnum::A));
    let enum_descriptor = TestEnum::script_host_type_descriptor().unwrap();
    assert_eq!(
        enum_descriptor.prototype_kind,
        ScriptHostPrototypeKind::Enum
    );

    #[crate::zircon_host_module(
        name = "test.macro.math",
        version = "0.1.0",
        capability = "test.math",
        documentation = "macro math"
    )]
    mod macro_math {
        use super::*;

        #[derive(crate::ZirconScriptType)]
        #[zircon_script(
            name = "Point",
            value_kind = ScriptHostValueKind::Float,
            allow_value_construction = true
        )]
        struct Point {
            #[zircon_script(type_name = "float")]
            x: f64,
        }

        pub fn point_fixture_x() -> f64 {
            Point { x: 3.5 }.x
        }

        #[crate::zircon_host_function(
            name = "double",
            return_type_name = "float",
            capability = "test.math",
            documentation = "double input"
        )]
        fn double(value: f64) -> f64 {
            value * 2.0
        }
    }

    let descriptor = macro_math::macro_math_host_module_descriptor().unwrap();
    assert_eq!(descriptor.name, "test.macro.math");
    assert_eq!(descriptor.capabilities, vec!["test.math".to_string()]);
    assert_eq!(descriptor.types[0].name, "Point");
    assert_eq!(macro_math::point_fixture_x(), 3.5);
    assert_eq!(descriptor.functions[0].name, "double");
    assert_eq!(
        descriptor.functions[0].parameters[0].type_ref.type_name,
        "float"
    );
    assert_eq!(descriptor.functions[0].return_type.type_name, "float");

    let exports = HostExportRegistry::default();
    macro_math::register_macro_math_host_module(&exports).unwrap();
    let value = exports
        .call_with_capabilities(
            "test.macro.math",
            "double",
            vec![ScriptHostValue::Float(2.5)],
            &CapabilitySet::default().with("test.math"),
        )
        .unwrap();
    assert_eq!(value, ScriptHostValue::Float(5.0));
}
