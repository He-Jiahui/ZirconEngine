use std::collections::HashMap;
use std::path::Path;

use super::super::*;
use super::support::*;

fn is_higher_layer_runtime_path(path: &[String]) -> bool {
    path.first().is_some_and(|segment| segment == "crate")
        && !path.starts_with(&["crate".to_owned(), "core".to_owned(), "resource".to_owned()])
}

fn resolved_dependency_paths(
    path: &[String],
    module_path: &[String],
    aliases: &ModuleAliasGraph,
) -> Vec<Vec<String>> {
    let root_is_local_alias = path.first().is_some_and(|root| {
        aliases
            .get(module_path)
            .is_some_and(|scope| scope.contains_key(root))
    });
    let normalized = match path.first().map(String::as_str) {
        Some("crate" | "self" | "super" | "zircon_runtime") => {
            normalize_module_path(path, module_path)
        }
        Some(_) if root_is_local_alias => normalize_module_path(path, module_path),
        _ => path.to_vec(),
    };
    resolved_module_paths(&normalized, aliases)
}

fn references_higher_layer_runtime_owner(source: &str, file_module_path: &[String]) -> bool {
    let tokens = rust_tokens(source);
    let mut aliases = ModuleAliasGraph::new();
    extend_module_alias_graph(&mut aliases, source, file_module_path);
    let mut index = 0;
    while index < tokens.len() {
        if tokens[index] == "use" {
            let start = index;
            let mut end = index + 1;
            while end < tokens.len() && tokens[end] != ";" {
                end += 1;
            }
            let (module_path, _) = module_context_at(&tokens, start, file_module_path);
            if rust_use_paths(&tokens[start..end.min(tokens.len())])
                .into_iter()
                .flat_map(|(path, _)| resolved_dependency_paths(&path, &module_path, &aliases))
                .any(|path| is_higher_layer_runtime_path(&path))
            {
                return true;
            }
            index = end.saturating_add(1);
            continue;
        }
        if !tokens[index]
            .chars()
            .all(|character| character == '_' || character.is_ascii_alphanumeric())
        {
            index += 1;
            continue;
        }
        let start = index;
        let mut path = vec![tokens[index].clone()];
        index += 1;
        while tokens.get(index).is_some_and(|token| token == "::")
            && tokens.get(index + 1).is_some()
        {
            path.push(tokens[index + 1].clone());
            index += 2;
        }
        if path.len() < 2 {
            continue;
        }
        let (module_path, _) = module_context_at(&tokens, start, file_module_path);
        if resolved_dependency_paths(&path, &module_path, &aliases)
            .iter()
            .any(|path| is_higher_layer_runtime_path(path))
        {
            return true;
        }
    }
    false
}

fn resource_module_path(resource_root: &Path, source_path: &Path) -> Vec<String> {
    let mut module_path = vec!["crate".to_owned(), "core".to_owned(), "resource".to_owned()];
    let Ok(relative) = source_path.strip_prefix(resource_root) else {
        return module_path;
    };
    let mut components = relative
        .components()
        .filter_map(|component| component.as_os_str().to_str().map(str::to_owned))
        .collect::<Vec<_>>();
    if components.last().is_some_and(|name| name == "mod.rs") {
        components.pop();
    } else if let Some(file_name) = components.last_mut() {
        *file_name = file_name
            .strip_suffix(".rs")
            .unwrap_or(file_name)
            .to_owned();
    }
    module_path.extend(components);
    module_path
}

