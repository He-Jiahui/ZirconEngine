use std::path::PathBuf;

use super::{NewPluginOptions, PluginKind};

pub(super) fn render_package_files(
    options: &NewPluginOptions<'_>,
    version: &str,
    sdk_api_version: &str,
    engine_compatibility: &str,
) -> Vec<(PathBuf, String)> {
    let id = options.id;
    let display_name = display_name(id);
    let constant_prefix = id.to_ascii_uppercase();
    let type_name = rust_type_name(id);
    let owner = if options.kind == PluginKind::Editor {
        "editor"
    } else {
        "runtime"
    };
    let category = match options.kind {
        PluginKind::Importer => "asset_importer",
        PluginKind::System => "runtime",
        PluginKind::Editor => "authoring",
    };
    let capability = match options.kind {
        PluginKind::Importer => format!("runtime.asset.importer.data.{id}"),
        PluginKind::System => format!("runtime.plugin.{id}"),
        PluginKind::Editor => format!("editor.extension.{id}"),
    };
    let mut files = vec![
        (
            PathBuf::from("plugin.toml"),
            manifest_template(
                options,
                version,
                sdk_api_version,
                &display_name,
                category,
                &capability,
                owner,
                engine_compatibility,
            ),
        ),
        (
            PathBuf::from(format!("{owner}/Cargo.toml")),
            owner_cargo_template(options, &display_name, owner),
        ),
        (
            PathBuf::from(format!("{owner}/src/capability.rs")),
            capability_template(
                options,
                &display_name,
                &constant_prefix,
                category,
                &capability,
                owner,
            ),
        ),
        (
            PathBuf::from(format!("{owner}/src/lib.rs")),
            lib_template(options, &constant_prefix),
        ),
        (
            PathBuf::from(format!("{owner}/src/plugin.rs")),
            plugin_template(options, &constant_prefix, &type_name, engine_compatibility),
        ),
    ];
    if options.native {
        files.push((
            PathBuf::from("dist/Cargo.toml"),
            dist_cargo_template(options, &display_name, owner),
        ));
        files.push((
            PathBuf::from("dist/src/lib.rs"),
            dist_lib_template(options, &capability, owner),
        ));
    }
    files
}

fn manifest_template(
    options: &NewPluginOptions<'_>,
    version: &str,
    sdk_api_version: &str,
    display_name: &str,
    category: &str,
    capability: &str,
    owner: &str,
    engine_compatibility: &str,
) -> String {
    let id = options.id;
    let crate_name = format!("zircon_plugin_{id}_{owner}");
    let target_modes = match options.kind {
        PluginKind::Importer => "\"client_runtime\", \"editor_host\"",
        PluginKind::System => "\"client_runtime\"",
        PluginKind::Editor => "\"editor_host\"",
    };
    let mut output = format!(
        "# @generated from Rust PluginDeclaration; do not edit by hand.\nid = \"{id}\"\nversion = \"{version}\"\nsdk_api_version = \"{sdk_api_version}\"\ndisplay_name = \"{display_name}\"\ncategory = \"{category}\"\ndescription = \"{display_name} plugin package.\"\nsupported_targets = [{target_modes}]\nsupported_platforms = [\"windows\", \"linux\", \"macos\"]\ncapabilities = [\"{capability}\"]\nmaturity = \"experimental\"\ndefault_packaging = [{}]\n",
        if options.native {
            "\"source_template\", \"library_embed\", \"native_dynamic\""
        } else {
            "\"source_template\", \"library_embed\""
        }
    );
    if options.native {
        output.push_str(&format!(
            "\n[distribution]\nforms = [\"dist\"]\ndefault_packaging = [\"native_dynamic\"]\nabi_version = 3\nengine_compat = \"{engine_compatibility}\"\ndist_crate = \"zircon_plugin_{id}_dist\"\ndescriptor_symbol = \"zircon_native_plugin_descriptor_v3\"\n{owner}_entry = \"zircon_plugin_{id}_{owner}_entry_v3\"\n"
        ));
    }
    if options.kind == PluginKind::Importer {
        output.push_str(&format!(
            "\n[[asset_importers]]\nid = \"{id}.{id}\"\nplugin_id = \"{id}\"\npriority = 100\nsource_extensions = [\"{id}\"]\noutput_kind = \"Data\"\nimporter_version = 1\nrequired_capabilities = [\"{capability}\"]\n"
        ));
    }
    output.push_str(&format!(
        "\n[[modules]]\nname = \"{id}.{owner}\"\nkind = \"{owner}\"\ncrate_name = \"{crate_name}\"\ntarget_modes = [{target_modes}]\ncapabilities = [\"{capability}\"]\n"
    ));
    if options.native {
        output.push_str(&format!(
            "\n[[modules]]\nname = \"{id}.dist\"\nkind = \"native\"\ncrate_name = \"zircon_plugin_{id}_dist\"\ntarget_modes = [{target_modes}]\ncapabilities = [\"{capability}\"]\n"
        ));
    }
    output
}

