use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote};
use syn::ItemMod;

use crate::args::HostModuleArgs;
use crate::attrs::{host_attr_function_ident, script_type_ident};

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
    let function_names = items
        .iter()
        .filter_map(host_attr_function_ident)
        .collect::<Vec<_>>();
    let type_names = items
        .iter()
        .filter_map(script_type_ident)
        .collect::<Vec<_>>();
    let function_descriptors = function_names.iter().map(|name| {
        let descriptor = format_ident!("__zircon_host_function_descriptor_{}", name);
        quote!(.with_function(#descriptor()))
    });
    let function_exports = function_names.iter().map(|name| {
        let export = format_ident!("__zircon_host_export_function_{}", name);
        quote!(#export())
    });
    let type_descriptors = type_names.iter().map(|name| {
        quote!(.with_type(<#name as ::zircon_runtime::core::framework::script::ZirconScriptType>::script_host_type_descriptor()))
    });

    Ok(quote! {
        #(#attrs)*
        #vis mod #mod_ident {
            #(#items)*

            pub fn #descriptor_ident() -> ::zircon_runtime::core::framework::script::ScriptHostModuleDescriptor {
                ::zircon_runtime::core::framework::script::ScriptHostModuleDescriptor::new(#module_name, #version)
                    #(#capabilities)*
                    #(#type_descriptors)*
                    #(#function_descriptors)*
                    #documentation
            }

            pub fn #register_ident(
                exports: &::zircon_runtime::script::HostExportRegistry,
            ) -> Result<::zircon_runtime::script::HostHandle, ::zircon_runtime::script::VmError> {
                exports.register_module(
                    #descriptor_ident(),
                    [#(#function_exports),*],
                )
            }
        }
    })
}