#[test]
fn resource_management_generation_is_owned_without_framework_backflow() {
    const ASSET_CONTRACT: &str = include_str!("../../../framework/asset.rs");
    const FRAMEWORK_ROOT: &str = include_str!("../../../framework/mod.rs");
    const MANAGEMENT_PROJECTION: &str = include_str!("../../manager/management_projection.rs");
    const RESOURCE_MANAGER: &str = include_str!("../../manager/resource_manager.rs");
    const OLD_FRAMEWORK_OWNER: &str = concat!("core::framework", "::asset");
    let old_owner = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("src/core/framework/asset/management_generation.rs");

    assert!(
        !old_owner.exists(),
        "the framework-owned declaration must stay deleted"
    );
    assert!(
        framework_root_has_external_asset_module(FRAMEWORK_ROOT),
        "framework asset must stay an external file module so inline relative-path re-exports cannot bypass the owner guard"
    );
    assert!(!ASSET_CONTRACT.contains("mod management_generation"));
    assert!(!ASSET_CONTRACT.contains("pub use management_generation"));
    assert!(
        !ASSET_CONTRACT.contains("pub use"),
        "framework asset must not expose lower resource declarations through compatibility re-exports"
    );
    assert!(
        asset_contract_has_only_resource_management_consumer(ASSET_CONTRACT),
        "framework asset must not recreate ResourceManagement aliases, wrappers, or re-exports"
    );
    assert!(
        asset_contract_has_no_generated_surface(ASSET_CONTRACT),
        "framework asset must remain a direct declaration contract without include, module, or macro-generated compatibility surfaces"
    );
    assert!(
        asset_contract_has_only_expected_public_items(ASSET_CONTRACT),
        "framework asset must not add another public compatibility surface"
    );
    assert!(
        !has_public_use(ASSET_CONTRACT),
        "framework asset must not recreate a public compatibility surface"
    );
    assert!(!MANAGEMENT_PROJECTION.contains(OLD_FRAMEWORK_OWNER));
    assert!(!RESOURCE_MANAGER.contains(OLD_FRAMEWORK_OWNER));

    let repo_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("zircon_runtime must be located under the repository root");
    let owner_path = repo_root.join("zircon_runtime/src/core/resource/management_generation.rs");
    let mut stale_consumers = Vec::new();
    for source_root in [
        "zircon_runtime",
        "zircon_runtime_interface",
        "zircon_editor",
        "zircon_app",
        "zircon_plugins",
        "zircon_hub",
        "zircon_reflect_derive",
        "examples",
        "tests",
    ] {
        visit_rust_sources(&repo_root.join(source_root), &mut |path, source| {
            if path != owner_path && references_old_resource_management_owner(source) {
                stale_consumers.push(path.to_path_buf());
            }
        });
    }
    assert!(
        stale_consumers.is_empty(),
        "old framework ResourceManagement consumers must stay deleted: {stale_consumers:?}"
    );

    let framework_root = repo_root.join("zircon_runtime/src/core/framework");
    let asset_contract_path = framework_root.join("asset.rs");
    let mut framework_sources = Vec::new();
    visit_rust_sources(&framework_root, &mut |path, source| {
        framework_sources.push((
            path.to_path_buf(),
            source.to_owned(),
            framework_module_path(&framework_root, path),
        ));
    });
    let mut framework_aliases = HashMap::new();
    for (_, source, module_path) in &framework_sources {
        extend_module_alias_graph(&mut framework_aliases, source, module_path);
    }
    let compatibility_declarations = framework_sources
        .iter()
        .filter(|(path, source, module_path)| {
            path != &asset_contract_path
                && (compact_rust_source(source).contains("ResourceManagement")
                    || reexports_resource_owner(source, module_path, &framework_aliases)
                    || imports_resource_owner_glob(source, module_path, &framework_aliases)
                    || declares_extern_crate(source)
                    || has_source_injection_surface(source))
        })
        .map(|(path, _, _)| path.clone())
        .collect::<Vec<_>>();
    assert!(
        compatibility_declarations.is_empty(),
        "framework ResourceManagement aliases and wrappers must stay deleted: {compatibility_declarations:?}"
    );
    let old_owner_directory = repo_root.join("zircon_runtime/src/core/framework/asset");
    if old_owner_directory.exists() {
        let mut old_owner_declarations = Vec::new();
        visit_rust_sources(&old_owner_directory, &mut |path, source| {
            if compact_rust_source(source).contains("ResourceManagement") {
                old_owner_declarations.push(path.to_path_buf());
            }
        });
        assert!(
            old_owner_declarations.is_empty(),
            "the old framework owner directory must not regain ResourceManagement APIs: {old_owner_declarations:?}"
        );
    }
}

#[test]
fn resource_owner_has_no_higher_layer_runtime_dependency() {
    let repo_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("zircon_runtime must be located under the repository root");
    let resource_root = repo_root.join("zircon_runtime/src/core/resource");
    let mut higher_layer_consumers = Vec::new();

    visit_rust_sources(&resource_root, &mut |path, source| {
        let module_path = resource_module_path(&resource_root, path);
        if references_higher_layer_runtime_owner(source, &module_path) {
            higher_layer_consumers.push(path.to_path_buf());
        }
    });

    assert!(
        higher_layer_consumers.is_empty(),
        "the layer-0 resource owner must not depend on higher Runtime owners: {higher_layer_consumers:?}"
    );
}