fn owner_cargo_template(options: &NewPluginOptions<'_>, display_name: &str, owner: &str) -> String {
    let id = options.id;
    if owner == "editor" {
        return format!(
            "[package]\nname = \"zircon_plugin_{id}_editor\"\nversion.workspace = true\nedition.workspace = true\nlicense.workspace = true\ndescription = \"{display_name} editor plugin package for Zircon.\"\n\n[features]\ndefault = [\"editor\"]\ndeclaration = [\"zircon_plugin_sdk/declaration\"]\neditor = [\"declaration\", \"dep:zircon_editor\", \"dep:zircon_runtime\", \"zircon_plugin_sdk/editor\"]\n\n[dependencies]\nzircon_editor = {{ workspace = true, optional = true }}\nzircon_plugin_sdk = {{ workspace = true, default-features = false }}\nzircon_runtime = {{ workspace = true, optional = true }}\n"
        );
    }
    format!(
        "[package]\nname = \"zircon_plugin_{id}_runtime\"\nversion.workspace = true\nedition.workspace = true\nlicense.workspace = true\ndescription = \"{display_name} runtime plugin package for Zircon.\"\n\n[features]\ndefault = [\"runtime\"]\ndeclaration = [\"zircon_plugin_sdk/declaration\"]\nruntime = [\"declaration\", \"dep:zircon_runtime\", \"zircon_plugin_sdk/runtime\"]\n\n[dependencies]\nzircon_plugin_sdk = {{ workspace = true, default-features = false }}\nzircon_runtime = {{ workspace = true, optional = true }}\n"
    )
}

fn capability_template(
    options: &NewPluginOptions<'_>,
    display_name: &str,
    constant_prefix: &str,
    category: &str,
    capability: &str,
    owner: &str,
) -> String {
    let id = options.id;
    let declaration = format!("{constant_prefix}_DECLARATION");
    let crate_constant = format!("{}_CRATE_NAME", owner.to_ascii_uppercase());
    let registration = if owner == "editor" {
        "editor_registration"
    } else {
        "runtime_registration"
    };
    let target_modes = match options.kind {
        PluginKind::Importer => "client_runtime, editor_host",
        PluginKind::System => "client_runtime",
        PluginKind::Editor => "editor_host",
    };
    let native_extensions = if options.kind == PluginKind::Importer {
        format!(
            "[{{\n                    point: \"runtime.asset.importer.data\",\n                    contribution: \"plugin.{id}.runtime\",\n                    schema: \"zircon.runtime.asset-importer.data/1\",\n                }}]"
        )
    } else {
        "[]".to_string()
    };
    let native_projection = if options.native {
        format!(
            "        native_projection: {{\n            plugin_id: NATIVE_PLUGIN_ID,\n            requested_capabilities: NATIVE_REQUESTED_CAPABILITIES,\n            {owner}: {{\n                entry: NATIVE_{}_ENTRY = \"zircon_plugin_{id}_{owner}_entry_v3\",\n                registration_manifest: NATIVE_{}_REGISTRATION_MANIFEST,\n                modules: [{{ name: \"{owner}\", kind: \"{owner}\" }}],\n                systems: [],\n                events: [],\n                extensions: {native_extensions},\n            }},\n        }},\n",
            owner.to_ascii_uppercase(),
            owner.to_ascii_uppercase()
        )
    } else {
        String::new()
    };
    format!(
        "zircon_plugin_sdk::declare_plugin! {{\n    pub {declaration} {{\n        id: PLUGIN_ID = \"{id}\",\n        display_name: \"{display_name}\",\n        category: {category},\n        module: MODULE_NAME = \"{id}.{owner}\",\n        crate_name: {crate_constant} = \"zircon_plugin_{id}_{owner}\",\n        module_description: \"{display_name} {owner} services\",\n        targets: [{}],\n        platforms: [windows, linux, macos],\n        capabilities: [\n            CAPABILITY = \"{capability}\" => {registration},\n        ],\n        maturity: experimental,\n        packaging: [{}],\n{native_projection}    }}\n}}\n\npub const {}_CAPABILITIES: &[&str] = &[CAPABILITY];\n",
        target_modes,
        if options.native {
            "source_template, library_embed, native_dynamic"
        } else {
            "source_template, library_embed"
        },
        owner.to_ascii_uppercase()
    )
}

