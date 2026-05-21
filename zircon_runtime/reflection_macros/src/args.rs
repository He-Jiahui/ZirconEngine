use syn::parse::{Parse, ParseStream};
use syn::spanned::Spanned;
use syn::{Expr, Ident, LitBool, LitStr, Path, Token};

#[derive(Default)]
pub(crate) struct ScriptTypeArgs {
    pub(crate) name: Option<String>,
    pub(crate) value_kind: Option<Path>,
    pub(crate) prototype: Option<Path>,
    pub(crate) allow_value_construction: Option<bool>,
    pub(crate) documentation: Option<String>,
}

#[derive(Default)]
pub(crate) struct FieldArgs {
    pub(crate) name: Option<String>,
    pub(crate) type_name: Option<String>,
    pub(crate) value_kind: Option<Path>,
    pub(crate) documentation: Option<String>,
    pub(crate) skip: bool,
}

#[derive(Default)]
pub(crate) struct HostFunctionArgs {
    pub(crate) name: Option<String>,
    pub(crate) return_type_name: Option<String>,
    pub(crate) return_value_kind: Option<Path>,
    pub(crate) capability: Vec<String>,
    pub(crate) documentation: Option<String>,
}

#[derive(Default)]
pub(crate) struct HostModuleArgs {
    pub(crate) name: Option<String>,
    pub(crate) version: Option<String>,
    pub(crate) capability: Vec<String>,
    pub(crate) documentation: Option<String>,
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

pub(crate) fn parse_lit_bool(value: syn::meta::ParseNestedMeta<'_>) -> syn::Result<bool> {
    Ok(value.value()?.parse::<LitBool>()?.value())
}

pub(crate) fn parse_lit_str(value: syn::meta::ParseNestedMeta<'_>) -> syn::Result<String> {
    Ok(value.value()?.parse::<LitStr>()?.value())
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
