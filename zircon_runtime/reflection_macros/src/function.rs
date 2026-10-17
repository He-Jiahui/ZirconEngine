//! 将无 receiver、非 async/泛型且具名参数的自由函数包装成宿主描述符及延迟调用的导出闭包。

use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote};
use syn::spanned::Spanned;
use syn::{FnArg, Ident, ItemFn, Pat, ReturnType, Type};

use crate::args::HostFunctionArgs;
use crate::tokens::{path_tokens, script_host_type_ref_tokens};

/// 生成宿主函数的描述工厂和同步导出工厂，并保留原函数定义。
///
/// ## 参数
///
/// `args` 是已解析的宿主名称、返回元数据、能力要求和文档；`item` 是拥有参数与返回签名的 Rust 函数 AST。helper 名使用 Rust 函数标识符，宿主名称覆盖同时用于描述和导出。
///
/// ## 返回
///
/// 成功返回原函数与两个私有 helper 的 token。async、非空泛型参数、receiver 和非标识符参数模式返回 `syn::Error`；生成代码的 trait 与名称解析仍由后续 Rust 编译检查。
///
/// ## 约束与生命周期
///
/// 描述工厂每次构造拥有的 DTO，声明固定的最小/最大参数数量和能力要求。导出工厂创建由注册表保存的 Arc 回调；每次执行按声明顺序短期访问参数视图，并以 `ScriptHostFromArgument` 产生不依赖该次视图借用的 Rust 值。转换错误通过 `?` 提前返回，全部成功才调用原函数。显式返回经 `ScriptHostIntoValue` 转为拥有值；省略返回类型后返回 Null。返回 type-ref 覆盖只修改描述，转换 trait 仍负责真实返回值。调用层在进入回调前验证数量与能力；包装直接使用返回转换 trait，不自动拆解 `Result`；本层同步回调不捕获原函数 panic。
///
/// ## 参考
///
/// 参阅 Runtime 的 `ScriptHostArguments::with_argument`、`ScriptHostFromArgument`、`ScriptHostIntoValue`、`HostExportFunction::new` 和 `ScriptCallSite::call`。
pub(crate) fn host_function_impl(
    args: HostFunctionArgs,
    item: ItemFn,
) -> syn::Result<TokenStream2> {
    let fn_ident = &item.sig.ident;
    if item.sig.asyncness.is_some() {
        return Err(syn::Error::new(
            item.sig.asyncness.span(),
            "zircon_host_function does not support async functions",
        ));
    }
    if !item.sig.generics.params.is_empty() {
        return Err(syn::Error::new(
            item.sig.generics.span(),
            "zircon_host_function does not support generic parameters",
        ));
    }
    let descriptor_ident = format_ident!("__zircon_host_function_descriptor_{}", fn_ident);
    let export_ident = format_ident!("__zircon_host_export_function_{}", fn_ident);
    let function_name = args.name.unwrap_or_else(|| fn_ident.to_string());
    let params = host_function_params(&item)?;
    let arity = params.len();
    let return_ty = match &item.sig.output {
        ReturnType::Default => None,
        ReturnType::Type(_, ty) => Some(ty.as_ref()),
    };
    let return_value_kind = args.return_value_kind.map(path_tokens).unwrap_or_else(|| {
        if let Some(return_ty) = return_ty {
            quote!(<#return_ty as ::zircon_runtime::core::framework::script::ScriptHostIntoValue>::script_host_type_ref().value_kind)
        } else {
            quote!(::zircon_runtime::core::framework::script::ScriptHostValueKind::Null)
        }
    });
    let return_type_ref = match (return_ty, args.return_type_name) {
        (Some(ty), Some(type_name)) => script_host_type_ref_tokens(
            ty,
            Some(return_value_kind.clone()),
            Some(type_name),
            quote!(::zircon_runtime::core::framework::script::ScriptHostIntoValue),
        ),
        (Some(ty), None) => script_host_type_ref_tokens(
            ty,
            Some(return_value_kind.clone()),
            None,
            quote!(::zircon_runtime::core::framework::script::ScriptHostIntoValue),
        ),
        (None, Some(type_name)) => quote! {
            ::zircon_runtime::core::framework::script::ScriptHostTypeRef::new(
                #return_value_kind,
                #type_name,
            )
        },
        (None, None) => quote! {
            ::zircon_runtime::core::framework::script::ScriptHostTypeRef::new(
                #return_value_kind,
                "void",
            )
        },
    };
    let documentation = args
        .documentation
        .map(|doc| quote!(.with_documentation(#doc)));
    let capabilities = args
        .capability
        .iter()
        .map(|capability| quote!(.with_required_capability(#capability)));

    let parameter_descriptors = params.iter().map(|param| {
        let name = &param.name;
        let ty = &param.ty;
        let type_ref = quote! {
            <#ty as ::zircon_runtime::core::framework::script::ScriptHostFromArgument>::script_host_type_ref()
        };
        quote! {
            .with_parameter({
                let type_ref = #type_ref;
                ::zircon_runtime::core::framework::script::ScriptHostParameterDescriptor::new(
                    #name,
                    type_ref.value_kind,
                )
                .with_type_ref(type_ref)
            })
        }
    });
    // 描述符阶段声明参数类型；值在回调调用时按索引转换，再交给原函数执行。
    // TODO: [CR-ZCA-R003-0002] 待确认输入参数 context 位于其它参数之前时，生成的 let 绑定是否遮蔽后续
    // context.arguments 接收者。输入 Ident 保留原 span，接收者使用 call-site span；尚缺真实宏编译证据，
    // 下一步用 context 置首、居中和末尾的 fixture 核对名称解析、编译器版本与诊断，再决定是否为缺陷。
    let conversions = params.iter().enumerate().map(|(index, param)| {
        let name = &param.ident;
        let ty = &param.ty;
        quote! {
            let #name = context.arguments.with_argument(#index, |value| {
                <#ty as ::zircon_runtime::core::framework::script::ScriptHostFromArgument>::from_script_host_argument(
                    value,
                    #index,
                )
            })?;
        }
    });
    let call_args = params.iter().map(|param| &param.ident);
    let call = if let Some(return_ty) = return_ty {
        quote! {
            let value: #return_ty = #fn_ident(#(#call_args),*);
            Ok(::zircon_runtime::core::framework::script::ScriptHostIntoValue::into_script_host_value(value))
        }
    } else {
        quote! {
            #fn_ident(#(#call_args),*);
            Ok(::zircon_runtime::core::framework::script::ScriptHostValue::Null)
        }
    };

    Ok(quote! {
        #item

        fn #descriptor_ident() -> ::zircon_runtime::core::framework::script::ScriptHostFunctionDescriptor {
            let return_type_ref = #return_type_ref;
            ::zircon_runtime::core::framework::script::ScriptHostFunctionDescriptor::new(
                #function_name,
                #arity,
                #arity,
                return_type_ref.value_kind,
            )
            .with_return_type(return_type_ref)
            #(#parameter_descriptors)*
            #(#capabilities)*
            #documentation
        }

        fn #export_ident() -> ::zircon_runtime::script::HostExportFunction {
            ::zircon_runtime::script::HostExportFunction::new(#function_name, |context| {
                #(#conversions)*
                #call
            })
        }
    })
}

struct HostParam {
    ident: Ident,
    name: String,
    ty: Type,
}

fn host_function_params(item: &ItemFn) -> syn::Result<Vec<HostParam>> {
    let mut params = Vec::new();
    for input in &item.sig.inputs {
        match input {
            FnArg::Receiver(receiver) => {
                return Err(syn::Error::new(
                    receiver.self_token.span,
                    "zircon_host_function does not support methods",
                ))
            }
            FnArg::Typed(typed) => {
                let Pat::Ident(pattern) = typed.pat.as_ref() else {
                    return Err(syn::Error::new(
                        typed.pat.span(),
                        "zircon_host_function parameters must be simple identifiers",
                    ));
                };
                params.push(HostParam {
                    ident: pattern.ident.clone(),
                    name: pattern.ident.to_string(),
                    ty: (*typed.ty).clone(),
                });
            }
        }
    }
    Ok(params)
}