#[test]
fn resource_owner_dependency_guard_rejects_higher_layer_aliases() {
    let module_path = ["crate", "core", "resource", "management_generation"].map(str::to_owned);
    let references_higher_layer =
        |source| references_higher_layer_runtime_owner(source, &module_path);

    assert!(references_higher_layer(
        "use crate::core::diagnostics::profiling::record_counter_batch;"
    ));
    assert!(references_higher_layer(
        "use crate::core as runtime_core; use runtime_core::diagnostics::profiling;"
    ));
    assert!(references_higher_layer("use crate::asset::AssetUri;"));
    assert!(references_higher_layer("use crate::core::*;"));
    assert!(references_higher_layer(
        "use super::super::diagnostics::profiling;"
    ));
    assert!(references_higher_layer(
        "use zircon_runtime::core::diagnostics::profiling;"
    ));
    assert!(references_higher_layer("use crate as runtime_root;"));
    assert!(!references_higher_layer(
        "use crate::core::resource::{ResourceId, ResourceRecord};"
    ));
    assert!(!references_higher_layer(
        "use super::ResourceId; pub(crate) fn resource_internal() {}"
    ));
    assert!(!references_higher_layer("use std::sync::Arc;"));
    assert!(!references_higher_layer(
        "// use crate::asset::*; const EXAMPLE: &str = \"crate::core::diagnostics\";"
    ));
}

