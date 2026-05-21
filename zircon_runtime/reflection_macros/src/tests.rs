use syn::{DeriveInput, ItemFn};

#[test]
fn host_function_rejects_async_functions() {
    let function: ItemFn = syn::parse_quote! {
        async fn load_value() -> f64 {
            1.0
        }
    };

    let error =
        crate::function::host_function_impl(crate::args::HostFunctionArgs::default(), function)
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

    let error =
        crate::function::host_function_impl(crate::args::HostFunctionArgs::default(), function)
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

    let error = crate::derive_type::derive_zircon_script_type_impl(input)
        .expect_err("generic script types should be rejected");

    assert!(error.to_string().contains("generic parameters"));
}

#[test]
fn host_function_rejects_methods() {
    let function: syn::ItemFn = syn::parse_quote! {
        fn length(&self) -> f64 {
            0.0
        }
    };

    let error =
        crate::function::host_function_impl(crate::args::HostFunctionArgs::default(), function)
            .expect_err("methods should be rejected");

    assert!(error.to_string().contains("methods"));
}

#[test]
fn host_function_rejects_non_identifier_parameters() {
    let function: syn::ItemFn = syn::parse_quote! {
        fn unpack((x, y): (f64, f64)) -> f64 {
            x + y
        }
    };

    let error =
        crate::function::host_function_impl(crate::args::HostFunctionArgs::default(), function)
            .expect_err("destructuring parameters should be rejected");

    assert!(error.to_string().contains("simple identifiers"));
}

#[test]
fn host_module_rejects_non_inline_modules() {
    let module: syn::ItemMod = syn::parse_quote! {
        mod external_math;
    };
    let args = crate::args::HostModuleArgs {
        name: Some("test.external".to_string()),
        ..Default::default()
    };

    let error = crate::module::host_module_impl(args, module)
        .expect_err("non-inline host modules should be rejected");

    assert!(error.to_string().contains("inline module body"));
}

#[test]
fn script_type_rejects_unions() {
    let input: syn::DeriveInput = syn::parse_quote! {
        union BadValue {
            int_value: i64,
            float_value: f64,
        }
    };

    let error = crate::derive_type::derive_zircon_script_type_impl(input)
        .expect_err("unions should be rejected");

    assert!(error.to_string().contains("unions"));
}

#[test]
fn script_type_expansion_preserves_metadata_and_skips_fields() {
    let input: syn::DeriveInput = syn::parse_quote! {
        #[zircon_script(
            name = "MetaVec3",
            value_kind = ScriptHostValueKind::Float,
            prototype = ScriptHostPrototypeKind::Struct,
            allow_value_construction = true,
            documentation = "vector docs"
        )]
        struct Vec3 {
            #[zircon_script(type_name = "float", documentation = "x docs")]
            x: f64,
            #[zircon_script(skip)]
            cached_length: f64,
        }
    };

    let tokens = crate::derive_type::derive_zircon_script_type_impl(input)
        .expect("script type expansion")
        .to_string();

    assert!(tokens.contains("MetaVec3"));
    assert!(tokens.contains("ScriptHostValueKind :: Float"));
    assert!(tokens.contains("ScriptHostPrototypeKind :: Struct"));
    assert!(tokens.contains("allow_value_construction (true)"));
    assert!(tokens.contains("with_documentation (\"vector docs\")"));
    assert!(tokens.contains("with_documentation (\"x docs\")"));
    assert!(!tokens.contains("cached_length"));
}

#[test]
fn host_function_expansion_preserves_descriptor_metadata() {
    let function: syn::ItemFn = syn::parse_quote! {
        fn length(value: f64) -> f64 {
            value
        }
    };
    let args = crate::args::HostFunctionArgs {
        name: Some("vec_length".to_string()),
        return_type_name: Some("float".to_string()),
        return_value_kind: None,
        capability: vec!["math.read".to_string()],
        documentation: Some("length docs".to_string()),
    };

    let tokens = crate::function::host_function_impl(args, function)
        .expect("host function expansion")
        .to_string();

    assert!(tokens.contains("vec_length"));
    assert!(tokens.contains("math.read"));
    assert!(tokens.contains("length docs"));
    assert!(tokens.contains("with_return_type"));
    assert!(tokens.contains("ScriptHostParameterDescriptor"));
}

#[test]
fn host_module_expansion_collects_types_and_functions() {
    let module: syn::ItemMod = syn::parse_quote! {
        mod math {
            #[derive(ZirconScriptType)]
            struct Vec3 {
                x: f64,
            }

            #[zircon_host_function(name = "length")]
            fn length(x: f64) -> f64 {
                x
            }
        }
    };
    let args = crate::args::HostModuleArgs {
        name: Some("test.math".to_string()),
        version: Some("0.2.0".to_string()),
        capability: vec!["math.read".to_string()],
        documentation: Some("math docs".to_string()),
    };

    let tokens = crate::module::host_module_impl(args, module)
        .expect("host module expansion")
        .to_string();

    assert!(tokens.contains("test.math"));
    assert!(tokens.contains("0.2.0"));
    assert!(tokens.contains("math.read"));
    assert!(tokens
        .contains("Vec3 as :: zircon_runtime :: core :: framework :: script :: ZirconScriptType"));
    assert!(tokens.contains("__zircon_host_function_descriptor_length"));
    assert!(tokens.contains("__zircon_host_export_function_length"));
}
