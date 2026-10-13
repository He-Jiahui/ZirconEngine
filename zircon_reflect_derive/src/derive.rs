//! 把 `syn::DeriveInput` 转换为运行时 `ZrReflect` trait 实现。
//! 类型注册元数据、字段名分派和当前字段槽位分派均由同一份规范化字段序列生成。

use proc_macro2::TokenStream;
use quote::quote;
use std::collections::BTreeSet;
use syn::spanned::Spanned;
use syn::{Data, DeriveInput};

use crate::attributes::{parse_container_attributes, ScriptVisibilityKind, SerializationKind};
use crate::fields::{
    collect_fields, collect_virtual_fields, field_info_tokens, read_arm_tokens,
    read_slot_arm_tokens, write_arm_tokens, write_slot_arm_tokens,
};

/// 根据一个 Rust 类型的语法树构造 `ZrReflect` 实现。
///
/// # 使用时机与生命周期
/// 由公开派生入口 [`crate::derive_zr_reflect`] 在编译期调用；输入和中间配置只用于构造本次展开结果。
///
/// # 参数
/// `input` 是 `syn` 解析后的目标项，容器和字段属性由对应解析器读取。
///
/// # 返回值
/// 返回实现运行时反射 trait 的 token；不支持的语法或配置返回带 span 的 `syn::Error`。
///
/// # 约束
/// 目标项不能有泛型参数且不能是 union；结构体成员或自定义回调须生成与反射值类型相容的读写结果。
/// 自动写入字段还须实现 `PartialEq`，用于判断写入是否改变值。枚举变体不转成普通字段。
///
/// # 参考
/// 输入配置由 [`crate::attributes::parse_container_attributes`] 解析，实体字段由 [`crate::fields::collect_fields`] 收集；
/// 结果实现 `zircon_runtime_interface::reflect::ZrReflect`，由运行时反射适配器调用。
pub(crate) fn derive_zr_reflect_impl(input: DeriveInput) -> syn::Result<TokenStream> {
    let input_span = input.span();
    if !input.generics.params.is_empty() {
        return Err(syn::Error::new(
            input.generics.span(),
            "ZrReflect does not support generic parameters",
        ));
    }

    let attributes = parse_container_attributes(&input.attrs)?;
    let ident = input.ident;
    let short_type_path = ident.to_string();
    let display_name = attributes
        .display_name
        .unwrap_or_else(|| short_type_path.clone());
    let type_path = attributes
        .type_path
        .clone()
        .map(|path| quote!(#path))
        .unwrap_or_else(|| quote!(concat!(module_path!(), "::", stringify!(#ident))));
    // 显式身份键可使字段 ID 不随类型路径改名而变化；缺省身份键仍是当前类型路径。
    let type_identity = attributes
        .identity
        .map(|identity| quote!(#identity))
        .unwrap_or_else(|| type_path.clone());

    // 枚举变体不作为普通字段暴露；union 的重叠存储无法由此反射访问器安全表达。
    let (kind, mut fields) = match input.data {
        Data::Struct(data) => (
            quote!(::zircon_runtime_interface::reflect::ReflectTypeKind::Struct),
            collect_fields(&data.fields)?,
        ),
        Data::Enum(_) => (
            quote!(::zircon_runtime_interface::reflect::ReflectTypeKind::Enum),
            Vec::new(),
        ),
        Data::Union(union) => {
            return Err(syn::Error::new(
                union.union_token.span(),
                "ZrReflect does not support unions",
            ))
        }
    };
    // 槽位顺序先取未跳过的实体字段，再接虚拟字段；稳定字段 ID 不依赖该顺序。
    fields.extend(collect_virtual_fields(attributes.virtual_fields)?);
    validate_unique_field_names(&fields, input_span)?;

    // 元数据字段顺序与两种分派共用该序列，因而生成槽位和注册索引保持一致。
    let field_info = fields
        .iter()
        .map(|field| field_info_tokens(field, &type_identity));
    let reflected_fields = if fields.is_empty() {
        quote!(::std::vec::Vec::new())
    } else {
        quote!(vec![#(#field_info),*])
    };
    let read_arms = fields.iter().map(read_arm_tokens);
    let read_slot_arms = fields
        .iter()
        .enumerate()
        .map(|(index, field)| read_slot_arm_tokens(field, index));
    let write_arms = fields
        .iter()
        .map(|field| write_arm_tokens(field, &type_path));
    let write_slot_arms = fields
        .iter()
        .enumerate()
        .map(|(index, field)| write_slot_arm_tokens(field, index, &type_path));
    let serialization = serialization_tokens(attributes.serialization);
    let component = attributes.component.then(|| quote!(.as_component()));
    let resource = attributes.resource.then(|| quote!(.as_resource()));
    let serializable = attributes
        .serializable
        .unwrap_or(!matches!(attributes.serialization, SerializationKind::None));
    let editor_visible = attributes.editor_visible.unwrap_or(true);
    let remote_visible = attributes.remote_visible.unwrap_or(false);
    let script_visibility = script_visibility_tokens(attributes.script_visibility);

    Ok(quote! {
        impl ::zircon_runtime_interface::reflect::ZrReflect for #ident {
            // 调用者请求注册信息时才构造路径、字段元数据和可见性配置。
            fn reflect_type_registration(
            ) -> Result<
                ::zircon_runtime_interface::reflect::ReflectTypeRegistration,
                ::zircon_runtime_interface::reflect::ReflectError,
            > {
                let type_path = ::zircon_runtime_interface::reflect::ReflectTypePath::new(
                    #type_path,
                    #short_type_path,
                )?;
                let fields = #reflected_fields;
                Ok(::zircon_runtime_interface::reflect::ReflectTypeRegistration::new(
                    type_path,
                    #display_name,
                    ::zircon_runtime_interface::reflect::ReflectTypeInfo::new(#kind, fields),
                    #serialization,
                )
                #component
                #resource
                .with_serializable(#serializable)
                .with_editor_visible(#editor_visible)
                .with_remote_visible(#remote_visible)
                .with_script_visibility(#script_visibility))
            }

            // 按规范字段名读取；具体字段访问由字段 token 生成器选择成员或自定义回调。
            fn read_reflected_field(
                &self,
                field_name: &str,
            ) -> Result<
                ::zircon_runtime_interface::reflect::ReflectedValue,
                ::zircon_runtime_interface::reflect::ReflectError,
            > {
                match field_name {
                    #(#read_arms)*
                    _ => Err(::zircon_runtime_interface::reflect::ReflectError::UnknownField {
                        type_path: (#type_path).to_string(),
                        field_name: field_name.to_string(),
                    }),
                }
            }

            // 按规范字段名写入；返回值沿用字段写分支报告的“值是否改变”。
            fn write_reflected_field(
                &mut self,
                field_name: &str,
                value: ::zircon_runtime_interface::reflect::ReflectedValue,
            ) -> Result<bool, ::zircon_runtime_interface::reflect::ReflectError> {
                match field_name {
                    #(#write_arms,)*
                    _ => Err(::zircon_runtime_interface::reflect::ReflectError::UnknownField {
                        type_path: (#type_path).to_string(),
                        field_name: field_name.to_string(),
                    }),
                }
            }

            // 槽位仅对应本次注册序列；未知槽位由该实现转成 UnknownField 错误。
            fn read_reflected_field_by_slot(
                &self,
                field_slot: u32,
            ) -> Result<
                ::zircon_runtime_interface::reflect::ReflectedValue,
                ::zircon_runtime_interface::reflect::ReflectError,
            > {
                match field_slot {
                    #(#read_slot_arms)*
                    _ => Err(::zircon_runtime_interface::reflect::ReflectError::UnknownField {
                        type_path: (#type_path).to_string(),
                        field_name: format!("#{field_slot}"),
                    }),
                }
            }

            // 按当前槽位写入，并保留字段写分支的变更标记和错误。
            fn write_reflected_field_by_slot(
                &mut self,
                field_slot: u32,
                value: ::zircon_runtime_interface::reflect::ReflectedValue,
            ) -> Result<bool, ::zircon_runtime_interface::reflect::ReflectError> {
                match field_slot {
                    #(#write_slot_arms,)*
                    _ => Err(::zircon_runtime_interface::reflect::ReflectError::UnknownField {
                        type_path: (#type_path).to_string(),
                        field_name: format!("#{field_slot}"),
                    }),
                }
            }
        }
    })
}

/// 确保实体字段和虚拟字段共用的反射名称空间中没有重名。
///
/// # 参数
/// `fields` 是跳过过滤后、虚拟字段追加后的规范字段序列；`span` 是重复名称诊断的位置。
///
/// # 返回值
/// 所有名称唯一时返回 `Ok(())`，遇到第二个同名字段时返回 `syn::Error`。
///
/// # 参考
/// 主流程在 [`derive_zr_reflect_impl`] 验证后才生成字段元数据；重名用例位于 `src/tests/cases.rs`。
fn validate_unique_field_names(
    fields: &[crate::fields::ReflectedField],
    span: proc_macro2::Span,
) -> syn::Result<()> {
    let mut names = BTreeSet::new();
    for field in fields {
        if !names.insert(field.name.as_str()) {
            return Err(syn::Error::new(
                span,
                format!("duplicate reflected field name `{}`", field.name),
            ));
        }
    }
    Ok(())
}

/// 将解析出的策略枚举映射为 runtime-interface 中对应的全限定路径 token。
fn serialization_tokens(kind: SerializationKind) -> TokenStream {
    let variant = match kind {
        SerializationKind::None => quote!(None),
        SerializationKind::Value => quote!(Value),
        SerializationKind::Json => quote!(Json),
        SerializationKind::ResourceHandle => quote!(ResourceHandle),
        SerializationKind::EntityReference => quote!(EntityReference),
    };
    quote!(::zircon_runtime_interface::reflect::ReflectSerializationStrategy::#variant)
}

/// 将解析出的脚本可见级别映射为 runtime-interface 的可见性枚举 token。
fn script_visibility_tokens(kind: ScriptVisibilityKind) -> TokenStream {
    let variant = match kind {
        ScriptVisibilityKind::Private => quote!(Private),
        ScriptVisibilityKind::Public => quote!(Public),
    };
    quote!(::zircon_runtime_interface::reflect::ReflectScriptVisibility::#variant)
}
