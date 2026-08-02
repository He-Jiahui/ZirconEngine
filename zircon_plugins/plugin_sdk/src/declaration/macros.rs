/// Declare package metadata once and project it into runtime and native ABI data.
///
/// The macro is deliberately data-only: registration tables and callback behavior
/// stay in the plugin crate, where their ownership remains visible to maintainers.
#[macro_export]
macro_rules! declare_plugin {
    (
        $(#[$metadata:meta])*
        $visibility:vis $declaration:ident {
            id: $id_constant:ident = $id:literal,
            display_name: $display_name:literal,
            category: $category:ident,
            module: $module_constant:ident = $module_name:literal,
            crate_name: $crate_name_constant:ident = $crate_name:literal,
            module_description: $module_description:literal,
            targets: [$($target:ident),+ $(,)?],
            platforms: [$($platform:ident),+ $(,)?],
            capabilities: [
                $first_capability_constant:ident = $first_capability:literal
                    => $first_capability_registration:ident
                $(
                    , $capability_constant:ident = $capability:literal
                        => $capability_registration:ident
                )* $(,)?
            ],
            maturity: $maturity:ident,
            packaging: [$($packaging:ident),+ $(,)?],
            $(
                native_projection: {
                    plugin_id: $native_plugin_id:ident,
                    requested_capabilities: $native_requested_capabilities:ident,
                    $(
                        runtime: {
                            entry: $native_runtime_entry:ident = $native_runtime_entry_name:literal,
                            registration_manifest: $native_runtime_registration_manifest:ident,
                            modules: [
                                $(
                                    {
                                        name: $native_runtime_module_name:literal,
                                        kind: $native_runtime_module_kind:literal $(,)?
                                    }
                                ),+ $(,)?
                            ],
                            systems: [
                                $(
                                    {
                                        id: $native_runtime_system_id:literal,
                                        module: $native_runtime_system_module:literal,
                                        stage: $native_runtime_system_stage:literal,
                                        order: $native_runtime_system_order:literal,
                                        sets: [
                                            $native_runtime_system_first_set:literal
                                            $(, $native_runtime_system_set:literal)* $(,)?
                                        ],
                                        access: [
                                            $native_runtime_system_first_access:literal
                                            $(, $native_runtime_system_access:literal)* $(,)?
                                        ],
                                        thread_affinity: $native_runtime_system_thread_affinity:literal,
                                        bridge_interface: $native_runtime_system_bridge_interface:literal,
                                        bridge_method: $native_runtime_system_bridge_method:literal $(,)?
                                    }
                                ),* $(,)?
                            ],
                            events: [
                                $(
                                    {
                                        namespace: $native_runtime_event_namespace:literal,
                                        name: $native_runtime_event_name:literal,
                                        stable_hash: $native_runtime_event_stable_hash:literal,
                                        schema: $native_runtime_event_schema:literal $(,)?
                                    }
                                ),* $(,)?
                            ],
                            extensions: [
                                $(
                                    {
                                        point: $native_runtime_extension_point:literal,
                                        contribution: $native_runtime_extension_contribution:literal,
                                        schema: $native_runtime_extension_schema:literal $(,)?
                                    }
                                ),* $(,)?
                            ] $(,)?
                        } $(,)?
                    )?
                    $(
                        editor: {
                            entry: $native_editor_entry:ident = $native_editor_entry_name:literal,
                            registration_manifest: $native_editor_registration_manifest:ident,
                            modules: [
                                $(
                                    {
                                        name: $native_editor_module_name:literal,
                                        kind: $native_editor_module_kind:literal $(,)?
                                    }
                                ),+ $(,)?
                            ],
                            systems: [
                                $(
                                    {
                                        id: $native_editor_system_id:literal,
                                        module: $native_editor_system_module:literal,
                                        stage: $native_editor_system_stage:literal,
                                        order: $native_editor_system_order:literal,
                                        sets: [
                                            $native_editor_system_first_set:literal
                                            $(, $native_editor_system_set:literal)* $(,)?
                                        ],
                                        access: [
                                            $native_editor_system_first_access:literal
                                            $(, $native_editor_system_access:literal)* $(,)?
                                        ],
                                        thread_affinity: $native_editor_system_thread_affinity:literal,
                                        bridge_interface: $native_editor_system_bridge_interface:literal,
                                        bridge_method: $native_editor_system_bridge_method:literal $(,)?
                                    }
                                ),* $(,)?
                            ],
                            events: [
                                $(
                                    {
                                        namespace: $native_editor_event_namespace:literal,
                                        name: $native_editor_event_name:literal,
                                        stable_hash: $native_editor_event_stable_hash:literal,
                                        schema: $native_editor_event_schema:literal $(,)?
                                    }
                                ),* $(,)?
                            ],
                            extensions: [
                                $(
                                    {
                                        point: $native_editor_extension_point:literal,
                                        contribution: $native_editor_extension_contribution:literal,
                                        schema: $native_editor_extension_schema:literal $(,)?
                                    }
                                ),* $(,)?
                            ] $(,)?
                        } $(,)?
                    )?
                } $(,)?
            )?
        }
    ) => {
        $(#[$metadata])*
        $visibility const $id_constant: &str = $id;
        $visibility const $module_constant: &str = $module_name;
        $visibility const $crate_name_constant: &str = $crate_name;
        $visibility const $first_capability_constant: &str = $first_capability;
        $(
            $visibility const $capability_constant: &str = $capability;
        )*
        $visibility const $declaration: $crate::PluginDeclaration =
            $crate::PluginDeclaration::new(
                $id_constant,
                $display_name,
                stringify!($category),
                $module_constant,
                $module_description,
                &[$($crate::declare_plugin!(@target $target)),+],
                &[$($crate::declare_plugin!(@platform $platform)),+],
                &[$first_capability_constant $(, $capability_constant)*],
                &[
                    $crate::declare_plugin!(@capability_role $first_capability_registration)
                    $(
                        , $crate::declare_plugin!(@capability_role $capability_registration)
                    )*
                ],
                $crate::declare_plugin!(@maturity $maturity),
                &[$($crate::declare_plugin!(@packaging $packaging)),+],
            );
        $crate::declare_plugin! {
            @native_projection
            [$visibility]
            [
                $first_capability => $first_capability_registration
                $(, $capability => $capability_registration)*
            ]
            $(
                {
                    plugin_id: $native_plugin_id,
                    id: $id,
                    requested_capabilities: $native_requested_capabilities,
                    $(
                        runtime: {
                            entry: $native_runtime_entry = $native_runtime_entry_name,
                            registration_manifest: $native_runtime_registration_manifest,
                            modules: [
                                $(
                                    {
                                        name: $native_runtime_module_name,
                                        kind: $native_runtime_module_kind,
                                    }
                                ),+
                            ],
                            systems: [
                                $(
                                    {
                                        id: $native_runtime_system_id,
                                        module: $native_runtime_system_module,
                                        stage: $native_runtime_system_stage,
                                        order: $native_runtime_system_order,
                                        sets: [
                                            $native_runtime_system_first_set
                                            $(, $native_runtime_system_set)*
                                        ],
                                        access: [
                                            $native_runtime_system_first_access
                                            $(, $native_runtime_system_access)*
                                        ],
                                        thread_affinity: $native_runtime_system_thread_affinity,
                                        bridge_interface: $native_runtime_system_bridge_interface,
                                        bridge_method: $native_runtime_system_bridge_method,
                                    }
                                ),*
                            ],
                            events: [
                                $(
                                    {
                                        namespace: $native_runtime_event_namespace,
                                        name: $native_runtime_event_name,
                                        stable_hash: $native_runtime_event_stable_hash,
                                        schema: $native_runtime_event_schema,
                                    }
                                ),*
                            ],
                            extensions: [
                                $(
                                    {
                                        point: $native_runtime_extension_point,
                                        contribution: $native_runtime_extension_contribution,
                                        schema: $native_runtime_extension_schema,
                                    }
                                ),*
                            ],
                        },
                    )?
                    $(
                        editor: {
                            entry: $native_editor_entry = $native_editor_entry_name,
                            registration_manifest: $native_editor_registration_manifest,
                            modules: [
                                $(
                                    {
                                        name: $native_editor_module_name,
                                        kind: $native_editor_module_kind,
                                    }
                                ),+
                            ],
                            systems: [
                                $(
                                    {
                                        id: $native_editor_system_id,
                                        module: $native_editor_system_module,
                                        stage: $native_editor_system_stage,
                                        order: $native_editor_system_order,
                                        sets: [
                                            $native_editor_system_first_set
                                            $(, $native_editor_system_set)*
                                        ],
                                        access: [
                                            $native_editor_system_first_access
                                            $(, $native_editor_system_access)*
                                        ],
                                        thread_affinity: $native_editor_system_thread_affinity,
                                        bridge_interface: $native_editor_system_bridge_interface,
                                        bridge_method: $native_editor_system_bridge_method,
                                    }
                                ),*
                            ],
                            events: [
                                $(
                                    {
                                        namespace: $native_editor_event_namespace,
                                        name: $native_editor_event_name,
                                        stable_hash: $native_editor_event_stable_hash,
                                        schema: $native_editor_event_schema,
                                    }
                                ),*
                            ],
                            extensions: [
                                $(
                                    {
                                        point: $native_editor_extension_point,
                                        contribution: $native_editor_extension_contribution,
                                        schema: $native_editor_extension_schema,
                                    }
                                ),*
                            ],
                        },
                    )?
                }
            )?
        }
    };
    (
        @native_projection
        [$visibility:vis]
        [
            $first_capability:literal => $first_capability_registration:ident
            $(, $capability:literal => $capability_registration:ident)*
        ]
    ) => {};
    (
        @native_projection
        [$visibility:vis]
        [
            $first_capability:literal => $first_capability_registration:ident
            $(, $capability:literal => $capability_registration:ident)*
        ]
        {
            plugin_id: $native_plugin_id:ident,
            id: $id:literal,
            requested_capabilities: $native_requested_capabilities:ident,
            $($native_projection_entry:tt)*
        }
    ) => {
        $visibility const $native_plugin_id: &[u8] = concat!($id, "\0").as_bytes();
        $visibility const $native_requested_capabilities: &[u8] = concat!(
            $first_capability
            $(, "\n", $capability)*,
            "\0"
        )
        .as_bytes();
        $crate::declare_plugin! {
            @native_projection_entries
            [$visibility]
            [
                $first_capability => $first_capability_registration
                $(, $capability => $capability_registration)*
            ]
            $($native_projection_entry)*
        }
    };
    (
        @native_projection_entries
        [$visibility:vis]
        [$($capability:literal => $capability_registration:ident),+]
    ) => {};
    (
        @native_projection_entries
        [$visibility:vis]
        [$($capability:literal => $capability_registration:ident),+]
        runtime: {
            entry: $native_entry:ident = $native_entry_name:literal,
            registration_manifest: $native_registration_manifest:ident,
            modules: [$($native_module:tt)*],
            systems: [$($native_system:tt)*],
            events: [$($native_event:tt)*],
            extensions: [$($native_extension:tt)*] $(,)?
        },
        $($remaining_projection_entry:tt)*
    ) => {
        $visibility const $native_entry: $crate::NativePluginEntryDeclaration =
            $crate::NativePluginEntryDeclaration::new(
                $native_entry_name,
                concat!($native_entry_name, "\0").as_bytes(),
            );
        $crate::declare_plugin! {
            @registration_manifest
            [$visibility]
            [runtime]
            [$native_registration_manifest]
            [$($capability => $capability_registration),+]
            {
                modules: [$($native_module)*],
                systems: [$($native_system)*],
                events: [$($native_event)*],
                extensions: [$($native_extension)*],
            }
        }
        $crate::declare_plugin! {
            @native_projection_entries
            [$visibility]
            [$($capability => $capability_registration),+]
            $($remaining_projection_entry)*
        }
    };
    (
        @native_projection_entries
        [$visibility:vis]
        [$($capability:literal => $capability_registration:ident),+]
        editor: {
            entry: $native_entry:ident = $native_entry_name:literal,
            registration_manifest: $native_registration_manifest:ident,
            modules: [$($native_module:tt)*],
            systems: [$($native_system:tt)*],
            events: [$($native_event:tt)*],
            extensions: [$($native_extension:tt)*] $(,)?
        },
        $($remaining_projection_entry:tt)*
    ) => {
        $visibility const $native_entry: $crate::NativePluginEntryDeclaration =
            $crate::NativePluginEntryDeclaration::new(
                $native_entry_name,
                concat!($native_entry_name, "\0").as_bytes(),
            );
        $crate::declare_plugin! {
            @registration_manifest
            [$visibility]
            [editor]
            [$native_registration_manifest]
            [$($capability => $capability_registration),+]
            {
                modules: [$($native_module)*],
                systems: [$($native_system)*],
                events: [$($native_event)*],
                extensions: [$($native_extension)*],
            }
        }
        $crate::declare_plugin! {
            @native_projection_entries
            [$visibility]
            [$($capability => $capability_registration),+]
            $($remaining_projection_entry)*
        }
    };
    (
        @registration_manifest
        [$visibility:vis]
        [$projection:ident]
        [$native_registration_manifest:ident]
        [$($capability:literal => $capability_registration:ident),+]
        {
            modules: [
                $(
                    {
                        name: $native_module_name:literal,
                        kind: $native_module_kind:literal $(,)?
                    }
                ),+ $(,)?
            ],
            systems: [
                $(
                    {
                        id: $native_system_id:literal,
                        module: $native_system_module:literal,
                        stage: $native_system_stage:literal,
                        order: $native_system_order:literal,
                        sets: [
                            $native_system_first_set:literal
                            $(, $native_system_set:literal)* $(,)?
                        ],
                        access: [
                            $native_system_first_access:literal
                            $(, $native_system_access:literal)* $(,)?
                        ],
                        thread_affinity: $native_system_thread_affinity:literal,
                        bridge_interface: $native_system_bridge_interface:literal,
                        bridge_method: $native_system_bridge_method:literal $(,)?
                    }
                ),* $(,)?
            ],
            events: [
                $(
                    {
                        namespace: $native_event_namespace:literal,
                        name: $native_event_name:literal,
                        stable_hash: $native_event_stable_hash:literal,
                        schema: $native_event_schema:literal $(,)?
                    }
                ),* $(,)?
            ],
            extensions: [
                $(
                    {
                        point: $native_extension_point:literal,
                        contribution: $native_extension_contribution:literal,
                        schema: $native_extension_schema:literal $(,)?
                    }
                ),* $(,)?
            ] $(,)?
        }
    ) => {
        $visibility const $native_registration_manifest: &[u8] = concat!(
            "schema = \"zircon.native.registration-manifest/3\"\n",
            "capabilities = [\n",
            $(
                $crate::declare_plugin!(
                    @registration_capability
                    $projection
                    $capability_registration
                    $capability
                ),
            )+
            "]\n",
            $(
                "[[modules]]\n",
                "name = \"", $native_module_name, "\"\n",
                "kind = \"", $native_module_kind, "\"\n",
            )+
            $(
                "[[systems]]\n",
                "id = \"", $native_system_id, "\"\n",
                "module = \"", $native_system_module, "\"\n",
                "stage = \"", $native_system_stage, "\"\n",
                "order = ", stringify!($native_system_order), "\n",
                "sets = [\"", $native_system_first_set, "\"",
                $(", \"", $native_system_set, "\"")*,
                "]\n",
                "access = [\"", $native_system_first_access, "\"",
                $(", \"", $native_system_access, "\"")*,
                "]\n",
                "thread_affinity = \"", $native_system_thread_affinity, "\"\n",
                "bridge_interface = \"", $native_system_bridge_interface, "\"\n",
                "bridge_method = \"", $native_system_bridge_method, "\"\n",
            )*
            $(
                "[[events]]\n",
                "namespace = \"", $native_event_namespace, "\"\n",
                "name = \"", $native_event_name, "\"\n",
                "stable_hash = ", stringify!($native_event_stable_hash), "\n",
                "schema = \"", $native_event_schema, "\"\n",
            )*
            $(
                "[[extensions]]\n",
                "point = \"", $native_extension_point, "\"\n",
                "contribution = \"", $native_extension_contribution, "\"\n",
                "schema = \"", $native_extension_schema, "\"\n",
            )*
            "\0",
        )
        .as_bytes();
    };
    (@registration_capability runtime runtime_registration $capability:literal) => {
        concat!("  \"", $capability, "\",\n")
    };
    (@registration_capability runtime runtime_editor_registration $capability:literal) => {
        concat!("  \"", $capability, "\",\n")
    };
    (@registration_capability runtime editor_registration $capability:literal) => {
        ""
    };
    (@registration_capability runtime requested_only $capability:literal) => {
        ""
    };
    (@registration_capability editor editor_registration $capability:literal) => {
        concat!("  \"", $capability, "\",\n")
    };
    (@registration_capability editor runtime_editor_registration $capability:literal) => {
        concat!("  \"", $capability, "\",\n")
    };
    (@registration_capability editor runtime_registration $capability:literal) => {
        ""
    };
    (@registration_capability editor requested_only $capability:literal) => {
        ""
    };
    (@capability_role runtime_registration) => {
        $crate::PluginCapabilityRole::RuntimeRegistration
    };
    (@capability_role editor_registration) => {
        $crate::PluginCapabilityRole::EditorRegistration
    };
    (@capability_role runtime_editor_registration) => {
        $crate::PluginCapabilityRole::RuntimeEditorRegistration
    };
    (@capability_role requested_only) => {
        $crate::PluginCapabilityRole::RequestedOnly
    };
    (@target client_runtime) => {
        $crate::PluginTarget::ClientRuntime
    };
    (@target server_runtime) => {
        $crate::PluginTarget::ServerRuntime
    };
    (@target editor_host) => {
        $crate::PluginTarget::EditorHost
    };
    (@platform windows) => {
        $crate::PluginPlatform::Windows
    };
    (@platform linux) => {
        $crate::PluginPlatform::Linux
    };
    (@platform macos) => {
        $crate::PluginPlatform::Macos
    };
    (@platform android) => {
        $crate::PluginPlatform::Android
    };
    (@platform ios) => {
        $crate::PluginPlatform::Ios
    };
    (@platform web_gpu) => {
        $crate::PluginPlatform::WebGpu
    };
    (@platform wasm) => {
        $crate::PluginPlatform::Wasm
    };
    (@platform headless) => {
        $crate::PluginPlatform::Headless
    };
    (@maturity core) => {
        $crate::PluginMaturityLevel::Core
    };
    (@maturity stable) => {
        $crate::PluginMaturityLevel::Stable
    };
    (@maturity beta) => {
        $crate::PluginMaturityLevel::Beta
    };
    (@maturity experimental) => {
        $crate::PluginMaturityLevel::Experimental
    };
    (@maturity externalized) => {
        $crate::PluginMaturityLevel::Externalized
    };
    (@maturity stub) => {
        $crate::PluginMaturityLevel::Stub
    };
    (@maturity deprecated) => {
        $crate::PluginMaturityLevel::Deprecated
    };
    (@packaging source_template) => {
        $crate::PluginPackaging::SourceTemplate
    };
    (@packaging library_embed) => {
        $crate::PluginPackaging::LibraryEmbed
    };
    (@packaging native_dynamic) => {
        $crate::PluginPackaging::NativeDynamic
    };
}