#[test]
fn resource_management_owner_guard_rejects_alias_and_multiline_consumer_forms() {
    const VALID_ASSET_CONTRACT: &str = "
        use std::sync::Arc;
        use crate::core::resource::{
            ResourceEventReceiver, ResourceManagementGeneration, ResourceRecord, ResourceState,
        };
        pub trait ResourceManager {
            fn resource_management_generation(&self) -> Arc<ResourceManagementGeneration>;
        }
    ";
    assert!(asset_contract_has_only_resource_management_consumer(
        VALID_ASSET_CONTRACT
    ));
    assert!(asset_contract_has_no_generated_surface(
        VALID_ASSET_CONTRACT
    ));
    const VALID_COMPLETE_ASSET_CONTRACT: &str = "
        #[derive(Clone, Copy, Debug, PartialEq, Eq)]
        pub struct ResourceCacheIdentity {
            pub revision: u64,
            pub state: ResourceState,
        }
        pub trait ResourceManager {}
    ";
    assert!(asset_contract_has_only_expected_public_items(
        VALID_COMPLETE_ASSET_CONTRACT
    ));
    assert!(framework_root_has_external_asset_module("pub mod asset;"));
    assert!(!framework_root_has_external_asset_module(
        "pub mod asset { pub use super::super::resource::*; }"
    ));
    assert!(!framework_root_has_external_asset_module(
        "pub mod asset; pub mod asset;"
    ));
    assert!(!framework_root_has_external_asset_module(
        "#[path = \"asset/compat.rs\"] pub mod asset;"
    ));
    assert!(!framework_root_has_external_asset_module(
        "#[cfg(feature = \"compat\")] pub mod asset;"
    ));
    assert!(!asset_contract_has_only_resource_management_consumer(
        &format!(
            "{VALID_ASSET_CONTRACT}\npub type ResourceManagementGeneration = crate::core::resource::ResourceManagementGeneration;"
        )
    ));
    assert!(!asset_contract_has_only_resource_management_consumer(
        &format!("{VALID_ASSET_CONTRACT}\npub use crate::core::resource::ResourceManagementPage;")
    ));
    assert!(!asset_contract_has_only_resource_management_consumer(
        "use crate::core::resource::ResourceManagementGeneration;\npub type ForgedConsumer = ResourceManagementGeneration;"
    ));
    assert!(asset_contract_has_only_resource_management_consumer(
        &format!(
            "{VALID_ASSET_CONTRACT}\n// pub type ResourceManagementPage = ();\nconst EXAMPLE: &str = \"ResourceManagementRow\";"
        )
    ));
    assert!(has_public_use(
        "pub /* compatibility */ use crate::core::resource::*;"
    ));
    assert!(has_public_use(
        "pub(crate) use crate::core::resource as resource;"
    ));
    assert!(has_public_use(
        "pub mod legacy { pub(super) use crate::core::resource::*; }"
    ));
    assert!(!has_public_use(
        "// pub use crate::core::resource::*;\nconst EXAMPLE: &str = \"pub use\";"
    ));
    let framework_module = ["crate", "core", "framework"].map(str::to_owned);
    let nested_framework_module = ["crate", "core", "framework", "nested"].map(str::to_owned);
    let mut framework_aliases = HashMap::new();
    extend_module_alias_graph(
        &mut framework_aliases,
        "use super::resource as lower_owner;",
        &framework_module,
    );
    assert!(reexports_resource_owner(
        "pub use crate::core::resource::*;",
        &framework_module,
        &framework_aliases,
    ));
    assert!(reexports_resource_owner(
        "use crate::core::resource as lower_owner;\npub use lower_owner::{ResourceManagementGeneration};",
        &framework_module,
        &framework_aliases,
    ));
    assert!(reexports_resource_owner(
        "pub use crate::core::{resource::{self as lower_owner}};",
        &framework_module,
        &framework_aliases,
    ));
    assert!(reexports_resource_owner(
        "pub use crate::core as core_root;",
        &framework_module,
        &framework_aliases,
    ));
    assert!(reexports_resource_owner(
        "pub use crate::*;",
        &framework_module,
        &framework_aliases,
    ));
    assert!(reexports_resource_owner(
        "pub use crate::core::*;",
        &framework_module,
        &framework_aliases,
    ));
    assert!(reexports_resource_owner(
        "pub use super::super::resource::*;",
        &nested_framework_module,
        &framework_aliases,
    ));
    assert!(reexports_resource_owner(
        "pub use super::lower_owner::*;",
        &["crate", "core", "framework", "audio"].map(str::to_owned),
        &framework_aliases,
    ));
    assert!(reexports_resource_owner(
        "pub use crate::core::framework::lower_owner::*;",
        &["crate", "core", "framework", "audio"].map(str::to_owned),
        &framework_aliases,
    ));
    assert!(reexports_resource_owner(
        "pub use zircon_runtime::core::resource::*;",
        &framework_module,
        &framework_aliases,
    ));
    assert!(reexports_resource_owner(
        "pub use ::r#zircon_runtime::{core::resource::*};",
        &framework_module,
        &framework_aliases,
    ));
    let inline_source = "
        use super::resource as lower_owner;
        pub mod audio { pub use super::lower_owner::*; }
    ";
    let mut inline_aliases = HashMap::new();
    extend_module_alias_graph(&mut inline_aliases, inline_source, &framework_module);
    assert!(reexports_resource_owner(
        inline_source,
        &framework_module,
        &inline_aliases,
    ));
    assert!(!reexports_resource_owner(
        "use crate::core::resource::ResourceManagementGeneration;",
        &framework_module,
        &framework_aliases,
    ));
    assert!(!reexports_resource_owner(
        "pub use resource::{ShaderResourceDescriptor};",
        &nested_framework_module,
        &framework_aliases,
    ));
    assert!(!reexports_resource_owner(
        "pub use super::ibl_bake_recipe::CANONICAL_SIZE;",
        &nested_framework_module,
        &framework_aliases,
    ));
    assert!(!reexports_resource_owner(
        "pub use crate::core::framework::tasks::TaskPoolKind;",
        &nested_framework_module,
        &framework_aliases,
    ));
    assert!(!reexports_resource_owner(
        "pub use zircon_runtime_interface::resource::ResourceKind;",
        &nested_framework_module,
        &framework_aliases,
    ));
    let local_alias_source = "
        fn build() { use crate::core::resource as lower_owner; }
        pub mod lower_owner { pub struct FrameworkOnly; }
        pub use lower_owner::FrameworkOnly;
    ";
    let mut local_aliases = HashMap::new();
    extend_module_alias_graph(&mut local_aliases, local_alias_source, &framework_module);
    assert!(!reexports_resource_owner(
        local_alias_source,
        &framework_module,
        &local_aliases,
    ));
    assert!(reexports_resource_owner(
        "macro_rules! compat { () => { pub use crate::core::resource::*; } }",
        &framework_module,
        &framework_aliases,
    ));
    assert!(imports_resource_owner_glob(
        "use crate::core::resource::*;\npub use ShaderResourceDescriptor as CompatibilityDescriptor;",
        &framework_module,
        &framework_aliases,
    ));
    let resource_alias_glob = "
        use crate::core::resource as lower_owner;
        use lower_owner::*;
        pub use ShaderResourceDescriptor as CompatibilityDescriptor;
    ";
    let mut resource_aliases = HashMap::new();
    extend_module_alias_graph(
        &mut resource_aliases,
        resource_alias_glob,
        &framework_module,
    );
    assert!(imports_resource_owner_glob(
        resource_alias_glob,
        &framework_module,
        &resource_aliases,
    ));
    assert!(!imports_resource_owner_glob(
        "use crate::core::resource::{ShaderResourceDescriptor};",
        &framework_module,
        &framework_aliases,
    ));
    assert!(declares_extern_crate("extern crate self as runtime_root;"));
    assert!(declares_extern_crate(
        "pub extern /* compatibility */ crate self as runtime_root;"
    ));
    assert!(!declares_extern_crate(
        "// extern crate self as runtime_root;\nconst EXAMPLE: &str = \"extern crate\";"
    ));
    assert!(has_source_injection_surface(
        "#[path = \"compat.rs\"] mod compatibility;"
    ));
    assert!(has_source_injection_surface(
        "#[cfg_attr(all(), path = \"../../resource/compat.rs\")] pub mod compatibility;"
    ));
    assert!(has_source_injection_surface(
        "include!(\"compatibility.rs\");"
    ));
    assert!(!has_source_injection_surface(
        "const SOURCE: &str = include_str!(\"source.rs\");"
    ));
    assert!(!has_source_injection_surface(
        "#[cfg_attr(all(), allow(dead_code))] pub mod framework_only;"
    ));
    assert!(!asset_contract_has_no_generated_surface(
        "include!(\"asset/compat.rs\");"
    ));
    assert!(!asset_contract_has_no_generated_surface("mod compat;"));
    assert!(!asset_contract_has_no_generated_surface(
        "compatibility_surface!();"
    ));
    assert!(!asset_contract_has_no_generated_surface(
        "#[compatibility_surface] pub trait ResourceManager {}"
    ));
    assert!(!asset_contract_has_no_generated_surface(
        "#[derive(CustomCompatibilitySurface)] pub struct ResourceCacheIdentity;"
    ));
    assert!(!asset_contract_has_only_expected_public_items(
        "pub struct ResourceCacheIdentity { pub revision: u64, pub state: ResourceState }\npub trait ResourceManager {}\npub extern crate self as resource;"
    ));
    assert!(!asset_contract_has_only_expected_public_items(
        "pub struct ResourceCacheIdentity { pub revision: u64, pub state: ResourceState }\npub trait ResourceManager {}\npub type Compatibility = ();"
    ));
    assert!(references_old_resource_management_owner(
        "use zircon_runtime::core::framework::asset::ResourceManagementGeneration;"
    ));
    assert!(references_old_resource_management_owner(
        "use zircon_runtime::core::framework::asset::{\nResourceManager,\nResourceManagementPage,\n};"
    ));
    assert!(references_old_resource_management_owner(
        "use zircon_runtime::core::framework /* hard cut */ ::r#asset::{ResourceManagementRow};"
    ));
    assert!(references_old_resource_management_owner(
        "use zircon_runtime::core::framework::{asset::{ResourceManagementPage}};"
    ));
    assert!(references_old_resource_management_owner(
        "use zircon_runtime::core::{framework::asset::ResourceManagementScan};"
    ));
    assert!(references_old_resource_management_owner(
        "use zircon_runtime::core::framework::asset as legacy_asset;\ntype Generation = legacy_asset::ResourceManagementGeneration;"
    ));
    assert!(references_old_resource_management_owner(
        "use zircon_runtime::core::framework as legacy_framework;\ntype Generation = legacy_framework::asset::ResourceManagementGeneration;"
    ));
    assert!(references_old_resource_management_owner(
        "use zircon_runtime::core::framework::asset::*;"
    ));
    assert!(references_old_resource_management_owner(
        "use zircon_runtime::core::framework::{asset::{self as legacy_asset}};\ntype Row = legacy_asset::ResourceManagementRow;"
    ));
    assert!(references_old_resource_management_owner(
        "use first_alias as second_alias;\nuse zircon_runtime::core::framework::asset as first_alias;\ntype Scan = second_alias::ResourceManagementScan;"
    ));
    assert!(references_old_resource_management_owner(
        "#[cfg(a)] use zircon_runtime::core::framework::asset as contract;\n#[cfg(b)] use zircon_runtime::core::resource as contract;\ntype Row = contract::ResourceManagementRow;"
    ));
    assert!(!references_old_resource_management_owner(
        "use zircon_runtime::core::resource::{ResourceManagementGeneration, ResourceManagementPage};"
    ));
    assert!(!references_old_resource_management_owner(
        "// core::framework::asset::ResourceManagementGeneration\nconst EXAMPLE: &str = r#\"core::framework::asset::{ResourceManagementRow}\"#;"
    ));
}