fn lib_template(options: &NewPluginOptions<'_>, constant_prefix: &str) -> String {
    let owner = if options.kind == PluginKind::Editor {
        "EDITOR"
    } else {
        "RUNTIME"
    };
    let native_exports = if options.native {
        format!(
            ", NATIVE_{owner}_ENTRY, NATIVE_{owner}_REGISTRATION_MANIFEST, NATIVE_PLUGIN_ID, NATIVE_REQUESTED_CAPABILITIES"
        )
    } else {
        String::new()
    };
    let plugin_module = if owner == "EDITOR" {
        "#[cfg(feature = \"editor\")]\nmod plugin;"
    } else {
        "#[cfg(feature = \"runtime\")]\nmod plugin;"
    };
    let plugin_export = if owner == "EDITOR" {
        "#[cfg(feature = \"editor\")]\npub use plugin::*;"
    } else {
        "#[cfg(feature = \"runtime\")]\npub use plugin::*;"
    };
    format!(
        "mod capability;\n{plugin_module}\n\npub use capability::{{CAPABILITY, {constant_prefix}_DECLARATION, {owner}_CAPABILITIES, {owner}_CRATE_NAME, MODULE_NAME, PLUGIN_ID{native_exports}}};\n{plugin_export}\n"
    )
}

fn plugin_template(
    options: &NewPluginOptions<'_>,
    constant_prefix: &str,
    type_name: &str,
    engine_compatibility: &str,
) -> String {
    let id = options.id;
    if options.kind == PluginKind::Editor {
        return editor_plugin_template(options, constant_prefix, type_name, engine_compatibility);
    }
    if options.kind == PluginKind::Importer {
        return importer_plugin_template(options, constant_prefix, type_name, engine_compatibility);
    }
    let native_entry_import = options
        .native
        .then_some(", NATIVE_RUNTIME_ENTRY")
        .unwrap_or("");
    let native_types = options
        .native
        .then_some(", PluginDistributionManifest, PluginModuleManifest")
        .unwrap_or("");
    let native_import = options
        .native
        .then_some("use zircon_runtime::core::framework::project::ExportPackagingStrategy;\n")
        .unwrap_or("");
    let native_chain = native_manifest_chain(options.native);
    let native_items = native_manifest_items(
        id,
        constant_prefix,
        "runtime",
        engine_compatibility,
        options.native,
    );
    let package_coordinate_items = package_coordinate_items();
    format!(
        "use crate::{{{constant_prefix}_DECLARATION, RUNTIME_CRATE_NAME{native_entry_import}}};\n{native_import}use zircon_runtime::plugin::{{PluginPackageManifest{native_types}, RuntimeExtensionRegistry, RuntimeExtensionRegistryError, RuntimePlugin, RuntimePluginDescriptor}};\n\n#[derive(Clone, Debug)]\npub struct {type_name}RuntimePlugin {{\n    descriptor: RuntimePluginDescriptor,\n}}\n\nimpl Default for {type_name}RuntimePlugin {{\n    fn default() -> Self {{\n        Self {{ descriptor: runtime_plugin_descriptor() }}\n    }}\n}}\n\nimpl RuntimePlugin for {type_name}RuntimePlugin {{\n    fn descriptor(&self) -> &RuntimePluginDescriptor {{ &self.descriptor }}\n\n    fn package_manifest(&self) -> PluginPackageManifest {{\n        generated_package_manifest(self.descriptor.package_manifest(){native_chain})\n    }}\n\n    fn register(&self, _registry: &mut RuntimeExtensionRegistry) -> Result<(), RuntimeExtensionRegistryError> {{\n        Ok(())\n    }}\n}}\n\npub fn runtime_plugin_descriptor() -> RuntimePluginDescriptor {{\n    {constant_prefix}_DECLARATION.runtime_descriptor(RUNTIME_CRATE_NAME)\n}}\n{package_coordinate_items}{native_items}\nzircon_plugin_sdk::runtime_plugin_exports!({type_name}RuntimePlugin);\n"
    )
}

