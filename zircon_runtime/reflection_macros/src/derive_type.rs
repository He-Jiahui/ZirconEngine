//! 将 zircon_script 元数据编译成框架 trait 实现：反射注册信息与脚本类型投影分别进入 ZirconScriptType 的两个方法。

use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::spanned::Spanned;
use syn::{Data, DeriveInput, Fields};

use crate::attrs::{parse_field_attrs, parse_script_type_attrs};
use crate::tokens::{path_tokens, script_host_type_ref_tokens};

/// 从派生输入生成反射注册描述和脚本投影的两个 trait 方法。
///
/// ## 参数
///
/// `input` 拥有 Rust 类型 AST 及其 `zircon_script` 属性。类型名称默认 Rust 标识符，类型身份键默认该名称；值种类默认 Null，原型默认 Struct 或 Enum，值构造标志默认 false。
///
/// ## 返回
///
/// 成功返回 `ZirconScriptType` 实现的 token；泛型参数、union、未知属性、语法错误或空白不规范的身份键返回 `syn::Error`，由公开入口转成编译诊断。
///
/// ## 约束与生命周期
///
/// 结构字段按声明顺序生成，`skip` 同时排除注册和投影；枚举仅生成 Enum 形状。字段 type-ref 的缺省部分从 `ScriptHostFromArgument` 读取，两个覆盖值均给出时直接构造指定描述。生成的注册描述使用 Value 角色、公共脚本可见性及 None 序列化策略；其构造是元数据操作，实例读写与序列化适配器需由其它层提供。类型路径错误在生成的注册方法被调用时返回；类型 descriptor 的组合还会校验注册字段与投影的一致性。每个生成方法重新构造拥有字符串和字段列表的数据。
///
/// ## 参考
///
/// 参阅 `field_tokens`、Runtime 的 `ZirconScriptType::script_host_type_descriptor` 与 Interface 的 `ReflectTypePath::new`、`ReflectTypeRegistration::new`。
pub(crate) fn derive_zircon_script_type_impl(input: DeriveInput) -> syn::Result<TokenStream2> {
    let ident = input.ident;
    if !input.generics.params.is_empty() {
        return Err(syn::Error::new(
            input.generics.span(),
            "ZirconScriptType does not support generic parameters",
        ));
    }
    let args = parse_script_type_attrs(&input.attrs)?;
    let type_name = args.name.unwrap_or_else(|| ident.to_string());
    let type_identity = args.identity.unwrap_or_else(|| type_name.clone());
    if type_identity.is_empty() || type_identity.trim() != type_identity {
        return Err(syn::Error::new(
            ident.span(),
            "zircon_script type identity must be non-empty and already trimmed",
        ));
    }
    let value_kind = args.value_kind.map(path_tokens).unwrap_or_else(|| {
        quote!(::zircon_runtime::core::framework::script::ScriptHostValueKind::Null)
    });
    let allow_value_construction = args.allow_value_construction.unwrap_or(false);
    let documentation = args
        .documentation
        .map(|doc| quote!(.with_documentation(#doc)));
    let (type_info, fields, default_prototype) = match input.data {
        Data::Struct(data) => {
            let (registrations, projections) = field_tokens(&data.fields, &type_identity)?;
            (
                quote!(::zircon_runtime::core::framework::script::__reflect::ReflectTypeInfo::struct_with_fields(
                    vec![#(#registrations),*]
                )),
                projections,
                quote!(::zircon_runtime::core::framework::script::ScriptHostPrototypeKind::Struct),
            )
        }
        Data::Enum(_) => (
            quote!(
                ::zircon_runtime::core::framework::script::__reflect::ReflectTypeInfo::new(
                    ::zircon_runtime::core::framework::script::__reflect::ReflectTypeKind::Enum,
                    Vec::new(),
                )
            ),
            Vec::new(),
            quote!(::zircon_runtime::core::framework::script::ScriptHostPrototypeKind::Enum),
        ),
        Data::Union(data) => {
            return Err(syn::Error::new(
                data.union_token.span(),
                "ZirconScriptType does not support unions",
            ))
        }
    };
    let prototype = args.prototype.map(path_tokens).unwrap_or(default_prototype);

    Ok(quote! {
        impl ::zircon_runtime::core::framework::script::ZirconScriptType for #ident {
            fn reflect_type_registration() -> Result<
                ::zircon_runtime::core::framework::script::__reflect::ReflectTypeRegistration,
                ::zircon_runtime::core::framework::script::__reflect::ReflectError,
            > {
                // 完整路径与短名都使用脚本公开名称，保持路径末段与注册描述一致。
                let type_path = ::zircon_runtime::core::framework::script::__reflect::ReflectTypePath::new(
                    concat!(module_path!(), "::", #type_name),
                    #type_name,
                )?;
                Ok(::zircon_runtime::core::framework::script::__reflect::ReflectTypeRegistration::new(
                    type_path,
                    #type_name,
                    #type_info,
                    ::zircon_runtime::core::framework::script::__reflect::ReflectSerializationStrategy::None,
                )
                .with_serializable(false)
                .with_editor_visible(false)
                .with_script_visibility(
                    ::zircon_runtime::core::framework::script::__reflect::ReflectScriptVisibility::Public,
                )
                #documentation)
            }

            fn script_host_type_projection() -> ::zircon_runtime::core::framework::script::ScriptHostTypeProjection {
                ::zircon_runtime::core::framework::script::ScriptHostTypeProjection::new(#value_kind)
                .with_prototype_kind(#prototype)
                .allow_value_construction(#allow_value_construction)
                #(#fields)*
            }
        }
    })
}

// 每个字段只解析一次属性，再同时生成反射字段与宿主投影；skip 会同时排除两侧。
/// 一次扫描结构字段，同时生成反射字段注册与脚本字段投影的 token 列表。
///
/// ## 参数
///
/// `fields` 借用结构字段 AST；`type_identity` 是生成稳定字段 ID 的类型身份键。字段显示名默认 Rust 字段名，元组字段使用原声明索引；字段身份键默认显示名。
///
/// ## 返回
///
/// 返回按非 skip 字段声明顺序排列的两个拥有 token 向量；属性解析或字段身份键校验失败时返回 `syn::Error`。
///
/// ## 约束与生命周期
///
/// 默认身份键随显示名变化；持久化消费者要保持 ID 时需保留显式 identity。type-ref token 在两份生成代码之间复用，每个运行期方法仍会独立求取自己的类型元数据。注册列表保存名称、类型名、alias 和文档；投影列表保存名称和值种类，由 descriptor 组合时建立一致性关系。AST 借用只用于本次宏展开。
///
/// ## 参考
///
/// 参阅 Interface 的 `ReflectFieldId::from_stable_keys`、`ReflectFieldInfo::new` 及 Runtime 的 `ScriptHostTypeDescriptor::from_reflect_registration`。
fn field_tokens(
    fields: &Fields,
    type_identity: &str,
) -> syn::Result<(Vec<TokenStream2>, Vec<TokenStream2>)> {
    let mut registrations = Vec::new();
    let mut projections = Vec::new();
    for (index, field) in fields.iter().enumerate() {
        let args = parse_field_attrs(&field.attrs)?;
        if args.skip {
            continue;
        }
        let field_name = match (&args.name, &field.ident) {
            (Some(name), _) => name.clone(),
            (None, Some(ident)) => ident.to_string(),
            (None, None) => index.to_string(),
        };
        let field_identity = args.identity.unwrap_or_else(|| field_name.clone());
        let aliases = args.aliases;
        let alias_tokens = aliases.iter().map(|alias| quote!(#alias.to_string()));
        if field_identity.is_empty() || field_identity.trim() != field_identity {
            return Err(syn::Error::new(
                field.span(),
                "zircon_script field identity must be non-empty and already trimmed",
            ));
        }
        let field_type = &field.ty;
        let registration_type_ref = script_host_type_ref_tokens(
            field_type,
            args.value_kind.clone().map(path_tokens),
            args.type_name.clone(),
            quote!(::zircon_runtime::core::framework::script::ScriptHostFromArgument),
        );
        let projection_type_ref = registration_type_ref.clone();
        let documentation = args
            .documentation
            .map(|doc| quote!(.with_documentation(#doc)));
        registrations.push(quote! {{
            let type_ref = #registration_type_ref;
            ::zircon_runtime::core::framework::script::__reflect::ReflectFieldInfo::new(
                ::zircon_runtime::core::framework::script::__reflect::ReflectFieldId::from_stable_keys(
                    #type_identity,
                    #field_identity,
                ),
                #field_name,
                type_ref.type_name,
                ::zircon_runtime::core::framework::script::__reflect::ReflectEditorHint::None,
            )
            .with_aliases(vec![#(#alias_tokens),*])
            .with_serializable(false)
            .with_editor_visible(false)
            #documentation
        }});
        projections.push(quote! {
            .with_field({
                let type_ref = #projection_type_ref;
                ::zircon_runtime::core::framework::script::ScriptHostFieldProjection::new(
                    #field_name,
                    type_ref.value_kind,
                )
            })
        });
    }
    Ok((registrations, projections))
}
