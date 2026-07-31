//! Compile-time native plugin manifest rendering.
//!
//! Native ABI descriptors borrow C strings from immutable storage, so their
//! package manifest must be rendered without allocation and end with a NUL.
//! This macro is metadata-only; entry points, callbacks, and host behavior
//! remain explicit in the consuming native plugin crate.

#[macro_export]
macro_rules! native_plugin_manifest_v3 {
    (
        $visibility:vis $manifest:ident {
            id: $id_constant:ident = $id:literal,
            requested_capabilities: $requested_capabilities:ident,
            version: $version:literal,
            sdk_api_version: $sdk_api_version:literal,
            display_name: $display_name:literal,
            category: $category:literal,
            description: $description:literal,
            maturity: $maturity:literal,
            targets: [$first_target:literal $(, $target:literal)* $(,)?],
            platforms: [$first_platform:literal $(, $platform:literal)* $(,)?],
            capabilities: [$first_capability:literal $(, $capability:literal)* $(,)?],
            packaging: [$first_packaging:literal $(, $packaging:literal)* $(,)?],
            distribution: {
                forms: [$first_form:literal $(, $form:literal)* $(,)?],
                default_packaging: [$first_distribution_packaging:literal $(, $distribution_packaging:literal)* $(,)?],
                abi_version: $abi_version:literal,
                engine_compat: $engine_compat:literal,
                dist_crate: $dist_crate:literal,
                descriptor_symbol: $descriptor_symbol:literal,
                runtime_entry: $runtime_entry:literal,
                editor_entry: $editor_entry:literal,
                assets: [$first_asset:literal $(, $asset:literal)* $(,)?] $(,)?
            },
            asset_importer: {
                id: $asset_importer_id_constant:ident = $asset_importer_id:literal,
                priority: $asset_importer_priority:literal,
                source_extensions: [$first_source_extension:literal $(, $source_extension:literal)* $(,)?],
                output_kind: $output_kind:literal,
                importer_version: $importer_version:literal,
                required_capabilities: [$first_required_capability:literal $(, $required_capability:literal)* $(,)?] $(,)?
            },
            modules: [
                $(
                    {
                        name: $module_name:literal,
                        kind: $module_kind:literal,
                        crate_name: $module_crate_name:literal,
                        target_modes: [$first_module_target:literal $(, $module_target:literal)* $(,)?],
                        capabilities: [$first_module_capability:literal $(, $module_capability:literal)* $(,)?] $(,)?
                    }
                ),+ $(,)?
            ],
            interface: {
                id: $interface_id:literal,
                methods: [
                    $(
                        {
                            name: $method_name:literal,
                            method_slot: $method_slot:literal $(,)?
                        }
                    ),+ $(,)?
                ] $(,)?
            } $(,)?
        }
    ) => {
        $visibility const $id_constant: &[u8] = concat!($id, "\0").as_bytes();
        $visibility const $requested_capabilities: &[u8] = concat!(
            $first_capability,
            "\n",
            $(
                $capability,
                "\n",
            )*
            "\0"
        )
        .as_bytes();
        $visibility const $asset_importer_id_constant: &str = $asset_importer_id;
        $visibility const $manifest: &str = concat!(
            "# @generated from zircon_plugin_sdk::native_plugin_manifest_v3!; do not edit by hand.\n",
            "id = \"",
            $id,
            "\"\n",
            "version = \"",
            $version,
            "\"\n",
            "sdk_api_version = \"",
            $sdk_api_version,
            "\"\n",
            "display_name = \"",
            $display_name,
            "\"\n",
            "category = \"",
            $category,
            "\"\n",
            "description = \"",
            $description,
            "\"\n",
            "maturity = \"",
            $maturity,
            "\"\n",
            "supported_targets = [\"",
            $first_target,
            "\"",
            $(
                ", \"",
                $target,
                "\"",
            )*
            "]\n",
            "supported_platforms = [\"",
            $first_platform,
            "\"",
            $(
                ", \"",
                $platform,
                "\"",
            )*
            "]\n",
            "capabilities = [\"",
            $first_capability,
            "\"",
            $(
                ", \"",
                $capability,
                "\"",
            )*
            "]\n",
            "default_packaging = [\"",
            $first_packaging,
            "\"",
            $(
                ", \"",
                $packaging,
                "\"",
            )*
            "]\n\n",
            "[distribution]\n",
            "forms = [\"",
            $first_form,
            "\"",
            $(
                ", \"",
                $form,
                "\"",
            )*
            "]\n",
            "default_packaging = [\"",
            $first_distribution_packaging,
            "\"",
            $(
                ", \"",
                $distribution_packaging,
                "\"",
            )*
            "]\n",
            "abi_version = ",
            stringify!($abi_version),
            "\n",
            "engine_compat = \"",
            $engine_compat,
            "\"\n",
            "dist_crate = \"",
            $dist_crate,
            "\"\n",
            "descriptor_symbol = \"",
            $descriptor_symbol,
            "\"\n",
            "runtime_entry = \"",
            $runtime_entry,
            "\"\n",
            "editor_entry = \"",
            $editor_entry,
            "\"\n",
            "assets = [\"",
            $first_asset,
            "\"",
            $(
                ", \"",
                $asset,
                "\"",
            )*
            "]\n\n",
            "[[asset_importers]]\n",
            "id = \"",
            $asset_importer_id,
            "\"\n",
            "plugin_id = \"",
            $id,
            "\"\n",
            "priority = ",
            stringify!($asset_importer_priority),
            "\n",
            "source_extensions = [\"",
            $first_source_extension,
            "\"",
            $(
                ", \"",
                $source_extension,
                "\"",
            )*
            "]\n",
            "output_kind = \"",
            $output_kind,
            "\"\n",
            "importer_version = ",
            stringify!($importer_version),
            "\n",
            "required_capabilities = [\"",
            $first_required_capability,
            "\"",
            $(
                ", \"",
                $required_capability,
                "\"",
            )*
            "]\n\n",
            $(
                "[[modules]]\n",
                "name = \"",
                $module_name,
                "\"\n",
                "kind = \"",
                $module_kind,
                "\"\n",
                "crate_name = \"",
                $module_crate_name,
                "\"\n",
                "target_modes = [\"",
                $first_module_target,
                "\"",
                $(
                    ", \"",
                    $module_target,
                    "\"",
                )*
                "]\n",
                "capabilities = [\"",
                $first_module_capability,
                "\"",
                $(
                    ", \"",
                    $module_capability,
                    "\"",
                )*
                "]\n\n",
            )+
            "[[provides_interfaces]]\n",
            "id = \"",
            $interface_id,
            "\"\n\n",
            $(
                "[[provides_interfaces.methods]]\n",
                "name = \"",
                $method_name,
                "\"\n",
                "method_slot = ",
                stringify!($method_slot),
                "\n",
            )+
            "\0"
        );
    };
}