fn editor_plugin_template(
    options: &NewPluginOptions<'_>,
    constant_prefix: &str,
    type_name: &str,
    engine_compatibility: &str,
) -> String {
    let id = options.id;
    let declaration = format!("{constant_prefix}_DECLARATION");
    let native_entry_import = options
        .native
        .then_some(", NATIVE_EDITOR_ENTRY")
        .unwrap_or("");
    let native_types = options
        .native
        .then_some(", PluginDistributionManifest, PluginModuleManifest")
        .unwrap_or("");
    let native_import = options
        .native
        .then_some("use zircon_runtime::core::framework::project::ExportPackagingStrategy;\n")
        .unwrap_or("");
    let native_chain = native_manifest_chain(options.native);
    let native_items = native_manifest_items(
        id,
        constant_prefix,
        "editor",
        engine_compatibility,
        options.native,
    );
    let package_coordinate_items = package_coordinate_items();
    format!(
        "use crate::{{EDITOR_CRATE_NAME, {declaration}{native_entry_import}}};\n{native_import}use zircon_runtime::plugin::{{PluginPackageManifest{native_types}}};\n\n#[derive(Clone, Debug)]\npub struct {type_name}EditorPlugin {{\n    descriptor: zircon_editor::EditorPluginDescriptor,\n}}\n\nimpl Default for {type_name}EditorPlugin {{\n    fn default() -> Self {{\n        Self {{ descriptor: editor_plugin_descriptor() }}\n    }}\n}}\n\nimpl zircon_editor::EditorPlugin for {type_name}EditorPlugin {{\n    fn descriptor(&self) -> &zircon_editor::EditorPluginDescriptor {{ &self.descriptor }}\n\n    fn register_editor_extensions(\n        &self,\n        _registry: &mut zircon_editor::core::editor_extension::EditorExtensionRegistry,\n    ) -> Result<(), zircon_editor::core::editor_extension::EditorExtensionRegistryError> {{\n        Ok(())\n    }}\n}}\n\npub fn editor_plugin_descriptor() -> zircon_editor::EditorPluginDescriptor {{\n    let declaration = {declaration};\n    declaration.capabilities().iter().fold(\n        zircon_editor::EditorPluginDescriptor::new(\n            declaration.id(),\n            declaration.display_name(),\n            EDITOR_CRATE_NAME,\n        )\n        .with_category(declaration.category()),\n        |descriptor, capability| descriptor.with_capability(*capability),\n    )\n}}\n\npub fn editor_plugin() -> {type_name}EditorPlugin {{ {type_name}EditorPlugin::default() }}\n\nfn base_manifest() -> PluginPackageManifest {{\n    let declaration = {declaration};\n    generated_package_manifest(\n        PluginPackageManifest::new(declaration.id(), declaration.display_name())\n            .with_category(declaration.category())\n            .with_supported_targets(declaration.target_modes())\n            .with_supported_platforms(declaration.supported_platforms())\n            .with_capabilities(declaration.capabilities().iter().copied())\n            .with_maturity(declaration.maturity())\n            .with_default_packaging(declaration.default_packaging()){native_chain},\n    )\n}}\n\npub fn package_manifest() -> PluginPackageManifest {{\n    zircon_editor::EditorPlugin::package_manifest(&editor_plugin(), base_manifest())\n}}\n\npub fn plugin_registration() -> zircon_editor::EditorPluginRegistrationReport {{\n    zircon_editor::EditorPluginRegistrationReport::from_plugin(&editor_plugin(), base_manifest())\n}}\n{package_coordinate_items}{native_items}"
    )
}

