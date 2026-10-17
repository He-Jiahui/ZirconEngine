//! 将 inline Rust 模块中的直接函数属性和类型 derive 汇总为模块描述符及注册函数；实际注册由调用方触发。

use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote};
use syn::ItemMod;

use crate::args::HostModuleArgs;
use crate::attrs::{host_attr_function_ident, script_type_ident};

/// 汇总内联模块的直接类型和函数子项，生成描述与注册 helper。
///
/// ## 参数
///
/// `args` 提供必填宿主模块名及可选版本、能力声明和文档；`item` 借其内容读取直接子项，并保留原属性、可见性和模块内容。版本缺省为 `0.1.0`。
///
/// ## 返回
///
/// 成功返回带两个公共 helper 的模块 token；缺少名称或外置模块声明返回 `syn::Error`。描述 helper 返回 `Result<ScriptHostModuleDescriptor, VmError>`；注册 helper 接收 `&HostExportRegistry` 并返回 `Result<HostHandle, VmError>`。
///
/// ## 约束与生命周期
///
/// 选择器按宏路径末段识别直接宿主函数与派生结构/枚举，收集结果用于对应的函数描述和导出工厂。描述 helper 被调用时求取各类型的 descriptor；反射错误映射为 Operation 并立即返回。注册 helper 先求描述，成功后创建回调数组并调用注册表接纳；描述成功仍需通过名称、capability 声明和回调匹配检查。registry 引用仅在本次同步注册使用，接纳后注册表拥有描述与 Arc 回调，调用方取得注册句柄。
///
/// ## 参考
///
/// 参阅 `attrs::host_attr_function_ident`、`attrs::script_type_ident`、Runtime 的 `ZirconScriptType::script_host_type_descriptor` 和 `HostExportRegistry::register_module`。
pub(crate) fn host_module_impl(args: HostModuleArgs, item: ItemMod) -> syn::Result<TokenStream2> {
    let attrs = &item.attrs;
    let vis = &item.vis;
    let mod_ident = &item.ident;
    let register_ident = format_ident!("register_{}_host_module", mod_ident);
    let descriptor_ident = format_ident!("{}_host_module_descriptor", mod_ident);
    let module_name = args.name.ok_or_else(|| {
        syn::Error::new(
            mod_ident.span(),
            "zircon_host_module requires name = \"...\"",
        )
    })?;
    let version = args.version.unwrap_or_else(|| "0.1.0".to_string());
    let documentation = args
        .documentation
        .map(|doc| quote!(.with_documentation(#doc)));
    let capabilities = args
        .capability
        .iter()
        .map(|capability| quote!(.with_capability(#capability)));

    let (_brace, items) = item.content.as_ref().ok_or_else(|| {
        syn::Error::new(
            item.ident.span(),
            "zircon_host_module requires an inline module body",
        )
    })?;
    let mut function_names = Vec::new();
    let mut type_names = Vec::new();
    // 只收集模块体内直接子项；同一函数名单随后生成描述符和导出回调。
    for item in items {
        if let Some(function) = host_attr_function_ident(item) {
            function_names.push(function);
        }
        if let Some(script_type) = script_type_ident(item) {
            type_names.push(script_type);
        }
    }
    let function_descriptors = function_names.iter().map(|name| {
        let descriptor = format_ident!("__zircon_host_function_descriptor_{}", name);
        quote!(.with_function(#descriptor()))
    });
    let function_exports = function_names.iter().map(|name| {
        let export = format_ident!("__zircon_host_export_function_{}", name);
        quote!(#export())
    });
    let type_descriptors = type_names.iter().map(|name| {
        quote!(.with_type(
            <#name as ::zircon_runtime::core::framework::script::ZirconScriptType>::script_host_type_descriptor()
                .map_err(|error| ::zircon_runtime::script::VmError::Operation(format!(
                    "script type reflection registration failed for {}: {error}",
                    stringify!(#name),
                )))?
        ))
    });

    Ok(quote! {
        #(#attrs)*
        #vis mod #mod_ident {
            #(#items)*

            pub fn #descriptor_ident() -> Result<
                ::zircon_runtime::core::framework::script::ScriptHostModuleDescriptor,
                ::zircon_runtime::script::VmError,
            > {
                Ok(::zircon_runtime::core::framework::script::ScriptHostModuleDescriptor::new(#module_name, #version)
                    #(#capabilities)*
                    #(#type_descriptors)*
                    #(#function_descriptors)*
                    #documentation)
            }

            pub fn #register_ident(
                exports: &::zircon_runtime::script::HostExportRegistry,
            ) -> Result<::zircon_runtime::script::HostHandle, ::zircon_runtime::script::VmError> {
                exports.register_module(
                    #descriptor_ident()?,
                    [#(#function_exports),*],
                )
            }
        }
    })
}
