use proc_macro2::TokenTree;
use syn::parse::{Parse, ParseStream};
use syn::{braced, bracketed, Ident, LitStr, Token, Visibility};

use super::PluginDeclarationProjection;

impl Parse for PluginDeclarationProjection {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let _: Visibility = input.parse()?;
        let _: Ident = input.parse()?;
        let body;
        braced!(body in input);

        parse_field_name(&body, "id")?;
        let _: Ident = body.parse()?;
        body.parse::<Token![=]>()?;
        let id: LitStr = body.parse()?;
        body.parse::<Token![,]>()?;

        parse_field_name(&body, "display_name")?;
        let display_name: LitStr = body.parse()?;
        body.parse::<Token![,]>()?;

        parse_field_name(&body, "category")?;
        let category: Ident = body.parse()?;
        body.parse::<Token![,]>()?;

        parse_field_name(&body, "module")?;
        let _: Ident = body.parse()?;
        body.parse::<Token![=]>()?;
        let module_name: LitStr = body.parse()?;
        body.parse::<Token![,]>()?;

        parse_field_name(&body, "crate_name")?;
        let _: Ident = body.parse()?;
        body.parse::<Token![=]>()?;
        let crate_name: LitStr = body.parse()?;
        body.parse::<Token![,]>()?;

        parse_field_name(&body, "module_description")?;
        let _: LitStr = body.parse()?;
        body.parse::<Token![,]>()?;

        parse_field_name(&body, "targets")?;
        let targets = parse_ident_list(&body)?;
        body.parse::<Token![,]>()?;

        parse_field_name(&body, "platforms")?;
        let platforms = parse_ident_list(&body)?;
        body.parse::<Token![,]>()?;

        parse_field_name(&body, "capabilities")?;
        let capability_projections = parse_capability_list(&body)?;
        let capabilities = capability_projections
            .iter()
            .map(|projection| projection.value.clone())
            .collect();
        let runtime_capabilities = capability_projections
            .iter()
            .filter(|projection| projection.role.projects_runtime())
            .map(|projection| projection.value.clone())
            .collect();
        let editor_capabilities = capability_projections
            .iter()
            .filter(|projection| projection.role.projects_editor())
            .map(|projection| projection.value.clone())
            .collect();
        body.parse::<Token![,]>()?;

        parse_field_name(&body, "maturity")?;
        let maturity: Ident = body.parse()?;
        body.parse::<Token![,]>()?;

        parse_field_name(&body, "packaging")?;
        let packaging = parse_ident_list(&body)?;
        body.parse::<Token![,]>()?;

        let mut runtime_entry = None;
        let mut editor_entry = None;
        if !body.is_empty() {
            parse_field_name(&body, "native_projection")?;
            let native;
            braced!(native in body);
            parse_field_name(&native, "plugin_id")?;
            let _: Ident = native.parse()?;
            native.parse::<Token![,]>()?;
            parse_field_name(&native, "requested_capabilities")?;
            let _: Ident = native.parse()?;
            native.parse::<Token![,]>()?;

            while !native.is_empty() {
                let projection: Ident = native.parse()?;
                native.parse::<Token![:]>()?;
                let projection_body;
                braced!(projection_body in native);
                parse_field_name(&projection_body, "entry")?;
                let _: Ident = projection_body.parse()?;
                projection_body.parse::<Token![=]>()?;
                let entry: LitStr = projection_body.parse()?;
                let entry = entry.value();
                match projection.to_string().as_str() {
                    "runtime" => runtime_entry = Some(entry),
                    "editor" => editor_entry = Some(entry),
                    other => {
                        return Err(syn::Error::new(
                            projection.span(),
                            format!("unsupported native projection `{other}`"),
                        ));
                    }
                }
                while !projection_body.is_empty() {
                    let _: TokenTree = projection_body.parse()?;
                }
                if native.peek(Token![,]) {
                    native.parse::<Token![,]>()?;
                }
            }
            if body.peek(Token![,]) {
                body.parse::<Token![,]>()?;
            }
        }

        if !body.is_empty() {
            return Err(body.error("unexpected field after plugin declaration"));
        }

        Ok(Self {
            id: id.value(),
            display_name: display_name.value(),
            category: category.to_string(),
            module_name: module_name.value(),
            crate_name: crate_name.value(),
            targets,
            platforms,
            capabilities,
            runtime_capabilities,
            editor_capabilities,
            maturity: maturity.to_string(),
            packaging,
            runtime_entry,
            editor_entry,
        })
    }
}

fn parse_field_name(input: ParseStream<'_>, expected: &str) -> syn::Result<()> {
    let field: Ident = input.parse()?;
    if field != expected {
        return Err(syn::Error::new(
            field.span(),
            format!("expected `{expected}` field, found `{field}`"),
        ));
    }
    input.parse::<Token![:]>()?;
    Ok(())
}

fn parse_ident_list(input: ParseStream<'_>) -> syn::Result<Vec<String>> {
    let content;
    bracketed!(content in input);
    let mut values = Vec::new();
    while !content.is_empty() {
        values.push(content.parse::<Ident>()?.to_string());
        if content.peek(Token![,]) {
            content.parse::<Token![,]>()?;
        }
    }
    Ok(values)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum CapabilityRole {
    RuntimeRegistration,
    EditorRegistration,
    RuntimeEditorRegistration,
    RequestedOnly,
}

impl CapabilityRole {
    fn parse(ident: Ident) -> syn::Result<Self> {
        match ident.to_string().as_str() {
            "runtime_registration" => Ok(Self::RuntimeRegistration),
            "editor_registration" => Ok(Self::EditorRegistration),
            "runtime_editor_registration" => Ok(Self::RuntimeEditorRegistration),
            "requested_only" => Ok(Self::RequestedOnly),
            other => Err(syn::Error::new(
                ident.span(),
                format!("unsupported capability projection role `{other}`"),
            )),
        }
    }

    fn projects_runtime(self) -> bool {
        matches!(
            self,
            Self::RuntimeRegistration | Self::RuntimeEditorRegistration
        )
    }

    fn projects_editor(self) -> bool {
        matches!(
            self,
            Self::EditorRegistration | Self::RuntimeEditorRegistration
        )
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct CapabilityProjection {
    value: String,
    role: CapabilityRole,
}

fn parse_capability_list(input: ParseStream<'_>) -> syn::Result<Vec<CapabilityProjection>> {
    let content;
    bracketed!(content in input);
    let mut values = Vec::new();
    while !content.is_empty() {
        let _: Ident = content.parse()?;
        content.parse::<Token![=]>()?;
        let value = content.parse::<LitStr>()?.value();
        content.parse::<Token![=>]>()?;
        let role = CapabilityRole::parse(content.parse()?)?;
        values.push(CapabilityProjection { value, role });
        if content.peek(Token![,]) {
            content.parse::<Token![,]>()?;
        }
    }
    Ok(values)
}