fn importer_plugin_template(
    options: &NewPluginOptions<'_>,
    constant_prefix: &str,
    type_name: &str,
    engine_compatibility: &str,
) -> String {
    let id = options.id;
    let native_entry_import = options
        .native
        .then_some(", NATIVE_RUNTIME_ENTRY")
        .unwrap_or("");
    let native_types = options
        .native
        .then_some(", PluginDistributionManifest, PluginModuleManifest")
        .unwrap_or("");
    let native_import = options
        .native
        .then_some("use zircon_runtime::core::framework::project::ExportPackagingStrategy;\n")
        .unwrap_or("");
    let native_chain = native_manifest_chain(options.native);
    let native_items = native_manifest_items(
        id,
        constant_prefix,
        "runtime",
        engine_compatibility,
        options.native,
    );
    let package_coordinate_items = package_coordinate_items();
    format!(
        "use crate::{{CAPABILITY, {constant_prefix}_DECLARATION, PLUGIN_ID, RUNTIME_CRATE_NAME{native_entry_import}}};\n{native_import}use zircon_runtime::asset::{{AssetImporterDescriptor, AssetKind}};\nuse zircon_runtime::plugin::{{PluginPackageManifest{native_types}, RuntimeExtensionRegistry, RuntimeExtensionRegistryError, RuntimePlugin, RuntimePluginDescriptor}};\n\n#[derive(Clone, Debug)]\npub struct {type_name}RuntimePlugin {{\n    descriptor: RuntimePluginDescriptor,\n}}\n\nimpl {type_name}RuntimePlugin {{\n    pub fn new() -> Self {{\n        Self {{ descriptor: runtime_plugin_descriptor() }}\n    }}\n}}\n\nimpl Default for {type_name}RuntimePlugin {{\n    fn default() -> Self {{ Self::new() }}\n}}\n\nimpl RuntimePlugin for {type_name}RuntimePlugin {{\n    fn descriptor(&self) -> &RuntimePluginDescriptor {{ &self.descriptor }}\n\n    fn package_manifest(&self) -> PluginPackageManifest {{\n        generated_package_manifest(\n            self.descriptor\n                .package_manifest()\n                .with_asset_importer(asset_importer_descriptor()){native_chain},\n        )\n    }}\n\n    fn register(&self, registry: &mut RuntimeExtensionRegistry) -> Result<(), RuntimeExtensionRegistryError> {{\n        registry.register_asset_importer_descriptor(asset_importer_descriptor())\n    }}\n}}\n\npub fn runtime_plugin_descriptor() -> RuntimePluginDescriptor {{\n    {constant_prefix}_DECLARATION.runtime_descriptor(RUNTIME_CRATE_NAME)\n}}\n\npub fn asset_importer_descriptor() -> AssetImporterDescriptor {{\n    AssetImporterDescriptor::new(\"{id}.{id}\", PLUGIN_ID, AssetKind::Data, 1)\n        .with_priority(100)\n        .with_source_extensions([\"{id}\"])\n        .with_required_capabilities([CAPABILITY])\n}}\n{package_coordinate_items}{native_items}\nzircon_plugin_sdk::runtime_plugin_exports!({type_name}RuntimePlugin);\n"
    )
}

fn native_manifest_chain(native: bool) -> &'static str {
    if native {
        "\n            .with_native_module(native_dist_module_manifest())\n            .with_distribution(native_distribution_manifest())"
    } else {
        ""
    }
}

fn package_coordinate_items() -> &'static str {
    "\nfn generated_package_manifest(\n    mut manifest: PluginPackageManifest,\n) -> PluginPackageManifest {\n    manifest.version = env!(\"CARGO_PKG_VERSION\").to_string();\n    manifest.sdk_api_version = zircon_plugin_sdk::SDK_API_VERSION.to_string();\n    manifest\n}\n"
}

fn native_manifest_items(
    id: &str,
    constant_prefix: &str,
    owner: &str,
    engine_compatibility: &str,
    native: bool,
) -> String {
    if !native {
        return String::new();
    }
    let declaration = format!("{constant_prefix}_DECLARATION");
    let entry = if owner == "editor" {
        "NATIVE_EDITOR_ENTRY"
    } else {
        "NATIVE_RUNTIME_ENTRY"
    };
    format!(
        "\nconst DIST_CRATE_NAME: &str = \"zircon_plugin_{id}_dist\";\nconst DIST_ENGINE_COMPAT: &str = \"{engine_compatibility}\";\nconst NATIVE_DESCRIPTOR_SYMBOL_V3: &str = \"zircon_native_plugin_descriptor_v3\";\nconst NATIVE_ABI_VERSION_V3: u32 = 3;\n\nfn native_dist_module_manifest() -> PluginModuleManifest {{\n    PluginModuleManifest::native(\n        format!(\"{{}}.dist\", {declaration}.id()),\n        DIST_CRATE_NAME,\n    )\n    .with_target_modes({declaration}.target_modes())\n    .with_capabilities({declaration}.capabilities().iter().copied())\n}}\n\nfn native_distribution_manifest() -> PluginDistributionManifest {{\n    PluginDistributionManifest {{\n        forms: vec![\"dist\".to_string()],\n        default_packaging: vec![ExportPackagingStrategy::NativeDynamic],\n        abi_version: Some(NATIVE_ABI_VERSION_V3),\n        engine_compat: DIST_ENGINE_COMPAT.to_string(),\n        dist_crate: DIST_CRATE_NAME.to_string(),\n        descriptor_symbol: NATIVE_DESCRIPTOR_SYMBOL_V3.to_string(),\n        {owner}_entry: {entry}.name().to_string(),\n        ..PluginDistributionManifest::default()\n    }}\n}}\n"
    )
}

