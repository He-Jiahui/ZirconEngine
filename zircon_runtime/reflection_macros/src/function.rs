use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote};
use syn::spanned::Spanned;
use syn::{FnArg, Ident, ItemFn, Pat, ReturnType, Type};

use crate::args::HostFunctionArgs;
use crate::tokens::{path_tokens, script_host_type_ref_tokens};

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
            <#ty as ::zircon_runtime::core::framework::script::ScriptHostFromValue>::script_host_type_ref()
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
    let conversions = params.iter().enumerate().map(|(index, param)| {
        let name = &param.ident;
        let ty = &param.ty;
        quote! {
            let #name = <#ty as ::zircon_runtime::core::framework::script::ScriptHostFromValue>::from_script_host_value(
                context.arguments.get(#index).ok_or_else(|| {
                    ::zircon_runtime::core::framework::script::ScriptHostError::new(format!("argument {} was not provided", #index))
                })?,
                #index,
            )?;
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
        #[allow(dead_code)]
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
