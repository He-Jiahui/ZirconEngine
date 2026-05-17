use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote};
use syn::parse::{Parse, ParseStream};
use syn::spanned::Spanned;
use syn::{
    parse_macro_input, Attribute, Data, DeriveInput, Expr, Fields, FnArg, Ident, ItemFn, ItemMod,
    LitBool, LitStr, Pat, Path, ReturnType, Token, Type,
};

#[proc_macro_derive(ZirconScriptType, attributes(zircon_script))]
pub fn derive_zircon_script_type(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    derive_zircon_script_type_impl(input)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

#[proc_macro_attribute]
pub fn zircon_host_function(attr: TokenStream, item: TokenStream) -> TokenStream {
    let args = parse_macro_input!(attr as HostFunctionArgs);
    let item = parse_macro_input!(item as ItemFn);
    host_function_impl(args, item)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

#[proc_macro_attribute]
pub fn zircon_host_module(attr: TokenStream, item: TokenStream) -> TokenStream {
    let args = parse_macro_input!(attr as HostModuleArgs);
    let item = parse_macro_input!(item as ItemMod);
    host_module_impl(args, item)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

#[derive(Default)]
struct ScriptTypeArgs {
    name: Option<String>,
    value_kind: Option<Path>,
    prototype: Option<Path>,
    allow_value_construction: Option<bool>,
    documentation: Option<String>,
}

#[derive(Default)]
struct FieldArgs {
    name: Option<String>,
    type_name: Option<String>,
    value_kind: Option<Path>,
    documentation: Option<String>,
    skip: bool,
}

#[derive(Default)]
struct HostFunctionArgs {
    name: Option<String>,
    return_type_name: Option<String>,
    return_value_kind: Option<Path>,
    capability: Vec<String>,
    documentation: Option<String>,
}

#[derive(Default)]
struct HostModuleArgs {
    name: Option<String>,
    version: Option<String>,
    capability: Vec<String>,
    documentation: Option<String>,
}

impl Parse for HostFunctionArgs {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        parse_key_values::<HostFunctionArgs>(input, |args, key, value| {
            match key.to_string().as_str() {
                "name" => args.name = Some(parse_lit_string(value, &key)?),
                "return_type_name" => args.return_type_name = Some(parse_lit_string(value, &key)?),
                "return_value_kind" => args.return_value_kind = Some(parse_expr_path(value, &key)?),
                "capability" => args.capability.push(parse_lit_string(value, &key)?),
                "documentation" | "doc" => {
                    args.documentation = Some(parse_lit_string(value, &key)?)
                }
                other => {
                    return Err(syn::Error::new(
                        key.span(),
                        format!("unknown zircon_host_function key `{other}`"),
                    ))
                }
            }
            Ok(())
        })
    }
}

impl Parse for HostModuleArgs {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        parse_key_values::<HostModuleArgs>(input, |args, key, value| {
            match key.to_string().as_str() {
                "name" => args.name = Some(parse_lit_string(value, &key)?),
                "version" => args.version = Some(parse_lit_string(value, &key)?),
                "capability" => args.capability.push(parse_lit_string(value, &key)?),
                "documentation" | "doc" => {
                    args.documentation = Some(parse_lit_string(value, &key)?)
                }
                other => {
                    return Err(syn::Error::new(
                        key.span(),
                        format!("unknown zircon_host_module key `{other}`"),
                    ))
                }
            }
            Ok(())
        })
    }
}

fn parse_key_values<T: Default>(
    input: ParseStream<'_>,
    mut set: impl FnMut(&mut T, Ident, Expr) -> syn::Result<()>,
) -> syn::Result<T> {
    let mut args = T::default();
    while !input.is_empty() {
        let key: Ident = input.parse()?;
        input.parse::<Token![=]>()?;
        let value: Expr = input.parse()?;
        set(&mut args, key, value)?;
        if input.peek(Token![,]) {
            input.parse::<Token![,]>()?;
        }
    }
    Ok(args)
}

fn derive_zircon_script_type_impl(input: DeriveInput) -> syn::Result<TokenStream2> {
    let ident = input.ident;
    if !input.generics.params.is_empty() {
        return Err(syn::Error::new(
            input.generics.span(),
            "ZirconScriptType does not support generic parameters",
        ));
    }
    let args = parse_script_type_attrs(&input.attrs)?;
    let type_name = args.name.unwrap_or_else(|| ident.to_string());
    let value_kind = args.value_kind.map(path_tokens).unwrap_or_else(|| {
        quote!(::zircon_runtime::core::framework::script::ScriptHostValueKind::Null)
    });
    let allow_value_construction = args.allow_value_construction.unwrap_or(false);
    let documentation = args
        .documentation
        .map(|doc| quote!(.with_documentation(#doc)));
    let (fields, default_prototype) = match input.data {
        Data::Struct(data) => (
            field_descriptor_tokens(&data.fields)?,
            quote!(::zircon_runtime::core::framework::script::ScriptHostPrototypeKind::Struct),
        ),
        Data::Enum(_) => (
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
            fn script_host_type_descriptor() -> ::zircon_runtime::core::framework::script::ScriptHostTypeDescriptor {
                ::zircon_runtime::core::framework::script::ScriptHostTypeDescriptor::new(
                    #type_name,
                    #value_kind,
                )
                .with_type_ref(::zircon_runtime::core::framework::script::ScriptHostTypeRef::new(#value_kind, #type_name))
                .with_prototype_kind(#prototype)
                .allow_value_construction(#allow_value_construction)
                #(#fields)*
                #documentation
            }
        }
    })
}

fn field_descriptor_tokens(fields: &Fields) -> syn::Result<Vec<TokenStream2>> {
    let mut descriptors = Vec::new();
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
        let field_type = &field.ty;
        let type_ref = script_host_type_ref_tokens(
            field_type,
            args.value_kind.map(path_tokens),
            args.type_name,
            quote!(::zircon_runtime::core::framework::script::ScriptHostFromValue),
        );
        let documentation = args
            .documentation
            .map(|doc| quote!(.with_documentation(#doc)));
        descriptors.push(quote! {
            .with_field({
                let type_ref = #type_ref;
                ::zircon_runtime::core::framework::script::ScriptHostFieldDescriptor::new(#field_name, type_ref.value_kind)
                    .with_type_ref(type_ref)
                    #documentation
            })
        });
    }
    Ok(descriptors)
}

fn host_function_impl(args: HostFunctionArgs, item: ItemFn) -> syn::Result<TokenStream2> {
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

fn host_module_impl(args: HostModuleArgs, item: ItemMod) -> syn::Result<TokenStream2> {
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

fn parse_script_type_attrs(attrs: &[Attribute]) -> syn::Result<ScriptTypeArgs> {
    let mut args = ScriptTypeArgs::default();
    for attr in attrs
        .iter()
        .filter(|attr| attr.path().is_ident("zircon_script"))
    {
        attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("name") {
                args.name = Some(meta.value()?.parse::<LitStr>()?.value());
            } else if meta.path.is_ident("value_kind") {
                args.value_kind = Some(meta.value()?.parse()?);
            } else if meta.path.is_ident("prototype") {
                args.prototype = Some(meta.value()?.parse()?);
            } else if meta.path.is_ident("allow_value_construction") {
                args.allow_value_construction = Some(meta.value()?.parse::<LitBool>()?.value());
            } else if meta.path.is_ident("documentation") || meta.path.is_ident("doc") {
                args.documentation = Some(meta.value()?.parse::<LitStr>()?.value());
            } else {
                return Err(meta.error("unknown zircon_script type key"));
            }
            Ok(())
        })?;
    }
    Ok(args)
}

fn parse_field_attrs(attrs: &[Attribute]) -> syn::Result<FieldArgs> {
    let mut args = FieldArgs::default();
    for attr in attrs
        .iter()
        .filter(|attr| attr.path().is_ident("zircon_script"))
    {
        attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("name") {
                args.name = Some(meta.value()?.parse::<LitStr>()?.value());
            } else if meta.path.is_ident("type_name") {
                args.type_name = Some(meta.value()?.parse::<LitStr>()?.value());
            } else if meta.path.is_ident("value_kind") {
                args.value_kind = Some(meta.value()?.parse()?);
            } else if meta.path.is_ident("documentation") || meta.path.is_ident("doc") {
                args.documentation = Some(meta.value()?.parse::<LitStr>()?.value());
            } else if meta.path.is_ident("skip") {
                args.skip = true;
            } else {
                return Err(meta.error("unknown zircon_script field key"));
            }
            Ok(())
        })?;
    }
    Ok(args)
}

fn host_attr_function_ident(item: &syn::Item) -> Option<Ident> {
    match item {
        syn::Item::Fn(function)
            if has_attr_with_last_segment(&function.attrs, "zircon_host_function") =>
        {
            Some(function.sig.ident.clone())
        }
        _ => None,
    }
}

fn script_type_ident(item: &syn::Item) -> Option<Ident> {
    match item {
        syn::Item::Struct(item) if derives_zircon_script_type(&item.attrs) => {
            Some(item.ident.clone())
        }
        syn::Item::Enum(item) if derives_zircon_script_type(&item.attrs) => {
            Some(item.ident.clone())
        }
        _ => None,
    }
}

fn has_attr_with_last_segment(attrs: &[Attribute], name: &str) -> bool {
    attrs
        .iter()
        .any(|attr| path_last_segment_is(attr.path(), name))
}

fn path_last_segment_is(path: &Path, name: &str) -> bool {
    path.segments
        .last()
        .is_some_and(|segment| segment.ident == name)
}

fn derives_zircon_script_type(attrs: &[Attribute]) -> bool {
    attrs.iter().any(|attr| {
        if !attr.path().is_ident("derive") {
            return false;
        }
        let mut found = false;
        let _ = attr.parse_nested_meta(|meta| {
            if path_last_segment_is(&meta.path, "ZirconScriptType") {
                found = true;
            }
            Ok(())
        });
        found
    })
}

fn parse_lit_string(value: Expr, key: &Ident) -> syn::Result<String> {
    match value {
        Expr::Lit(lit) => match lit.lit {
            syn::Lit::Str(value) => Ok(value.value()),
            _ => Err(syn::Error::new(
                lit.lit.span(),
                format!("{key} expects a string literal"),
            )),
        },
        _ => Err(syn::Error::new(
            value.span(),
            format!("{key} expects a string literal"),
        )),
    }
}

fn parse_expr_path(value: Expr, key: &Ident) -> syn::Result<Path> {
    match value {
        Expr::Path(path) => Ok(path.path),
        _ => Err(syn::Error::new(
            value.span(),
            format!("{key} expects a path"),
        )),
    }
}

fn path_tokens(path: Path) -> TokenStream2 {
    quote!(#path)
}

fn script_host_type_ref_tokens(
    ty: &Type,
    value_kind: Option<TokenStream2>,
    type_name: Option<String>,
    trait_path: TokenStream2,
) -> TokenStream2 {
    match (value_kind, type_name) {
        (Some(value_kind), Some(type_name)) => quote! {
            ::zircon_runtime::core::framework::script::ScriptHostTypeRef::new(#value_kind, #type_name)
        },
        (Some(value_kind), None) => quote! {{
            let mut type_ref = <#ty as #trait_path>::script_host_type_ref();
            type_ref.value_kind = #value_kind;
            type_ref
        }},
        (None, Some(type_name)) => quote! {{
            let mut type_ref = <#ty as #trait_path>::script_host_type_ref();
            type_ref.type_name = #type_name.to_string();
            type_ref
        }},
        (None, None) => quote! {
            <#ty as #trait_path>::script_host_type_ref()
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn host_function_rejects_async_functions() {
        let function: ItemFn = syn::parse_quote! {
            async fn load_value() -> f64 {
                1.0
            }
        };

        let error = host_function_impl(HostFunctionArgs::default(), function)
            .expect_err("async host functions should be rejected");

        assert!(error.to_string().contains("async functions"));
    }

    #[test]
    fn host_function_rejects_generic_functions() {
        let function: ItemFn = syn::parse_quote! {
            fn identity<T>(value: T) -> T {
                value
            }
        };

        let error = host_function_impl(HostFunctionArgs::default(), function)
            .expect_err("generic host functions should be rejected");

        assert!(error.to_string().contains("generic parameters"));
    }

    #[test]
    fn script_type_rejects_generic_types() {
        let input: DeriveInput = syn::parse_quote! {
            struct Wrapper<T> {
                value: T,
            }
        };

        let error = derive_zircon_script_type_impl(input)
            .expect_err("generic script types should be rejected");

        assert!(error.to_string().contains("generic parameters"));
    }
}