fn dist_cargo_template(options: &NewPluginOptions<'_>, display_name: &str, owner: &str) -> String {
    let id = options.id;
    format!(
        "[package]\nname = \"zircon_plugin_{id}_dist\"\nversion.workspace = true\nedition.workspace = true\nlicense.workspace = true\ndescription = \"{display_name} native plugin distribution for Zircon.\"\n\n[lib]\ncrate-type = [\"cdylib\"]\n\n[features]\ndefault = [\"dist\"]\ndist = [\"zircon_plugin_{id}_{owner}/declaration\", \"zircon_plugin_sdk/native\"]\n\n[dependencies]\nzircon_plugin_sdk = {{ workspace = true, default-features = false }}\nzircon_plugin_{id}_{owner} = {{ path = \"../{owner}\", default-features = false }}\n"
    )
}

fn dist_lib_template(options: &NewPluginOptions<'_>, capability: &str, owner: &str) -> String {
    let id = options.id;
    let upper_owner = owner.to_ascii_uppercase();
    let macro_name = if owner == "editor" {
        "native_dist_editor_plugin_v3"
    } else {
        "native_dist_runtime_plugin_v3"
    };
    format!(
        "use zircon_plugin_{id}_{owner}::{{NATIVE_{upper_owner}_ENTRY, NATIVE_{upper_owner}_REGISTRATION_MANIFEST, NATIVE_PLUGIN_ID, NATIVE_REQUESTED_CAPABILITIES}};\nuse zircon_plugin_sdk::native::ZIRCON_NATIVE_PLUGIN_ABI_VERSION;\n\nconst PLUGIN_MANIFEST: &str = concat!(include_str!(\"../../plugin.toml\"), \"\\0\");\nconst DIAGNOSTICS: &[u8] = b\"{id} {owner} entry ready\\0\";\nconst MISSING_HOST_DIAGNOSTICS: &[u8] = b\"{id} requires {capability}\\0\";\nconst EMPTY_MANIFEST: &[u8] = b\"\\0\";\n\nzircon_plugin_sdk::{macro_name}! {{\n    plugin_id: NATIVE_PLUGIN_ID,\n    package_manifest: PLUGIN_MANIFEST,\n    descriptor_abi_version: ZIRCON_NATIVE_PLUGIN_ABI_VERSION,\n    {owner}_entry: zircon_plugin_{id}_{owner}_entry_v3,\n    {owner}_entry_name: NATIVE_{upper_owner}_ENTRY.cstr(),\n    requested_capabilities: NATIVE_REQUESTED_CAPABILITIES,\n    missing_host_diagnostics: MISSING_HOST_DIAGNOSTICS,\n    {owner}: {{\n        required_capabilities: [\"{capability}\"],\n        denied_capabilities: [],\n        negotiated_capabilities: NATIVE_REQUESTED_CAPABILITIES,\n        diagnostics: DIAGNOSTICS,\n        is_stateless: true,\n        state_schema_version: 0,\n        command_manifest_schema: None,\n        event_manifest_schema: None,\n        registration_manifest_schema: Some(zircon_plugin_sdk::native::NATIVE_REGISTRATION_MANIFEST_SCHEMA_V3),\n        command_manifest: Some(EMPTY_MANIFEST),\n        event_manifest: Some(EMPTY_MANIFEST),\n        registration_manifest: Some(NATIVE_{upper_owner}_REGISTRATION_MANIFEST),\n        invoke_command: None,\n        save_state: None,\n        restore_state: None,\n        unload: None,\n        bridge_methods: [],\n        on_host_ready: None,\n    }},\n}}\n"
    )
}

fn display_name(id: &str) -> String {
    id.split('_')
        .map(|part| {
            let mut characters = part.chars();
            match characters.next() {
                Some(first) => first.to_ascii_uppercase().to_string() + characters.as_str(),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn rust_type_name(id: &str) -> String {
    id.split('_')
        .map(|part| {
            let mut characters = part.chars();
            match characters.next() {
                Some(first) => first.to_ascii_uppercase().to_string() + characters.as_str(),
                None => String::new(),
            }
        })
        .collect()
}
