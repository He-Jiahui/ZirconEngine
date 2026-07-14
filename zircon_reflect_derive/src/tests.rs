use syn::DeriveInput;

#[test]
fn derive_round_trips_reflect_type_info() {
    let input: DeriveInput = syn::parse_quote! {
        #[zr_reflect(
            component,
            script_visibility = "public",
            display_name = "Health"
        )]
        struct Health {
            #[zr_reflect(editor_hint = "Scalar")]
            current: f32,
            #[zr_reflect(editor_hint = "Scalar", readonly)]
            maximum: f32,
        }
    };

    let expansion = crate::derive::derive_zr_reflect_impl(input)
        .expect("valid reflection input should expand")
        .to_string();

    assert!(
        expansion.contains("impl :: zircon_runtime_interface :: reflect :: ZrReflect for Health")
    );
    assert!(expansion.contains("ReflectTypeKind :: Struct"));
    assert!(expansion.contains("ReflectSerializationStrategy :: Value"));
    assert!(expansion.contains(". as_component ()"));
    assert!(expansion.contains("ReflectScriptVisibility :: Public"));
    assert!(expansion.contains(". with_remote_visible (false)"));
    assert!(expansion.contains("ReflectFieldInfo :: new (\"current\" , \"Scalar\""));
    assert!(expansion.contains("ReflectFieldInfo :: new (\"maximum\" , \"Scalar\""));
    assert!(expansion.contains(". with_editable (false)"));
    assert!(expansion.contains("read_reflected_field"));
    assert!(expansion.contains("write_reflected_field"));
    assert!(expansion.contains("read_reflected_field_by_slot"));
    assert!(expansion.contains("write_reflected_field_by_slot"));
    assert!(expansion.contains("0u32 =>"));
    assert!(expansion.contains("1u32 =>"));
}

#[test]
fn derive_preserves_enum_shape_without_inventing_fields() {
    let input: DeriveInput = syn::parse_quote! {
        #[zr_reflect(script_visibility = "private")]
        enum GameplayState {
            Idle,
            Running,
        }
    };

    let expansion = crate::derive::derive_zr_reflect_impl(input)
        .expect("fieldless enum reflection should expand")
        .to_string();

    assert!(expansion.contains("ReflectTypeKind :: Enum"));
    assert!(expansion.contains("Vec :: new ()"));
    assert!(expansion.contains("ReflectScriptVisibility :: Private"));
}

#[test]
fn derive_rejects_conflicting_component_and_resource_flags() {
    let input: DeriveInput = syn::parse_quote! {
        #[zr_reflect(component, resource)]
        struct Invalid {
            value: bool,
        }
    };

    let error = crate::derive::derive_zr_reflect_impl(input)
        .expect_err("a reflected type cannot be both component and resource");

    assert!(error
        .to_string()
        .contains("cannot be both a component and a resource"));
}

#[test]
fn derive_rejects_unknown_field_type_without_explicit_value_path() {
    let input: DeriveInput = syn::parse_quote! {
        struct UnknownFieldType {
            value: DomainSpecificValue,
        }
    };

    let error = crate::derive::derive_zr_reflect_impl(input)
        .expect_err("unknown field types must declare their reflected value path");

    assert!(error
        .to_string()
        .contains("requires value_type_path = \"...\""));
}

#[test]
fn derive_expands_virtual_fields_through_custom_accessors() {
    let input: DeriveInput = syn::parse_quote! {
        #[zr_reflect(
            component,
            field(
                name = "translation",
                value_type_path = "Vec3",
                editor_hint = "Vec3",
                read = "reflection::read_translation",
                write = "reflection::write_translation"
            ),
            field(
                name = "rotation",
                value_type_path = "Vec4",
                editor_hint = "Vec4",
                read = "reflection::read_rotation",
                readonly
            )
        )]
        struct LocalTransform {
            #[zr_reflect(skip)]
            transform: Transform,
        }
    };

    let expansion = crate::derive::derive_zr_reflect_impl(input)
        .expect("virtual reflected fields should expand")
        .to_string();

    assert!(expansion.contains("reflection :: read_translation (self)"));
    assert!(expansion.contains("reflection :: write_translation (self , value)"));
    assert!(expansion.contains("reflection :: read_rotation (self)"));
    assert!(expansion.contains("NonEditableField"));
    assert!(!expansion.contains("self . transform"));
}

#[test]
fn derive_rejects_duplicate_reflected_field_names() {
    let input: DeriveInput = syn::parse_quote! {
        #[zr_reflect(
            component,
            field(
                name = "value",
                value_type_path = "Scalar",
                read = "reflection::read_value",
                readonly
            )
        )]
        struct DuplicateField {
            value: f32,
        }
    };

    let error = crate::derive::derive_zr_reflect_impl(input)
        .expect_err("named and virtual reflected fields must share one unique namespace");

    assert!(error
        .to_string()
        .contains("duplicate reflected field name `value`"));
}

#[test]
fn derive_requires_explicit_accessors_for_platform_sized_integers() {
    for field_type in ["isize", "usize"] {
        let source = format!("struct PlatformSized {{ value: {field_type} }}");
        let input = syn::parse_str::<DeriveInput>(&source).expect("test input should parse");

        let error = crate::derive::derive_zr_reflect_impl(input)
            .expect_err("platform-sized integers must not infer a portable reflection layout");

        assert!(error
            .to_string()
            .contains("requires value_type_path = \"...\""));
    }
}
