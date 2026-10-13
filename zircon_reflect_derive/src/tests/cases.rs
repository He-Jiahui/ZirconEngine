//! 派生实现的单元覆盖：测试直接构造 `syn::DeriveInput` 并检查展开结果或诊断。
//! 这些用例验证 proc-macro 内部展开契约；运行时 trait 调用另由 `zircon_runtime` 的反射测试覆盖。

use syn::DeriveInput;

/// 检查显式容器/字段元数据进入注册 token，并生成名称与槽位读写方法。
#[test]
fn derive_round_trips_reflect_type_info() {
    let input: DeriveInput = syn::parse_quote! {
        #[zr_reflect(
            component,
            identity = "gameplay.health",
            script_visibility = "public",
            display_name = "Health"
        )]
        struct Health {
            #[zr_reflect(identity = "current-value", alias = "health", editor_hint = "Scalar")]
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
    assert!(!expansion.contains("with_plugin_owned"));
    assert!(expansion.contains("ReflectScriptVisibility :: Public"));
    assert!(expansion.contains(". with_remote_visible (false)"));
    assert!(expansion.contains("ReflectFieldId :: from_stable_keys"));
    assert!(expansion.contains("\"gameplay.health\" , \"current-value\""));
    assert!(expansion.contains("\"current\" , \"Scalar\""));
    assert!(expansion.contains("with_aliases (vec ! [\"health\" . to_string ()])"));
    assert!(expansion.contains("\"maximum\" , \"Scalar\""));
    assert!(expansion.contains(". with_editable (false)"));
    assert!(expansion.contains("read_reflected_field"));
    assert!(expansion.contains("write_reflected_field"));
    assert!(expansion.contains("read_reflected_field_by_slot"));
    assert!(expansion.contains("write_reflected_field_by_slot"));
    assert!(expansion.contains("0u32 =>"));
    assert!(expansion.contains("1u32 =>"));
}

/// 检查类型身份键和字段身份键带首尾空白时都会拒绝展开。
#[test]
fn derive_rejects_non_canonical_stable_identity_keys() {
    let type_input: DeriveInput = syn::parse_quote! {
        #[zr_reflect(identity = " gameplay.health")]
        struct Health {
            current: f32,
        }
    };
    let error = crate::derive::derive_zr_reflect_impl(type_input)
        .expect_err("type identity keys must not hide leading whitespace");
    assert!(error.to_string().contains("non-empty and already trimmed"));

    let field_input: DeriveInput = syn::parse_quote! {
        struct Health {
            #[zr_reflect(identity = "current-value ")]
            current: f32,
        }
    };
    let error = crate::derive::derive_zr_reflect_impl(field_input)
        .expect_err("field identity keys must not hide trailing whitespace");
    assert!(error.to_string().contains("non-empty and already trimmed"));
}

/// 拒绝已退役的 `plugin_owned` 容器键，避免派生宏重新生成旧投影。
#[test]
fn derive_rejects_retired_plugin_owned_projection() {
    let input: DeriveInput = syn::parse_quote! {
        #[zr_reflect(component, plugin_owned)]
        struct Invalid {
            value: bool,
        }
    };

    let error = crate::derive::derive_zr_reflect_impl(input)
        .expect_err("plugin ownership must come from the canonical type path");

    assert!(error
        .to_string()
        .contains("unknown zr_reflect type attribute"));
}

/// 检查枚举保留 `Enum` 类型形状，且不会把变体伪装成普通字段。
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

/// 组件与资源角色同时声明时必须在宏展开阶段返回错误。
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

/// 未识别 Rust 字段类型没有显式反射值路径时必须拒绝展开。
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

/// 检查已支持的列表元素类型递归生成嵌套 `List<...>` schema 路径。
#[test]
fn derive_infers_typed_list_value_paths() {
    let input: DeriveInput = syn::parse_quote! {
        struct TypedLists {
            names: Vec<String>,
            weights: Vec<Vec<f32>>,
        }
    };

    let expansion = crate::derive::derive_zr_reflect_impl(input)
        .expect("supported list element types should infer a typed reflection path")
        .to_string();

    assert!(expansion.contains("\"names\" , \"List<String>\""));
    assert!(expansion.contains("\"weights\" , \"List<List<Scalar>>\""));
}

/// 未识别的列表元素不能退化为缺少元素类型的宽泛 `List` 路径。
#[test]
fn derive_requires_an_explicit_path_for_unknown_list_elements() {
    let input: DeriveInput = syn::parse_quote! {
        struct UnknownListElement {
            values: Vec<DomainSpecificValue>,
        }
    };

    let error = crate::derive::derive_zr_reflect_impl(input)
        .expect_err("unknown list elements must not degrade to an untyped List declaration");

    assert!(error
        .to_string()
        .contains("requires value_type_path = \"...\""));
}

/// 检查虚拟字段通过自定义回调读写，且只读回调不生成可编辑字段。
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

/// 实体字段与容器声明的虚拟字段共享唯一的反射名称空间。
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

/// 平台位宽整数不自动推断可移植 schema，必须显式选择反射值类型路径。
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
